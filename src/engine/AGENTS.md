# src/engine/ — 引擎目录

## 职责

llama-server 和 rpc-server 的进程管理、状态机、日志聚合。

## 文件清单

| 文件        | 核心类型                           | 职责                                      |
|-----------|--------------------------------|-----------------------------------------|
| server.rs | `ServerManager`, `ServerState` | llama-server 进程生命周期 + launch_command 捕获 |
| rpc.rs    | `RpcManager`, `RpcState`       | rpc-server 进程生命周期                       |
| mod.rs    | `LogEntry`, `LogType`          | 日志聚合、枚举定义                               |

## ServerManager 状态机

```
Idle → Starting → Running → Idle          (正常停止)
Idle → Starting → Running → Error(_)      (崩溃/非零退出)
Running → Stopping → Idle                  (停止中)
```

- `start()`: 构建 Command(含所有 --args) → spawn → stdout/stderr BufReader → 捕获 launch_command
- `stop()`: `child.kill()` + `child.wait().ok()` → 清理状态
- `poll_logs()`: try_wait() 检测进程退出 → Error(exit_code)
- `is_listening()`: 检查是否已监听到端口 (用于 WebClient 按钮启用/禁用)

## RpcManager 状态机

```
Idle → Starting → Running → Idle          (正常停止)
Running → Stopping → Idle                  (停止中)
```

- `start()`: --host/--port/--threads/--device + rpc_cache?--cache
- `poll()`: try_wait() 检测进程退出 → Error(exit_code)

## 进程管理通用模式

- `Arc<Mutex<InnerState>>` 包裹 `std::process::Child`
- stdout/stderr: 各一个 `thread::spawn`, BufReader→lines→push_back
- Windows: CREATE_NO_WINDOW (0x08000000) + windows() cfg 分支
- Drop trait 自动 stop() — App 退出时清理子进程

## Log 聚合 (mod.rs)

- `LogEntry { source: LogType, timestamp, message }`
- `LogType::Server` / `LogType::Rpc`
- 日志环形缓冲区: VecDeque<String>, 容量限制 2000 行

## 约束

- start() 前置检查: path 非空 + is_file(); 已运行则直接返回 (幂等)
- 错误消息走 i18n, 禁止硬编码中文/英文
- 日志线程在 stop() 时通过 child.take() 自然终止
