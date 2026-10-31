//! # perknow-core（L1）
//!
//! 跨模块共用的基础能力，**不含业务语义**：
//!
//! - `ids`：UUIDv7 生成（`contracts/README.md` §4）。
//! - `time`：epoch 毫秒 ↔ RFC 3339 UTC 毫秒。
//! - `hash`：SHA-256 与 `chunk.content` 归一化（`data-model.md` §2.5）。
//! - `path`：`rel_path` 校验与越界防护（BAN-12）。
//! - `cancel`：`CancelToken`（§1.2 取消语义）。
//! - `events`：`EventSink` 抽象（BAN-05）。
//! - `error`：脱敏错误构造（BAN-16）。
//!
//! 边界：只允许依赖 `perknow-contracts`；**禁止** `tauri`/`rusqlite`/`qdrant-client`/`tantivy`/`ort`，
//! 也禁止依赖任何 L2/L3（`contracts/architecture-boundaries.md` §2）。

#![deny(unsafe_code)]
#![warn(rust_2018_idioms)]

pub mod cancel;
pub mod error;
pub mod events;
pub mod hash;
pub mod ids;
pub mod path;
pub mod time;

pub use cancel::CancelToken;
pub use error::{internal_error, io_error, kind_name, not_implemented};
pub use events::{EventSink, NoopEventSink, RecordingEventSink};
pub use hash::{content_hash, normalize_content, sha256_file, sha256_hex};
pub use ids::{
    new_chunk_id, new_correlation_id, new_file_id, new_id, new_kb_id, new_op_id, new_request_id,
};
pub use path::{
    ensure_within_root, rel_from_root, resolve_existing, resolve_for_create, to_posix,
    validate_rel_path, MAX_REL_PATH_LEN,
};
pub use time::{now_ms, now_rfc3339_ms, to_rfc3339_ms};
