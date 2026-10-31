//! 知识库与注册表对象（`data-model.md` §2.1；服务见 `interface-contracts.md` §3.1）。
//!
//! `root_dir`/`files_root` 是**唯一**允许出现绝对路径的字段（`data-model.md` §1.2 硬规则 2）。

use serde::{Deserialize, Serialize};

use crate::ids::{KbId, Timestamp};
use crate::ops::OperationHandle;
use crate::settings::KbSettings;

/// 知识库状态机（`data-model.md` §5.1）：`creating → ready ⇄ busy → ready`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum KbStatus {
    /// 正在构建（`<storage_root>/<kb_id>.tmp/`），注册表不可见。
    Creating,
    /// 可用。
    Ready,
    /// 有排他操作（融合/重构/导入/导出/同步写）进行中。
    Busy,
    /// 目录被移走或删除。
    Missing,
    /// 校验或迁移失败。
    Corrupted,
}

/// 统计缓存值（允许最终一致，UI 以 `kb_list` 为准）。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct KbStats {
    /// 文件节点数（不含目录）。
    pub files: u64,
    /// 块数。
    pub chunks: u64,
    /// 向量数。
    pub vectors: u64,
    /// 失败文件数。
    pub fail_count: u32,
    /// `files/` 占用字节数。
    pub size_bytes: u64,
}

/// `KnowledgeBase`：打开中的知识库（`kb_open`/`kb_switch` 返回）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct KnowledgeBase {
    /// 全局唯一 id，同时是目录名。
    pub kb_id: KbId,
    /// 显示名（用户可改，不影响目录）。
    pub name: String,
    /// `<storage_root>/<kb_id>`，绝对路径。
    pub root_dir: String,
    /// `<root_dir>/files`，UI 跳转基准，绝对路径。
    pub files_root: String,
    /// 状态。
    pub status: KbStatus,
    /// 创建时间。
    pub created_at: Timestamp,
    /// 更新时间。
    pub updated_at: Timestamp,
    /// 库表版本。
    pub schema_version: u32,
    /// 建库时契约版本。
    pub contract_version: String,
    /// 建库时应用版本。
    pub app_version: String,
    /// 每次成功重建/恢复索引 +1。
    pub index_version: u64,
    /// 建库/改配置时的设置快照，用于检测“配置已变更需重建”。
    pub settings_snapshot: KbSettings,
    /// 统计缓存。
    pub stats: KbStats,
}

/// `KbSummary`：`kb_list` 的列表行（不携带完整设置快照）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct KbSummary {
    /// 知识库 id。
    pub kb_id: KbId,
    /// 显示名。
    pub name: String,
    /// 绝对根目录。
    pub root_dir: String,
    /// 状态。
    pub status: KbStatus,
    /// 最近打开时间；从未打开为 `null`。
    pub last_opened_at: Option<Timestamp>,
    /// 统计缓存。
    pub stats: KbStats,
    /// 建库时应用版本。
    pub app_version: String,
    /// 库表版本。
    pub schema_version: u32,
    /// 在途长任务；空闲为 `null`。
    pub in_flight_op: Option<OperationHandle>,
}

/// `kb_meta` 表的一行（`data-model.md` §4.2；DB 映射类型，不直接暴露给 UI）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct KbMetaRow {
    /// 知识库 id。
    pub kb_id: KbId,
    /// 显示名。
    pub name: String,
    /// 绝对根目录。
    pub root_dir: String,
    /// 状态。
    pub status: KbStatus,
    /// 创建时间（epoch 毫秒）。
    pub created_at: i64,
    /// 更新时间（epoch 毫秒）。
    pub updated_at: i64,
    /// 库表版本。
    pub schema_version: u32,
    /// 契约版本。
    pub contract_version: String,
    /// 应用版本。
    pub app_version: String,
    /// 索引版本。
    pub index_version: u64,
    /// 向量模型 id。
    pub vector_model: String,
    /// 向量维度。
    pub vector_dim: u32,
    /// 设置快照 JSON。
    pub settings_snapshot: KbSettings,
    /// `stats_files` 缓存值。
    pub stats_files: u64,
    /// `stats_chunks` 缓存值。
    pub stats_chunks: u64,
    /// `stats_vectors` 缓存值。
    pub stats_vectors: u64,
    /// `stats_failed` 缓存值。
    pub stats_failed: u32,
}
