//! `AppState`：打开中的知识库与会话状态。
//!
//! Phase-0 只保留“当前打开的库”；Phase-1 起加入服务句柄（`KnowledgeBaseService` 等）与在途操作表。
//! 状态访问一律走 `Mutex`，锁中毒按“无状态”处理（不 panic，BAN-06 的边界要求）。

use std::sync::Mutex;

use perknow_contracts::{KbId, OperationHandle};

/// 已打开的知识库摘要。
#[derive(Debug, Clone, PartialEq)]
pub struct OpenKb {
    /// 知识库 id。
    pub kb_id: KbId,
    /// 在途长任务（同一库同时最多一个排他操作）。
    pub in_flight_op: Option<OperationHandle>,
}

/// 应用会话状态。
#[derive(Debug, Default)]
pub struct AppState {
    opened: Mutex<Option<OpenKb>>,
}

impl AppState {
    /// 当前打开的库；未打开返回 `None`。
    pub fn opened_kb(&self) -> Option<OpenKb> {
        self.opened.lock().ok().and_then(|guard| guard.clone())
    }

    /// 设置/清空当前库（`kb_open`/`kb_close`/`kb_switch` 调用）。
    pub fn set_opened_kb(&self, kb: Option<OpenKb>) {
        if let Ok(mut guard) = self.opened.lock() {
            *guard = kb;
        }
    }
}
