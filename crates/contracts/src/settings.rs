//! 应用级与库级设置（`data-model.md` §2.14/§5.4；服务见 `interface-contracts.md` §3.2）。
//!
//! 分两层（DEC-20261026-07）：`AppSettings` 全局一份；`KbSettings` 每库一份并可覆盖应用级检索参数。
//! 变更 `chunk.strategy`/`chunk.max_chars`/`embedding.model_id`/`embedding.dim` 必然
//! `requires_rebuild=true`（DEC-20261026-05），**不自动执行**重构。

use serde::{Deserialize, Serialize};

use crate::llm::LlmCfg;
use crate::ops::RebuildPlan;
use crate::search::HybridSpec;

/// 历史保留上限。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct HistoryLimit {
    /// 最多保留条数。
    pub max_items: u32,
    /// 最多占用字节。
    pub max_bytes: u64,
}

/// 存储设置。修改 `storage_root` 本期只对新库生效。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct StorageSettings {
    /// 知识库根目录（绝对路径）。
    pub storage_root: String,
    /// 历史上限。
    pub history_limit: HistoryLimit,
}

/// 监听设置（FR-08：默认 1 分钟）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct WatchSettings {
    /// 监听周期秒，10..=3600。
    #[serde(default = "default_interval_secs")]
    pub interval_secs: u32,
    /// `fs_*` 写操作后是否主动触发同步。
    pub active_sync: bool,
}

/// 默认监听周期（秒）。
fn default_interval_secs() -> u32 {
    60
}

/// 检索设置（应用级默认值）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct SearchSettings {
    /// 默认通道配置。
    #[serde(default)]
    pub default_hybrid: HybridSpec,
    /// 默认返回条数。
    #[serde(default = "default_top_k")]
    pub top_k: u32,
    /// 默认 RRF 常数。
    #[serde(default = "default_rrf_k")]
    pub rrf_k: u32,
}

/// 默认返回条数。
fn default_top_k() -> u32 {
    10
}

/// 默认 RRF 常数。
fn default_rrf_k() -> u32 {
    60
}

/// 展示设置。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct DisplaySettings {
    /// 可选条数档位。
    pub top_k_options: Vec<u32>,
    /// 预览字符上限 80..=2000，默认 240。
    #[serde(default = "default_preview_chars")]
    pub preview_chars: u32,
}

/// 默认预览字符数。
fn default_preview_chars() -> u32 {
    240
}

/// LLM 设置组：`LlmCfg` 字段 + `enabled`（`data-model.md` §2.14）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct LlmSettings {
    /// 具体配置（扁平展开，密钥不在此处）。
    #[serde(flatten)]
    pub cfg: LlmCfg,
    /// 是否启用总结模式。
    pub enabled: bool,
}

/// UI 设置。`max_tabs` 固定 5，不可由用户修改。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct UiSettings {
    /// 最大标签数（契约固定 5）。
    #[serde(default = "default_max_tabs")]
    pub max_tabs: u32,
    /// 语言，默认 `zh-CN`。
    pub locale: String,
    /// 主题。
    pub theme: String,
}

/// 最大标签数（固定值）。
fn default_max_tabs() -> u32 {
    5
}

/// 应用级设置（`app.sqlite3.app_settings` 单行 JSON）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct AppSettings {
    /// 存储。
    pub storage: StorageSettings,
    /// 监听。
    pub watch: WatchSettings,
    /// 检索。
    pub search: SearchSettings,
    /// 展示。
    pub display: DisplaySettings,
    /// LLM。
    pub llm: LlmSettings,
    /// UI。
    pub ui: UiSettings,
}

/// 分块策略（`data-model.md` §2.14）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum ChunkStrategy {
    /// 递归切分。
    Recursive,
    /// 语义切分。
    Semantic,
    /// 父子分块。
    ParentChild,
    /// AST 感知。
    Ast,
}

/// 分块配置。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ChunkCfg {
    /// 策略。
    pub strategy: ChunkStrategy,
    /// 单块最大字符数。
    pub max_chars: u32,
    /// 重叠字符数。
    pub overlap_chars: u32,
    /// 代码文件按 AST 感知。
    pub code_aware: bool,
}

/// 嵌入配置。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct EmbeddingCfg {
    /// 模型 id，例 `embeddinggemma-2-768`。
    pub model_id: String,
    /// 维度（768/512/256/128）。
    pub dim: u32,
    /// 批大小。
    pub batch_size: u32,
    /// 是否索引图片。
    pub image_enabled: bool,
}

/// MinerU 解析档位（`data-model.md` §2.14）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum MineruMode {
    /// 最快，纯文本。
    Flash,
    /// 基础版面。
    Basic,
    /// 标准（默认）。
    Standard,
    /// 高精度（含表格/公式）。
    Advanced,
}

/// 解析配置。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ParseCfg {
    /// 解析档位。
    pub mineru_mode: MineruMode,
    /// 是否启用解析缓存。
    pub cache_enabled: bool,
    /// 导出时是否打包缓存。
    pub keep_cache_on_export: bool,
}

/// 库级检索覆盖项（缺省即继承应用级）。
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct KbSearchOverrides {
    /// 覆盖默认通道配置。
    pub default_hybrid: Option<HybridSpec>,
    /// 覆盖 RRF 常数。
    pub rrf_k: Option<u32>,
}

/// 库级检索设置。
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct KbSearchCfg {
    /// 覆盖项。
    #[serde(default)]
    pub overrides: KbSearchOverrides,
}

/// 建库来源信息。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct CreatedFrom {
    /// 建库时应用版本。
    pub app_version: String,
    /// 建库时契约版本。
    pub contract_version: String,
}

/// 库级设置（`kb.sqlite3.settings` 单行 JSON）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct KbSettings {
    /// 分块。
    pub chunk: ChunkCfg,
    /// 嵌入。
    pub embedding: EmbeddingCfg,
    /// 解析。
    pub parse: ParseCfg,
    /// 检索覆盖。
    pub search: KbSearchCfg,
    /// 建库来源。
    pub created_from: CreatedFrom,
}

/// `settings_update_kb` 的返回。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct UpdateKbSettingsResult {
    /// 更新后的设置。
    pub settings: KbSettings,
    /// 是否需要重建索引（为 `true` 时不自动执行）。
    pub requires_rebuild: bool,
    /// 需要重建时给出的计划；否则 `null`。
    pub plan: Option<RebuildPlan>,
}

/// 存储组的补丁（字段级，缺省即不变）。
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct StorageSettingsPatch {
    /// 新存储根目录。
    pub storage_root: Option<String>,
    /// 新历史上限。
    pub history_limit: Option<HistoryLimit>,
}

/// 监听组的补丁。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct WatchSettingsPatch {
    /// 新监听周期（10..=3600）。
    pub interval_secs: Option<u32>,
    /// 新主动同步开关。
    pub active_sync: Option<bool>,
}

/// 检索组的补丁。
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct SearchSettingsPatch {
    /// 新默认通道配置。
    pub default_hybrid: Option<HybridSpec>,
    /// 新默认条数。
    pub top_k: Option<u32>,
    /// 新 RRF 常数。
    pub rrf_k: Option<u32>,
}

/// 展示组的补丁。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct DisplaySettingsPatch {
    /// 新档位列表。
    pub top_k_options: Option<Vec<u32>>,
    /// 新预览字符上限。
    pub preview_chars: Option<u32>,
}

/// LLM 组的补丁。**不接受** `api_key`（走 `settings_set_llm_secret`）。
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct LlmSettingsPatch {
    /// 新配置。
    pub cfg: Option<LlmCfg>,
    /// 新启用状态。
    pub enabled: Option<bool>,
}

/// UI 组的补丁。`max_tabs` 不可改。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct UiSettingsPatch {
    /// 新语言。
    pub locale: Option<String>,
    /// 新主题。
    pub theme: Option<String>,
}

/// `settings_update_app` 的入参：字段级部分更新，未提供即不变。
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct AppSettingsPatch {
    /// 存储组。
    pub storage: Option<StorageSettingsPatch>,
    /// 监听组。
    pub watch: Option<WatchSettingsPatch>,
    /// 检索组。
    pub search: Option<SearchSettingsPatch>,
    /// 展示组。
    pub display: Option<DisplaySettingsPatch>,
    /// LLM 组。
    pub llm: Option<LlmSettingsPatch>,
    /// UI 组。
    pub ui: Option<UiSettingsPatch>,
}

/// 分块组的补丁。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ChunkCfgPatch {
    /// 新策略。
    pub strategy: Option<ChunkStrategy>,
    /// 新最大字符数。
    pub max_chars: Option<u32>,
    /// 新重叠字符数。
    pub overlap_chars: Option<u32>,
    /// 新 AST 开关。
    pub code_aware: Option<bool>,
}

/// 嵌入组的补丁。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct EmbeddingCfgPatch {
    /// 新模型 id。
    pub model_id: Option<String>,
    /// 新维度。
    pub dim: Option<u32>,
    /// 新批大小。
    pub batch_size: Option<u32>,
    /// 新图片开关。
    pub image_enabled: Option<bool>,
}

/// 解析组的补丁。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ParseCfgPatch {
    /// 新解析档位。
    pub mineru_mode: Option<MineruMode>,
    /// 新缓存开关。
    pub cache_enabled: Option<bool>,
    /// 新导出缓存开关。
    pub keep_cache_on_export: Option<bool>,
}

/// `settings_update_kb` 的入参：字段级部分更新。
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct KbSettingsPatch {
    /// 分块组。
    pub chunk: Option<ChunkCfgPatch>,
    /// 嵌入组。
    pub embedding: Option<EmbeddingCfgPatch>,
    /// 解析组。
    pub parse: Option<ParseCfgPatch>,
    /// 检索组。
    pub search: Option<KbSearchCfg>,
}
