//! 建库/导入导出请求与导出包清单（`data-model.md` §2.12/§6）。
//!
//! 导入校验顺序固定：`format` → `format_version` → `contract_version` → `schema_version` →
//! `checksums.sha256` → 文件数一致，全部通过后才允许落盘。

use serde::{Deserialize, Serialize};

use crate::ids::{KbId, Timestamp};
use crate::settings::KbSettings;

/// 建库可选参数。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct CreateKbOptions {
    /// 解析档位（缺省用应用默认）。
    pub parse_mode: Option<String>,
    /// 排除的 glob 列表。
    #[serde(default)]
    pub exclude_globs: Vec<String>,
}

/// `kb_create` 入参：完整复制源目录（不支持“原地建库”）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct CreateKbRequest {
    /// 显示名。
    pub name: String,
    /// 源目录（绝对路径）。
    pub source_dir: String,
    /// 可选参数。
    pub options: Option<CreateKbOptions>,
}

/// `kb_import` 入参。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ImportKbRequest {
    /// 导出包路径（绝对路径）。
    pub zip_path: String,
    /// 覆盖库名。
    pub target_name: Option<String>,
    /// 覆盖存储根目录。
    pub target_storage_root: Option<String>,
}

/// `kb_export` 入参。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ExportRequest {
    /// 目标 zip 路径（绝对路径）。
    pub zip_path: String,
    /// 是否打包解析缓存，默认 `false`。
    #[serde(default)]
    pub include_cache: bool,
    /// 是否打包索引，默认 `true`。
    #[serde(default = "default_true")]
    pub include_index: bool,
}

/// `bool` 的 serde 默认值 `true`。
fn default_true() -> bool {
    true
}

/// 清单中的知识库信息。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ManifestKb {
    /// 原库 id。
    pub kb_id: KbId,
    /// 库名。
    pub name: String,
    /// 建库时间。
    pub created_at: Timestamp,
    /// 导出时的设置快照。
    pub settings_snapshot: KbSettings,
}

/// 清单中的索引信息。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ManifestIndex {
    /// 引擎，固定 `qdrant`。
    pub engine: String,
    /// `kb_<kb_id 去连字符>`。
    pub collection: String,
    /// 向量模型 id。
    pub vector_model: String,
    /// 维度。
    pub dim: u32,
    /// 索引版本。
    pub index_version: u64,
    /// 是否含 Tantivy 目录。
    pub tantivy: bool,
}

/// 清单中的单个文件条目。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ManifestFile {
    /// 相对路径。
    pub rel_path: String,
    /// 字节数。
    pub size: u64,
    /// SHA-256 小写 hex。
    pub content_hash: String,
    /// 副本 mtime。
    pub mtime: Timestamp,
}

/// 清单中的计数。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ManifestCounts {
    /// 文件数。
    pub files: u64,
    /// 块数。
    pub chunks: u64,
    /// 向量数。
    pub vectors: u64,
}

/// 校验和文件描述。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ManifestChecksums {
    /// 算法，固定 `sha256`。
    pub algorithm: String,
    /// 校验和文件名。
    pub file: String,
}

/// `manifest.json`（导出包根，UTF-8）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ExportManifest {
    /// 固定 `perknowbase.kb`。
    pub format: String,
    /// 包格式版本。
    pub format_version: u32,
    /// 契约版本。
    pub contract_version: String,
    /// 库表版本。
    pub schema_version: u32,
    /// 导出时应用版本。
    pub app_version: String,
    /// 导出时间。
    pub created_at: Timestamp,
    /// 库信息。
    pub kb: ManifestKb,
    /// 索引信息。
    pub index: ManifestIndex,
    /// 文件清单。
    pub files: Vec<ManifestFile>,
    /// 计数。
    pub counts: ManifestCounts,
    /// 是否含缓存。
    pub includes_cache: bool,
    /// 校验和描述。
    pub checksums: ManifestChecksums,
}
