//! 事件发布抽象（BAN-05：L1–L3 不得 `use tauri`、不得直接 emit）。
//!
//! 只有 `apps/desktop` 实现本 trait（转成 Tauri emit）；其余层只持有 `Arc<dyn EventSink>`。
//! 事件名与载荷类型来自 `perknow_contracts::events`（§5 事件契约，只有 A 可改）。

use std::sync::Mutex;

use perknow_contracts::Event;

/// 事件出口。实现必须**非阻塞**且**不 panic**（事件失败不能影响业务）。
pub trait EventSink: Send + Sync {
    /// 发布一条事件。
    fn emit(&self, event: Event);
}

/// 空实现：单元测试与不需要事件的环境使用。
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopEventSink;

impl EventSink for NoopEventSink {
    fn emit(&self, _event: Event) {}
}

/// 记录型实现：测试断言“发了哪些事件”。
#[derive(Debug, Default)]
pub struct RecordingEventSink {
    events: Mutex<Vec<Event>>,
}

impl RecordingEventSink {
    /// 新建空记录器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 取已记录的事件快照。
    pub fn events(&self) -> Vec<Event> {
        match self.events.lock() {
            Ok(guard) => guard.clone(),
            // 锁中毒说明别处 panic；事件只是提示，取空即可，不二次 panic。
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }
}

impl EventSink for RecordingEventSink {
    fn emit(&self, event: Event) {
        if let Ok(mut guard) = self.events.lock() {
            guard.push(event);
        }
    }
}
