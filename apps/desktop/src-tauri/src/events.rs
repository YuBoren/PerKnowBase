//! `EventSink` 的 Tauri 实现——**全仓唯一**的 emit 点（BAN-05）。
//!
//! 事件名与载荷来自 `perknow_contracts::Event`（§5 事件契约）：
//! `Event` 保证事件名在封闭集合内，payload 已按契约结构序列化。
//! emit 失败（窗口已关闭等）**不得**升级为业务错误：事件只是通知（BAN-17）。

use perknow_contracts::Event;
use perknow_core::EventSink;
use tauri::{AppHandle, Emitter};

/// 把 `Event` 转发成 Tauri 事件。
#[derive(Clone)]
pub struct TauriEventSink {
    app: AppHandle,
}

impl TauriEventSink {
    /// 用应用句柄构造。
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl EventSink for TauriEventSink {
    fn emit(&self, event: Event) {
        // 事件名是契约固定串（如 `kb://op-done`），前端按同名监听。
        let _ = self.app.emit(event.name.as_str(), event.payload);
    }
}
