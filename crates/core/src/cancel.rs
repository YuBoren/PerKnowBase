//! 取消原语（`interface-contracts.md` §1.2：长任务必须支持取消且取消幂等）。
//!
//! `CancelToken` 是 `Clone` 的：编排层 clone 给各阶段共享，`cancel` 幂等、可重复调用。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use perknow_contracts::AppResult;
use tokio::sync::Notify;

/// 可共享的取消信号。
#[derive(Debug, Clone)]
pub struct CancelToken {
    inner: Arc<CancelInner>,
}

#[derive(Debug, Default)]
struct CancelInner {
    cancelled: AtomicBool,
    notify: Notify,
}

impl CancelToken {
    /// 新建未取消的令牌。
    pub fn new() -> Self {
        Self {
            inner: Arc::new(CancelInner::default()),
        }
    }

    /// 请求取消；重复调用无副作用。
    pub fn cancel(&self) {
        self.inner.cancelled.store(true, Ordering::SeqCst);
        self.inner.notify.notify_waiters();
    }

    /// 是否已取消。
    pub fn is_cancelled(&self) -> bool {
        self.inner.cancelled.load(Ordering::SeqCst)
    }

    /// 已取消则返回 `CANCELLED`，用于阶段边界快速失败。
    pub fn check(&self) -> AppResult<()> {
        if self.is_cancelled() {
            Err(perknow_contracts::AppError::cancelled())
        } else {
            Ok(())
        }
    }

    /// 等待取消（已取消则立即返回）。
    pub async fn cancelled(&self) {
        if self.is_cancelled() {
            return;
        }
        self.inner.notify.notified().await;
    }
}

impl Default for CancelToken {
    fn default() -> Self {
        Self::new()
    }
}
