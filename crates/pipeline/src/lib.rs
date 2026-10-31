//! perknow-pipeline（L2）— `DocumentParser`（native + sidecar 客户端）、`Chunker`（纯函数）、`Embedder`（ONNX）、解析缓存读写。。
//!
//! Phase-0 状态：**空壳**。只有文档注释，没有实现（AGENTS.md §5：第 0 周不写业务实现）。
//!
//! # 契约
//!
//! - `contracts/interface-contracts.md` §3.5
//! - §6
//! - `contracts/data-model.md` §2.4/§2.5/§7
//!
//! # 边界
//!
//! - 允许依赖：`perknow-contracts`、`perknow-core`、`perknow-storage`（仅端口）
//! - 禁止：`tauri`、`qdrant-client`、`tantivy`、任一 L3；不得调用 `perknow-index`（BAN-13）
//! - 公共 API 只从本文件 `pub use`（BAN-09）；错误统一 `perknow_contracts::AppError`。
