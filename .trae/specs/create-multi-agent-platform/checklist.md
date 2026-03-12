# 多 AI Agent 协同开发平台 - 验证清单

## CLI 命令系统验证
- [x] `overclock --help` 显示完整的命令帮助信息
- [x] `overclock --version` 显示正确的版本号
- [x] `overclock init --help` 显示 init 命令的详细帮助
- [x] `overclock run --help` 显示 run 命令的详细帮助
- [x] `overclock agent --help` 显示 agent 子命令的帮助

## 项目初始化验证
- [x] `overclock init test-project` 成功创建项目目录
- [x] 创建的项目包含 .ai/ 目录
- [x] .ai/ 目录包含 feature_list.md 文件
- [x] .ai/ 目录包含 progress.md 文件
- [x] .ai/ 目录包含 architecture.md 文件
- [x] .ai/prompts/ 目录包含所有角色提示词文件
- [x] 项目根目录包含 CORE_GUIDELINES.md 文件
- [x] 项目包含 .ai/config.toml 配置文件
- [x] 目录已存在时显示适当的错误信息
- [x] `--path` 参数正确指定项目路径

## 配置系统验证
- [x] config.toml 包含 PM 角色配置
- [x] config.toml 包含 Architect 角色配置
- [x] config.toml 包含 Developer 角色配置
- [x] config.toml 包含 Tester 角色配置
- [x] config.toml 包含 Reviewer 角色配置
- [x] 配置文件可正确解析为 Config 结构
- [x] 无效配置文件返回明确的错误信息
- [x] codebuddy 路径配置正确读取

## Agent 角色验证
- [x] PM 角色有正确的职责描述
- [x] Architect 角色有正确的职责描述
- [x] Developer 角色有正确的职责描述
- [x] Tester 角色有正确的职责描述
- [x] Reviewer 角色有正确的职责描述
- [x] 角色枚举可从字符串正确解析
- [x] 角色映射到正确的提示词文件路径

## codebuddy 集成验证
- [x] 可检测 codebuddy 是否已安装
- [x] codebuddy 未安装时显示安装提示
- [x] 正确构建 codebuddy 调用命令
- [x] 角色信息正确传递给 codebuddy
- [x] 职责描述正确传递给 codebuddy

## Agent 会话验证
- [x] `overclock run pm` 成功启动 PM Agent
- [x] `overclock run architect` 成功启动 Architect Agent
- [x] `overclock run developer` 成功启动 Developer Agent
- [x] Agent 进程可接收用户输入
- [x] Agent 输出正确显示在终端
- [x] Agent 进程可正常退出
- [x] Agent 进程异常退出时显示错误信息

## agent 子命令验证
- [x] `overclock agent list` 列出所有可用角色
- [x] `overclock agent show pm` 显示 PM 角色详情
- [x] `overclock agent show architect` 显示 Architect 角色详情
- [x] 角色详情包含职责描述
- [x] 角色详情包含提示词文件路径

## 错误处理验证
- [x] 无效命令返回非零退出码
- [x] 无效角色名返回清晰的错误信息
- [x] 配置文件格式错误提供修复建议
- [x] codebuddy 未找到提供安装指引
- [x] 权限错误提供解决方案建议

## 代码质量验证
- [x] `cargo build` 无编译错误
- [x] `cargo test` 所有测试通过
- [x] `cargo clippy` 无警告
- [x] `cargo fmt --check` 格式正确
- [x] 代码有适当的文档注释
