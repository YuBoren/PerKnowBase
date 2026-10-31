//! 同步对象（`data-model.md` §2.10/§5.4；服务见 `interface-contracts.md` §3.6）。
//!
//! `SyncRouter` 必须纯路由、无副作用；路由表即 `data-model.md` §5.4 的事件→动作映射。

use serde::{Deserialize, Serialize};

use crate::file::FileKind;
use crate::ids::{KbId, Timestamp};

/// 变更类型（`data-model.md` §5.4）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum SyncEventKind {
    /// 新增。
    Create,
    /// 修改。
    Modify,
    /// 删除。
    Delete,
    /// 移动/重命名。
    Move,
}

/// 事件来源：`fs` 服务主动写（active）或 watcher 发现（passive）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum SyncSource {
    /// `fs_*` 写操作后主动发出。
    Active,
    /// `notify` 监听发现。
    Passive,
}

/// 文件系统变更事件（由 `fs` 发出、`sync` 消费）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct SyncEvent {
    /// 事件 id（UUIDv7）。
    pub event_id: String,
    /// 知识库 id。
    pub kb_id: KbId,
    /// 变更后路径。
    pub rel_path: String,
    /// 变更类型。
    pub kind: SyncEventKind,
    /// 移动前路径（`Move` 必填）。
    pub old_rel_path: Option<String>,
    /// 目录事件需递归展开。
    pub is_dir: bool,
    /// 发现时间。
    pub detected_at: Timestamp,
    /// 来源。
    pub source: SyncSource,
}

/// 路由动作（`data-model.md` §5.4）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum SyncActionKind {
    /// 解析。
    Parse,
    /// 分块。
    Chunk,
    /// 嵌入。
    Embed,
    /// 写入索引。
    IndexUpsert,
    /// 删除索引。
    IndexDelete,
    /// 只改索引里的路径字段。
    IndexPathUpdate,
    /// 更新元数据。
    MetaUpdate,
    /// 删除元数据。
    MetaDelete,
}

/// 单条路由结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct SyncAction {
    /// 动作类型。
    pub kind: SyncActionKind,
    /// 作用路径。
    pub rel_path: String,
    /// 已存在的文件 id（`create` 前为 `null`）。
    pub file_id: Option<String>,
}

/// 同步范围。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum SyncScope {
    /// 全量对账。
    All,
    /// 只处理指定路径。
    Paths {
        /// 相对路径列表。
        rel_paths: Vec<String>,
    },
}

/// 任务触发方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum SyncTrigger {
    /// 用户手动触发。
    Manual,
    /// 定时监听触发。
    Watch,
    /// `fs_*` 主动触发。
    Active,
}

/// 同步任务状态机（`data-model.md` §5.4）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum SyncTaskStatus {
    /// 排队。
    Queued,
    /// 执行中。
    Running,
    /// 全部成功。
    Succeeded,
    /// 部分失败（必须带 `errors[]`）。
    PartialFailed,
    /// 失败。
    Failed,
    /// 取消。
    Cancelled,
}

/// 单条同步错误（`errors[]`，最多 100 条）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct SyncError {
    /// 相对路径。
    pub rel_path: String,
    /// 错误码。
    pub code: String,
    /// 说明。
    pub message: String,
}

/// 同步任务。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct SyncTask {
    /// 任务 id（UUIDv7）。
    pub task_id: String,
    /// 知识库 id。
    pub kb_id: KbId,
    /// 状态。
    pub status: SyncTaskStatus,
    /// 范围。
    pub scope: SyncScope,
    /// 总项数。
    pub total: u32,
    /// 完成项数。
    pub done: u32,
    /// 失败项数。
    pub failed: u32,
    /// 错误明细（≤100 条）。
    #[serde(default)]
    pub errors: Vec<SyncError>,
    /// 创建时间。
    pub created_at: Timestamp,
    /// 开始时间；未开始为 `null`。
    pub started_at: Option<Timestamp>,
    /// 结束时间；未结束为 `null`。
    pub finished_at: Option<Timestamp>,
    /// 触发方式。
    pub trigger: SyncTrigger,
}

/// 同步状态（`sync_status` 返回）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct SyncStatus {
    /// 运行中的任务 id；空闲为 `null`。
    pub running_task_id: Option<String>,
    /// 待处理事件数。
    pub pending_events: u32,
    /// watcher 是否在跑。
    pub watched: bool,
    /// 监听周期秒（10..=3600）。
    pub interval_secs: u32,
    /// 最近成功时间。
    pub last_success_at: Option<Timestamp>,
    /// 最近错误摘要（脱敏）。
    pub last_error: Option<String>,
}

/// 文件节点在同步路由中的轻量视图（避免 `sync` 依赖 `fs` 的私有类型）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct SyncFileHint {
    /// 相对路径。
    pub rel_path: String,
    /// 类型。
    pub kind: FileKind,
    /// 是否目录。
    pub is_dir: bool,
}
