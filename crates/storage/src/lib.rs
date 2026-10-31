//! perknow-storage（L2）— SQLite 连接与 PRAGMA、迁移执行与账本、`MetadataRepo`/`AppRepo` 端口与实现、事务边界（唯一 SQL 出口）。。
//!
//! Phase-0 状态：**空壳**。只有文档注释，没有实现（AGENTS.md §5：第 0 周不写业务实现）。
//!
//! # 契约
//!
//! - `contracts/interface-contracts.md` §3.3
//! - `contracts/data-model.md` §4
//! - `crates/storage/migrations/README.md`
//!
//! # 边界
//!
//! - 允许依赖：`perknow-contracts`、`perknow-core`
//! - 禁止：`tauri`、`qdrant-client`、`tantivy`、`ort`、任一 L3（BAN-03）
//! - 公共 API 只从本文件 `pub use`（BAN-09）；错误统一 `perknow_contracts::AppError`。
