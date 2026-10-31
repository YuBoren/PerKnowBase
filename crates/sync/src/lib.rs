//! perknow-sync（L3）— `SyncService`：`notify` 监听、去抖 500ms、`SyncRouter` 纯路由、增量任务执行与幂等。。
//!
//! Phase-0 状态：**空壳**。只有文档注释，没有实现（AGENTS.md §5：第 0 周不写业务实现）。
//!
//! # 契约
//!
//! - `contracts/interface-contracts.md` §3.6/§7.2
//! - `contracts/data-model.md` §5.4
//!
//! # 边界
//!
//! - 允许依赖：`contracts`、`core`、`storage`、`pipeline`、`index`、`fs`
//! - 禁止：`tauri`、`kb`、`search`、`llm`、`history`
//! - 公共 API 只从本文件 `pub use`（BAN-09）；错误统一 `perknow_contracts::AppError`。
