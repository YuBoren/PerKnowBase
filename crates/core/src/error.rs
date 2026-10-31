//! 错误构造辅助：把底层错误**脱敏**成 `AppError`（BAN-16：不得带绝对路径/正文/密钥）。
//!
//! `std::io::Error` 的 `Display` 常包含绝对路径，因此这里只取 `ErrorKind`。

use perknow_contracts::{AppError, ErrorCode};

/// `io::Error` → `IO_ERROR`（可重试）。`context` 是固定短语，调用方不得拼接路径。
pub fn io_error(context: &str, error: &std::io::Error) -> AppError {
    AppError::new(
        ErrorCode::IoError,
        format!("{context}: {}", kind_name(error.kind())),
    )
}

/// 锁中毒（线程 panic）等内部不变量破坏 → `INTERNAL`。
pub fn internal_error(context: &str) -> AppError {
    AppError::new(ErrorCode::Internal, context.to_owned())
}

/// 尚未实现的路径（Phase-0 骨架会用到；Phase-1 起应逐项消灭）。
pub fn not_implemented(feature: &str) -> AppError {
    AppError::new(ErrorCode::Internal, format!("not implemented: {feature}"))
}

/// `io::ErrorKind` 的稳定小写名字（只用于日志/错误文本）。
pub fn kind_name(kind: std::io::ErrorKind) -> &'static str {
    match kind {
        std::io::ErrorKind::NotFound => "not_found",
        std::io::ErrorKind::PermissionDenied => "permission_denied",
        std::io::ErrorKind::AlreadyExists => "already_exists",
        std::io::ErrorKind::InvalidInput => "invalid_input",
        std::io::ErrorKind::InvalidData => "invalid_data",
        std::io::ErrorKind::TimedOut => "timed_out",
        std::io::ErrorKind::Interrupted => "interrupted",
        std::io::ErrorKind::UnexpectedEof => "unexpected_eof",
        std::io::ErrorKind::WriteZero => "write_zero",
        std::io::ErrorKind::WouldBlock => "would_block",
        // 其余 ErrorKind（多数仍为 unstable / 随版本变化）统一归为 other，避免绑定具体 Rust 版本。
        _ => "other",
    }
}
