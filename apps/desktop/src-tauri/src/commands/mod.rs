//! 命令注册表（A 独占，`interface-contracts.md` §4）。
//!
//! 规则：
//! - 命令清单以 `table::COMMANDS` 为准（39 条；`scripts/validate-contracts.ps1` 会与契约表比对）。
//! - 实际注册点在 `lib.rs` 的 `.invoke_handler(tauri::generate_handler![...])`：新增命令必须先改
//!   契约表与 `table.rs`，再在 `lib.rs` 增一行。
//! - 每个命令函数 ≤ 20 行，只做 DTO → service → `AppError` 映射（BAN-06）。
//! - 未实现的命令**不注册**：前端拿到的是“命令不存在”，而不是一个假成功的空实现。

pub mod app_info;
pub mod table;
