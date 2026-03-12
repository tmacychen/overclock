use crate::cli::AgentCommands;
use crate::role::Role;
use anyhow::Result;
use colored::Colorize;

pub fn handle_agent(agent_command: AgentCommands) -> Result<()> {
    match agent_command {
        AgentCommands::List => list_agents(),
        AgentCommands::Show { role } => show_agent(&role),
    }
}

fn list_agents() -> Result<()> {
    println!("{}", "可用角色列表:".green().bold());
    println!();

    for role in Role::all() {
        println!(
            "  {} {} - {}",
            "•".cyan(),
            role.as_str().yellow().bold(),
            role.name()
        );
        println!("    {}", role.description().dimmed());
        println!();
    }

    println!("{}", "使用 'overclock run <role>' 启动 Agent".dimmed());
    Ok(())
}

fn show_agent(role_name: &str) -> Result<()> {
    let role: Role = match role_name.parse() {
        Ok(r) => r,
        Err(e) => return Err(anyhow::anyhow!("{}", e)),
    };

    println!("{}", format!("角色: {}", role.name()).green().bold());
    println!();
    println!("{} {}", "标识:".cyan(), role.as_str());
    println!("{} {}", "提示词文件:".cyan(), role.prompt_file());
    println!("{} {}", "触发条件:".cyan(), role.trigger());
    println!();
    println!("{} {}", "职责描述:".cyan(), role.description());

    Ok(())
}
