use crate::config::Config;
use crate::role::Role;
use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;
use std::process::Command;

pub fn handle_run(role_name: &str) -> Result<()> {
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

    let config = Config::load(&config_path).context("无法加载配置文件")?;

    println!("{} {} Agent...", "启动".green(), role.name().cyan());
    println!();

    run_codebuddy(&config, &role, &cwd)
}

fn run_codebuddy(config: &Config, role: &Role, project_path: &Path) -> Result<()> {
    let codebuddy_path = find_codebuddy()?;
    let prompt_path = project_path.join(role.prompt_file());

    if !prompt_path.exists() {
        return Err(anyhow::anyhow!(
            "提示词文件不存在: {}",
            prompt_path.display()
        ));
    }

    let system_prompt = create_system_prompt(role, &prompt_path)?;
    let temp_prompt_path = project_path.join(format!(".ai/.system_{}.md", role.as_str()));

    std::fs::write(&temp_prompt_path, system_prompt)
        .context("无法创建临时系统提示词文件")?;

    // 生成角色固定的 session-id，实现角色独立记忆
    let session_id = format!("overclock-{}-{}", config.project.name, role.as_str());

    println!("{}", format!("角色: {}", role.name()).yellow().bold());
    println!("{}", format!("职责: {}", role.description()).dimmed());
    println!("{}", format!("会话 ID: {}", session_id).dimmed());
    println!("{}", format!("提示词: {}", role.prompt_file()).dimmed());
    println!();
    println!("{}", "正在启动 codebuddy...".dimmed());
    println!();

    let mut child = Command::new(&codebuddy_path)
        .current_dir(project_path)
        .arg("--system-prompt-file")
        .arg(&temp_prompt_path)
        .arg("--session-id")
        .arg(&session_id)
        .env("OVERCLOCK_ROLE", role.as_str())
        .env("OVERCLOCK_ROLE_NAME", role.name())
        .env("OVERCLOCK_ROLE_DESCRIPTION", role.description())
        .spawn()
        .context("无法启动 codebuddy")?;

    let status = child.wait().context("等待 codebuddy 进程时出错")?;

    let _ = std::fs::remove_file(&temp_prompt_path);

    if status.success() {
        println!();
        println!("{}", "Agent 会话已结束".green());
    } else {
        println!();
        println!(
            "{} Agent 以状态 {} 退出",
            "警告:".yellow(),
            status.code().unwrap_or(-1)
        );
    }

    Ok(())
}

fn create_system_prompt(role: &Role, prompt_path: &Path) -> Result<String> {
    let base_prompt = std::fs::read_to_string(prompt_path)
        .context("无法读取提示词文件")?;

    Ok(format!(
        r#"# 系统角色设定

你当前的角色是: **{}**
你的职责是: {}

---

{}

---

现在开始你的工作。记住始终保持角色身份，按照上述职责执行任务。
"#,
        role.name(),
        role.description(),
        base_prompt
    ))
}

fn find_codebuddy() -> Result<String> {
    if let Ok(path) = which::which("codebuddy") {
        return Ok(path.to_string_lossy().to_string());
    }

    Err(anyhow::anyhow!(
        "未找到 codebuddy CLI 工具。请确保已安装并添加到 PATH。\n\
         安装说明: https://github.com/your-org/codebuddy"
    ))
}
