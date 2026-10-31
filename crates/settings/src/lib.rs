//! perknow-settings（L3）— `SettingsService`：应用级/库级设置、密钥加密存储、导入导出、`requires_rebuild` 判定。。
//!
//! Phase-0 状态：**空壳**。只有文档注释，没有实现（AGENTS.md §5：第 0 周不写业务实现）。
//!
//! # 契约
//!
//! - `contracts/interface-contracts.md` §3.2
//! - `contracts/data-model.md` §2.14/§5.4
//!
//! # 边界
//!
//! - 允许依赖：`contracts`、`core`、`storage`
//! - 禁止：`tauri`、`index`、`pipeline`
//! - 公共 API 只从本文件 `pub use`（BAN-09）；错误统一 `perknow_contracts::AppError`。
