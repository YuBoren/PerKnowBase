//! 文件节点、跳转定位与分页结果（`data-model.md` §2.2/§2.3/§2.15）。
//!
//! `rel_path` 一律相对 `files/`、POSIX 分隔、不含前导 `/`、不含 `..`（`data-model.md` §1.2）。

use serde::{Deserialize, Serialize};

use crate::ids::{FileId, KbId, Timestamp};

/// 文件类型矩阵（`data-model.md` §5.2）。`.xls` 尽力支持，不计入验收。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum FileKind {
    /// .txt/.log
    Text,
    /// .md/.markdown
    Markdown,
    /// .pdf
    Pdf,
    /// .docx
    Docx,
    /// .html/.htm
    Html,
    /// .csv
    Csv,
    /// .xlsx
    Xlsx,
    /// .json
    Json,
    /// .ipynb
    Notebook,
    /// 源码
    Code,
    /// .jpg/.jpeg/.png
    Image,
    /// 其他（标记 `unsupported`）
    Other,
}

/// 解析状态（`data-model.md` §5.2）。目录恒为 `skipped`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum ParseStatus {
    /// 待解析。
    Pending,
    /// 解析中。
    Parsing,
    /// 已解析。
    Parsed,
    /// 解析失败。
    Failed,
    /// 格式不支持。
    Unsupported,
    /// 目录或未参与解析。
    Skipped,
}

/// 索引状态（`data-model.md` §5.2）。路径/元数据变更后仍为 `indexed`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum IndexStatus {
    /// 待索引。
    Pending,
    /// 索引中。
    Indexing,
    /// 已索引。
    Indexed,
    /// 索引陈旧（引擎不可用或需重建）。
    Stale,
    /// 索引失败。
    Failed,
}

/// 最近一次失败原因（对应 DDL 的 `error_code`/`error_message`）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct FileError {
    /// 错误码（`ErrorCode` 的字符串值）。
    pub code: String,
    /// 开发者可读文本，不含正文与绝对路径。
    pub message: String,
}

/// 跳转定位（联合类型，`kind` 判别，`data-model.md` §2.3）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum SourceLocation {
    /// 文本/markdown/代码/html/json：行号 1-based。
    Text {
        /// 起始行（含），≥1。
        line_start: u32,
        /// 结束行（含），≥ `line_start`。
        line_end: u32,
        /// 起始列，未知填 0。
        col_start: u32,
        /// 结束列，未知填 0。
        col_end: u32,
    },
    /// PDF：页码 1-based，`bbox` 为左上原点坐标。
    Pdf {
        /// 页码，≥1。
        page: u32,
        /// 高亮框 `[x0, y0, x1, y1]`；可选。
        bbox: Option<Vec<f32>>,
    },
    /// 图片：只定位到文件。
    Image {
        /// 图片 `rel_path`。
        file: String,
    },
    /// 表格：csv 的 `sheet` 为文件名；xlsx 按 sheet 必填。
    Table {
        /// sheet 名或 csv 文件名。
        sheet: String,
        /// 行号（0-based），可选。
        row: Option<u32>,
        /// 列号（0-based），可选。
        col: Option<u32>,
    },
    /// Jupyter：cell 序号 0-based。
    Notebook {
        /// cell 序号。
        cell: u32,
        /// `code` 或 `markdown`。
        cell_kind: String,
    },
}

/// 文件/目录节点（`data-model.md` §2.2）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct FileNode {
    /// 节点 id（目录也分配）。
    pub file_id: FileId,
    /// 所属知识库。
    pub kb_id: KbId,
    /// 父节点；根为 `null`。
    pub parent_id: Option<FileId>,
    /// 相对 `files/` 的 POSIX 路径。
    pub rel_path: String,
    /// `rel_path` 最后一段。
    pub name: String,
    /// 类型。
    pub kind: FileKind,
    /// 小写扩展名（不含点）；目录为 `null`。
    pub ext: Option<String>,
    /// 字节数；目录为 0。
    pub size: u64,
    /// 副本 mtime。
    pub mtime: Timestamp,
    /// SHA-256 小写 hex；目录为 `null`。
    pub content_hash: Option<String>,
    /// 解析状态。
    pub parse_status: ParseStatus,
    /// 索引状态。
    pub index_status: IndexStatus,
    /// 最近失败原因。
    pub error: Option<FileError>,
    /// 原始元数据（EXIF/PDF 标题/页数等），前后端透传。
    pub metadata: Option<serde_json::Value>,
    /// 创建时间。
    pub created_at: Timestamp,
    /// 更新时间。
    pub updated_at: Timestamp,
}

/// 分页参数（`limit <= 200`，`interface-contracts.md` §1.6）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Page {
    /// 偏移。
    pub offset: u32,
    /// 上限。
    pub limit: u32,
}

/// `fs_list_children` 的返回：目录在前、名称本地化排序，单目录 >5000 项返回 `has_more`。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ChildPage {
    /// 当前页节点。
    pub items: Vec<FileNode>,
    /// 该目录总项数。
    pub total: u32,
    /// 是否还有下一页。
    pub has_more: bool,
}

/// 添加文件/目录的逐项失败。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct AddFailure {
    /// 源路径（绝对路径，仅返回给出调用方的这一处诊断信息用途）。
    pub source: String,
    /// 错误码。
    pub code: String,
    /// 说明。
    pub message: String,
}

/// `fs_add_files`/`fs_add_folder` 结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct AddResult {
    /// 新增数量。
    pub added: u32,
    /// 跳过数量（已存在且 hash 相同）。
    pub skipped: u32,
    /// 失败项。
    pub failed: Vec<AddFailure>,
}

/// 删除的逐项失败。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct RemoveFailure {
    /// 相对路径。
    pub rel_path: String,
    /// 错误码。
    pub code: String,
    /// 说明。
    pub message: String,
}

/// `fs_remove` 结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct RemoveResult {
    /// 已删除数量。
    pub removed: u32,
    /// 失败项。
    pub failed: Vec<RemoveFailure>,
}

/// 移动的逐项失败。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct MoveFailure {
    /// 源相对路径。
    pub from: String,
    /// 错误码。
    pub code: String,
    /// 说明。
    pub message: String,
}

/// `fs_move` 结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct MoveResult {
    /// 已移动数量。
    pub moved: u32,
    /// 失败项。
    pub failed: Vec<MoveFailure>,
}

/// 单次移动操作：`{from, to_parent}`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct MoveOp {
    /// 源相对路径。
    pub from: String,
    /// 目标父目录相对路径（空串表示根）。
    pub to_parent: String,
}
