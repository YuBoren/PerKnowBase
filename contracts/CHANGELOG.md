# 契约变更记录（CHANGELOG）

> 记录 `contract_version` 的每一次变化。格式：`## [版本] - 日期 - 级别`，级别见 `contracts/README.md` §2。
> **只追加，不删改历史条目**；版本号必须与 `contracts/README.md` 顶部、`crates/contracts/src/version.rs`（Phase-1）、`apps/ui/src/lib/tauri-api/version.ts`（Phase-1）一致。

## [未发布] - 计划中

- 待 `data-model.md` / `interface-contracts.md` 评审完成后冻结为 0.1.0。
- 待建 `contracts/schemas/*.schema.json` 五件套（app-settings / kb-settings / export-manifest / events / sidecar）。

## [0.1.0] - 2026-10-26 - MINOR（初始冻结候选）

**范围**：首次冻结，无历史版本可比，按“全部新增”记录。

### 新增 — 接口

- Rust trait 契约：`KnowledgeBaseService`、`SettingsService`、`MetadataRepo`、`FileSystemService`、`DocumentParser`、`Chunker`、`Embedder`、`SyncService`、`IndexService`、`SearchService`、`Retriever`、`PromptBuilder`、`LlmGateway`、`HistoryService`。
- Tauri 命令 39 条（`kb_*` 11、`fs_*` 8、`search_*` 4、`history_*` 4、`settings_*` 7、`sync_*` 3、`llm_probe`、`app_info`）。
- 事件 7 个：`sync://progress`、`index://status`、`kb://changed`、`kb://op-progress`、`kb://op-done`、`kb://rebuild-required`、`error://toast`。
- sidecar JSON-RPC 协议 v1.0：`initialize`/`health`/`parse_file`/`cancel`/`shutdown`，NDJSON over stdio。
- `AppError` 30 个错误码 + `user_message_key` 映射规则。

### 新增 — 数据模型

- 领域对象：`KnowledgeBase`、`FileNode`、`SourceLocation`（5 变体）、`ParsedDocument`/`Block`/`ParseWarning`、`DocumentChunk`、`EmbeddingRecord`、`SearchRequest/Result/Hit`、`Citation`、`PromptPackage`、`SummaryResponse`、`LlmRequest/Response`、`SyncEvent/SyncAction/SyncTask/SyncStatus`、`MergeRequest/MergeJournalEntry`、`RebuildPlan`、`OperationHandle`、`ExportManifest`、`HistoryRecord/HistoryHit`、`AppSettings`、`KbSettings`。
- 命令 DTO / 端口类型 25 项（`CreateKbRequest`、`ImportKbRequest`、`ExportRequest`、`AppSettingsPatch`、`KbSettingsPatch`、`HistoryFilter`、`HistoryPage`、`FileRef`、`ParseOptions`、`ChunkOptions`、`ImageRef`、`IndexItem`、`VectorCfg`、`VectorQuery`、`Bm25Query`、`ScoredChunk`、`LlmProbeResult`、`ParserHealth`、`IndexHealth`、`HashRecord`、`KbMetaRow`、`MergeOpRow`、`RebuildStateRow`、`RepoTx`、`AppInfo`）。
- 枚举 20 组：`KbStatus`、`ParseStatus`、`IndexStatus`、`FileKind`、`SearchMode`、`SearchScope`、`HybridKind`、`ChunkType`、`SyncEventKind`、`SyncSource`、`SyncActionKind`、`SyncTaskStatus`、`MergeMode`、`MergeJournalStatus`、`OpKind`、`OpStatus`、`RebuildReason`、`KbChangeKind`、`LlmProvider`、`FinishReason`。
- SQLite：应用级 3 表（`app_settings`、`kb_registry`、`app_schema_migrations`），库级 12 表（`kb_meta`、`file_node`、`file_hash`、`chunk`、`settings`、`sync_task`、`history`、`history_hit`、`merge_op`、`merge_journal`、`rebuild_state`、`schema_migrations`）。
- 索引：Qdrant collection `kb_<kb_id>` + payload 11 字段；Tantivy 8 字段（jieba 分词）。
- 磁盘布局、解析缓存布局、导出 ZIP 布局、`manifest.json` 结构。

### 新增 — 边界

- 分层 L0–L4 + 外部进程；crate 依赖矩阵；22 条禁令（BAN-01..BAN-22）；11 类共享模块与 6 步变更流程；数据唯一写者矩阵；质量闸门 8 项。

### 裁决引用的决策

`DEC-20261026-01` ~ `DEC-20261026-14`（见 `decisions.md`）。

### 已知待办（不阻塞 0.1.0）

- `DEC-20261026-10`（全局搜索是否本期）与 `DEC-20261026-11`（F-14/15/16 验收条目回填）仍为 `Proposed`。
- JSON Schema 五件套未落地；Phase-1 首周补齐。
- 移动端（C-03）不进入 0.1.0 契约范围。

## 版本规划

| 版本 | 目标时间 | 计划内容 |
|---|---|---|
| 0.1.0 | 第 0 周末 | 本文档冻结 |
| 0.2.0 | 第 2 周末 | 按实现反馈补：过滤条件细化、`warnings` 结构化、历史清理策略参数 |
| 0.3.0 | 第 4 周末 | 融合/重构的断点续跑字段、导出增量包（可选） |
| 1.0.0 | 验收前 | 冻结对外语义；移动端【扩展】接口预留 |
