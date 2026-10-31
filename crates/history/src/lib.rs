//! perknow-history（L3）— `HistoryService`：历史写入/查询/删除/裁剪、软引用轻量快照。。
//!
//! Phase-0 状态：**空壳**。只有文档注释，没有实现（AGENTS.md §5：第 0 周不写业务实现）。
//!
//! # 契约
//!
//! - `contracts/interface-contracts.md` §3.8
//! - `contracts/data-model.md` §2.13
//!
//! # 边界
//!
//! - 允许依赖：`contracts`、`core`、`storage`
//! - 禁止：`tauri`、`index`、`llm`、`search`、`kb`
//! - 公共 API 只从本文件 `pub use`（BAN-09）；错误统一 `perknow_contracts::AppError`。
