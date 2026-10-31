//! 解析产物、块与嵌入对象（`data-model.md` §2.4/§2.5/§2.6；缓存布局见 §7）。
//!
//! `Chunker` 必须确定性（`interface-contracts.md` §3.5）；`chunk.content` 是 BM25 与嵌入的
//! **唯一**文本源（BAN-07）。

use serde::{Deserialize, Serialize};

use crate::file::{FileKind, SourceLocation};
use crate::ids::{ChunkId, FileId, KbId, Timestamp};

/// 解析器标识：`{name, version, mode}`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ParserInfo {
    /// 例 `mineru` / `native`。
    pub name: String,
    /// 例 `4.x` / `1`。
    pub version: String,
    /// 例 `standard` / `raw`。
    pub mode: String,
}

/// 块类型（取值按 `data-model.md` §2.4 字面，大写）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockType {
    /// 正文段落。
    #[serde(rename = "TEXT")]
    Text,
    /// 标题。
    #[serde(rename = "TITLE")]
    Title,
    /// 表格。
    #[serde(rename = "TABLE")]
    Table,
    /// 代码。
    #[serde(rename = "CODE")]
    Code,
    /// 图片。
    #[serde(rename = "IMAGE")]
    Image,
    /// 列表。
    #[serde(rename = "LIST")]
    List,
}

/// 表格块内容：`{sheet, rows}`。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct TableData {
    /// sheet 名；csv 为文件名。
    pub sheet: String,
    /// 行 × 列的字符串矩阵。
    pub rows: Vec<Vec<String>>,
}

/// 图片块的相对引用：`{rel_path}`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ImageRefData {
    /// 图片 `rel_path`（相对 `files/`）。
    pub rel_path: String,
}

/// 解析产物的一个块。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Block {
    /// 块 id。
    pub block_id: String,
    /// 全局顺序（0-based）。
    pub order: u32,
    /// 块类型。
    pub block_type: BlockType,
    /// 文本内容（图片块为 `null`）。
    pub text: Option<String>,
    /// 定位符。
    pub source_location: SourceLocation,
    /// 标题层级（仅 `TITLE` 有意义）。
    pub level: Option<u32>,
    /// 表格数据。
    pub table: Option<TableData>,
    /// 图片引用。
    pub image: Option<ImageRefData>,
    /// 附加元数据。
    pub metadata: Option<serde_json::Value>,
}

/// 解析警告码（大写，`data-model.md` §2.4）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParseWarningCode {
    /// 加密文件。
    #[serde(rename = "ENCRYPTED")]
    Encrypted,
    /// 部分内容缺失。
    #[serde(rename = "PARTIAL")]
    Partial,
    /// 未抽到文本。
    #[serde(rename = "EMPTY_TEXT")]
    EmptyText,
    /// 超过体积上限。
    #[serde(rename = "TOO_LARGE")]
    TooLarge,
    /// 需要 OCR（本期为【扩展】）。
    #[serde(rename = "OCR_REQUIRED")]
    OcrRequired,
    /// 已降级（如解析器回退）。
    #[serde(rename = "FALLBACK")]
    Fallback,
}

/// 解析警告。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ParseWarning {
    /// 警告码。
    pub code: ParseWarningCode,
    /// 说明。
    pub message: String,
    /// 定位符（可选）。
    pub source_location: Option<SourceLocation>,
}

/// 解析产物（sidecar `parse_file` 的 `result`，`data-model.md` §2.4）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ParsedDocument {
    /// 文件 id。
    pub file_id: FileId,
    /// 相对 `files/` 的路径。
    pub rel_path: String,
    /// 类型。
    pub kind: FileKind,
    /// 解析器标识。
    pub parser: ParserInfo,
    /// 有序块。
    pub blocks: Vec<Block>,
    /// 警告（可为空）。
    pub warnings: Vec<ParseWarning>,
    /// PDF 页数；其他类型为 `null`。
    pub page_count: Option<u32>,
    /// 命中/写入的缓存目录（绝对路径，仅内部使用）。
    pub cache_path: Option<String>,
    /// 解析时间。
    pub parsed_at: Timestamp,
    /// 耗时毫秒。
    pub elapsed_ms: u64,
}

/// 块类型（小写，`data-model.md` §5.3）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum ChunkType {
    /// 正文。
    Text,
    /// 标题。
    Heading,
    /// 表格。
    Table,
    /// 代码。
    Code,
    /// 图片。
    Image,
}

/// 送入 BM25 与嵌入的最小单元（`data-model.md` §2.5）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct DocumentChunk {
    /// 块 id（重解析后新生成，`content_hash` 相同时可复用）。
    pub chunk_id: ChunkId,
    /// 所属知识库。
    pub kb_id: KbId,
    /// 所属文件。
    pub file_id: FileId,
    /// 文件内 0-based 顺序，`(kb_id, file_id, seq)` 唯一。
    pub seq: u32,
    /// **唯一**送入 BM25/嵌入的文本。
    pub content: String,
    /// `sha256(normalized_content)`，归一化=去首尾空白 + CRLF→LF。
    pub content_hash: String,
    /// 定位符（必填）。
    pub source_location: SourceLocation,
    /// 块类型。
    pub chunk_type: ChunkType,
    /// 父子分块时指向父块。
    pub parent_id: Option<ChunkId>,
    /// 估算 token 数。
    pub token_count: Option<u32>,
    /// 附加元数据（`heading_path`/`sheet`/`row_range` 等）。
    pub metadata: Option<serde_json::Value>,
}

/// 向量引用：只存引用，不存裸向量（`data-model.md` §2.6）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct VectorRef {
    /// 固定 `qdrant`。
    pub store: String,
    /// `kb_<kb_id 去连字符>`。
    pub collection: String,
    /// `chunk_id`。
    pub point_id: String,
}

/// 嵌入记录（`data-model.md` §2.6）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct EmbeddingRecord {
    /// 块 id。
    pub chunk_id: ChunkId,
    /// 模型 id，例 `embeddinggemma-2-768`。
    pub model_id: String,
    /// 维度（768/512/256/128，Matryoshka）。
    pub dim: u32,
    /// 向量存放位置。
    pub vector_ref: VectorRef,
    /// 与 chunk 一致的 `content_hash`，用于跳过重复嵌入。
    pub content_hash: String,
    /// 创建时间。
    pub created_at: Timestamp,
}

/// 解析选项（内部类型，`data-model.md` §2.16）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ParseOptions {
    /// `flash|basic|standard|advanced`。
    pub mineru_mode: String,
    /// 是否读写解析缓存。
    pub cache_enabled: bool,
    /// OCR（【扩展】，本期恒 `false`）。
    pub ocr: bool,
    /// 页范围（PDF），可选。
    pub page_range: Option<PageRange>,
}

/// 页范围：`{start, end}`，1-based 闭区间。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct PageRange {
    /// 起始页。
    pub start: u32,
    /// 结束页。
    pub end: u32,
}

/// 分块选项（内部类型）。变更这些字段会触发重构（`RebuildPlan`）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ChunkOptions {
    /// `recursive|semantic|parent_child|ast`。
    pub strategy: String,
    /// 单块最大字符数。
    pub max_chars: u32,
    /// 相邻块重叠字符数。
    pub overlap_chars: u32,
    /// 代码文件按 AST 感知分块。
    pub code_aware: bool,
}

/// 解析输入引用（内部类型）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct FileRef {
    /// 所属知识库。
    pub kb_id: KbId,
    /// 文件 id。
    pub file_id: FileId,
    /// 相对 `files/` 路径。
    pub rel_path: String,
    /// 绝对路径（`<root_dir>/files/<rel_path>`）。
    pub abs_path: String,
    /// 类型。
    pub kind: FileKind,
    /// 字节数。
    pub size: u64,
    /// 已算出的哈希（可选）。
    pub content_hash: Option<String>,
}

/// 图片引用（内部类型）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ImageRef {
    /// 所属知识库。
    pub kb_id: KbId,
    /// 文件 id。
    pub file_id: FileId,
    /// 相对路径。
    pub rel_path: String,
    /// 绝对路径。
    pub abs_path: String,
}

/// 解析服务健康（`ParserHealth`，`data-model.md` §2.16）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ParserHealth {
    /// 是否可用。
    pub ok: bool,
    /// sidecar 协议版本。
    pub protocol_version: String,
    /// MinerU 版本。
    pub mineru_version: String,
    /// 在途请求数。
    pub inflight: u32,
    /// 支持格式。
    pub supported_formats: Vec<String>,
}
