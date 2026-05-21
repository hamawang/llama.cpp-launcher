# src/ui/ — UI 面板目录

## 职责

7 个 egui 面板 + 模型文件标签解析。纯 UI 渲染，不包含业务逻辑。

## 文件清单

| 文件                 | 面板        | 核心功能                                                                                  |
|--------------------|-----------|---------------------------------------------------------------------------------------|
| launch_commands.rs | 启动命令      | 只读展示 server/RPC 最终启动命令                                                                |
| log_panel.rs       | 日志        | 实时日志流 (ServerLog + RpcLog 聚合), 清空按钮                                                   |
| model_panel.rs     | 模型        | 目录浏览、GGUF 文件列表、彩色标签解析、mmproj 切换                                                       |
| params_panel.rs    | 推理参数      | n_ctx/n_predict/temperature/top_p/top_k/repeat_penalty/kv_offload/cache_type/GPU 全部参数 |
| rpc_panel.rs       | RPC 配置    | rpc-server 路径/端口/threads/device/cache + 启动/停止 + 状态显示                                  |
| server_panel.rs    | Server 配置 | llama-server 路径/端口/槽位 + 启动/停止/重启 + 状态 + RPC 模式开关                                      |
| presets_panel.rs   | 预设管理      | 预设保存/应用/删除/自启动，返回 bool 表示是否应启动 Server                                                 |

## 渲染约定

- 每帧 `App::update()` 按 `self.state.active_panel` 索引路由到对应面板
- 面板函数签名统一：`fn ui(&mut Ui, settings: &mut AppSettings, lang: &Language)`
- `model_panel.rs` 例外：额外接收 `&mut ServerManager, &mut RpcManager`，支持 Main/Mmproj/DFlash 三种文件模式切换 (
  `FileMode`)。auto_detect_model_dir() 优先查找 "models"，其次 "model"（大小写不敏感）；DFlash 草稿文件通过 is_dflash_file()
  过滤展示
- `log_panel.rs` 额外接收 `&mut ServerManager, &mut RpcManager`
- `presets_panel.rs` 例外：返回 bool 表示是否应启动 Server。当应用预设且 auto_start_preset_name ==
  preset.name（或对应自启动开关启用）时，返回 true 触发 Server 启动

## 标签解析规则 (model_panel::parse_tags)

文件名去掉 `.gguf` 后缀后按 `-` 分割:

- 含数字且以 b/m/k 结尾 → 紫色 (参数量：7b, 32k)
- q 开头 → 橙色 (量化：q4_0, q8_0)
- 纯数字/小数点 → 灰色 (版本号：1.5)
- 含 instruct/chat/sft/rlhf/dpo/orpo/grpo → 绿色 (训练方式)
- 其他 → 蓝色 (模型名：llama3, qwen)

## 约束

- 所有文本通过 `i18n::t(Key, lang)`, 禁止硬编码
- rfd 仅用于 server/rpc 面板的 `pick_file()` 和 model 面板的 `pick_folder()`
- egui 0.29 API, 不使用 0.28/0.30 特有方法