use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum OverclockError {
    #[error("配置文件错误: {0}")]
    ConfigError(String),

    #[error("角色错误: {0}")]
    RoleError(String),

    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("模板错误: {0}")]
    TemplateError(String),

    #[error("Agent 错误: {0}")]
    AgentError(String),

    #[error(
        "codebuddy 未找到。请确保已安装并添加到 PATH。\n安装说明: https://github.com/your-org/codebuddy"
    )]
    CodebuddyNotFound,

    #[error("项目目录不存在: {0}")]
    ProjectNotFound(String),

    #[error("目录已存在: {0}")]
    DirectoryExists(String),
}

impl From<toml::de::Error> for OverclockError {
    fn from(e: toml::de::Error) -> Self {
        OverclockError::ConfigError(format!("TOML 解析错误: {}", e))
    }
}

impl From<String> for OverclockError {
    fn from(e: String) -> Self {
        OverclockError::RoleError(e)
    }
}

#[allow(dead_code)]
pub fn print_error_hint(error: &OverclockError) {
    use colored::Colorize;

    match error {
        OverclockError::CodebuddyNotFound => {
            println!();
            println!("{}", "修复建议:".yellow().bold());
            println!("  1. 安装 codebuddy CLI 工具");
            println!("  2. 确保 codebuddy 在 PATH 环境变量中");
            println!("  3. 运行 'codebuddy --version' 验证安装");
        }
        OverclockError::ProjectNotFound(path) => {
            println!();
            println!("{}", "修复建议:".yellow().bold());
            println!("  1. 确认路径是否正确: {}", path);
            println!("  2. 使用 'overclock init' 创建新项目");
        }
        OverclockError::DirectoryExists(path) => {
            println!();
            println!("{}", "修复建议:".yellow().bold());
            println!("  1. 使用不同的项目名称");
            println!("  2. 使用 --path 参数指定其他路径");
            println!("  3. 删除现有目录: rm -rf {}", path);
        }
        OverclockError::ConfigError(msg) => {
            println!();
            println!("{}", "修复建议:".yellow().bold());
            println!("  1. 检查 .ai/config.toml 文件格式");
            println!("  2. 确保所有必需字段都已配置");
            if msg.contains("TOML") {
                println!("  3. 使用 TOML 验证工具检查语法");
            }
        }
        _ => {}
    }
}
