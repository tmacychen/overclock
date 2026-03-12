# 多 Agent 协作系统设计文档 v2.0

## 概述

实现 PM 作为唯一入口的多 Agent 协作开发平台，支持：
- PM 内部创建和管理其他 Agent
- 多个同角色 Agent 并行工作
- 用户通过 ID 接入任意 Agent
- 保留 codebuddy CLI 指令
- 任务完成自动退出

## 架构

```
用户 ←→ PM Agent (唯一入口)
          │
          ├── arch-001 (后台)
          ├── dev-001 (后台)
          ├── dev-002 (后台)
          └── test-001 (后台)
```

## Agent ID 系统

### ID 格式
```
{role}-{序号}
例如: pm-001, arch-001, dev-001, dev-002, test-001
```

### ID 分配规则
- 每个角色独立计数
- 重启后从 001 重新开始
- ID 存储在 `.ai/agent_counter.json`

## CLI 命令

### 用户命令

```bash
# 初始化项目
overclock init my_project

# 启动 PM（唯一入口）
overclock run

# 查看 Agent 状态
overclock agent show

# 接入 Agent
overclock agent attach dev-002
```

### 内部命令（隐藏）

```bash
# PM 调用，启动后台 Agent
overclock start-agent developer \
  --task "实现 F001" \
  --model glm-4.7 \
  --parent pm-001
```

## 文件结构

```
.ai/
├── agent_status.json      # 运行中的 Agent 状态
├── agent_counter.json     # ID 计数器
├── pipes/
│   ├── dev-001_in         # 输入管道
│   ├── dev-001_out        # 输出管道
│   └── ...
├── logs/
│   ├── dev-001.log        # Agent 日志
│   └── ...
├── completed/
│   ├── dev-001.md         # 完成报告
│   └── ...
├── tasks/
│   ├── dev-001.md         # 任务描述
│   └── ...
└── ...
```

## 数据结构

### agent_status.json

```json
{
  "agents": [
    {
      "id": "pm-001",
      "role": "pm",
      "pid": 12345,
      "model": "glm-5.0",
      "status": "running",
      "task": "主会话",
      "started_at": "2026-03-12T18:00:00Z",
      "parent": null
    },
    {
      "id": "dev-001",
      "role": "developer",
      "pid": 12346,
      "model": "glm-4.7",
      "status": "running",
      "task": "实现 F001",
      "started_at": "2026-03-12T18:05:00Z",
      "parent": "pm-001"
    }
  ]
}
```

### agent_counter.json

```json
{
  "pm": 1,
  "architect": 1,
  "developer": 2,
  "tester": 0,
  "reviewer": 0
}
```

## 工作流程

### 1. PM 创建 Agent

```
PM 输出: [FORK:developer] 实现 F001

overclock 捕获指令:
  1. 解析角色: developer
  2. 解析任务: "实现 F001"
  3. 分配 ID: dev-002 (developer 计数器 +1)
  4. 创建管道: .ai/pipes/dev-002_in, dev-002_out
  5. 启动后台进程: overclock start-agent developer --task "实现 F001"
  6. 更新 agent_status.json
  7. 返回 ID 给 PM: "dev-002 已启动"
```

### 2. 用户接入 Agent

```
$ overclock agent show
运行中的 Agent:
  ID        Role        Status          Model
  ─────────────────────────────────────────────
  pm-001    pm          运行中 (主会话)  glm-5.0
  dev-001   developer   开发中           glm-4.7
  dev-002   developer   开发中           glm-4.7

$ overclock agent attach dev-002

[接入 dev-002 - Developer]
Developer: 正在实现 F001...
已完成: 数据库模型设计

> /model glm-5.0        # 切换模型（codebuddy 指令）
已切换到 glm-5.0

> 继续实现 API 接口
Developer: 好的，继续...
```

### 3. Agent 完成任务

```
Agent 完成工作后:
  1. 更新 agent_status.json: status = "completed"
  2. 写入完成报告: .ai/completed/dev-001.md
  3. 退出进程
  4. PM 检测到完成，读取报告
  5. PM 向用户汇报
```

## PM 提示词扩展

```markdown
## Agent 编排能力

你可以创建和管理其他 Agent 来完成工作。

### 创建 Agent
输出以下格式的指令：

[FORK:{role}] {任务描述}

示例：
[FORK:architect] 基于 feature_list.md 设计系统架构
[FORK:developer] 实现 F001: 用户登录功能
[FORK:tester] 测试 F001 的登录功能

### 可用角色
- architect: 架构师，负责技术设计
- developer: 开发者，负责功能实现（可创建多个）
- tester: 测试员，负责测试验证
- reviewer: 审查员，负责代码审查

### 监控进度
读取 .ai/progress.md 查看所有 Agent 的工作进度

### 接收完成通知
定期检查 .ai/completed/ 目录，读取 Agent 的完成报告

### 汇报结果
当 Agent 完成后，读取其输出文件，向用户汇报结果
```

## 实现模块

### 1. AgentManager

```rust
pub struct AgentManager {
    status_file: PathBuf,
    counter_file: PathBuf,
    pipes_dir: PathBuf,
    logs_dir: PathBuf,
}

impl AgentManager {
    pub fn new(project_path: &Path) -> Self;
    
    // 分配唯一 ID
    pub fn allocate_id(&mut self, role: &Role) -> String;
    
    // 启动后台 Agent
    pub fn start_agent(&mut self, role: Role, task: String, model: Option<String>) -> Result<String>;
    
    // 获取运行中的 Agent
    pub fn list_running(&self) -> Vec<AgentInfo>;
    
    // 接入 Agent
    pub fn attach(&self, agent_id: &str) -> Result<()>;
    
    // 标记完成
    pub fn mark_completed(&mut self, agent_id: &str, report: String) -> Result<()>;
}
```

### 2. 命名管道通信

```rust
pub struct AgentPipe {
    input: File,   // .ai/pipes/{id}_in
    output: File,  // .ai/pipes/{id}_out
}

impl AgentPipe {
    pub fn create(id: &str) -> Result<Self>;
    pub fn send(&mut self, message: &str) -> Result<()>;
    pub fn receive(&mut self) -> Result<String>;
}
```

### 3. 自动退出检测

```rust
// 在 Agent 提示词中添加
"完成任务后，输出以下指令退出：
[COMPLETE]
完成报告：
- 实现了什么
- 修改了哪些文件
- 测试结果
[END]"

// overclock 捕获 [COMPLETE] 指令，执行退出流程
```

## 验收标准

### AC-1: 多个同角色 Agent
- **Given**: PM 创建了两个 developer
- **When**: 运行 `overclock agent show`
- **Then**: 显示 dev-001 和 dev-002 两个 Agent
- **验证方式**: `programmatic`

### AC-2: 通过 ID 接入
- **Given**: dev-001 和 dev-002 都在运行
- **When**: 运行 `overclock agent attach dev-002`
- **Then**: 成功接入 dev-002，可以对话
- **验证方式**: `manual`

### AC-3: 保留 CLI 指令
- **Given**: 已接入 dev-001
- **When**: 输入 `/model glm-5.0`
- **Then**: 模型切换成功
- **验证方式**: `manual`

### AC-4: 自动退出
- **Given**: Agent 完成任务
- **When**: Agent 输出 [COMPLETE] 指令
- **Then**: Agent 自动退出，状态更新为 completed
- **验证方式**: `programmatic`

## 变更记录

| 版本 | 日期 | 变更内容 | 作者 |
|------|------|----------|------|
| 1.0 | 2026-03-12 | 初始设计 | overclock |
| 2.0 | 2026-03-12 | 添加 ID 系统、attach、自动退出 | overclock |
