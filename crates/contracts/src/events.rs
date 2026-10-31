//! 事件名称与载荷（`contracts/interface-contracts.md` §5）。
//!
//! 规则（§5）：事件是通知不是数据源；载荷不得含正文/密钥/绝对路径；只有 Rust 侧发出；
//! 订阅方必须容忍未知字段与未知 `phase` 字符串。发布者只能发布自己域的事件。

use serde::{Deserialize, Serialize};

use crate::error::{AppError, ErrorCode};
use crate::ids::{KbId, OpId};
use crate::ops::{OpKind, OpStatus, RebuildPlan};

/// 事件名（固定集合，`域://事件`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventName {
    #[serde(rename = "sync://progress")]
    SyncProgress,
    #[serde(rename = "index://status")]
    IndexStatus,
    #[serde(rename = "kb://changed")]
    KbChanged,
    #[serde(rename = "kb://op-progress")]
    KbOpProgress,
    #[serde(rename = "kb://op-done")]
    KbOpDone,
    #[serde(rename = "kb://rebuild-required")]
    KbRebuildRequired,
    #[serde(rename = "error://toast")]
    ErrorToast,
}

impl EventName {
    /// 全部事件名（顺序与契约表一致）。
    pub const ALL: [EventName; 7] = [
        EventName::SyncProgress,
        EventName::IndexStatus,
        EventName::KbChanged,
        EventName::KbOpProgress,
        EventName::KbOpDone,
        EventName::KbRebuildRequired,
        EventName::ErrorToast,
    ];

    /// 线上字符串值。
    pub const fn as_str(self) -> &'static str {
        match self {
            EventName::SyncProgress => "sync://progress",
            EventName::IndexStatus => "index://status",
            EventName::KbChanged => "kb://changed",
            EventName::KbOpProgress => "kb://op-progress",
            EventName::KbOpDone => "kb://op-done",
            EventName::KbRebuildRequired => "kb://rebuild-required",
            EventName::ErrorToast => "error://toast",
        }
    }
}

/// `kb://changed` 的 `what`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum KbChangeKind {
    /// 元数据变化。
    Meta,
    /// 设置变化。
    Settings,
    /// 文件树变化。
    Files,
    /// 索引状态变化。
    Index,
}

/// `index://status` 的 `state`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum IndexState {
    /// 空闲。
    Idle,
    /// 索引中。
    Indexing,
    /// 索引陈旧。
    Stale,
}

/// `sync://progress` 载荷。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct SyncProgressEvent {
    /// 知识库 id。
    pub kb_id: KbId,
    /// 任务 id。
    pub task_id: String,
    /// 阶段名（订阅方必须容忍未知值）。
    pub phase: String,
    /// 已完成。
    pub done: u32,
    /// 总数（0 表示未知）。
    pub total: u32,
    /// 当前文件相对路径（可选）。
    pub current_path: Option<String>,
}

/// `index://status` 载荷。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct IndexStatusEvent {
    /// 知识库 id。
    pub kb_id: KbId,
    /// 已索引数。
    pub indexed: u64,
    /// 总数。
    pub total: u64,
    /// 失败数。
    pub failed: u32,
    /// 状态。
    pub state: IndexState,
    /// 最近错误摘要（脱敏）。
    pub last_error: Option<String>,
}

/// `kb://changed` 载荷。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct KbChangedEvent {
    /// 知识库 id。
    pub kb_id: KbId,
    /// 变更类型。
    pub what: KbChangeKind,
}

/// `kb://op-progress` 载荷。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct KbOpProgressEvent {
    /// 操作 id。
    pub op_id: OpId,
    /// 知识库 id。
    pub kb_id: KbId,
    /// 种类。
    pub kind: OpKind,
    /// 阶段名。
    pub phase: String,
    /// 已完成。
    pub done: u32,
    /// 总数（0 表示未知）。
    pub total: u32,
    /// 附言（禁止正文）。
    pub message: Option<String>,
}

/// `kb://op-done` 载荷。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct KbOpDoneEvent {
    /// 操作 id。
    pub op_id: OpId,
    /// 知识库 id。
    pub kb_id: KbId,
    /// 种类。
    pub kind: OpKind,
    /// 终态。
    pub status: OpStatus,
    /// 结果摘要（不含正文）。
    pub result: Option<serde_json::Value>,
    /// 失败详情。
    pub error: Option<AppError>,
}

/// `kb://rebuild-required` 载荷。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct KbRebuildRequiredEvent {
    /// 知识库 id。
    pub kb_id: KbId,
    /// 重构计划（用户确认后调 `kb_rebuild`）。
    pub plan: RebuildPlan,
}

/// `error://toast` 载荷。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ErrorToastEvent {
    /// 错误码。
    pub code: ErrorCode,
    /// 可读文本（已脱敏）。
    pub message: String,
    /// 是否可重试。
    pub retryable: bool,
    /// 关联 id。
    pub correlation_id: String,
    /// 附加上下文。
    pub context: Option<serde_json::Value>,
}

/// 一条待发事件：事件名 + JSON 载荷。
///
/// `perknow_core::EventSink` 只接受本类型，从而保证事件名集合封闭、发布点单一（BAN-05）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Event {
    /// 事件名。
    pub name: EventName,
    /// 载荷（各事件结构见本模块）。
    pub payload: serde_json::Value,
}

impl Event {
    /// 用任意可序列化载荷构造事件；序列化失败时退化为 `null`（不 panic，BAN-16）。
    pub fn new<T: Serialize>(name: EventName, payload: &T) -> Self {
        Self {
            name,
            payload: serde_json::to_value(payload).unwrap_or(serde_json::Value::Null),
        }
    }

    /// 线上事件名。
    pub fn name_str(&self) -> &'static str {
        self.name.as_str()
    }

    /// `sync://progress`。
    pub fn sync_progress(payload: &SyncProgressEvent) -> Self {
        Self::new(EventName::SyncProgress, payload)
    }

    /// `index://status`。
    pub fn index_status(payload: &IndexStatusEvent) -> Self {
        Self::new(EventName::IndexStatus, payload)
    }

    /// `kb://changed`。
    pub fn kb_changed(payload: &KbChangedEvent) -> Self {
        Self::new(EventName::KbChanged, payload)
    }

    /// `kb://op-progress`。
    pub fn kb_op_progress(payload: &KbOpProgressEvent) -> Self {
        Self::new(EventName::KbOpProgress, payload)
    }

    /// `kb://op-done`。
    pub fn kb_op_done(payload: &KbOpDoneEvent) -> Self {
        Self::new(EventName::KbOpDone, payload)
    }

    /// `kb://rebuild-required`。
    pub fn kb_rebuild_required(payload: &KbRebuildRequiredEvent) -> Self {
        Self::new(EventName::KbRebuildRequired, payload)
    }

    /// `error://toast`。
    pub fn error_toast(payload: &ErrorToastEvent) -> Self {
        Self::new(EventName::ErrorToast, payload)
    }
}

/// `error://toast` 的便捷构造：由 `AppError` 直接生成，避免各处手拼字段。
pub fn error_toast_from(err: &AppError, context: Option<serde_json::Value>) -> Event {
    Event::error_toast(&ErrorToastEvent {
        code: err.code,
        message: err.message.clone(),
        retryable: err.retryable,
        correlation_id: err.correlation_id.clone(),
        context,
    })
}
