# 待开发功能需求文档

## 创建日期
2026-03-12

## 功能需求：PM 自动创建并管理 Architect

### 需求描述

用户希望 PM 能够自动创建 Architect Agent 来设计项目的技术方案，并且用户可以通过 `attach` 命令接入到 Architect 的会话中，观察其工作过程。

### 用户场景

```
用户: 我想做一个 Todo List 应用

PM: 好的，让我分析需求...
    [编写 feature_list.md]
    
PM: 需求分析完成，现在创建 Architect 进行架构设计
    [FORK:architect] 基于 feature_list.md 设计系统架构
    
# 系统自动执行：
# overclock start-agent architect --task "基于 feature_list.md 设计系统架构"

PM: Architect 已启动 (ID: architect-001)
    你可以通过以下方式观察：
    - 查看状态: overclock agent status
    - 接入会话: overclock agent attach architect-001
    - 查看日志: tail -f .ai/logs/architect-001.log

# 用户在新终端接入
$ overclock agent attach architect-001

[接入 Architect 会话]
Architect: 正在分析 feature_list.md...
Architect: 设计数据库架构...
Architect: 选择技术栈: Rust + Axum + PostgreSQL

> 为什么选择 PostgreSQL？

Architect: 考虑到 Todo List 需要关系型数据存储...
```

### 技术实现方案

#### 1. 输出捕获与指令解析

在 `src/commands/run.rs` 中，需要捕获 PM (codebuddy) 的输出流，并解析特殊指令。

```rust
// 伪代码
pub fn handle_run() -> Result<()> {
    let mut child = spawn_codebuddy_pm()?;
    
    // 捕获 stdout
    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    
    for line in reader.lines() {
        let line = line?;
        
        // 解析 FORK 指令
        if let Some(fork_cmd) = parse_fork_command(&line) {
            handle_fork_command(fork_cmd)?;
        }
        
        // 同时输出到用户
        println!("{}", line);
    }
    
    Ok(())
}

fn parse_fork_command(line: &str) -> Option<ForkCommand> {
    // 解析格式: [FORK:architect] 任务描述
    let re = Regex::new(r"\[FORK:(\w+)\]\s*(.+)").unwrap();
    re.captures(line).map(|caps| {
        ForkCommand {
            role: caps[1].to_string(),
            task: caps[2].to_string(),
        }
    })
}

fn handle_fork_command(cmd: ForkCommand) -> Result<()> {
    let mut manager = AgentManager::new(&cwd);
    let (agent_id, _) = manager.start_agent(
        cmd.role.parse()?,
        cmd.task,
        None,
        Some(current_pm_id),
    )?;
    
    // 输出提示信息
    println!("\n[系统] Agent {} 已启动", agent_id);
    println!("[系统] 接入命令: overclock agent attach {}", agent_id);
    
    Ok(())
}
```

#### 2. 修改 PM 系统提示词

确保 PM 知道如何使用 FORK 指令（已在当前实现中添加）：

```markdown
## Agent 编排能力

### 创建 Agent
输出以下格式的指令：

[FORK:{role}] {任务描述}

示例：
[FORK:architect] 基于 feature_list.md 设计系统架构
[FORK:developer] 实现 F001: 用户登录功能
```

#### 3. 实时输出流处理

需要处理 codebuddy 的输出流，有两种方案：

**方案 A：管道捕获**
```rust
let mut child = Command::new("codebuddy")
    .stdout(Stdio::piped())
    .spawn()?;

let stdout = child.stdout.take().unwrap();
// 实时读取并解析
```

**方案 B：文件监控**
```rust
// codebuddy 输出到文件
// overclock 监控文件变化并解析
let mut watcher = notify::recommended_watcher(|event| {
    // 解析新增内容
})?;
```

推荐方案 A，更简单直接。

#### 4. Attach 功能增强

当前 `attach` 功能使用 `--continue` 恢复会话，需要确保：

1. Agent 的 session-id 正确
2. 能看到 Agent 的历史输出
3. 能与 Agent 交互

可能需要修改为：

```rust
pub fn handle_agent_attach(agent_id: &str) -> Result<()> {
    // 获取 Agent 信息
    let agent = manager.get_agent(agent_id)?;
    
    // 显示 Agent 的历史输出（从日志文件）
    let log_file = format!(".ai/logs/{}.log", agent_id);
    if Path::new(&log_file).exists() {
        println!("=== Agent 历史输出 ===");
        let content = std::fs::read_to_string(&log_file)?;
        println!("{}", content);
        println!("=== 继续会话 ===\n");
    }
    
    // 启动交互式会话
    let mut child = Command::new("codebuddy")
        .arg("--session-id")
        .arg(&agent.session_id)
        .arg("--continue")
        .spawn()?;
    
    child.wait()?;
    Ok(())
}
```

### 文件修改清单

| 文件 | 修改内容 |
|------|---------|
| `src/commands/run.rs` | 添加输出捕获和 FORK 指令解析 |
| `src/agent_manager.rs` | 可能需要添加 `start_agent_async` 方法 |
| `src/commands/agent.rs` | 增强 `handle_agent_attach` 功能 |
| `template/.ai/prompts/pm_prompt.md` | 确认 FORK 指令说明完整 |

### 验收标准

#### AC-1: PM 自动创建 Architect
- **Given**: PM 运行中，用户完成需求沟通
- **When**: PM 输出 `[FORK:architect] 设计架构`
- **Then**: Architect Agent 自动启动
- **验证方式**: `programmatic`

#### AC-2: 用户收到通知
- **Given**: Architect 已启动
- **When**: 系统检测到 FORK 指令
- **Then**: 显示 Agent ID 和接入命令
- **验证方式**: `manual`

#### AC-3: Attach 显示历史
- **Given**: Architect 已工作一段时间
- **When**: 用户执行 `overclock agent attach architect-001`
- **Then**: 显示 Architect 的历史输出
- **验证方式**: `manual`

#### AC-4: Attach 可以交互
- **Given**: 用户已 attach 到 Architect
- **When**: 用户输入问题
- **Then**: Architect 回答
- **验证方式**: `manual`

### 开发优先级

1. **高优先级**: 输出捕获和 FORK 解析
2. **高优先级**: Agent 自动启动
3. **中优先级**: Attach 显示历史
4. **低优先级**: 错误处理和边界情况

### 技术难点

1. **输出流实时处理**: 需要同时显示给用户和解析指令
2. **并发控制**: PM 和 Architect 同时运行，需要正确管理进程
3. **会话隔离**: 确保 PM 和 Architect 的会话独立

### 相关文档

- [multi-agent-design.md](./multi-agent-design.md) - 多 Agent 系统设计
- [agent-fork-design.md](./agent-fork-design.md) - Agent Fork 机制设计

### 更新记录

| 日期 | 更新内容 |
|------|---------|
| 2026-03-12 | 初始需求文档 |
