# Overclock - 多 AI Agent 协同开发平台

基于 [AI-Driven Development Specification (ADDS)](https://github.com/tmacychen/ai-driven-dev-spec) 构建的多 AI Agent 协同开发 CLI 工具。

## 功能特性

- **多角色 Agent 支持**: PM、Architect、Developer、Tester、Reviewer
- **项目模板初始化**: 一键生成符合 ADDS 规范的项目结构
- **codebuddy 集成**: 通过本地 codebuddy CLI 启动 AI Agent 会话
- **配置管理**: TOML 格式的项目配置文件

## 安装

### 前置要求

- Rust 1.70+
- codebuddy CLI 工具

### 从源码构建

```bash
git clone https://github.com/your-org/overclock.git
cd overclock
cargo build --release
```

编译后的二进制文件位于 `target/release/overclock`。

## 使用方法

### 初始化项目

```bash
# 在当前目录创建项目
overclock init my-project

# 指定路径创建项目
overclock init my-project --path /path/to/projects
```

初始化后的项目结构：

```
my-project/
├── .ai/
│   ├── prompts/
│   │   ├── pm_prompt.md
│   │   ├── architect_prompt.md
│   │   ├── developer_prompt.md
│   │   ├── tester_prompt.md
│   │   └── reviewer_prompt.md
│   ├── config.toml
│   ├── feature_list.md
│   ├── progress.md
│   └── architecture.md
└── CORE_GUIDELINES.md
```

### 管理 Agent 角色

```bash
# 列出所有可用角色
overclock agent list

# 查看角色详情
overclock agent show pm
overclock agent show architect
```

### 启动 Agent 会话

```bash
# 进入项目目录
cd my-project

# 启动 PM Agent
overclock run pm

# 启动 Architect Agent
overclock run architect

# 启动 Developer Agent
overclock run developer
```

## Agent 角色

| 角色 | 标识 | 职责 |
|------|------|------|
| Project Manager | `pm` | 需求分析、任务分解、进度跟踪 |
| Architect | `architect` | 技术设计、架构选型、环境配置 |
| Developer | `developer` | 功能实现、单元测试、自我验证 |
| Tester | `tester` | 测试验证、回归测试、质量保证 |
| Reviewer | `reviewer` | 代码审查、安全审计、质量门控 |

## 配置文件

项目配置位于 `.ai/config.toml`：

```toml
[project]
name = "my-project"
version = "0.1.0"

[codebuddy]
path = ".ai"

[agents.pm]
name = "Project Manager"
prompt = ".ai/prompts/pm_prompt.md"
trigger = "project_start,requirements_change"

[agents.architect]
name = "Architect"
prompt = ".ai/prompts/architect_prompt.md"
trigger = "pm_complete,architecture_missing"

# ... 其他角色配置
```

## 开发

### 构建

```bash
cargo build
```

### 测试

```bash
cargo test
```

### 代码检查

```bash
cargo clippy
cargo fmt
```

## 许可证

MIT License

## 相关项目

- [AI-Driven Development Specification (ADDS)](https://github.com/tmacychen/ai-driven-dev-spec) - AI 驱动开发规范
- [codebuddy](https://github.com/your-org/codebuddy) - 本地 AI Agent CLI 工具
