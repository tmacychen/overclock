mod agent_manager;
mod cli;
mod commands;
mod config;
mod error;
mod role;

use clap::Parser;
use cli::{AgentCommands, Cli, Commands};
use colored::Colorize;

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { name, path } => commands::handle_init(&name, path.as_deref()),
        Commands::Agent { agent_command } => handle_agent_command(agent_command),
        Commands::Run => commands::handle_run(),
        Commands::StartAgent {
            role,
            task,
            parent_session,
        } => commands::handle_start_agent(&role, task.as_deref(), parent_session.as_deref()),
    };

    if let Err(e) = result {
        eprintln!("{} {}", "错误:".red(), e);

        if let Some(hint) = extract_hint(&e) {
            println!();
            println!("{}", "提示:".yellow().bold());
            println!("{}", hint);
        }

        std::process::exit(1);
    }
}

fn handle_agent_command(agent_command: AgentCommands) -> anyhow::Result<()> {
    match agent_command {
        AgentCommands::List => commands::handle_agent_list(),
        AgentCommands::Show { role } => commands::handle_agent_show(&role),
        AgentCommands::Status => commands::handle_agent_status(),
        AgentCommands::Attach { agent_id } => commands::handle_agent_attach(&agent_id),
        AgentCommands::Completed => commands::handle_agent_completed(),
    }
}

fn extract_hint(error: &anyhow::Error) -> Option<String> {
    let error_str = error.to_string();

    if error_str.contains("codebuddy") {
        Some("请确保已安装 codebuddy CLI 工具并添加到 PATH".to_string())
    } else if error_str.contains("config.toml") {
        Some("请先使用 'overclock init' 初始化项目".to_string())
    } else if error_str.contains("已存在") {
        Some("使用 --path 参数指定其他路径，或使用不同的项目名称".to_string())
    } else if error_str.contains("未知角色") {
        Some("可用角色: pm, architect, developer, tester, reviewer".to_string())
    } else if error_str.contains("提示词文件不存在") {
        Some("请确保项目已正确初始化，包含 .ai/prompts/ 目录".to_string())
    } else if error_str.contains("Agent not found") || error_str.contains("未找到 Agent") {
        Some("运行 'overclock agent status' 查看运行中的 Agent".to_string())
    } else {
        None
    }
}
