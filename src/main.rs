mod cli;
mod commands;
mod config;
mod error;
mod role;

use clap::Parser;
use cli::{Cli, Commands};
use colored::Colorize;

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { name, path } => commands::handle_init(&name, path.as_deref()),
        Commands::Agent { agent_command } => commands::handle_agent(agent_command),
        Commands::Run { role } => commands::handle_run(&role),
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
    } else {
        None
    }
}
