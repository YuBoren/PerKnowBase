# 接口契约（interface-contracts）

> 版本：`contract_version = 0.1.0`　|　状态：Phase-0 冻结候选　|　owner：A（sidecar 段 C，检索/索引段 B）
> 本文件规定：模块间 Rust 接口、前端↔后端命令与事件、Rust↔Python sidecar 协议、错误码与关键时序。
> 字段级定义在 `data-model.md`；分层与禁令在 `architecture-boundaries.md`。**本文件与 `data-model.md` 冲突时，以 `data-model.md` 的字段定义为准，并立即提 PR 修正本文件。**

---

## 1. 通用约定

1. **异步模型**：所有 service trait 方法为 `async`、`Send + Sync`；不得内部阻塞超过 50ms（阻塞操作走 `spawn_blocking`）。禁止在 trait 方法内开线程池等待（并发由 shell/编排层控制）。
2. **取消**：长任务通过 `CancellationToken` 传入；`search_*`、`sync_*`、`kb_import/export/merge` 必须支持取消；取消后资源必须释放、不留半成品（见 §7 时序）。
3. **长任务一律异步化**：`kb_create/kb_import/kb_export/kb_merge/kb_delete/kb_rebuild/kb_switch` 返回 `OperationHandle`，进度走事件；**禁止在 command 内同步等待解析/索引完成**（1 万文件会超时）。
4. **幂等**：`fs_add_files`、`fs_move`、`sync_trigger`、`index_upsert`、`kb_import` 按 `(kb_id, rel_path, content_hash)` 幂等；重放不得产生重复行或重复向量。
5. **错误**：统一 `AppError`（§2）。**禁止** `unwrap`/`expect` 穿透到 command 边界；**禁止**把第三方错误字符串直接作为 `code`。
6. **时间与排序**：`SearchResult.hits` 按 `rank` 升序（0 为最佳）；`history_list` 按 `created_at` 降序；`fs_list_children` 目录在前、按名称本地化排序。
7. **分页**：列表接口必须分页（`offset/limit`，`limit <= 200`），`fs_list_children` 单目录超过 5000 项时返回 `has_more`。
8. **日志与关联**：每次 command 生成 `correlation_id`（UUIDv7），贯穿 Rust/sidecar 日志与 `error://toast`；日志不得包含文件正文与 API 密钥。
9. **DTO 校验**：命令入参是独立 DTO（`#[serde(deny_unknown_fields)]`），不得直接把领域对象当入参，避免内部字段被前端写入。
10. **禁止跨层调用**：command 层不得直接调用 `crates/index`/`crates/pipeline`/Qdrant/SQLite，只能调应用服务 trait（见 `architecture-boundaries.md`）。

---

## 2. 错误契约

```json
{ "code": "KB_NOT_FOUND", "message": "知识库不存在或已被移动",
  "details": {"kb_id": "0199..."}, "retryable": false,
  "correlation_id": "0199a1f0-...", "user_message_key": "error.kb.not_found" }
```

| code | 触发条件 | retryable | user_message_key |
|---|---|---|---|
| `KB_NOT_FOUND` | kb_id 不在注册表或目录缺失 | false | `error.kb.not_found` |
| `KB_ALREADY_OPEN` | 重复打开同一 KB | false | `error.kb.already_open` |
| `KB_BUSY` | 同一 KB 有排他操作（融合/重构/导入）进行中 | true | `error.kb.busy` |
| `KB_CORRUPTED` | `kb.sqlite3` 校验/迁移失败 | false | `error.kb.corrupted` |
| `KB_SCHEMA_TOO_NEW` | 库 `schema_version` > 应用支持版本 | false | `error.kb.schema_too_new` |
| `PATH_NOT_FOUND` | 相对/绝对路径不存在 | false | `error.fs.path_not_found` |
| `PATH_OUTSIDE_KB` | 解析后的路径逃逸出 KB 根（含 `..`、符号链接） | false | `error.fs.path_outside` |
| `PATH_CONFLICT` | 移动/添加目标已存在 | false | `error.fs.conflict` |
| `FILE_LOCKED` | 文件被其他进程占用 | true | `error.fs.locked` |
| `PARSE_FAILED` | 单个文件解析失败（不中断批次） | true | `error.parse.failed` |
| `PARSE_UNSUPPORTED` | 格式不在支持矩阵 | false | `error.parse.unsupported` |
| `PARSER_UNAVAILABLE` | sidecar 启动失败或崩溃重启耗尽 | true | `error.parse.unavailable` |
| `EMBED_FAILED` | ONNX 推理失败 | true | `error.embed.failed` |
| `MODEL_NOT_READY` | 模型文件缺失/未下载 | false | `error.embed.model_missing` |
| `INDEX_UNAVAILABLE` | Qdrant 未就绪/连接失败 | true | `error.index.unavailable` |
| `INDEX_VERSION_MISMATCH` | 索引版本与元数据不一致，需重建 | false | `error.index.rebuild_required` |
| `SEARCH_INVALID_REQUEST` | 参数非法（top_k 越界、mode/hybrid 组合非法） | false | `error.search.invalid` |
| `SEARCH_CANCELLED` | 调用取消 | false | `error.search.cancelled` |
| `LLM_NOT_CONFIGURED` | 未配置 API，且模式需 LLM | false | `error.llm.not_configured` |
| `LLM_AUTH_FAILED` | 401/403 | false | `error.llm.auth` |
| `LLM_RATE_LIMITED` | 429 | true | `error.llm.rate_limited` |
| `LLM_TIMEOUT` | 超时/网络失败 | true | `error.llm.timeout` |
| `LLM_RESPONSE_INVALID` | 响应不可解析或引用越界 | true | `error.llm.invalid` |
| `SETTINGS_INVALID` | JSON 导入校验失败/密钥缺失 | false | `error.settings.invalid` |
| `EXPORT_INVALID` | manifest/校验和不符 | false | `error.export.invalid` |
| `EXPORT_VERSION_UNSUPPORTED` | 包 `contract_version`/`schema_version` 过高 | false | `error.export.version` |
| `SYNC_IN_PROGRESS` | 同 KB 同步任务已运行 | true | `error.sync.in_progress` |
| `CANCELLED` | 通用取消 | false | `error.cancelled` |
| `IO_ERROR` | 其他文件系统错误 | true | `error.io` |
| `INTERNAL` | 未分类缺陷 | false | `error.internal` |

**聚合规则**：批处理（导入、同步、解析）不因单文件失败而整体失败；失败计入 `warnings[]`/`errors[]` 并让任务状态为 `partial_failed`。只有致命错误才 `failed`。

---

## 3. Rust 服务接口契约

> 签名是规范；`Result<T> = std::result::Result<T, AppError>`。所有 trait 由应用服务层持有，实现放在各自 crate。

### 3.1 知识库（A）

```rust
#[async_trait]
pub trait KnowledgeBaseService: Send + Sync {
    async fn list(&self) -> Result<Vec<KbSummary>>;                       // 注册表 + 在途操作
    async fn create(&self, req: CreateKbRequest) -> Result<OperationHandle>;   // 复制副本、解析、索引
    async fn open(&self, kb_id: &str) -> Result<KnowledgeBase>;           // 打开库（校验 schema、挂载索引）
    async fn close(&self, kb_id: &str) -> Result<()>;                     // 关闭句柄、停 watcher、释放索引
    async fn switch(&self, kb_id: &str) -> Result<KnowledgeBase>;         // close 当前 + open 目标（原子）
    async fn delete(&self, kb_id: &str, keep_export: Option<ExportRequest>) -> Result<OperationHandle>;
    async fn import(&self, req: ImportKbRequest) -> Result<OperationHandle>;
    async fn export(&self, req: ExportRequest) -> Result<OperationHandle>;
    async fn merge(&self, req: MergeRequest) -> Result<OperationHandle>;
    async fn rebuild(&self, plan: RebuildPlan) -> Result<OperationHandle>;
    async fn cancel(&self, op_id: &str) -> Result<()>;
}
```

语义约束：

- `create`：先在 `storage_root/<kb_id>.tmp/` 构建，**全部成功后原子 rename** 为 `storage_root/<kb_id>/`；中途崩溃只留 `.tmp`（启动时清理）。
- `switch`：先 `close` 旧库并 `flush`，再 `open` 新库；失败必须回滚到旧库仍可用。
- `delete`：默认删除副本 + 索引 + 元数据；`keep_export=true` 时先导出再删。
- `merge`：见 §7.5；失败必须按 `merge_journal` 回滚到合并前快照。
- `rebuild`：由 `SettingsService` 判定高级配置变更后发起；中断可续跑（进度落 `merge_journal` 类似的 `rebuild_state`）。

### 3.2 设置（A）

```rust
#[async_trait]
pub trait SettingsService: Send + Sync {
    async fn get_app(&self) -> Result<AppSettings>;                  // 密钥字段只返回 has_key: bool
    async fn update_app(&self, patch: AppSettingsPatch) -> Result<AppSettings>;
    async fn get_kb(&self, kb_id: &str) -> Result<KbSettings>;
    async fn update_kb(&self, kb_id: &str, patch: KbSettingsPatch) -> Result<UpdateKbSettingsResult>; // 含 requires_rebuild
    async fn export_json(&self) -> Result<String>;                   // 密钥不导出，标记 has_key
    async fn import_json(&self, json: &str) -> Result<AppSettings>;
    async fn set_llm_secret(&self, provider: &str, secret: &str) -> Result<()>; // 走系统密钥库/加密存储
}
```

- 密钥永不出现在 `get_app`/`export_json` 的明文里；日志与事件不得包含密钥。
- `update_kb` 若变更 `chunk.strategy` 或 `embedding.model`，返回 `requires_rebuild=true` 并**不**自动执行；由 shell 提示用户后调 `kb_rebuild`（DEC-20261026-05）。

### 3.3 存储（A，唯一 SQL 出口）

```rust
pub trait MetadataRepo: Send + Sync {
    // 知识库/文件
    fn kb_meta_get(&self, kb_id: &str) -> Result<Option<KbMetaRow>>;
    fn file_upsert(&self, nodes: &[FileNode]) -> Result<()>;
    fn file_get(&self, kb_id: &str, rel_path: &str) -> Result<Option<FileNode>>;
    fn file_delete(&self, kb_id: &str, paths: &[String]) -> Result<usize>;
    fn file_children(&self, kb_id: &str, rel_dir: &str, offset: u32, limit: u32) -> Result<Vec<FileNode>>;
    // 哈希
    fn file_hash_get(&self, kb_id: &str, rel_path: &str) -> Result<Option<HashRecord>>;
    fn file_hash_put(&self, rec: &HashRecord) -> Result<()>;
    // 块
    fn chunk_replace_file(&self, kb_id: &str, file_id: &str, chunks: &[DocumentChunk]) -> Result<()>; // 单事务
    fn chunk_get(&self, kb_id: &str, chunk_id: &str) -> Result<Option<DocumentChunk>>;
    fn chunk_ids_by_file(&self, kb_id: &str, file_id: &str) -> Result<Vec<String>>;
    // 同步任务 / 历史 / 合并日志
    fn sync_task_create(&self, t: &SyncTask) -> Result<()>;
    fn sync_task_update(&self, t: &SyncTask) -> Result<()>;
    fn history_insert(&self, h: &HistoryRecord) -> Result<()>;
    fn history_list(&self, kb_id: &str, f: HistoryFilter) -> Result<Vec<HistoryRecord>>;
    fn history_delete(&self, kb_id: &str, ids: &[String]) -> Result<usize>;
    fn history_trim(&self, kb_id: &str, keep: u32) -> Result<usize>;
    fn journal_append(&self, e: &MergeJournalEntry) -> Result<()>;
    fn merge_op_put(&self, op: &MergeOpRow) -> Result<()>;          // 融合整体状态（merge_op）
    fn rebuild_state_put(&self, s: &RebuildStateRow) -> Result<()>; // 重构断点续跑
    fn tx<T>(&self, f: impl FnOnce(&dyn RepoTx) -> Result<T>) -> Result<T>;  // 显式事务
}
```

- **禁止** `crates/*` 之外的任何代码持有 SQLite 连接；禁止在 `MetadataRepo` 之外拼 SQL（禁令 BAN-05）。
- 一个文件的重解析 + 块替换必须在**单事务**内完成：删除旧块 → 插入新块 → 更新 `file_node.index_status`。

### 3.4 文件系统（C）

```rust
#[async_trait]
pub trait FileSystemService: Send + Sync {
    async fn list_children(&self, kb_id: &str, rel_dir: &str, p: Page) -> Result<ChildPage>;
    async fn add_files(&self, kb_id: &str, target_rel_dir: &str, sources: &[PathBuf]) -> Result<AddResult>;
    async fn add_folder(&self, kb_id: &str, target_rel_dir: &str, source: &Path) -> Result<AddResult>;
    async fn remove(&self, kb_id: &str, rel_paths: &[String]) -> Result<RemoveResult>;
    async fn move_entries(&self, kb_id: &str, ops: &[MoveOp]) -> Result<MoveResult>;
    async fn open_external(&self, kb_id: &str, rel_path: &str, at: Option<SourceLocation>) -> Result<()>;
    async fn reveal(&self, kb_id: &str, rel_paths: &[String]) -> Result<()>;
    async fn refresh(&self, kb_id: &str, rel_dir: Option<String>) -> Result<OperationHandle>; // 全量对账
}
```

- 所有写操作先落磁盘再发 `SyncEvent{source: active}`，由 `SyncRouter` 统一处理 → **禁止** fs 层直接调 `index`/`pipeline`。
- `open_external` 只允许系统默认应用/资源管理器；若文件已被删除或历史记录悬空，返回 `PATH_NOT_FOUND`，前端显示“原文件已变更，无法跳转”。

### 3.5 管线（C）

```rust
#[async_trait]
pub trait DocumentParser: Send + Sync {
    async fn parse(&self, file: FileRef, opts: ParseOptions, cancel: CancellationToken) -> Result<ParsedDocument>;
    fn supports(&self, kind: FileKind, ext: &str) -> bool;
    async fn health(&self) -> Result<ParserHealth>;
}
pub trait Chunker: Send + Sync {
    fn chunk(&self, doc: &ParsedDocument, opts: &ChunkOptions) -> Result<Vec<DocumentChunk>>; // 纯函数、可重放
}
#[async_trait]
pub trait Embedder: Send + Sync {
    async fn embed_texts(&self, texts: &[String]) -> Result<Vec<EmbeddingRecord>>;
    async fn embed_images(&self, images: &[ImageRef]) -> Result<Vec<EmbeddingRecord>>;
    fn dim(&self) -> u32;
    fn model_id(&self) -> &str;
}
```

- `Chunker` 必须**确定性**：同一 `ParsedDocument + ChunkOptions` 产出完全一致（用于 `content_hash` 去重与增量）。
- `ParsedDocument` 的 `cache_path` 必须命中解析缓存布局；命中缓存时不重新调 sidecar。

### 3.6 同步（C）

```rust
#[async_trait]
pub trait SyncService: Send + Sync {
    async fn start(&self, kb_id: &str) -> Result<()>;
    async fn stop(&self, kb_id: &str) -> Result<()>;
    async fn trigger(&self, kb_id: &str, scope: SyncScope) -> Result<SyncTask>;
    async fn status(&self, kb_id: &str) -> Result<SyncStatus>;
    async fn set_interval(&self, kb_id: &str, secs: u32) -> Result<()>; // 10..=3600，默认 60
    async fn route(&self, ev: SyncEvent) -> Result<Vec<SyncAction>>;     // 纯路由，无副作用
}
```

`SyncAction` 取值与落库映射见 `data-model.md` §5.3；路由必须满足“移动只更元数据路径”。

### 3.7 索引（B）

```rust
#[async_trait]
pub trait IndexService: Send + Sync {
    async fn ensure_collection(&self, kb_id: &str, cfg: &VectorCfg) -> Result<()>;
    async fn upsert(&self, kb_id: &str, items: &[IndexItem]) -> Result<()>;       // 向量 + Tantivy 双写
    async fn delete(&self, kb_id: &str, chunk_ids: &[String]) -> Result<()>;
    async fn update_path(&self, kb_id: &str, file_id: &str, rel_path: &str) -> Result<()>; // 移动：只改 payload/字段
    async fn search_vector(&self, kb_id: &str, q: &VectorQuery) -> Result<Vec<ScoredChunk>>;
    async fn search_bm25(&self, kb_id: &str, q: &Bm25Query) -> Result<Vec<ScoredChunk>>;
    async fn snapshot(&self, kb_id: &str, out: &Path) -> Result<()>;
    async fn restore(&self, kb_id: &str, snapshot: &Path) -> Result<()>;
    async fn health(&self) -> Result<IndexHealth>;
}
```

- 双写顺序：先 Qdrant upsert，再 Tantivy commit；任一失败标记 `index_status=stale` 并返回 `INDEX_UNAVAILABLE`（可重试），**不得**只写一半后报告成功。
- `search_vector`/`search_bm25` 只返回 `ScoredChunk{chunk_id, score, retriever}`，**不返回预览文本**；预览由 `MetadataRepo.chunk_get` 补齐（避免两套文本源）。

### 3.8 检索与 AI（B）

```rust
#[async_trait]
pub trait SearchService: Send + Sync {
    async fn search(&self, req: SearchRequest, cancel: CancellationToken) -> Result<SearchResult>;      // Minimal
    async fn summarize(&self, req: SearchRequest, cancel: CancellationToken) -> Result<SummaryResponse>; // Summary
    async fn build_prompt(&self, req: SearchRequest) -> Result<PromptPackage>;                          // Test
    async fn cancel(&self, request_id: &str) -> Result<()>;
}
pub trait Retriever: Send + Sync { async fn retrieve(&self, req: &SearchRequest) -> Result<SearchResult>; }
pub trait PromptBuilder: Send + Sync { fn build(&self, r: &SearchResult, mode: SearchMode) -> Result<PromptPackage>; }
#[async_trait]
pub trait LlmGateway: Send + Sync {
    async fn complete(&self, req: LlmRequest) -> Result<LlmResponse>;
    async fn probe(&self, cfg: &LlmCfg) -> Result<LlmProbeResult>;
}
#[async_trait]
pub trait HistoryService: Send + Sync {
    async fn record(&self, rec: HistoryRecord) -> Result<()>;
    async fn list(&self, kb_id: &str, f: HistoryFilter) -> Result<Vec<HistoryRecord>>;
    async fn delete(&self, kb_id: &str, ids: &[String]) -> Result<usize>;
    async fn get(&self, kb_id: &str, id: &str) -> Result<HistoryRecord>;
    async fn trim(&self, kb_id: &str, keep: u32, max_bytes: u64) -> Result<u32>;
}
```

模式契约：

| 模式 | 流程 | LLM | 降级 |
|---|---|---|---|
| Minimal | 混合检索 → 取预览 → 返回 `SearchResult` | 否 | — |
| Summary | 混合检索 → PromptBuilder → LlmGateway → 解析引用 → `SummaryResponse` | 是 | 未配置/失败：`degraded=true`，返回“拼接上下文”文本 + 原样引用，错误进 `warnings` |
| Test | 混合检索 → PromptBuilder → 返回 `PromptPackage` | 否 | — |

`SearchRequest.hybrid` 默认 `Hybrid{weights:{bm25:0.5, vector:0.5}, rrf_k:60}`（DEC-20261026-08）；`top_k` 默认 10，范围 1..=50。

### 3.9 应用外壳（A 拥有注册，实现按 owner）

```rust
pub trait CommandRouter: Send + Sync { fn register(&self, app: &mut tauri::App); }  // 仅 A 可改
```

- shell 只做：参数反序列化 → 调 service → `AppError` 映射（含 `user_message_key`）→ 返回 DTO。
- shell **不得**：直接 `use qdrant_client`/`rusqlite`/`tantivy`、写业务分支、拼接 prompt、计算 hash。

---

## 4. Tauri 命令契约

> 命名 `snake_case`，`invoke('kb_create', payload)`。所有命令返回 `Result<T, AppError>`。`OperationHandle` 见 `data-model.md`。

| 命令 | 入参（DTO） | 返回 | owner | 主要错误 |
|---|---|---|---|---|
| `kb_list` | — | `Vec<KbSummary>` | A | — |
| `kb_create` | `CreateKbRequest` | `OperationHandle` | A | `PATH_NOT_FOUND`、`KB_BUSY`、`PARSE_FAILED`(部分) |
| `kb_open` | `{kb_id}` | `KnowledgeBase` | A | `KB_NOT_FOUND`、`KB_SCHEMA_TOO_NEW`、`KB_CORRUPTED` |
| `kb_close` | `{kb_id}` | `()` | A | `KB_NOT_FOUND` |
| `kb_switch` | `{kb_id}` | `KnowledgeBase` | A | `KB_NOT_FOUND`、`KB_BUSY` |
| `kb_delete` | `{kb_id, keep_export?}` | `OperationHandle` | A | `KB_BUSY` |
| `kb_export` | `ExportRequest` | `OperationHandle` | A | `IO_ERROR` |
| `kb_import` | `ImportKbRequest` | `OperationHandle` | A | `EXPORT_INVALID`、`EXPORT_VERSION_UNSUPPORTED` |
| `kb_merge` | `MergeRequest` | `OperationHandle` | A | `KB_BUSY`、`PATH_CONFLICT` |
| `kb_rebuild` | `{kb_id}` | `OperationHandle` | A | `KB_BUSY`、`MODEL_NOT_READY` |
| `kb_cancel_op` | `{op_id}` | `()` | A | `CANCELLED` |
| `fs_list_children` | `{kb_id, rel_dir, offset?, limit?}` | `ChildPage` | C | `PATH_NOT_FOUND`、`PATH_OUTSIDE_KB` |
| `fs_add_files` | `{kb_id, target_rel_dir, source_paths[]}` | `AddResult` | C | `PATH_CONFLICT`、`FILE_LOCKED` |
| `fs_add_folder` | `{kb_id, target_rel_dir, source_path}` | `AddResult` | C | 同上 |
| `fs_remove` | `{kb_id, rel_paths[]}` | `RemoveResult` | C | `PATH_NOT_FOUND` |
| `fs_move` | `{kb_id, ops[{from, to_parent}]}` | `MoveResult` | C | `PATH_CONFLICT` |
| `fs_open` | `{kb_id, rel_path, at?}` | `()` | C | `PATH_NOT_FOUND` |
| `fs_reveal` | `{kb_id, rel_paths[]}` | `()` | C | `PATH_NOT_FOUND` |
| `fs_refresh` | `{kb_id, rel_dir?}` | `OperationHandle` | C | `KB_BUSY` |
| `search_minimal` | `SearchRequest` | `SearchResult` | B | `SEARCH_INVALID_REQUEST`、`INDEX_UNAVAILABLE` |
| `search_summary` | `SearchRequest` | `SummaryResponse` | B | 上述 + `LLM_*`（可降级） |
| `search_test` | `SearchRequest` | `PromptPackage` | B | `SEARCH_INVALID_REQUEST` |
| `search_cancel` | `{request_id}` | `()` | B | `SEARCH_CANCELLED` |
| `history_list` | `{kb_id, filter, offset, limit}` | `HistoryPage` | B | — |
| `history_delete` | `{kb_id, ids[]}` | `{deleted}` | B | — |
| `history_clear` | `{kb_id, before?}` | `{deleted}` | B | — |
| `history_get` | `{kb_id, id}` | `HistoryRecord` | B | `PATH_NOT_FOUND`(快照悬空时不报错，返回标记) |
| `settings_get_app` | — | `AppSettings` | A | — |
| `settings_update_app` | `AppSettingsPatch` | `AppSettings` | A | `SETTINGS_INVALID` |
| `settings_get_kb` | `{kb_id}` | `KbSettings` | A | `KB_NOT_FOUND` |
| `settings_update_kb` | `{kb_id, patch}` | `UpdateKbSettingsResult` | A | `KB_BUSY` |
| `settings_export` | — | `{json}` | A | — |
| `settings_import` | `{json}` | `AppSettings` | A | `SETTINGS_INVALID` |
| `settings_set_llm_secret` | `{provider, secret}` | `()` | A | `SETTINGS_INVALID` |
| `sync_trigger` | `{kb_id, scope}` | `SyncTask` | C | `SYNC_IN_PROGRESS` |
| `sync_status` | `{kb_id}` | `SyncStatus` | C | — |
| `sync_set_interval` | `{kb_id, secs}` | `()` | C | `SETTINGS_INVALID`(10..=3600 越界) |
| `llm_probe` | `{cfg?}` | `LlmProbeResult` | B | `LLM_*` |
| `app_info` | — | `AppInfo{version, contract_version, schema_version, platform}` | A | — |

**命令禁令**：不得新增未列入本表的 command；新增必须 PR 更新本表 + `crates/contracts` + 生成 client（MAJOR/MINOR 按 `contracts/README.md` §2）。

---

## 5. 事件契约

> 事件名固定为 `域://事件`，载荷为 JSON。**发布者只能发布自己域的事件**；事件定义（字段）只能由 A 在本节修改。

| 事件 | 载荷 | 发布者 | 订阅者 | 频率/合并 |
|---|---|---|---|---|
| `sync://progress` | `{kb_id, task_id, phase, done, total, current_path?}` | C | D | 节流 ≥200ms，按 task 合并 |
| `index://status` | `{kb_id, indexed, total, failed, state: idle\|indexing\|stale, last_error?}` | B | D | 节流 ≥500ms |
| `kb://changed` | `{kb_id, what: META\|SETTINGS\|FILES\|INDEX}` | A | D | 直接转发 |
| `kb://op-progress` | `{op_id, kb_id, kind, phase, done, total, message?}` | A | D | 节流 ≥200ms |
| `kb://op-done` | `{op_id, kb_id, kind, status: succeeded\|partial_failed\|failed\|cancelled, result?, error?}` | A | D | 每 op 一次 |
| `kb://rebuild-required` | `{kb_id, plan: RebuildPlan}` | A | D（提示用户）→ 用户确认后调 `kb_rebuild` | 每变更一次 |
| `error://toast` | `{code, message, retryable, correlation_id, context?}` | 全员 | D | 同类合并 3s |

规则：

1. 事件是**通知**不是数据源：前端收到事件后仍需调用命令拉取权威状态（避免事件丢失导致 UI 漂移）。
2. 事件载荷不得包含文件正文、Prompt 全文、API 密钥、绝对路径（`root_dir` 除外）。
3. 事件只由 Rust 侧发出；sidecar 不发事件（由 C 转译）。
4. 订阅方必须容忍未知字段与未知 `phase` 字符串。

---

## 6. Rust ↔ Python sidecar 协议（C 定义，A 评审）

### 6.1 传输与帧格式

- **stdio**，`stdin`/`stdout` 为协议通道；**一行一个 JSON 对象（NDJSON，UTF-8，无 BOM，无多余空白行）**；不允许在 stdout 打日志（日志走 stderr）。
- JSON-RPC 2.0：请求 `{"jsonrpc":"2.0","id":"<uuid>","method":"...","params":{...}}`；响应 `{"jsonrpc":"2.0","id":"...","result":{...}}` 或 `{"jsonrpc":"2.0","id":"...","error":{"code":-32000,"message":"...","data":{...}}}`。
- 单行上限 32 MiB；超限视为协议错误并重启 sidecar。

### 6.2 方法

| 方法 | 方向 | 说明 |
|---|---|---|
| `initialize` | Rust→sidecar（请求） | `{protocol_version:"1.0", app_version, cache_root, log_level, max_parallel}`；返回 `{protocol_version, mineru_version, supported_formats[], device}`；版本不兼容返回 `-32001 PROTOCOL_MISMATCH` |
| `health` | Rust→sidecar | `{}` → `{ok, uptime_ms, inflight, visited}`；用于启动探活与心跳 |
| `parse_file` | Rust→sidecar | `{file_id, kb_id, abs_path, format, cache_dir, options:{mode:"flash\|basic\|standard\|advanced", page_range?, keep_cache:true, ocr:false}}` → `ParsedDocument`（见 `data-model.md` §3.6） |
| `cancel` | Rust→sidecar | `{id:"<被取消的 parse_file id>"}` → `{cancelled:true}`；被取消请求返回 `-32002 CANCELLED` |
| `shutdown` | Rust→sidecar | `{}` → `{ok:true}` 后进程退出 |

### 6.3 错误映射

```json
{"code": -32010, "message": "parse failed", "data": {"app_code": "PARSE_FAILED",
 "retryable": true, "file_id": "...", "reason": "encrypted pdf", "page": 3}}
```

| JSON-RPC code | app_code | 说明 |
|---|---|---|
| -32001 | `PARSER_UNAVAILABLE` | 协议版本不匹配 |
| -32002 | `CANCELLED` | 被取消 |
| -32010 | `PARSE_FAILED` | 单文件解析失败（可重试） |
| -32011 | `PARSE_UNSUPPORTED` | 格式不支持 |
| -32012 | `PATH_NOT_FOUND` | 文件在调用时已被删除 |
| -32099 | `INTERNAL` | sidecar 内部异常 |

### 6.4 生命周期（A 实现进程管理，C 提供可执行入口）

1. **按需启动**：首个解析请求时拉起；启动超时 20s（首次可放宽到 120s 用于模型下载提示）。
2. **探活**：`initialize` 成功后 5s 一次 `health`；连续 3 次失败判定不可用。
3. **重启**：崩溃后指数退避重启，最多 3 次；仍失败 → `PARSER_UNAVAILABLE`，任务 `partial_failed`，UI 提示“解析服务不可用”。
4. **空闲回收**：无在途请求 5 分钟后 `shutdown`；再次需要时重启（冷启动要能被 UI 感知，`index://status.state` 不变，但 `PARSE` 进度可能停顿）。
5. **并发**：sidecar 内最多 `max_parallel`（默认 2）并发解析，其余在 Rust 队列排队；**禁止** Rust 侧无界并发拉 sidecar。
6. **路径**：sidecar 只接受绝对路径，且必须位于当前 KB 副本根或待导入源目录；Rust 侧负责校验（BAN-12）。
7. **数据边界**：sidecar **不得**写 `kb.sqlite3`、不得直连 Qdrant、不得改 `files/` 下原文件；只写 `cache_dir`。

### 6.5 版本

`protocol_version` 独立于 `contract_version`，`MAJOR.MINOR`。MAJOR 不同 → 拒绝启动并提示重装；MINOR 不同 → 以降级能力运行（缺方法时返回 `-32601`）。

---

## 7. 关键时序

### 7.1 创建知识库（`kb_create`）

```
UI → kb_create{dest_name, source_dir, options}
A  : 生成 kb_id → 建 storage_root/<kb_id>.tmp/ → 复制副本（保留 mtime/权限）
   : 建 kb.sqlite3（schema v1）→ 写 kb_meta → 发 kb://op-progress(phase=copy)
   : 标记全部 file_node.parse_status=pending, index_status=pending
   : 发 kb://op-progress(phase=parse_index) → C: 管线并行（≤2）解析+分块+嵌入
   : 每文件完成 → B: index.upsert（Qdrant+Tantivy）→ A: 事务更新状态
   : 全部完成 → 原子 rename 为 <kb_id>/ → 注册表写入 → kb://op-done
   : 中途取消/崩溃 → 保留 .tmp 供诊断，启动时清理；注册表不写
```

### 7.2 增量同步（自动监听 / `fs_*` 主动操作）

```
C  : watch 事件(passive) 或 fs 操作(active) → 去抖(默认 500ms) → SyncEvent
   : SyncRouter.route(ev) → SyncAction:
      create/modify → [parse, chunk, embed, index_upsert, meta_update]
      delete        → [index_delete, meta_delete]
      move          → [meta_update(rel_path), index_path_update]  ← 不重新解析
   : 若 content_hash 未变（move/仅 mtime）→ 只更元数据
   : 执行中发 sync://progress；单文件失败 → 记录 warnings、index_status=failed，任务 partial_failed
```

### 7.3 极简 / 总结 / 测试检索

```
UI → search_minimal(req)  [req.request_id 由前端生成]
B  : 校验 req（top_k、hybrid 组合、kb 是否打开）
   : Retriever → BM25(top 50) + Vector(top 50) → RRF(k=60) → 过滤 → 截断 top_k
   : MetadataRepo.chunk_get 补 preview（截断到 settings.display.preview_chars）
   : 返回 SearchResult{hits ordered by rank, index_version}
summary: 同上 → PromptBuilder(mode=summary) → LlmGateway → 解析 [n] 引用 →
         校验每个 citation 的 hit_ref 存在（否则 LLM_RESPONSE_INVALID→降级）
test   : 同上 → PromptBuilder(mode=test) → 返回 PromptPackage（不调 LLM）
历史     : 成功后 HistoryService.record（异步、失败不影响检索结果）
```

### 7.4 导入 / 导出

```
export: 校验 KB 已关闭写窗口（持 KB 写锁）→ 打包 manifest.json + files/ + kb.sqlite3 +
        index/(tantivy + qdrant.snapshot) + checksums.sha256(+ 可选 cache/) → 写 temp → rename
import: 解压到 .import-tmp → 校验 manifest.contract_version/schema_version/checksums →
        版本过高 → EXPORT_VERSION_UNSUPPORTED；一致 → rename 到 storage_root/<kb_id>/ →
        C 重建缺失的解析缓存；B restore 索引快照（失败则整库重建索引）
```

### 7.5 融合（`kb_merge`）

```
A: 预检（目标根存在、目录不冲突、磁盘空间）→ 生成 merge_journal(status=planned) 并 fsync
 : 逐文件 复制(A→B) 或 移动 → 更新目标库 file_node/chunk 的 kb_id+rel_path
 : B: index.upsert 目标库（新 chunk_id 或沿用 chunk_id，payload 改 kb_id/rel_path）
 : 全部成功 → committed；move 模式 → 删除源库（副本+索引+元数据）
 : 任一步失败 → 按 journal 逆序回滚（删除已复制文件、恢复被移动文件、删除新写入索引）
```

### 7.6 重构（`kb_rebuild`，由 `kb://rebuild-required` 触发）

```
A: 校验无其他排他操作 → 记 rebuild_state(step, done, total) → 冻结 watcher
 : C: 清空 cache（策略/模型变更）→ 重新解析+分块+嵌入（分批 200 文件）
 : B: 删除旧 collection/full-text 索引 → 用新配置 ensure_collection → 重新 upsert
 : A: 更新 kb_meta.settings_snapshot 与 index_version → 解冻 watcher → kb://op-done
 : 中断可续跑：rebuild_state 记录已完成批次；重新启动后从未完成批次继续
```

---

## 8. 兼容性与废弃

1. 同一 MAJOR 内：新增命令/事件/可选字段必须保持旧客户端可用；旧字段只标记废弃不删除。
2. `contract_version` 提升时，`app_info` 立即反映；前端启动检查 `contract_version` 与生成 client 一致，不一致提示重装（避免 Tauri IPC 类型漂移）。
3. sidecar 协议与 `contract_version` 解耦，但 `app_info` 同时返回 `parser_protocol_version`。
4. 废弃项需在 `contracts/CHANGELOG.md` 标注 `deprecated_since` 与移除版本，并在 `decisions.md` 留 DEC。
