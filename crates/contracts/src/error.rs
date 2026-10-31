//! 统一错误契约（`contracts/interface-contracts.md` §2、`contracts/data-model.md` §2.17）。
//!
//! 规则：
//! - 30 个 `code` 是**封闭集合**；新增必须走契约（MINOR）+ `crates/contracts` 同步。
//! - `message` 是开发者可读文本，**不得**含正文/密钥/绝对路径（BAN-16）。
//! - `retryable` 与 `user_message_key` 由 `code` 推导，调用方不得自行覆盖。

use serde::{Deserialize, Serialize};

use crate::ids::CorrelationId;

/// 错误码（线上取值即契约表左列，大写蛇形）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCode {
    #[serde(rename = "KB_NOT_FOUND")]
    KbNotFound,
    #[serde(rename = "KB_ALREADY_OPEN")]
    KbAlreadyOpen,
    #[serde(rename = "KB_BUSY")]
    KbBusy,
    #[serde(rename = "KB_CORRUPTED")]
    KbCorrupted,
    #[serde(rename = "KB_SCHEMA_TOO_NEW")]
    KbSchemaTooNew,
    #[serde(rename = "PATH_NOT_FOUND")]
    PathNotFound,
    #[serde(rename = "PATH_OUTSIDE_KB")]
    PathOutsideKb,
    #[serde(rename = "PATH_CONFLICT")]
    PathConflict,
    #[serde(rename = "FILE_LOCKED")]
    FileLocked,
    #[serde(rename = "PARSE_FAILED")]
    ParseFailed,
    #[serde(rename = "PARSE_UNSUPPORTED")]
    ParseUnsupported,
    #[serde(rename = "PARSER_UNAVAILABLE")]
    ParserUnavailable,
    #[serde(rename = "EMBED_FAILED")]
    EmbedFailed,
    #[serde(rename = "MODEL_NOT_READY")]
    ModelNotReady,
    #[serde(rename = "INDEX_UNAVAILABLE")]
    IndexUnavailable,
    #[serde(rename = "INDEX_VERSION_MISMATCH")]
    IndexVersionMismatch,
    #[serde(rename = "SEARCH_INVALID_REQUEST")]
    SearchInvalidRequest,
    #[serde(rename = "SEARCH_CANCELLED")]
    SearchCancelled,
    #[serde(rename = "LLM_NOT_CONFIGURED")]
    LlmNotConfigured,
    #[serde(rename = "LLM_AUTH_FAILED")]
    LlmAuthFailed,
    #[serde(rename = "LLM_RATE_LIMITED")]
    LlmRateLimited,
    #[serde(rename = "LLM_TIMEOUT")]
    LlmTimeout,
    #[serde(rename = "LLM_RESPONSE_INVALID")]
    LlmResponseInvalid,
    #[serde(rename = "SETTINGS_INVALID")]
    SettingsInvalid,
    #[serde(rename = "EXPORT_INVALID")]
    ExportInvalid,
    #[serde(rename = "EXPORT_VERSION_UNSUPPORTED")]
    ExportVersionUnsupported,
    #[serde(rename = "SYNC_IN_PROGRESS")]
    SyncInProgress,
    #[serde(rename = "CANCELLED")]
    Cancelled,
    #[serde(rename = "IO_ERROR")]
    IoError,
    #[serde(rename = "INTERNAL")]
    Internal,
}

impl ErrorCode {
    /// 全部错误码（顺序与契约表一致），供校验脚本与测试遍历。
    pub const ALL: [ErrorCode; 30] = [
        ErrorCode::KbNotFound,
        ErrorCode::KbAlreadyOpen,
        ErrorCode::KbBusy,
        ErrorCode::KbCorrupted,
        ErrorCode::KbSchemaTooNew,
        ErrorCode::PathNotFound,
        ErrorCode::PathOutsideKb,
        ErrorCode::PathConflict,
        ErrorCode::FileLocked,
        ErrorCode::ParseFailed,
        ErrorCode::ParseUnsupported,
        ErrorCode::ParserUnavailable,
        ErrorCode::EmbedFailed,
        ErrorCode::ModelNotReady,
        ErrorCode::IndexUnavailable,
        ErrorCode::IndexVersionMismatch,
        ErrorCode::SearchInvalidRequest,
        ErrorCode::SearchCancelled,
        ErrorCode::LlmNotConfigured,
        ErrorCode::LlmAuthFailed,
        ErrorCode::LlmRateLimited,
        ErrorCode::LlmTimeout,
        ErrorCode::LlmResponseInvalid,
        ErrorCode::SettingsInvalid,
        ErrorCode::ExportInvalid,
        ErrorCode::ExportVersionUnsupported,
        ErrorCode::SyncInProgress,
        ErrorCode::Cancelled,
        ErrorCode::IoError,
        ErrorCode::Internal,
    ];

    /// 线上字符串值（与 `interface-contracts.md` §2 表左列逐字一致）。
    pub const fn as_str(self) -> &'static str {
        match self {
            ErrorCode::KbNotFound => "KB_NOT_FOUND",
            ErrorCode::KbAlreadyOpen => "KB_ALREADY_OPEN",
            ErrorCode::KbBusy => "KB_BUSY",
            ErrorCode::KbCorrupted => "KB_CORRUPTED",
            ErrorCode::KbSchemaTooNew => "KB_SCHEMA_TOO_NEW",
            ErrorCode::PathNotFound => "PATH_NOT_FOUND",
            ErrorCode::PathOutsideKb => "PATH_OUTSIDE_KB",
            ErrorCode::PathConflict => "PATH_CONFLICT",
            ErrorCode::FileLocked => "FILE_LOCKED",
            ErrorCode::ParseFailed => "PARSE_FAILED",
            ErrorCode::ParseUnsupported => "PARSE_UNSUPPORTED",
            ErrorCode::ParserUnavailable => "PARSER_UNAVAILABLE",
            ErrorCode::EmbedFailed => "EMBED_FAILED",
            ErrorCode::ModelNotReady => "MODEL_NOT_READY",
            ErrorCode::IndexUnavailable => "INDEX_UNAVAILABLE",
            ErrorCode::IndexVersionMismatch => "INDEX_VERSION_MISMATCH",
            ErrorCode::SearchInvalidRequest => "SEARCH_INVALID_REQUEST",
            ErrorCode::SearchCancelled => "SEARCH_CANCELLED",
            ErrorCode::LlmNotConfigured => "LLM_NOT_CONFIGURED",
            ErrorCode::LlmAuthFailed => "LLM_AUTH_FAILED",
            ErrorCode::LlmRateLimited => "LLM_RATE_LIMITED",
            ErrorCode::LlmTimeout => "LLM_TIMEOUT",
            ErrorCode::LlmResponseInvalid => "LLM_RESPONSE_INVALID",
            ErrorCode::SettingsInvalid => "SETTINGS_INVALID",
            ErrorCode::ExportInvalid => "EXPORT_INVALID",
            ErrorCode::ExportVersionUnsupported => "EXPORT_VERSION_UNSUPPORTED",
            ErrorCode::SyncInProgress => "SYNC_IN_PROGRESS",
            ErrorCode::Cancelled => "CANCELLED",
            ErrorCode::IoError => "IO_ERROR",
            ErrorCode::Internal => "INTERNAL",
        }
    }

    /// 是否可重试（前端据此显示“重试”），逐条对齐契约表。
    pub const fn retryable(self) -> bool {
        matches!(
            self,
            ErrorCode::KbBusy
                | ErrorCode::FileLocked
                | ErrorCode::ParseFailed
                | ErrorCode::ParserUnavailable
                | ErrorCode::EmbedFailed
                | ErrorCode::IndexUnavailable
                | ErrorCode::LlmRateLimited
                | ErrorCode::LlmTimeout
                | ErrorCode::LlmResponseInvalid
                | ErrorCode::SyncInProgress
                | ErrorCode::IoError
        )
    }

    /// i18n key（前端优先展示），逐条对齐契约表。
    pub const fn user_message_key(self) -> &'static str {
        match self {
            ErrorCode::KbNotFound => "error.kb.not_found",
            ErrorCode::KbAlreadyOpen => "error.kb.already_open",
            ErrorCode::KbBusy => "error.kb.busy",
            ErrorCode::KbCorrupted => "error.kb.corrupted",
            ErrorCode::KbSchemaTooNew => "error.kb.schema_too_new",
            ErrorCode::PathNotFound => "error.fs.path_not_found",
            ErrorCode::PathOutsideKb => "error.fs.path_outside",
            ErrorCode::PathConflict => "error.fs.conflict",
            ErrorCode::FileLocked => "error.fs.locked",
            ErrorCode::ParseFailed => "error.parse.failed",
            ErrorCode::ParseUnsupported => "error.parse.unsupported",
            ErrorCode::ParserUnavailable => "error.parse.unavailable",
            ErrorCode::EmbedFailed => "error.embed.failed",
            ErrorCode::ModelNotReady => "error.embed.model_missing",
            ErrorCode::IndexUnavailable => "error.index.unavailable",
            ErrorCode::IndexVersionMismatch => "error.index.rebuild_required",
            ErrorCode::SearchInvalidRequest => "error.search.invalid",
            ErrorCode::SearchCancelled => "error.search.cancelled",
            ErrorCode::LlmNotConfigured => "error.llm.not_configured",
            ErrorCode::LlmAuthFailed => "error.llm.auth",
            ErrorCode::LlmRateLimited => "error.llm.rate_limited",
            ErrorCode::LlmTimeout => "error.llm.timeout",
            ErrorCode::LlmResponseInvalid => "error.llm.invalid",
            ErrorCode::SettingsInvalid => "error.settings.invalid",
            ErrorCode::ExportInvalid => "error.export.invalid",
            ErrorCode::ExportVersionUnsupported => "error.export.version",
            ErrorCode::SyncInProgress => "error.sync.in_progress",
            ErrorCode::Cancelled => "error.cancelled",
            ErrorCode::IoError => "error.io",
            ErrorCode::Internal => "error.internal",
        }
    }
}

/// 统一错误载荷：命令返回 `Result<T, AppError>`，事件 `error://toast` 复用同一结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct AppError {
    /// 错误码。
    pub code: ErrorCode,
    /// 开发者可读文本；禁止包含正文、密钥、绝对路径。
    pub message: String,
    /// 结构化上下文（`kb_id`/`rel_path`/`op_id` 等）；无内容时为 `null`（字段本身必填）。
    pub details: Option<serde_json::Value>,
    /// 是否可重试，由 `code` 推导。
    pub retryable: bool,
    /// 与日志关联的 id（UUIDv7）。
    pub correlation_id: CorrelationId,
    /// i18n key，由 `code` 推导。
    pub user_message_key: String,
}

impl AppError {
    /// 按错误码构造；`retryable`/`user_message_key` 自动填充，`correlation_id` 新建。
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
            retryable: code.retryable(),
            correlation_id: new_correlation_id(),
            user_message_key: code.user_message_key().to_owned(),
        }
    }

    /// 附带结构化上下文。
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    /// 复用外层（命令/任务）已生成的关联 id，避免一次操作出现两个 id。
    pub fn with_correlation(mut self, correlation_id: impl Into<CorrelationId>) -> Self {
        self.correlation_id = correlation_id.into();
        self
    }

    /// 未分类缺陷（`INTERNAL`）。仅在确实无法归类时使用。
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, message)
    }

    /// 调用方主动取消。
    pub fn cancelled() -> Self {
        Self::new(ErrorCode::Cancelled, "operation cancelled by caller")
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for AppError {}

/// 生成关联 id（UUIDv7，小写带连字符）。
fn new_correlation_id() -> CorrelationId {
    uuid::Uuid::now_v7().to_string()
}

/// 通用 `Result` 别名（`interface-contracts.md` §3 的 `Result<T>`）。
pub type AppResult<T> = std::result::Result<T, AppError>;
