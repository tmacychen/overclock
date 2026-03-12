# Agent Fork 机制设计文档

## 概述

实现 PM Agent 动态创建（fork）其他 Agent（如 Architect）的能力，实现多 Agent 并行协作。

## 使用场景

1. PM 完成需求分析后，自动创建 Architect 进行架构设计
2. PM 在开发过程中，需要 Tester 并行进行测试
3. PM 需要 Reviewer 审查已完成的代码

## 设计方案

### 1. Fork 指令格式

PM 在对话中输出特定格式的指令：

```
[FORK:architect] 请设计用户认证模块的架构
[FORK:tester] 对登录功能进行测试
[FORK:reviewer] 审查 F005 的实现代码
```

### 2. 指令解析流程

```
overclock run pm
    ↓
PM 输出: "需求分析完成"
    ↓
PM 输出: "[FORK:architect] 请设计系统架构"
    ↓
overclock 捕获指令
    ↓
解析角色: architect
解析任务: "请设计系统架构"
    ↓
异步启动: overclock run architect
    ↓
PM 继续运行，Architect 并行工作
```

### 3. 实现架构

#### 3.1 修改 PM 系统提示词

添加 Fork 能力说明：

```markdown
## Fork Agent 能力

当你需要其他角色协助时，可以 Fork 创建新的 Agent。

### Fork 指令格式
[FORK:{role}] {任务描述}

### 可用角色
- architect: 架构师，负责技术设计
- developer: 开发者，负责功能实现  
- tester: 测试员，负责测试验证
- reviewer: 审查员，负责代码审查

### 使用示例
需求分析完成后，需要架构设计：
[FORK:architect] 基于 feature_list.md 设计系统架构

需要并行测试：
[FORK:tester] 对 F001 功能进行测试

### 注意事项
- Fork 后新 Agent 会独立运行
- 你可以继续你的工作
- 通过文件（如 progress.md）查看其他 Agent 的进展
```

#### 3.2 修改 run 命令实现

```rust
pub fn handle_run(role_name: &str, fork_mode: bool) -> Result<()> {
    // ... 现有代码 ...
    
    if fork_mode {
        // Fork 模式：在后台运行，捕获输出
        run_codebuddy_forked(&config, &role, &cwd, task_description)
    } else {
        // 普通模式：交互式运行
        run_codebuddy_interactive(&config, &role, &cwd)
    }
}
```

#### 3.3 添加 Agent 管理器

```rust
// src/agent_manager.rs

pub struct AgentManager {
    running_agents: HashMap<String, Child>,
}

impl AgentManager {
    pub fn fork_agent(&mut self, role: Role, task: String) -> Result<String> {
        // 1. 创建系统提示词（包含任务描述）
        // 2. 启动 codebuddy 进程
        // 3. 保存进程句柄
        // 4. 返回 Agent ID
    }
    
    pub fn list_running(&self) -> Vec<AgentInfo> {
        // 返回运行中的 Agent 列表
    }
    
    pub fn wait_for(&mut self, agent_id: &str) -> Result<ExitStatus> {
        // 等待指定 Agent 完成
    }
}
```

### 4. 通信机制

#### 方案 A：文件共享（推荐）

每个 Agent 通过读写共享文件通信：

```
.ai/agent_communication/
├── pm_to_architect.md      # PM 给 Architect 的消息
├── architect_to_pm.md      # Architect 给 PM 的消息
├── shared_context.md       # 共享上下文
└── agent_status.json       # Agent 状态
```

#### 方案 B：管道通信

使用 Unix socket 或命名管道实现实时通信。

### 5. 用户界面

```bash
# 查看运行中的 Agent
$ overclock agent status
运行中的 Agent:
  • pm (PID: 12345) - 需求分析中
  • architect (PID: 12346) - 架构设计中 [由 PM Fork]

# Fork 模式启动（内部使用）
$ overclock run architect --fork --task "设计认证模块"

# 等待特定 Agent
$ overclock agent wait architect
```

## 实现阶段

### Phase 1: 基础 Fork 功能
- [ ] 修改 PM 提示词，添加 Fork 指令说明
- [ ] 实现指令解析器
- [ ] 实现异步 Agent 启动
- [ ] 添加简单的文件通信

### Phase 2: Agent 管理
- [ ] 实现 AgentManager
- [ ] 添加 agent status 命令
- [ ] 添加 agent wait 命令
- [ ] 进程监控和自动清理

### Phase 3: 高级功能
- [ ] 双向实时通信
- [ ] Agent 间任务委托
- [ ] 结果汇总和报告

## 验收标准

### AC-1: Fork 指令解析
- **Given**: PM 输出 `[FORK:architect] 设计架构`
- **When**: overclock 捕获输出
- **Then**: 成功解析角色和任务
- **验证方式**: `programmatic`

### AC-2: 异步启动
- **Given**: PM 运行中
- **When**: PM Fork Architect
- **Then**: Architect 在后台启动，PM 继续运行
- **验证方式**: `manual`

### AC-3: 独立记忆
- **Given**: PM 和 Architect 同时运行
- **When**: 分别进行对话
- **Then**: 各自有独立的会话历史
- **验证方式**: `programmatic`

## 相关文件

- `src/commands/run.rs` - 启动逻辑
- `src/agent_manager.rs` - Agent 管理（新增）
- `template/.ai/prompts/pm_prompt.md` - PM 提示词
- `docs/agent-fork-design.md` - 本文档

## 变更记录

| 版本 | 日期 | 变更内容 | 作者 |
|------|------|----------|------|
| 1.0 | 2026-03-12 | 初始设计文档 | overclock |
