//! perknow-llm（L2）— OpenAI 兼容客户端、超时/重试/降级、Prompt 渲染工具。。
//!
//! Phase-0 状态：**空壳**。只有文档注释，没有实现（AGENTS.md §5：第 0 周不写业务实现）。
//!
//! # 契约
//!
//! - `contracts/interface-contracts.md` §3.8
//! - `contracts/data-model.md` §2.9
//!
//! # 边界
//!
//! - 允许依赖：`perknow-contracts`、`perknow-core`
//! - 禁止：`tauri`、`rusqlite`、`qdrant-client`、`tantivy`、任一 L3
//! - 公共 API 只从本文件 `pub use`（BAN-09）；错误统一 `perknow_contracts::AppError`。
