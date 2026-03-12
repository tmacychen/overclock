use crate::agent_manager::AgentManager;
use crate::config::Config;
use crate::role::Role;
use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;

pub fn handle_run() -> Result<()> {
    let cwd = std::env::current_dir().context("无法获取当前工作目录")?;
    let config_path = cwd.join(".ai/config.toml");

    if !config_path.exists() {
        return Err(anyhow::anyhow!(
            "未找到配置文件 .ai/config.toml。请先在项目目录中运行此命令。"
        ));
    }

    let config = Config::load(&config_path).context("无法加载配置文件")?;
    let role = Role::Pm;

    println!("{} {} Agent...", "启动".green(), role.name().cyan());
    println!();

    run_pm(&config, &role, &cwd)
}

fn run_pm(config: &Config, role: &Role, project_path: &Path) -> Result<()> {
    let codebuddy_path = find_codebuddy()?;
    let prompt_path = project_path.join(role.prompt_file());

    if !prompt_path.exists() {
        return Err(anyhow::anyhow!(
            "提示词文件不存在: {}",
            prompt_path.display()
        ));
    }

    let mut manager = AgentManager::new(project_path);
    let id = manager.allocate_id(role)?;

    let system_prompt = create_pm_system_prompt(role, &prompt_path, &id)?;
    let temp_prompt_path = project_path.join(format!(".ai/.system_{}.md", &id));

    std::fs::write(&temp_prompt_path, system_prompt)
        .context("无法创建临时系统提示词文件")?;

    // 使用配置文件中的项目名称，确保与 codebuddy 的存储机制匹配
    let config_name = config.project.name.clone();
    let session_id = format!("overclock-{}-{}", config_name, &id);

    println!("{}", format!("角色: {}", role.name()).yellow().bold());
    println!("{}", format!("Agent ID: {}", id).dimmed());
    println!("{}", format!("职责: {}", role.description()).dimmed());
    println!("{}", format!("会话 ID: {}", session_id).dimmed());
    println!("{}", format!("提示词: {}", role.prompt_file()).dimmed());
    println!();
    println!("{}", "正在启动 codebuddy...".dimmed());
    println!();

    let mut child = std::process::Command::new(&codebuddy_path)
        .current_dir(project_path)
        .arg("--system-prompt-file")
        .arg(&temp_prompt_path)
        .arg("--session-id")
        .arg(&session_id)
        .env("OVERCLOCK_AGENT_ID", &id)
        .env("OVERCLOCK_ROLE", role.as_str())
        .env("OVERCLOCK_ROLE_NAME", role.name())
        .env("OVERCLOCK_ROLE_DESCRIPTION", role.description())
        .spawn()
        .context("无法启动 codebuddy")?;

    let started_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let agent_info = crate::agent_manager::AgentInfo {
        id: id.clone(),
        role: role.as_str().to_string(),
        pid: child.id(),
        model: "glm-5.0".to_string(),
        status: "running".to_string(),
        task: "主会话".to_string(),
        started_at,
        parent: None,
    };

    manager.add_agent(&agent_info)?;

    let status = child.wait().context("等待 codebuddy 进程时出错")?;

    let _ = std::fs::remove_file(&temp_prompt_path);

    if status.success() {
        println!();
        println!("{}", "PM 会话已结束".green());
    } else {
        println!();
        println!(
            "{} PM 以状态 {} 退出",
            "警告:".yellow(),
            status.code().unwrap_or(-1)
        );
    }

    Ok(())
}

pub fn handle_start_agent(
    role_name: &str,
    task: Option<&str>,
    parent_session: Option<&str>,
) -> Result<()> {
    let role: Role = match role_name.parse() {
        Ok(r) => r,
        Err(e) => return Err(anyhow::anyhow!("{}", e)),
    };

    let cwd = std::env::current_dir().context("无法获取当前工作目录")?;
    let config_path = cwd.join(".ai/config.toml");

    if !config_path.exists() {
        return Err(anyhow::anyhow!(
            "未找到配置文件 .ai/config.toml。请先在项目目录中运行此命令。"
        ));
    }

    let task = task
        .map(|s| s.to_string())
        .unwrap_or_else(|| "未指定任务".to_string());

    println!("{} {} Agent (后台)...", "启动".green(), role.name().cyan());
    println!();

    let mut manager = AgentManager::new(&cwd);
    let (agent_id, mut child) = manager.start_agent(
        role,
        task.clone(),
        None,
        parent_session.map(|s| s.to_string()),
    )?;

    println!("{}", format!("Agent ID: {}", agent_id).yellow().bold());
    println!("{}", format!("任务: {}", task).dimmed());
    println!();
    println!("{}", "Agent 已在后台启动".dimmed());
    println!("{}", format!("查看状态: overclock agent status").dimmed());
    println!(
        "{}",
        format!("查看日志: tail -f .ai/logs/{}.log", agent_id).dimmed()
    );
    println!(
        "{}",
        format!("接入会话: overclock agent attach {}", agent_id).dimmed()
    );

    let status = child.wait().context("等待 codebuddy 进程时出错")?;

    if status.success() {
        println!();
        println!("{}", format!("Agent {} 已完成任务", agent_id).green());
    } else {
        println!();
        println!(
            "{} Agent {} 以状态 {} 退出",
            "警告:".yellow(),
            agent_id,
            status.code().unwrap_or(-1)
        );
    }

    Ok(())
}

fn create_pm_system_prompt(role: &Role, prompt_path: &Path, agent_id: &str) -> Result<String> {
    let base_prompt = std::fs::read_to_string(prompt_path).context("无法读取提示词文件")?;

    Ok(format!(
        r#"# 系统角色设定

你当前的角色是: **{}**
你的 Agent ID 是: **{}**
你的职责是: {}

---

{}

---

## Agent 编排能力

你可以创建和管理其他 Agent 来完成工作。

### 创建 Agent
输出以下格式的指令：

[FORK:{{role}}] {{任务描述}}

示例：
[FORK:architect] 基于 feature_list.md 设计系统架构
[FORK:developer] 实现 F001: 用户登录功能
[FORK:tester] 测试 F001 的登录功能

### 可用角色
- architect: 架构师，负责技术设计
- developer: 开发者，负责功能实现（可创建多个）
- tester: 测试员，负责测试验证
- reviewer: 审查员，负责代码审查

### 监控进度
读取 .ai/progress.md 查看其他 Agent 的工作进度

### 更新进度
定期更新 .ai/progress.md 文件，记录：
- 当前工作状态
- 已完成的任务
- 遇到的问题
- 下一步计划

### 接收完成通知
定期检查 .ai/completed/ 目录，读取 Agent 的完成报告

### 汇报结果
当 Agent 完成后，读取其输出文件，向用户汇报结果

现在开始你的工作。记住始终保持角色身份，按照上述职责执行任务。
"#,
        role.name(),
        agent_id,
        role.description(),
        base_prompt
    ))
}

fn find_codebuddy() -> Result<String> {
    if let Ok(path) = which::which("codebuddy") {
        return Ok(path.to_string_lossy().to_string());
    }

    Err(anyhow::anyhow!(
        "未找到 codebuddy CLI 工具。请确保已安装并添加到 PATH。"
    ))
}
