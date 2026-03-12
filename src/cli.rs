use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "overclock")]
#[command(version = "0.1.0")]
#[command(about = "Overclock CLI - Multi-Agent Development Platform")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new project with ADDS template
    Init {
        name: String,
        #[arg(short, long)]
        path: Option<String>,
    },
    /// Manage agents (list, show, status, attach)
    Agent {
        #[command(subcommand)]
        agent_command: AgentCommands,
    },
    /// Start the Project Manager (main entry point)
    /// PM will orchestrate other agents automatically
    Run,
    /// Internal command: Start a specific agent (used by PM)
    #[command(hide = true)]
    StartAgent {
        role: String,
        #[arg(long)]
        task: Option<String>,
        #[arg(long)]
        parent_session: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum AgentCommands {
    /// List all available agent roles
    List,
    /// Show details of a specific agent role
    Show { role: String },
    /// Show status of running agents
    Status,
    /// Attach to a running agent by ID
    Attach { agent_id: String },
    /// Show completed agents and their reports
    Completed,
}
