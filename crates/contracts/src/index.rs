//! 索引层内部类型（`data-model.md` §3；服务见 `interface-contracts.md` §3.7）。
//!
//! `ScoredChunk` 只回 `chunk_id/score/retriever`，**不含预览文本**：预览统一从元数据库补。

use serde::{Deserialize, Serialize};

use crate::file::{FileKind, SourceLocation};
use crate::ids::{ChunkId, FileId, KbId};
use crate::parse::ChunkType;
use crate::search::SearchFilters;

/// 单路来源（`data-model.md` §5.3）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum Retriever {
    /// BM25（Tantivy）。
    Bm25,
    /// 向量（Qdrant）。
    Vector,
    /// RRF 融合。
    Rrf,
}

/// 向量距离度量。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum Distance {
    /// 余弦距离（契约唯一取值）。
    Cosine,
}

/// 写入索引的一条记录（向量 + Tantivy 双写，`data-model.md` §2.16）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct IndexItem {
    /// 块 id（同时是 Qdrant point id）。
    pub chunk_id: ChunkId,
    /// 知识库 id。
    pub kb_id: KbId,
    /// 文件 id。
    pub file_id: FileId,
    /// 相对路径（移动时只改此字段）。
    pub rel_path: String,
    /// 文件类型。
    pub file_type: FileKind,
    /// 块类型。
    pub chunk_type: ChunkType,
    /// 块正文（BAN-07：只能来自 `chunk.content`）。
    pub content: String,
    /// 块内容哈希。
    pub content_hash: String,
    /// 向量模型 id。
    pub model_id: String,
    /// 文件内序号。
    pub seq: u32,
    /// 定位符。
    pub source_location: SourceLocation,
    /// 向量（f32）。
    pub vector: Vec<f32>,
}

/// 向量集合配置（`data-model.md` §3.1）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct VectorCfg {
    /// `kb_<kb_id 去连字符>`。
    pub collection: String,
    /// 维度，与 `KbSettings.embedding.dim` 一致。
    pub dim: u32,
    /// 距离度量。
    pub distance: Distance,
    /// 是否落盘。
    pub on_disk: bool,
}

/// 向量检索请求。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct VectorQuery {
    /// 查询向量。
    pub vector: Vec<f32>,
    /// 候选上限（契约取 50）。
    pub top_k: u32,
    /// 过滤条件。
    pub filters: Option<SearchFilters>,
}

/// BM25 检索请求。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Bm25Query {
    /// 查询串。
    pub query: String,
    /// 候选上限（契约取 50）。
    pub top_k: u32,
    /// 过滤条件。
    pub filters: Option<SearchFilters>,
}

/// 单路命中（无预览文本）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ScoredChunk {
    /// 块 id。
    pub chunk_id: ChunkId,
    /// 原始分。
    pub score: f32,
    /// 来源。
    pub retriever: Retriever,
}

/// 索引健康。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct IndexHealth {
    /// Qdrant 是否可用。
    pub qdrant_ok: bool,
    /// Tantivy 是否可用。
    pub tantivy_ok: bool,
    /// 已建集合。
    pub collections: Vec<String>,
    /// 错误摘要（脱敏）。
    pub error: Option<String>,
}
