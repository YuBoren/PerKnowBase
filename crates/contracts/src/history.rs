//! 检索历史（`data-model.md` §2.13；服务见 `interface-contracts.md` §3.8）。
//!
//! 软引用规则：快照永不随文件变更而改写；`alive=false` 时 UI 置灰且禁止跳转。

use serde::{Deserialize, Serialize};

use crate::file::{FileKind, SourceLocation};
use crate::ids::{ChunkId, FileId, KbId, Timestamp};
use crate::search::{HybridSpec, SearchMode};

/// 历史快照中的一条命中（轻量，≤20 条）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct HistoryHit {
    /// 块 id。
    pub chunk_id: ChunkId,
    /// 文件 id。
    pub file_id: FileId,
    /// 当次记录的相对路径（不随文件移动改写）。
    pub rel_path: String,
    /// 文件类型。
    pub file_type: FileKind,
    /// 定位符。
    pub source_location: SourceLocation,
    /// 预览（≤240 字）。
    pub preview: String,
    /// 融合分。
    pub score: f32,
    /// 当次排名。
    pub rank: u32,
    /// 文件当前是否仍存在且 hash 未变。
    pub alive: bool,
}

/// 一条检索历史。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct HistoryRecord {
    /// 历史 id（UUIDv7）。
    pub history_id: String,
    /// 知识库 id。
    pub kb_id: KbId,
    /// 查询串。
    pub query: String,
    /// 模式。
    pub mode: SearchMode,
    /// 便于复现的通道配置。
    pub hybrid: HybridSpec,
    /// 返回条数。
    pub top_k: u32,
    /// 纯文本摘要（≤200 字，不存 LLM 全文）。
    pub result_summary: String,
    /// 命中总数。
    pub hit_count: u32,
    /// 是否降级。
    pub degraded: bool,
    /// 轻量快照（≤20 条）。
    #[serde(default)]
    pub hits: Vec<HistoryHit>,
    /// 创建时间。
    pub created_at: Timestamp,
}

/// 历史筛选条件。
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct HistoryFilter {
    /// 查询串模糊匹配。
    pub query: Option<String>,
    /// 模式。
    pub mode: Option<SearchMode>,
    /// 起始时间（含）。
    pub from: Option<Timestamp>,
    /// 结束时间（含）。
    pub to: Option<Timestamp>,
}

/// 历史分页结果。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct HistoryPage {
    /// 当前页。
    pub items: Vec<HistoryRecord>,
    /// 总条数。
    pub total: u32,
    /// 是否还有下一页。
    pub has_more: bool,
}
