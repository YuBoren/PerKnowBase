//! perknow-kb（L3）— `KnowledgeBaseService`：建库/打开/切换/关闭/删除、导入导出、融合与回滚、重构编排。。
//!
//! Phase-0 状态：**空壳**。只有文档注释，没有实现（AGENTS.md §5：第 0 周不写业务实现）。
//!
//! # 契约
//!
//! - `contracts/interface-contracts.md` §3.1/§7
//! - `contracts/data-model.md` §2.11/§2.12/§6
//!
//! # 边界
//!
//! - 允许依赖：`contracts`、`core`、`storage`、`pipeline`、`index`、`fs`、`settings`、`sync`
//! - 禁止：`tauri`、`perknow-search`（禁止编排检索）
//! - 公共 API 只从本文件 `pub use`（BAN-09）；错误统一 `perknow_contracts::AppError`。
