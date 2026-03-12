use crate::agent_manager::AgentManager;
use crate::role::Role;
use anyhow::Result;
use colored::Colorize;

pub fn handle_agent_list() -> Result<()> {
    println!("{}", "可用角色列表:".green().bold());
    println!();

    for role in Role::all() {
        let aliases = match role {
            Role::Pm => "(p)",
            Role::Architect => "(a, arch)",
            Role::Developer => "(d, dev)",
            Role::Tester => "(t, test)",
            Role::Reviewer => "(r, review)",
        };
        println!(
            "  {} {} {} - {}",
            "•".cyan(),
            role.as_str().yellow().bold(),
            aliases.dimmed(),
            role.name()
        );
        println!("    {}", role.description().dimmed());
        println!();
    }

    println!("{}", "使用 'overclock run' 启动 PM（主入口）".dimmed());
    println!("{}", "PM 会自动创建和管理其他 Agent".dimmed());
    Ok(())
}

pub fn handle_agent_show(_role: &str) -> Result<()> {
    println!("{}", "注意: 'agent show' 命令已废弃".yellow());
    println!("{}", "请使用 'overclock agent status' 查看运行中的 Agent".dimmed());
    Ok(())
}

pub fn handle_agent_status() -> Result<()> {
    let cwd = std::env::current_dir()
        .map_err(|_| anyhow::anyhow!("无法获取当前工作目录"))?;
    
    let config_path = cwd.join(".ai/config.toml");
    if !config_path.exists() {
        return Err(anyhow::anyhow!(
            "未找到配置文件 .ai/config.toml。请先在项目目录中运行此命令。"
        ));
    }

    let manager = AgentManager::new(&cwd);
    let agents = manager.list_running()?;

    if agents.is_empty() {
        println!("{}", "当前没有运行中的 Agent".yellow());
        return Ok(());
    }

    println!("{}", "运行中的 Agent:".green().bold());
    println!();
    println!(
        "  {:<12} {:<12} {:<20} {:<10} {}",
        "ID".cyan(),
        "Role".cyan(),
        "Status".cyan(),
        "Model".cyan(),
        "Task".cyan()
    );
    println!("  {}", "─".repeat(70).dimmed());

    for agent in agents {
        let status_display = if agent.role == "pm" {
            "运行中 (主会话)".to_string()
        } else {
            format!("运行中 [父: {}]", agent.parent.as_deref().unwrap_or("-"))
        };

        println!(
            "  {:<12} {:<12} {:<20} {:<10} {}",
            agent.id.yellow(),
            agent.role,
            status_display.dimmed(),
            agent.model,
            if agent.task.chars().count() > 30 {
                format!("{}...", agent.task.chars().take(27).collect::<String>())
            } else {
                agent.task.clone()
            }
        );
    }

    println!();
    println!("{}", "接入 Agent:".dimmed());
    println!("{}", "  overclock agent attach <ID>".dimmed());
    println!("{}", "查看日志:".dimmed());
    println!("{}", "  tail -f .ai/logs/<ID>.log".dimmed());

    Ok(())
}

pub fn handle_agent_attach(agent_id: &str) -> Result<()> {
    let cwd = std::env::current_dir()
        .map_err(|_| anyhow::anyhow!("无法获取当前工作目录"))?;
    
    let config_path = cwd.join(".ai/config.toml");
    if !config_path.exists() {
        return Err(anyhow::anyhow!(
            "未找到配置文件 .ai/config.toml。请先在项目目录中运行此命令。"
        ));
    }

    let manager = AgentManager::new(&cwd);
    
    let agent = manager.get_agent(agent_id)?
        .ok_or_else(|| anyhow::anyhow!("未找到 Agent: {}", agent_id))?;

    if agent.status != "running" {
        return Err(anyhow::anyhow!(
            "Agent {} 状态为 '{}'，无法接入",
            agent_id,
            agent.status
        ));
    }

    let project_name = cwd
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");
    let session_id = format!("overclock-{}-{}", project_name, agent_id);

    println!("{}", format!("接入 Agent: {}", agent_id).green().bold());
    println!("{}", format!("角色: {}", agent.role).dimmed());
    println!("{}", format!("任务: {}", agent.task).dimmed());
    println!("{}", format!("会话 ID: {}", session_id).dimmed());
    println!();
    println!("{}", "正在启动交互式会话...".dimmed());
    println!();

    let codebuddy_path = find_codebuddy()?;

    let mut child = std::process::Command::new(&codebuddy_path)
        .current_dir(&cwd)
        .arg("--session-id")
        .arg(&session_id)
        .arg("--continue")
        .env("OVERCLOCK_AGENT_ID", &agent_id)
        .env("OVERCLOCK_ROLE", &agent.role)
        .spawn()
        .map_err(|e| anyhow::anyhow!("无法启动 codebuddy: {}", e))?;

    let status = child.wait().map_err(|e| anyhow::anyhow!("等待 codebuddy 进程时出错: {}", e))?;

    if status.success() {
        println!();
        println!("{}", "会话已结束".green());
    } else {
        println!();
        println!(
            "{} 会话以状态 {} 退出",
            "警告:".yellow(),
            status.code().unwrap_or(-1)
        );
    }

    Ok(())
}

pub fn handle_agent() -> Result<()> {
    handle_agent_list()
}

pub fn handle_agent_completed() -> Result<()> {
    let cwd = std::env::current_dir()
        .map_err(|_| anyhow::anyhow!("无法获取当前工作目录"))?;
    
    let config_path = cwd.join(".ai/config.toml");
    if !config_path.exists() {
        return Err(anyhow::anyhow!(
            "未找到配置文件 .ai/config.toml。请先在项目目录中运行此命令。"
        ));
    }

    let manager = AgentManager::new(&cwd);
    let completed = manager.list_completed()?;

    if completed.is_empty() {
        println!("{}", "没有已完成的 Agent".yellow());
        return Ok(());
    }

    println!("{}", "已完成的 Agent:".green().bold());
    println!();

    for agent in completed {
        println!("{}", format!("Agent: {}", agent.id).yellow().bold());
        println!("{}", format!("  角色: {}", agent.role).dimmed());
        println!("{}", format!("  任务: {}", agent.task).dimmed());
        
        if let Ok(Some(report)) = manager.get_completed_report(&agent.id) {
            println!("{}", "  完成报告:".dimmed());
            for line in report.lines().take(10) {
                println!("    {}", line);
            }
            if report.lines().count() > 10 {
                println!("    ...");
            }
        }
        println!();
    }

    println!("{}", "查看完整报告:".dimmed());
    println!("{}", "  cat .ai/completed/<ID>.md".dimmed());

    Ok(())
}

fn find_codebuddy() -> Result<String> {
    if let Ok(path) = which::which("codebuddy") {
        return Ok(path.to_string_lossy().to_string());
    }

    Err(anyhow::anyhow!(
        "未找到 codebuddy CLI 工具。请确保已安装并添加到 PATH。"
    ))
}
