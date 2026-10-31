//! perknow-index（L2）— Qdrant/Tantivy 适配、Qdrant 进程生命周期与端口回退、快照、BM25 分词（唯一索引出口）。。
//!
//! Phase-0 状态：**空壳**。只有文档注释，没有实现（AGENTS.md §5：第 0 周不写业务实现）。
//!
//! # 契约
//!
//! - `contracts/data-model.md` §3
//! - `contracts/interface-contracts.md` §3.7
//!
//! # 边界
//!
//! - 允许依赖：`perknow-contracts`、`perknow-core`
//! - 禁止：`tauri`、`rusqlite`、`perknow-storage`、`perknow-pipeline`（BAN-04/BAN-13）
//! - 公共 API 只从本文件 `pub use`（BAN-09）；错误统一 `perknow_contracts::AppError`。
