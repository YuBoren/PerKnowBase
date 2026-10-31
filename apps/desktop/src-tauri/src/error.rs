//! 命令层错误映射。
//!
//! 契约已经把错误载荷定义成 `AppError`（`data-model.md` §2.17），因此映射是 **1:1**：
//! shell 不再包装一层 DTO，也不把第三方错误字符串直接当 `code`（`interface-contracts.md` §1.5）。
//! 需要补的只有 `correlation_id` 与日志关联——`AppError::new` 已自动生成，
//! 命令内若已有外层 id，用 `with_correlation` 复用，避免一次操作出现两个 id。

use perknow_contracts::AppError;

/// 命令返回类型（`Result<T, AppError>`）。
pub type CommandResult<T> = std::result::Result<T, AppError>;

/// 未实现命令的占位错误（Phase-0 只有 `app_info` 已实现，其余登记为 `Pending`）。
pub fn not_implemented(command: &str) -> AppError {
    AppError::internal(format!("command not implemented yet: {command}"))
}
