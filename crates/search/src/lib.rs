//! perknow-search（L3）— `SearchService` 三模式编排、BM25+向量+RRF、`PromptBuilder`、引用校验。。
//!
//! Phase-0 状态：**空壳**。只有文档注释，没有实现（AGENTS.md §5：第 0 周不写业务实现）。
//!
//! # 契约
//!
//! - `contracts/interface-contracts.md` §3.8/§7.3
//! - `contracts/data-model.md` §2.7/§2.8
//!
//! # 边界
//!
//! - 允许依赖：`contracts`、`core`、`storage`、`index`、`llm`
//! - 禁止：`tauri`、`pipeline`（禁止解析）、`kb`、`fs`、`sync`
//! - 公共 API 只从本文件 `pub use`（BAN-09）；错误统一 `perknow_contracts::AppError`。
