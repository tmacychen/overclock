use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub project: ProjectConfig,
    pub codebuddy: CodebuddyConfig,
    pub agents: AgentsConfig,
    pub workflow: WorkflowConfig,
    pub files: FilesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebuddyConfig {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentsConfig {
    pub pm: AgentDefinition,
    pub architect: AgentDefinition,
    pub developer: AgentDefinition,
    pub tester: AgentDefinition,
    pub reviewer: AgentDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub name: String,
    pub prompt: String,
    pub trigger: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub feature_lifecycle: Vec<String>,
    pub regression_priority: String,
    pub max_retries: u32,
    pub session_timeout_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesConfig {
    pub feature_list: String,
    pub progress: String,
    pub architecture: String,
    pub core_guidelines: String,
}

impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    #[allow(dead_code)]
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn default_config() -> Self {
        Config {
            project: ProjectConfig {
                name: "overclock".to_string(),
                version: "0.1.0".to_string(),
                description: "ADDS Project Template".to_string(),
            },
            codebuddy: CodebuddyConfig {
                path: ".ai".to_string(),
            },
            agents: AgentsConfig {
                pm: AgentDefinition {
                    name: "Project Manager".to_string(),
                    prompt: ".ai/prompts/pm_prompt.md".to_string(),
                    trigger: "project_start,requirements_change".to_string(),
                },
                architect: AgentDefinition {
                    name: "Architect".to_string(),
                    prompt: ".ai/prompts/architect_prompt.md".to_string(),
                    trigger: "pm_complete,architecture_missing".to_string(),
                },
                developer: AgentDefinition {
                    name: "Developer".to_string(),
                    prompt: ".ai/prompts/developer_prompt.md".to_string(),
                    trigger: "architecture_approved,feature_assigned".to_string(),
                },
                tester: AgentDefinition {
                    name: "Tester".to_string(),
                    prompt: ".ai/prompts/tester_prompt.md".to_string(),
                    trigger: "feature_testing,regression_check".to_string(),
                },
                reviewer: AgentDefinition {
                    name: "Reviewer".to_string(),
                    prompt: ".ai/prompts/reviewer_prompt.md".to_string(),
                    trigger: "tests_passed,security_audit".to_string(),
                },
            },
            workflow: WorkflowConfig {
                feature_lifecycle: vec![
                    "pending".to_string(),
                    "in_progress".to_string(),
                    "testing".to_string(),
                    "completed".to_string(),
                ],
                regression_priority: "high".to_string(),
                max_retries: 3,
                session_timeout_minutes: 30,
            },
            files: FilesConfig {
                feature_list: ".ai/feature_list.md".to_string(),
                progress: ".ai/progress.md".to_string(),
                architecture: ".ai/architecture.md".to_string(),
                core_guidelines: "CORE_GUIDELINES.md".to_string(),
            },
        }
    }
}
