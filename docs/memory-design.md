# 角色独立记忆机制设计文档

## 概述

本文档描述 overclock 平台的角色独立记忆机制设计，实现每个 AI Agent 角色拥有独立的持久化记忆，支持长期对话历史的保存和恢复。

## 背景

codebuddy CLI 工具本身支持会话记忆功能：
- 会话数据持久化存储在 `~/.codebuddy/projects/{project-path}/{session-id}.jsonl`
- 通过 `--session-id` 参数可以指定固定会话 ID
- 相同 session-id 的调用会自动关联到同一会话历史

## 设计目标

1. **角色隔离**：每个角色拥有独立的记忆空间
2. **持久化**：记忆保存到本地磁盘，重启后依然可用
3. **可识别**：用户能清楚知道当前使用的是哪个角色的记忆
4. **可扩展**：为未来记忆压缩功能预留接口

## 技术方案

### 1. 会话 ID 生成策略

为每个角色分配固定的 session-id，格式：

```
overclock-{project_name}-{role}
```

示例：
- PM 角色：`overclock-my_project-pm`
- Architect 角色：`overclock-my_project-architect`
- Developer 角色：`overclock-my_project-developer`
- Tester 角色：`overclock-my_project-tester`
- Reviewer 角色：`overclock-my_project-reviewer`

### 2. 存储位置

codebuddy 自动管理存储路径：

```
~/.codebuddy/projects/{encoded-project-path}/
└── {session-id}.jsonl          # 完整对话历史
```

示例：
```
~/.codebuddy/projects/Users-tmacy-Documents-my_project/
├── overclock-my_project-pm.jsonl
├── overclock-my_project-architect.jsonl
├── overclock-my_project-developer.jsonl
└── ...
```

### 3. 会话文件格式

JSON Lines 格式，每行一个 JSON 对象：

```json
{"type":"message","role":"user","content":"...","timestamp":1234567890}
{"type":"reasoning","content":"...","timestamp":1234567891}
{"type":"message","role":"assistant","content":"...","timestamp":1234567892}
```

包含：
- 用户消息
- AI 回复
- 思考过程 (reasoning)
- 文件历史快照
- 元数据（时间戳、模型、token 用量等）

### 4. 实现方案

修改 `src/commands/run.rs`：

```rust
fn run_codebuddy(config: &Config, role: &Role, project_path: &Path) -> Result<()> {
    // ... 现有代码 ...
    
    // 生成角色固定的 session-id
    let session_id = format!("overclock-{}-{}", 
        config.project.name, 
        role.as_str()
    );
    
    println!("{}", format!("会话 ID: {}", session_id).dimmed());
    
    let mut child = Command::new(&codebuddy_path)
        .current_dir(project_path)
        .arg("--system-prompt-file").arg(&temp_prompt_path)
        .arg("--session-id").arg(&session_id)  // 固定会话ID
        .env("OVERCLOCK_ROLE", role.as_str())
        // ... 其他参数 ...
        .spawn()
        .context("无法启动 codebuddy")?;
    
    // ... 后续代码 ...
}
```

### 5. 用户交互流程

```
用户执行: overclock run pm

输出:
启动 Project Manager Agent...

角色: Project Manager
职责: 负责需求分析、任务分解和进度跟踪...
会话 ID: overclock-my_project-pm
提示词: .ai/prompts/pm_prompt.md

正在启动 codebuddy...
[进入交互式会话，包含历史记忆]
```

## 未来扩展：记忆压缩

### 问题
长期对话会导致：
- 会话文件过大
- token 用量增加
- 响应速度变慢

### 解决方案（预留）

#### 方案 A：摘要压缩
定期将历史对话生成摘要，替换原始对话：

```rust
// 伪代码
if conversation_length > threshold {
    let summary = generate_summary(&history);
    replace_history_with_summary(&session_file, summary);
}
```

#### 方案 B：分层存储
- 近期对话：完整保留（如最近 10 轮）
- 中期对话：保留关键决策和文件修改记录
- 早期对话：仅保留摘要

#### 方案 C：智能选择
codebuddy 本身支持上下文窗口管理，可以依赖其内部机制。

### 实现时机
当实际使用中遇到 token 限制或性能问题时再实现。

## 验收标准

### AC-1: 角色独立记忆
- **Given**: 项目已初始化，名为 "my_project"
- **When**: 执行 `overclock run pm` 两次
- **Then**: 第二次启动能看到第一次的对话历史
- **验证方式**: `manual`

### AC-2: 角色隔离
- **Given**: 已启动过 PM 和 Architect 角色
- **When**: 查看 `~/.codebuddy/projects/` 目录
- **Then**: 存在两个独立的会话文件
- **验证方式**: `programmatic`

### AC-3: 持久化
- **Given**: 已完成一次 PM 对话
- **When**: 重启电脑后再次执行 `overclock run pm`
- **Then**: 能看到之前的对话历史
- **验证方式**: `manual`

## 相关文件

- `src/commands/run.rs` - 主要实现文件
- `src/config.rs` - 配置定义（包含 project.name）
- `src/role.rs` - 角色定义

## 变更记录

| 版本 | 日期 | 变更内容 | 作者 |
|------|------|----------|------|
| 1.0 | 2026-03-12 | 初始设计文档 | overclock |
