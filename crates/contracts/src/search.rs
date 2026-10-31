//! 检索请求/结果与提示词包（`data-model.md` §2.7/§2.8；编排见 `interface-contracts.md` §3.8）。
//!
//! `hits` 按 `rank` 升序；`preview` 只由 `chunk.content` 截断而来（`include_preview=false` 时为空串，
//! **不得为 `null`**）。

use serde::{Deserialize, Serialize};

use crate::file::{FileKind, SourceLocation};
use crate::ids::{ChunkId, FileId, KbId, RequestId, Timestamp};
use crate::llm::Usage;
use crate::parse::ChunkType;

/// 检索模式（`data-model.md` §5.3）。命令与 mode 必须一致。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum SearchMode {
    /// 极简：只返回命中。
    Minimal,
    /// 总结：调用 LLM 生成带引用的答案。
    Summary,
    /// 测试：只返回可直接复制的提示词包。
    Test,
}

/// 检索范围。`global` 为【扩展】预留，本期传 `global` 返回 `SEARCH_INVALID_REQUEST`。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum SearchScope {
    /// 当前库（默认）。
    #[default]
    Local,
    /// 跨库（本期拒绝）。
    Global,
}

/// 混合权重：`bm25 + vector = 1`（容差 1e-3）。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct HybridWeights {
    /// BM25 权重。
    pub bm25: f32,
    /// 向量权重。
    pub vector: f32,
}

/// 检索通道（`data-model.md` §2.7）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum HybridSpec {
    /// 只用 BM25。
    Bm25Only,
    /// 只用向量。
    VectorOnly,
    /// 双路 + RRF。
    Hybrid {
        /// 权重（和为 1）。
        weights: HybridWeights,
        /// RRF 平滑常数，10..=200。
        rrf_k: u32,
    },
}

impl HybridSpec {
    /// 契约默认值：`hybrid{bm25:0.5, vector:0.5, rrf_k:60}`（DEC-20261026-08）。
    pub fn default_hybrid() -> Self {
        HybridSpec::Hybrid {
            weights: HybridWeights {
                bm25: 0.5,
                vector: 0.5,
            },
            rrf_k: 60,
        }
    }
}

impl Default for HybridSpec {
    fn default() -> Self {
        HybridSpec::default_hybrid()
    }
}

/// 检索过滤条件。
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct SearchFilters {
    /// 文件类型白名单；空表示不限制。
    #[serde(default)]
    pub file_types: Vec<FileKind>,
    /// 相对路径前缀（POSIX）；`null` 表示不限制。
    pub rel_path_prefix: Option<String>,
    /// 块类型白名单；空表示不限制。
    #[serde(default)]
    pub chunk_types: Vec<ChunkType>,
    /// mtime 下界；`null` 表示不限制。
    pub mtime_after: Option<Timestamp>,
}

/// 预览选项。`preview_chars` 不得超过 `AppSettings.display.preview_chars` 上限。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct SearchOptions {
    /// 是否返回预览文本。
    pub include_preview: bool,
    /// 预览字符上限（80..=2000）。
    pub preview_chars: u32,
}

impl Default for SearchOptions {
    fn default() -> Self {
        SearchOptions {
            include_preview: true,
            preview_chars: 240,
        }
    }
}

/// 检索请求（前端生成 `request_id`，用于取消与历史关联）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct SearchRequest {
    /// 请求 id（UUIDv7）。
    pub request_id: RequestId,
    /// 必须已打开的知识库。
    pub kb_id: KbId,
    /// 查询串，1..=500 字符。
    pub query: String,
    /// 模式。
    pub mode: SearchMode,
    /// 返回条数，1..=50，默认 10。
    #[serde(default = "default_top_k")]
    pub top_k: u32,
    /// 通道配置。
    #[serde(default)]
    pub hybrid: HybridSpec,
    /// 过滤条件。
    pub filters: Option<SearchFilters>,
    /// 范围。
    #[serde(default)]
    pub scope: SearchScope,
    /// UI 上下文，仅用于事件回推，后端不持久化。
    pub tab_id: Option<String>,
    /// 预览选项。
    #[serde(default)]
    pub options: SearchOptions,
}

/// `top_k` 默认值。
fn default_top_k() -> u32 {
    10
}

/// 检索警告（降级原因等）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct SearchWarning {
    /// 警告码（大写）。
    pub code: String,
    /// 说明。
    pub message: String,
}

/// 检索过程统计。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct RetrievalStats {
    /// BM25 候选数。
    pub bm25_candidates: u32,
    /// 向量候选数。
    pub vector_candidates: u32,
    /// 融合后条数。
    pub fused: u32,
    /// 是否降级（如索引不可用走单路）。
    pub degraded: bool,
}

/// 单路原始分（调试/权重展示）。
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct RetrieverScores {
    /// BM25 分。
    pub bm25: Option<f32>,
    /// 向量分。
    pub vector: Option<f32>,
}

/// 一条检索命中。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct SearchHit {
    /// 块 id。
    pub chunk_id: ChunkId,
    /// 文件 id。
    pub file_id: FileId,
    /// 相对路径（用于打开/定位）。
    pub rel_path: String,
    /// 文件类型（图标与筛选）。
    pub file_type: FileKind,
    /// 跳转定位（必填）。
    pub source_location: SourceLocation,
    /// 预览文本；`include_preview=false` 时为空串。
    pub preview: String,
    /// 预览是否被截断。
    pub preview_truncated: bool,
    /// 相对 `preview` 的字节偏移半开区间。
    pub highlight_ranges: Vec<(u32, u32)>,
    /// RRF 融合分，越大越相关。
    pub score: f32,
    /// 0-based 排名，升序。
    pub rank: u32,
    /// 单路原始分。
    pub retriever_scores: RetrieverScores,
    /// 块类型。
    pub chunk_type: ChunkType,
}

/// 检索结果。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct SearchResult {
    /// 请求 id。
    pub request_id: RequestId,
    /// 知识库 id。
    pub kb_id: KbId,
    /// 模式。
    pub mode: SearchMode,
    /// 原始查询。
    pub query: String,
    /// 命中（按 `rank` 升序）。
    pub hits: Vec<SearchHit>,
    /// 耗时毫秒。
    pub elapsed_ms: u64,
    /// 索引版本（与 `kb_meta.index_version` 比对可发现陈旧）。
    pub index_version: u64,
    /// 统计。
    pub retrieval: RetrievalStats,
    /// 警告。
    #[serde(default)]
    pub warnings: Vec<SearchWarning>,
}

/// 引用（`ref_no` 与正文 `[n]` 一一对应）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Citation {
    /// 1-based 引用号。
    pub ref_no: u32,
    /// 块 id。
    pub chunk_id: ChunkId,
    /// 文件 id。
    pub file_id: FileId,
    /// 相对路径。
    pub rel_path: String,
    /// 定位符。
    pub source_location: SourceLocation,
    /// 引文（≤200 字）。
    pub quote: Option<String>,
}

/// 提示词上下文块。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ContextBlock {
    /// 引用号（1-based）。
    pub ref_no: u32,
    /// 相对路径。
    pub rel_path: String,
    /// 定位符。
    pub source_location: SourceLocation,
    /// 正文（来自 `chunk.content`）。
    pub text: String,
}

/// 提示词包（测试模式的返回值，可直接复制到任意大模型）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct PromptPackage {
    /// 原始查询。
    pub query: String,
    /// `summary` 或 `test`。
    pub mode: SearchMode,
    /// 上下文块，与 `hit_refs` 一一对应。
    pub context_blocks: Vec<ContextBlock>,
    /// 系统指令（含 `[n]` 引用格式要求）。
    pub instructions: String,
    /// 完整可复制文本。
    pub rendered_prompt: String,
    /// `chunk_id` 顺序 = `ref_no` 顺序。
    pub hit_refs: Vec<ChunkId>,
    /// token 估算。
    pub token_estimate: u32,
    /// 是否因预算截断。
    pub truncated: bool,
}

/// 总结模式响应（LLM 不可用时 `degraded=true` 且返回拼接上下文）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct SummaryResponse {
    /// 请求 id。
    pub request_id: RequestId,
    /// 答案正文。
    pub answer: String,
    /// 引用列表。
    pub citations: Vec<Citation>,
    /// 是否降级。
    pub degraded: bool,
    /// 降级原因（`degraded=false` 时为 `null`）。
    pub degrade_reason: Option<String>,
    /// 用量。
    pub usage: Option<Usage>,
    /// 耗时毫秒。
    pub elapsed_ms: u64,
    /// 模型名。
    pub model: String,
    /// 警告。
    #[serde(default)]
    pub warnings: Vec<SearchWarning>,
}
