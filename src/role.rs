use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Pm,
    Architect,
    Developer,
    Tester,
    Reviewer,
}

impl Role {
    pub fn all() -> Vec<Role> {
        vec![
            Role::Pm,
            Role::Architect,
            Role::Developer,
            Role::Tester,
            Role::Reviewer,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Role::Pm => "Project Manager",
            Role::Architect => "Architect",
            Role::Developer => "Developer",
            Role::Tester => "Tester",
            Role::Reviewer => "Reviewer",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Role::Pm => {
                "负责需求分析、任务分解和进度跟踪。分析 app_spec.md，生成 feature_list.md，分配优先级和依赖关系。"
            }
            Role::Architect => {
                "负责技术设计、架构选型和环境配置。设计系统架构，选择技术栈，生成 architecture.md 和 init.sh。"
            }
            Role::Developer => {
                "负责功能实现。每个会话实现一个功能，编写单元测试，自我验证实现，更新功能状态为 testing。"
            }
            Role::Tester => {
                "负责测试验证和质量保证。运行所有测试用例，验证验收标准，运行回归测试，记录测试结果。"
            }
            Role::Reviewer => {
                "负责代码审查和安全审计。审查代码质量，检查安全漏洞，验证架构合规性，批准或拒绝功能。"
            }
        }
    }

    pub fn prompt_file(&self) -> &'static str {
        match self {
            Role::Pm => ".ai/prompts/pm_prompt.md",
            Role::Architect => ".ai/prompts/architect_prompt.md",
            Role::Developer => ".ai/prompts/developer_prompt.md",
            Role::Tester => ".ai/prompts/tester_prompt.md",
            Role::Reviewer => ".ai/prompts/reviewer_prompt.md",
        }
    }

    pub fn trigger(&self) -> &'static str {
        match self {
            Role::Pm => "project_start,requirements_change",
            Role::Architect => "pm_complete,architecture_missing",
            Role::Developer => "architecture_approved,feature_assigned",
            Role::Tester => "feature_testing,regression_check",
            Role::Reviewer => "tests_passed,security_audit",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Pm => "pm",
            Role::Architect => "architect",
            Role::Developer => "developer",
            Role::Tester => "tester",
            Role::Reviewer => "reviewer",
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pm" | "project_manager" => Ok(Role::Pm),
            "architect" => Ok(Role::Architect),
            "developer" | "dev" => Ok(Role::Developer),
            "tester" => Ok(Role::Tester),
            "reviewer" => Ok(Role::Reviewer),
            _ => Err(format!(
                "未知角色: '{}'。可用角色: pm, architect, developer, tester, reviewer",
                s
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_from_str() {
        assert_eq!(Role::from_str("pm").unwrap(), Role::Pm);
        assert_eq!(Role::from_str("architect").unwrap(), Role::Architect);
        assert_eq!(Role::from_str("developer").unwrap(), Role::Developer);
        assert_eq!(Role::from_str("tester").unwrap(), Role::Tester);
        assert_eq!(Role::from_str("reviewer").unwrap(), Role::Reviewer);
    }

    #[test]
    fn test_role_from_str_invalid() {
        assert!(Role::from_str("invalid").is_err());
    }

    #[test]
    fn test_role_display() {
        assert_eq!(format!("{}", Role::Pm), "pm");
        assert_eq!(format!("{}", Role::Architect), "architect");
    }

    #[test]
    fn test_all_roles() {
        let roles = Role::all();
        assert_eq!(roles.len(), 5);
    }
}
