//! perknow-fs（L3）— `FileSystemService`：知识依据浏览/增删/移动/打开、路径安全、发 `SyncEvent{source:active}`。。
//!
//! Phase-0 状态：**空壳**。只有文档注释，没有实现（AGENTS.md §5：第 0 周不写业务实现）。
//!
//! # 契约
//!
//! - `contracts/interface-contracts.md` §3.4
//! - `contracts/data-model.md` §1.2/§2.2
//!
//! # 边界
//!
//! - 允许依赖：`contracts`、`core`、`storage`
//! - 禁止：`tauri`、`pipeline`、`index`、`sync`、`search`、`kb`
//! - 公共 API 只从本文件 `pub use`（BAN-09）；错误统一 `perknow_contracts::AppError`。
