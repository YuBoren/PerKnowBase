//! 命令注册表（A 独占，`interface-contracts.md` §4）。
//!
//! 规则：
//! - `handler()` 是**唯一**的注册处；新增命令必须先改契约表与 `table::COMMANDS`。
//! - 每个命令函数 ≤ 20 行，只做 DTO → service → `AppError` 映射（BAN-06）。
//! - 未实现的命令**不注册**：前端拿到的是“命令不存在”，而不是一个假成功的空实现。

pub mod app_info;
pub mod table;

/// 生成 Tauri 的 invoke handler（当前只注册已实现的命令）。
pub fn handler() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![app_info::app_info]
}
