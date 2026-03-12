use crate::config::Config;
use crate::role::Role;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub role: String,
    pub pid: u32,
    pub model: String,
    pub status: String,
    pub task: String,
    pub started_at: u64,
    pub parent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentStatus {
    pub agents: Vec<AgentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentCounter {
    pub pm: u32,
    pub architect: u32,
    pub developer: u32,
    pub tester: u32,
    pub reviewer: u32,
}

pub struct AgentManager {
    project_path: PathBuf,
    status_file: PathBuf,
    counter_file: PathBuf,
    pipes_dir: PathBuf,
    logs_dir: PathBuf,
    completed_dir: PathBuf,
    tasks_dir: PathBuf,
}

impl AgentManager {
    pub fn new(project_path: &std::path::Path) -> Self {
        let ai_dir = project_path.join(".ai");
        Self {
            project_path: project_path.to_path_buf(),
            status_file: ai_dir.join("agent_status.json"),
            counter_file: ai_dir.join("agent_counter.json"),
            pipes_dir: ai_dir.join("pipes"),
            logs_dir: ai_dir.join("logs"),
            completed_dir: ai_dir.join("completed"),
            tasks_dir: ai_dir.join("tasks"),
        }
    }

    pub fn with_config(project_path: &std::path::Path) -> Result<Self> {
        let ai_dir = project_path.join(".ai");
        
        // 确保必要的目录存在
        std::fs::create_dir_all(&ai_dir)?;
        
        Ok(Self {
            project_path: project_path.to_path_buf(),
            status_file: ai_dir.join("agent_status.json"),
            counter_file: ai_dir.join("agent_counter.json"),
            pipes_dir: ai_dir.join("pipes"),
            logs_dir: ai_dir.join("logs"),
            completed_dir: ai_dir.join("completed"),
            tasks_dir: ai_dir.join("tasks"),
        })
    }

    pub fn allocate_id(&mut self, role: &Role) -> Result<String> {
        let mut counter = self.load_counter()?;

        let count = match role {
            Role::Pm => &mut counter.pm,
            Role::Architect => &mut counter.architect,
            Role::Developer => &mut counter.developer,
            Role::Tester => &mut counter.tester,
            Role::Reviewer => &mut counter.reviewer,
        };

        *count += 1;
        let id = format!("{}-{:03}", role.as_str(), count);

        self.save_counter(&counter)?;

        Ok(id)
    }

    pub fn start_agent(
        &mut self,
        role: Role,
        task: String,
        model: Option<String>,
        parent_id: Option<String>,
    ) -> Result<(String, std::process::Child)> {
        self.ensure_directories()?;

        let id = self.allocate_id(&role)?;
        let model = model.unwrap_or_else(|| "glm-5.0".to_string());

        let task_file = self.tasks_dir.join(format!("{}.md", &id));
        std::fs::write(&task_file, &task).context("无法写入任务文件")?;

        let log_file = self.logs_dir.join(format!("{}.log", &id));

        // 使用项目名称生成 session-id，与配置文件保持一致
        let project_name = self.project_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project");
        
        // 尝试从配置文件获取项目名称，如果失败则使用路径名称
        let config_name = match self.load_config_name() {
            Ok(name) => name,
            Err(_) => project_name.to_string(),
        };
        
        let session_id = format!("overclock-{}-{}", config_name, &id);

        let prompt_path = self.project_path.join(role.prompt_file());
        let system_prompt = self.create_system_prompt_with_task(&role, &prompt_path, &task, &id)?;
        let temp_prompt_path = self.project_path.join(format!(".ai/.system_{}.md", &id));

        std::fs::write(&temp_prompt_path, system_prompt)
            .context("无法创建临时系统提示词文件")?;

        let codebuddy_path = self.find_codebuddy()?;

        let log_file_handle = std::fs::File::create(&log_file)
            .context("无法创建日志文件")?;

        let child = std::process::Command::new(&codebuddy_path)
            .current_dir(&self.project_path)
            .arg("--system-prompt-file")
            .arg(&temp_prompt_path)
            .arg("--session-id")
            .arg(&session_id)
            .arg("--model")
            .arg(&model)
            .env("OVERCLOCK_AGENT_ID", &id)
            .env("OVERCLOCK_ROLE", role.as_str())
            .env("OVERCLOCK_ROLE_NAME", role.name())
            .env("OVERCLOCK_ROLE_DESCRIPTION", role.description())
            .stdout(std::process::Stdio::from(log_file_handle))
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .context("无法启动 codebuddy")?;

        let started_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let agent_info = AgentInfo {
            id: id.clone(),
            role: role.as_str().to_string(),
            pid: child.id(),
            model,
            status: "running".to_string(),
            task,
            started_at,
            parent: parent_id,
        };

        self.add_agent(&agent_info)?;

        Ok((id, child))
    }

    pub fn list_running(&self) -> Result<Vec<AgentInfo>> {
        let status = self.load_status()?;
        Ok(status.agents.into_iter().filter(|a| a.status == "running").collect())
    }

    pub fn get_agent(&self, agent_id: &str) -> Result<Option<AgentInfo>> {
        let status = self.load_status()?;
        Ok(status.agents.into_iter().find(|a| a.id == agent_id))
    }

    pub fn mark_completed(&mut self, agent_id: &str, report: String) -> Result<()> {
        let mut status = self.load_status()?;
        
        if let Some(agent) = status.agents.iter_mut().find(|a| a.id == agent_id) {
            agent.status = "completed".to_string();
        }

        self.save_status(&status)?;

        let report_file = self.completed_dir.join(format!("{}.md", agent_id));
        std::fs::write(&report_file, report).context("无法写入完成报告")?;

        Ok(())
    }

    pub fn list_completed(&self) -> Result<Vec<AgentInfo>> {
        let status = self.load_status()?;
        Ok(status.agents.into_iter().filter(|a| a.status == "completed").collect())
    }

    pub fn get_completed_report(&self, agent_id: &str) -> Result<Option<String>> {
        let report_file = self.completed_dir.join(format!("{}.md", agent_id));
        if report_file.exists() {
            let content = std::fs::read_to_string(&report_file)
                .context("无法读取完成报告")?;
            Ok(Some(content))
        } else {
            Ok(None)
        }
    }

    fn ensure_directories(&self) -> Result<()> {
        std::fs::create_dir_all(&self.pipes_dir).context("无法创建 pipes 目录")?;
        std::fs::create_dir_all(&self.logs_dir).context("无法创建 logs 目录")?;
        std::fs::create_dir_all(&self.completed_dir).context("无法创建 completed 目录")?;
        std::fs::create_dir_all(&self.tasks_dir).context("无法创建 tasks 目录")?;
        Ok(())
    }

    fn load_status(&self) -> Result<AgentStatus> {
        if !self.status_file.exists() {
            return Ok(AgentStatus::default());
        }

        let content = std::fs::read_to_string(&self.status_file)
            .context("无法读取 agent_status.json")?;
        
        serde_json::from_str(&content).context("无法解析 agent_status.json")
    }

    fn save_status(&self, status: &AgentStatus) -> Result<()> {
        let content = serde_json::to_string_pretty(status)
            .context("无法序列化 agent_status.json")?;
        
        std::fs::write(&self.status_file, content)
            .context("无法写入 agent_status.json")
    }

    fn load_counter(&self) -> Result<AgentCounter> {
        if !self.counter_file.exists() {
            return Ok(AgentCounter::default());
        }

        let content = std::fs::read_to_string(&self.counter_file)
            .context("无法读取 agent_counter.json")?;
        
        serde_json::from_str(&content).context("无法解析 agent_counter.json")
    }

    fn save_counter(&self, counter: &AgentCounter) -> Result<()> {
        let content = serde_json::to_string_pretty(counter)
            .context("无法序列化 agent_counter.json")?;
        
        std::fs::write(&self.counter_file, content)
            .context("无法写入 agent_counter.json")
    }

    pub fn add_agent(&self, info: &AgentInfo) -> Result<()> {
        let mut status = self.load_status()?;
        status.agents.push(info.clone());
        self.save_status(&status)
    }

    fn create_system_prompt_with_task(
        &self,
        role: &Role,
        prompt_path: &std::path::Path,
        task: &str,
        agent_id: &str,
    ) -> Result<String> {
        let base_prompt = std::fs::read_to_string(prompt_path)
            .context("无法读取提示词文件")?;

        Ok(format!(
            r#"# 系统角色设定

你当前的角色是: **{}**
你的 Agent ID 是: **{}**
你的职责是: {}

---

## 当前任务

{}

---

{}

---

## 进度更新

定期更新 .ai/progress.md 文件，记录你的工作进度：
- 当前正在做什么
- 已完成的内容
- 遇到的问题
- 下一步计划

## 完成任务

完成任务后，请输出以下格式的指令：

[COMPLETE]
完成报告：
- 实现了什么
- 修改了哪些文件
- 测试结果
[END]

现在开始你的工作。记住始终保持角色身份，按照上述职责执行任务。
"#,
            role.name(),
            agent_id,
            role.description(),
            task,
            base_prompt
        ))
    }

    fn find_codebuddy(&self) -> Result<String> {
        if let Ok(path) = which::which("codebuddy") {
            return Ok(path.to_string_lossy().to_string());
        }

        Err(anyhow::anyhow!(
            "未找到 codebuddy CLI 工具。请确保已安装并添加到 PATH。"
        ))
    }

    fn load_config_name(&self) -> Result<String> {
        let config_path = self.project_path.join(".ai/config.toml");
        if config_path.exists() {
            let config = Config::load(&config_path)?;
            Ok(config.project.name)
        } else {
            Err(anyhow::anyhow!("配置文件不存在"))
        }
    }
}
