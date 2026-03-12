use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "overclock")]
#[command(version = "0.1.0")]
#[command(about = "Overclock CLI - Agent management tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Init {
        name: String,
        #[arg(short, long)]
        path: Option<String>,
    },
    Agent {
        #[command(subcommand)]
        agent_command: AgentCommands,
    },
    Run {
        role: String,
    },
}

#[derive(Subcommand)]
pub enum AgentCommands {
    List,
    Show { role: String },
}
