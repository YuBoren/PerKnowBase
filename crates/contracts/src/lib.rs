//! # perknow-contracts（L0）
//!
//! 契约的**唯一代码形态**：领域类型、枚举、错误码、事件名、版本常量。
//! 文档形态是同目录外的 `contracts/*.md`；两者必须同 PR/同版本修改（`AGENTS.md` §4.2）。
//!
//! ## 约束
//!
//! - 本 crate **不依赖任何 workspace crate**（防循环），也不依赖 `tauri`/`rusqlite`/`qdrant-client`/
//!   `tantivy`/`ort`（`contracts/architecture-boundaries.md` §2）。
//! - 只有数据定义与纯函数式的派生（`retryable`/`user_message_key`/`as_str`），**不含业务逻辑**。
//! - `ts` feature 打开时用 `ts-rs` 导出 TS 类型；产物目录由 D 的 `scripts/gen-types` 通过
//!   `TS_RS_EXPORT_DIR` 注入，生成物禁止手改（BAN-08）。
//!
//! ## 与文档的对应
//!
//! | 模块 | 契约章节 |
//! |---|---|
//! | `kb` / `file` / `parse` | `contracts/data-model.md` §2.1–§2.6 |
//! | `search` / `llm` / `index` / `history` | §2.7–§2.9、§2.13、§3 |
//! | `sync` / `ops` / `settings` / `export` | §2.10–§2.14 |
//! | `events` / `error` / `app` | `contracts/interface-contracts.md` §2、§4、§5 |

#![deny(unsafe_code)]
#![warn(rust_2018_idioms)]

pub mod app;
pub mod error;
pub mod events;
pub mod export;
pub mod file;
pub mod history;
pub mod ids;
pub mod index;
pub mod kb;
pub mod llm;
pub mod ops;
pub mod parse;
pub mod search;
pub mod settings;
pub mod sync;
pub mod version;

pub use app::AppInfo;
pub use error::{AppError, AppResult, ErrorCode};
pub use events::{
    error_toast_from, ErrorToastEvent, Event, EventName, IndexState, IndexStatusEvent,
    KbChangeKind, KbChangedEvent, KbOpDoneEvent, KbOpProgressEvent, KbRebuildRequiredEvent,
    SyncProgressEvent,
};
pub use export::{
    CreateKbOptions, CreateKbRequest, ExportManifest, ExportRequest, ImportKbRequest,
    ManifestChecksums, ManifestCounts, ManifestFile, ManifestIndex, ManifestKb,
};
pub use file::{
    AddFailure, AddResult, ChildPage, FileError, FileKind, FileNode, IndexStatus, MoveFailure,
    MoveOp, MoveResult, Page, ParseStatus, RemoveFailure, RemoveResult, SourceLocation,
};
pub use history::{HistoryFilter, HistoryHit, HistoryPage, HistoryRecord};
pub use ids::{ChunkId, CorrelationId, FileId, KbId, OpId, RequestId, Timestamp};
pub use index::{
    Bm25Query, Distance, IndexHealth, IndexItem, Retriever, ScoredChunk, VectorCfg, VectorQuery,
};
pub use kb::{KbMetaRow, KbStats, KbStatus, KbSummary, KnowledgeBase};
pub use llm::{
    FinishReason, LlmCfg, LlmMessage, LlmProbeResult, LlmProvider, LlmRequest, LlmResponse,
    LlmRole, Usage,
};
pub use ops::{
    MergeConflict, MergeJournalAction, MergeJournalEntry, MergeJournalEntryStatus,
    MergeJournalStatus, MergeMode, MergeOpRow, MergeRequest, OpKind, OpStatus, OperationHandle,
    RebuildPlan, RebuildReason, RebuildReasonDetail, RebuildStateRow, RebuildStep,
};
pub use parse::{
    Block, BlockType, ChunkOptions, ChunkType, DocumentChunk, EmbeddingRecord, FileRef, ImageRef,
    ImageRefData, PageRange, ParseOptions, ParseWarning, ParseWarningCode, ParsedDocument,
    ParserHealth, ParserInfo, TableData, VectorRef,
};
pub use search::{
    Citation, ContextBlock, HybridSpec, HybridWeights, PromptPackage, RetrievalStats,
    RetrieverScores, SearchFilters, SearchHit, SearchMode, SearchOptions, SearchRequest,
    SearchResult, SearchScope, SearchWarning, SummaryResponse,
};
pub use settings::{
    AppSettings, AppSettingsPatch, ChunkCfg, ChunkCfgPatch, ChunkStrategy, CreatedFrom,
    DisplaySettings, DisplaySettingsPatch, EmbeddingCfg, EmbeddingCfgPatch, HistoryLimit,
    KbSearchCfg, KbSearchOverrides, KbSettings, KbSettingsPatch, LlmSettings, LlmSettingsPatch,
    MineruMode, ParseCfg, ParseCfgPatch, SearchSettings, SearchSettingsPatch, StorageSettings,
    StorageSettingsPatch, UiSettings, UiSettingsPatch, UpdateKbSettingsResult, WatchSettings,
    WatchSettingsPatch,
};
pub use sync::{
    SyncAction, SyncActionKind, SyncError, SyncEvent, SyncEventKind, SyncFileHint, SyncScope,
    SyncSource, SyncStatus, SyncTask, SyncTaskStatus, SyncTrigger,
};
pub use version::{
    CONTRACT_VERSION, EXPORT_FORMAT_ID, EXPORT_FORMAT_VERSION, PARSER_PROTOCOL_VERSION,
    SCHEMA_VERSION,
};
