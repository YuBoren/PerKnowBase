//! 长任务、融合与重构对象（`data-model.md` §2.11/§5.5；编排见 `interface-contracts.md` §3.1/§7）。
//!
//! 长任务一律返回 `OperationHandle`，进度走事件；事件只做提示，UI 用 `kb_list`/`sync_status`
//! 拉取权威进度（BAN-17）。

use serde::{Deserialize, Serialize};

use crate::ids::{KbId, OpId, Timestamp};

/// 长任务种类（`data-model.md` §5.5）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum OpKind {
    /// 建库。
    KbCreate,
    /// 导入。
    KbImport,
    /// 导出。
    KbExport,
    /// 融合。
    KbMerge,
    /// 删除。
    KbDelete,
    /// 重构。
    KbRebuild,
    /// 全量对账刷新。
    FsRefresh,
}

/// 长任务状态机（`data-model.md` §5.5）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum OpStatus {
    /// 执行中。
    Running,
    /// 全部成功。
    Succeeded,
    /// 部分失败。
    PartialFailed,
    /// 失败。
    Failed,
    /// 取消。
    Cancelled,
}

/// 长任务句柄。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct OperationHandle {
    /// 操作 id（UUIDv7）。
    pub op_id: OpId,
    /// 所属知识库。
    pub kb_id: KbId,
    /// 种类。
    pub kind: OpKind,
    /// 开始时间。
    pub started_at: Timestamp,
}

/// 融合模式：`move` 成功后**自动删除源库**（立项报告 2.3，无独立布尔字段）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum MergeMode {
    /// 复制，保留源库。
    Copy,
    /// 移动，成功后删除源库。
    Move,
}

/// 同名冲突策略，默认 `rename`（追加 “ (2)”）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum MergeConflict {
    /// 跳过。
    Skip,
    /// 覆盖。
    Overwrite,
    /// 重命名新文件（默认）。
    #[default]
    Rename,
}

/// 融合请求。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct MergeRequest {
    /// 源库。
    pub source_kb_id: KbId,
    /// 目标库。
    pub target_kb_id: KbId,
    /// 目标库内相对目录（不存在则创建）。
    pub target_rel_dir: String,
    /// 模式。
    pub mode: MergeMode,
    /// 冲突策略。
    #[serde(default)]
    pub conflict: MergeConflict,
}

/// 融合日志条目动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum MergeJournalAction {
    /// 复制文件。
    CopyFile,
    /// 移动文件。
    MoveFile,
    /// 更新元数据。
    UpdateMeta,
    /// 写入索引。
    UpsertIndex,
    /// 删除源库。
    DeleteSource,
}

/// 融合日志**条目级**状态（`data-model.md` §2.11）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum MergeJournalEntryStatus {
    /// 已计划。
    Planned,
    /// 已完成。
    Done,
    /// 已回滚。
    RolledBack,
}

/// 融合操作**整体**状态机（`data-model.md` §5.5）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum MergeJournalStatus {
    /// 已写日志、未开始搬运。
    Planned,
    /// 复制中。
    Copying,
    /// 更新元数据中。
    MetaUpdating,
    /// 更新索引中。
    Indexing,
    /// 已提交。
    Committed,
    /// 回滚中。
    RollingBack,
    /// 已回滚。
    RolledBack,
    /// 失败且未能回滚。
    Failed,
}

/// 融合日志条目（回滚依据）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct MergeJournalEntry {
    /// 条目 id。
    pub entry_id: String,
    /// 操作 id。
    pub op_id: OpId,
    /// 顺序号。
    pub seq: u32,
    /// 动作。
    pub action: MergeJournalAction,
    /// 源路径。
    pub source_path: Option<String>,
    /// 目标路径。
    pub target_path: Option<String>,
    /// 条目状态。
    pub status: MergeJournalEntryStatus,
    /// 创建时间。
    pub created_at: Timestamp,
}

/// `merge_op` 表的一行（`data-model.md` §4.2）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct MergeOpRow {
    /// 操作 id。
    pub op_id: OpId,
    /// 源库。
    pub source_kb_id: KbId,
    /// 目标库。
    pub target_kb_id: KbId,
    /// 目标相对目录。
    pub target_rel_dir: String,
    /// 模式。
    pub mode: MergeMode,
    /// 冲突策略。
    pub conflict: MergeConflict,
    /// 整体状态。
    pub status: MergeJournalStatus,
    /// 条目总数。
    pub entry_count: u32,
    /// 完成条目数。
    pub done_count: u32,
    /// 错误（JSON）。
    pub error: Option<serde_json::Value>,
    /// 创建时间（epoch 毫秒）。
    pub created_at: i64,
    /// 结束时间（epoch 毫秒）。
    pub finished_at: Option<i64>,
}

/// 重构原因（`data-model.md` §5.5）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum RebuildReason {
    /// 分块策略变更。
    ChunkStrategy,
    /// 嵌入模型变更。
    EmbeddingModel,
    /// 解析模式变更。
    ParseMode,
    /// 索引损坏。
    IndexCorrupt,
    /// 用户手动触发。
    Manual,
}

/// 重构步骤（可断点续跑）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum RebuildStep {
    /// 清缓存。
    ClearCache,
    /// 重新解析。
    Reparse,
    /// 重新分块。
    Rechunk,
    /// 重新嵌入。
    Reembed,
    /// 重建索引。
    Reindex,
    /// 校验。
    Verify,
}

/// 触发重构的字段差异。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct RebuildReasonDetail {
    /// 字段路径，例 `chunk.strategy`。
    pub field: String,
    /// 旧值。
    pub old: String,
    /// 新值。
    pub new: String,
}

/// 重构计划（`kb://rebuild-required` 载荷，需用户确认）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct RebuildPlan {
    /// 知识库 id。
    pub kb_id: KbId,
    /// 操作 id。
    pub op_id: OpId,
    /// 原因分类。
    pub reason: RebuildReason,
    /// 逐字段差异。
    #[serde(default)]
    pub reasons_detail: Vec<RebuildReasonDetail>,
    /// 计划步骤。
    pub steps: Vec<RebuildStep>,
    /// 受影响文件数。
    pub affected_files: u32,
    /// 预计耗时毫秒；未知为 `null`。
    pub eta_ms: Option<u64>,
    /// 是否必须用户确认（契约恒 `true`）。
    pub requires_confirm: bool,
}

/// `rebuild_state` 表的一行（断点续跑依据）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct RebuildStateRow {
    /// 知识库 id。
    pub kb_id: KbId,
    /// 操作 id。
    pub op_id: OpId,
    /// 原因。
    pub reason: RebuildReason,
    /// 当前步骤。
    pub step: RebuildStep,
    /// 已完成数。
    pub done: u32,
    /// 总数。
    pub total: u32,
    /// 开始时间（epoch 毫秒）。
    pub started_at: i64,
    /// 更新时间（epoch 毫秒）。
    pub updated_at: i64,
}
