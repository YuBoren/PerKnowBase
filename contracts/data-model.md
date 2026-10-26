# 数据模型契约（data-model）

> 版本：`contract_version = 0.1.0`　|　owner：A（存储/表结构）、B（Qdrant payload、Tantivy schema）、C（解析缓存、ParsedDocument）
> 规定：磁盘布局、核心领域对象字段、枚举与状态机、SQLite 表结构、向量/全文索引结构、导出包、版本与迁移。
> **字段命名、类型、必填性是硬约束**；实现偏离即视为契约违规。

---

## 1. 存储总览

### 1.1 应用级目录 `<app_data>`

| 平台 | 路径 |
|---|---|
| Windows | `%APPDATA%\PerKnowBase\` |
| Linux | `$XDG_DATA_HOME/PerKnowBase/`（默认 `~/.local/share/PerKnowBase/`） |

```
<app_data>/
├─ app.sqlite3            # 应用级：知识库注册表 + 应用设置（A）
├─ qdrant/                # Qdrant 单进程数据（B 管理）
│  └─ storage/
├─ tantivy/<kb_id>/       # 每库一个全文索引（B）
├─ models/                # ONNX 嵌入模型（C）
├─ parsers/               # sidecar 运行时/venv（C 提供产物，A 打包安装）
├─ logs/                  # 本地日志（A）
└─ tmp/                   # 临时文件，启动时清理
```

### 1.2 知识库目录 `storage_root/<kb_id>/`

`storage_root` 可由用户在设置中修改（`AppSettings.storage_root`），默认 `<app_data>/kbs`。

```
<storage_root>/
├─ <kb_id>/               # 一个知识库 = 一个自包含目录（库间不互通）
│  ├─ kb.sqlite3          # 元数据库（WAL 模式）
│  ├─ kb.sqlite3-wal / -shm
│  ├─ files/              # 知识依据副本根（保留目录结构与 mtime）
│  ├─ cache/<file_id>/<content_hash>/   # 解析缓存（C）
│  ├─ locks/kb.lock       # 单实例写锁（flock）
│  └─ tmp/                # 原子写中转
├─ <kb_id>.tmp/           # 创建中的暂存，成功后原子 rename；启动时清理
└─ .import-tmp/           # 导入解压暂存
```

**硬规则**

1. `rel_path` 一律相对于 `files/`，POSIX 分隔，不含前导 `/`，不含 `..`。
2. `root_dir`（绝对路径）只出现在 `KnowledgeBase` 与注册表；其余契约只用 `rel_path`。
3. 库内一切数据（含索引快照）不得跨库引用；跨库检索属于【扩展】。
4. `files/` 是唯一的知识依据事实源；`cache/` 可随时删除并重建。

---

## 2. 领域对象

> 类型记号：`string`、`u32/u64/i64`、`bool`、`f32`、`T[]`、`T|null`、`map<string, any>`。时间见 `README.md` §4。
> 所有对象序列化为 `snake_case` JSON；Rust 侧 `#[serde(rename_all = "snake_case")]`。

### 2.1 `KnowledgeBase`

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `kb_id` | string(UUIDv7) | ✔ | 全局唯一，同时是目录名 |
| `name` | string | ✔ | 显示名（用户可改，不影响目录） |
| `root_dir` | string(abs) | ✔ | `<storage_root>/<kb_id>` |
| `files_root` | string(abs) | ✔ | `<root_dir>/files`，UI 跳转基准 |
| `status` | `KbStatus` | ✔ | 见 §5.1 |
| `created_at` | time | ✔ | |
| `updated_at` | time | ✔ | |
| `schema_version` | u32 | ✔ | 库表版本 |
| `contract_version` | string | ✔ | 建库时契约版本 |
| `app_version` | string | ✔ | 建库时应用版本 |
| `index_version` | u64 | ✔ | 每次成功重建/恢复 +1，检索结果携带 |
| `settings_snapshot` | `KbSettings` | ✔ | 建库/改配置时快照，用于检测“配置已变更需重建” |
| `stats` | `KbStats` | ✔ | `{files, chunks, vectors, fail_count, size_bytes}` |

`KbSummary`（`kb_list` 用）：`kb_id/name/root_dir/status/last_opened_at/stats/app_version/schema_version/in_flight_op?`。

### 2.2 `FileNode`

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `file_id` | string(UUIDv7) | ✔ | 目录节点也分配 id（便于移动稳定引用） |
| `kb_id` | string | ✔ | |
| `parent_id` | string\|null | ✔ | 根为 null |
| `rel_path` | string | ✔ | 相对 `files/`，POSIX |
| `name` | string | ✔ | 同 `rel_path` 最后一段 |
| `kind` | `FileKind` | ✔ | 由扩展名 + 探测决定 |
| `ext` | string\|null | ✔ | 小写，不含点；目录为 null |
| `size` | u64 | ✔ | 字节；目录为 0 |
| `mtime` | time | ✔ | 副本 mtime |
| `content_hash` | string\|null | ✔ | 目录为 null；文件为 SHA-256 |
| `parse_status` | `ParseStatus` | ✔ | 目录恒为 `skipped` |
| `index_status` | `IndexStatus` | ✔ | |
| `error` | `{code, message}\|null` | ✔ | 最近一次失败原因（对应 DDL 的 `error_code`/`error_message`） |
| `metadata` | map<string,any>\|null | ✔ | 原始元数据（EXIF/PDF 标题/页数等），前后端可透传 |
| `created_at` / `updated_at` | time | ✔ | |

**派生规则**：`rel_path` 变更时必须同步更新 `chunk` 以外所有引用（`history_hit.rel_path` 走软引用，不改写）。

### 2.3 `SourceLocation`（联合类型，`kind` 判别）

```json
{"kind":"text","line_start":42,"line_end":48,"col_start":0,"col_end":0}
{"kind":"pdf","page":7,"bbox":[72.1,110.4,510.2,300.9]}
{"kind":"image","file":"notes/pics/a.png"}
{"kind":"table","sheet":"Sheet1","row":12,"col":3}
{"kind":"notebook","cell":5,"cell_kind":"code"}
```

| 变体 | 必填字段 | 备注 |
|---|---|---|
| `text` | `line_start`, `line_end` (1-based) | 文本/markdown/代码/html/json：行号必须可跳转 |
| `pdf` | `page` (1-based) | `bbox` 可选（PDF 坐标，左上原点）；用于高亮 |
| `image` | `file` | 图片只定位到文件 |
| `table` | `sheet`, 可选 `row`/`col` | csv/xlsx/csv 类 |
| `notebook` | `cell`(0-based), `cell_kind` | ipynb code/markdown 分离后回指 |

**不变量**：`text.line_end >= line_start >= 1`；`pdf.page >= 1`；`image.file` 必须等于该 chunk 所属 `rel_path`。

### 2.4 `ParsedDocument`

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `file_id` | string | ✔ | |
| `rel_path` | string | ✔ | |
| `kind` | `FileKind` | ✔ | |
| `parser` | `{name, version, mode}` | ✔ | 例 `{mineru, 4.x, standard}` / `{native, 1, raw}` |
| `blocks` | `Block[]` | ✔ | 有序 |
| `warnings` | `ParseWarning[]` | ✔ | 可为空 |
| `page_count` | u32\|null | ✔ | PDF 页数；其他 null |
| `cache_path` | string(abs)\|null | ✔ | 命中/写入的缓存目录 |
| `parsed_at` | time | ✔ | |
| `elapsed_ms` | u64 | ✔ | |

`Block`：`block_id`(string)、`order`(u32, 0-based 全局序)、`block_type`(`TEXT|TITLE|TABLE|CODE|IMAGE|LIST`)、`text`(string\|null)、`source_location`(`SourceLocation`)、`level`(u32\|null，标题层级)、`table`(`{sheet, rows: string[][]}|null`)、`image`(`{rel_path}|\null`)、`metadata`(map\|null)。

`ParseWarning`：`code`(`ENCRYPTED|PARTIAL|EMPTY_TEXT|TOO_LARGE|OCR_REQUIRED|FALLBACK`)、`message`、`source_location?`。

### 2.5 `DocumentChunk`

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `chunk_id` | string(UUIDv7) | ✔ | 重解析后新生成；`content_hash` 相同时可复用 |
| `kb_id` | string | ✔ | |
| `file_id` | string | ✔ | |
| `seq` | u32 | ✔ | 文件内 0-based 顺序，`(kb_id,file_id,seq)` 唯一 |
| `content` | string | ✔ | **送入 BM25 与嵌入的唯一文本源** |
| `content_hash` | string | ✔ | `sha256(normalized_content)`；归一化=去首尾空白、CRLF→LF |
| `source_location` | `SourceLocation` | ✔ | 必填，用于溯源跳转 |
| `chunk_type` | `ChunkType` | ✔ | |
| `parent_id` | string\|null | ✔ | 父子分块时指向父块 |
| `token_count` | u32\|null | ✔ | 估算值 |
| `metadata` | map\|null | ✔ | `{heading_path[], sheet, row_range...}` |

### 2.6 `EmbeddingRecord`

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `chunk_id` | string | ✔ | |
| `model_id` | string | ✔ | 例 `embeddinggemma-2-768` |
| `dim` | u32 | ✔ | 768/512/256/128（Matryoshka） |
| `vector_ref` | `{store:"qdrant", collection, point_id}` | ✔ | 只存引用，不存裸向量 |
| `content_hash` | string | ✔ | 与 chunk 一致，用于跳过重复嵌入 |
| `created_at` | time | ✔ | |

### 2.7 `SearchRequest` / `SearchResult` / `SearchHit`

```json
{
  "request_id": "0199a1f0-5f1a-7c3a-9d2e-8b1f2a3c4d5e",
  "tab_id": "tab-3",
  "kb_id": "0199a1f0-1111-7c3a-9d2e-000000000001",
  "query": "硅的折射率",
  "mode": "minimal",
  "top_k": 10,
  "hybrid": {"kind":"hybrid","weights":{"bm25":0.5,"vector":0.5},"rrf_k":60},
  "filters": {"file_types":["csv","pdf"], "rel_path_prefix":"optics/", "chunk_types":["text","table"]},
  "scope": "local",
  "options": {"include_preview":true, "preview_chars":240}
}
```

| `SearchRequest` 字段 | 类型 | 必填 | 约束 |
|---|---|---|---|
| `request_id` | string(UUIDv7) | ✔ | 前端生成，用于取消与历史关联 |
| `kb_id` | string | ✔ | 必须已打开 |
| `query` | string | ✔ | 1..=500 字符，去首尾空白后非空 |
| `mode` | `SearchMode` | ✔ | 决定走哪个命令，命令与 mode 必须匹配 |
| `top_k` | u32 | ✔ | 1..=50，默认 10 |
| `hybrid` | `HybridSpec` | ✔ | 默认 `hybrid{0.5,0.5,60}` |
| `filters` | `SearchFilters`\|null | ✔ | 见下 |
| `scope` | `SearchScope` | ✔ | `local`；`global` 为【扩展】预留，本期传 `global` 返回 `SEARCH_INVALID_REQUEST` |
| `tab_id` | string\|null | ✔ | UI 上下文，仅用于事件回推，后端不持久化 |
| `options` | `{include_preview, preview_chars}` | ✔ | `preview_chars` 受 `AppSettings.display.preview_chars` 上限约束 |

`SearchFilters`：`file_types: FileKind[]`、`rel_path_prefix: string|null`、`chunk_types: ChunkType[]`、`mtime_after: time|null`。

`HybridSpec`：
- `{"kind":"bm25_only"}`
- `{"kind":"vector_only"}`
- `{"kind":"hybrid","weights":{"bm25":f32,"vector":f32},"rrf_k":u32}`，权重 ≥0 且和为 1（容差 1e-3），`rrf_k` 10..=200。

`SearchResult`：`request_id`、`kb_id`、`mode`、`query`、`hits: SearchHit[]`、`elapsed_ms`(u64)、`index_version`(u64)、`retrieval`(`{bm25_candidates, vector_candidates, fused, degraded:bool}`)、`warnings: [{code,message}]`。

`SearchHit`：

| 字段 | 类型 | 说明 |
|---|---|---|
| `chunk_id` / `file_id` | string | |
| `rel_path` | string | 用于打开/定位 |
| `file_type` | `FileKind` | 用于图标与筛选 |
| `source_location` | `SourceLocation` | **必填**，跳转依据 |
| `preview` | string | 由 `chunk.content` 截断；`include_preview=false` 时为空串（**不得为 null**） |
| `preview_truncated` | bool | `preview` 是否因 `preview_chars` 被截断（前端显示“…”） |
| `highlight_ranges` | `[u32,u32][]` | 相对 `preview` 的字节偏移半开区间 |
| `score` | f32 | RRF 融合分（越大越相关） |
| `rank` | u32 | 0-based，升序 |
| `retriever_scores` | `{bm25?:f32, vector?:f32}` | 单路原始分，调试/权重展示 |
| `chunk_type` | `ChunkType` | |

### 2.8 `Citation` / `PromptPackage` / `SummaryResponse`

`Citation`：`ref_no`(u32, 1-based, 与正文 `[n]` 对应)、`chunk_id`、`file_id`、`rel_path`、`source_location`、`quote?`(string，≤200 字)。

`PromptPackage`：

| 字段 | 类型 | 说明 |
|---|---|---|
| `query` | string | 原始查询 |
| `mode` | `SearchMode` | `summary` 或 `test` |
| `context_blocks` | `{ref_no, rel_path, source_location, text}[]` | 与 `hit_refs` 一一对应 |
| `instructions` | string | 系统指令（含引用格式要求 `[n]`） |
| `rendered_prompt` | string | **可直接复制到任意大模型**的完整文本 |
| `hit_refs` | `string[]` | `chunk_id` 顺序 = `ref_no` 顺序 |
| `token_estimate` | u32 | 估算 |
| `truncated` | bool | 是否因预算截断 |

`SummaryResponse`：`request_id`、`answer`(string)、`citations: Citation[]`、`degraded`(bool)、`degrade_reason?`、`usage`(`{prompt_tokens, completion_tokens, total_tokens}`\|null)、`elapsed_ms`、`model`、`warnings[]`。

**引用校验**：`answer` 中出现的 `[n]` 必须存在于 `citations`；LLM 返回越界引用 → 丢弃该引用并记 `warnings`；全部越界 → `LLM_RESPONSE_INVALID` 并降级。

### 2.9 `LlmRequest` / `LlmResponse`

`LlmRequest`：`model`、`messages:[{role:system|user|assistant, content}]`、`temperature`(f32, 0..=2)、`max_tokens`(u32\|null)、`stream`(bool，本期一律 false)、`timeout_ms`(u64)。

`LlmResponse`：`model`、`content`、`usage`、`finish_reason`(`stop|length|content_filter|error`)、`elapsed_ms`。

`LlmCfg`（存于设置，密钥单独加密）：`provider`(`LlmProvider`)、`base_url`、`model`、`timeout_ms`、`max_tokens`、`has_key`(bool，只读)。

### 2.10 同步对象

`SyncEvent`：

| 字段 | 类型 | 说明 |
|---|---|---|
| `event_id` | string(UUIDv7) | |
| `kb_id` | string | |
| `rel_path` | string | 变更后路径 |
| `kind` | `SyncEventKind` | CREATE/MODIFY/DELETE/MOVE |
| `old_rel_path` | string\|null | MOVE 时必填 |
| `is_dir` | bool | 目录事件需要递归展开 |
| `detected_at` | time | |
| `source` | `SyncSource` | ACTIVE（fs 服务）/PASSIVE（watcher） |

`SyncAction`：`{kind: SyncActionKind, rel_path, file_id?}`；`SyncActionKind` 见 §5.4。

`SyncTask`：

| 字段 | 类型 | 说明 |
|---|---|---|
| `task_id` | string(UUIDv7) | |
| `kb_id` | string | |
| `status` | `SyncTaskStatus` | |
| `scope` | `SyncScope`：`{kind:"all"}` 或 `{kind:"paths","rel_paths":[...]}` | |
| `total`/`done`/`failed` | u32 | |
| `errors` | `{rel_path, code, message}[]` | 每条最多 100 条，超出计数截断 |
| `created_at`/`started_at`/`finished_at` | time\|null | |
| `trigger` | `manual`\|`watch`\|`active` | |

`SyncStatus`：`{running_task_id?, pending_events, watched:bool, interval_secs, last_success_at?, last_error?}`。

### 2.11 融合 / 重构 / 长任务

`MergeRequest`：

| 字段 | 类型 | 说明 |
|---|---|---|
| `source_kb_id` / `target_kb_id` | string | 必须都已在注册表；`source != target` |
| `target_rel_dir` | string | 目标库内相对目录，不存在则创建 |
| `mode` | `MergeMode` | `COPY` 保留源库；`MOVE` 成功后**自动删除源库**（立项报告 2.3，无独立布尔字段） |
| `conflict` | `skip\|overwrite\|rename` | 同名冲突策略，默认 `rename`（追加 ` (2)`） |

`MergeJournalEntry`：`entry_id`、`op_id`、`seq`、`action`(`copy_file|move_file|update_meta|upsert_index|delete_source`)、`source_path`、`target_path`、`status`(`planned|done|rolled_back`，**条目级**；整体状态见 §5.5 `MergeJournalStatus`)、`created_at`。

`RebuildPlan`：`kb_id`、`op_id`、`reason`(`RebuildReason`)、`reasons_detail`(`{field, old, new}[]`)、`steps`(`CLEAR_CACHE|REPARSE|RECHUNK|REEMBED|REINDEX|VERIFY`)、`affected_files`(u32)、`eta_ms`(u64\|null)、`requires_confirm`(bool=true)。

`OperationHandle`：`op_id`、`kb_id`、`kind`(`OpKind`)、`started_at`。UI 用 `kb_list`/`sync_status` 拉取权威进度，事件只做提示。

### 2.12 导入导出

`ExportManifest`（`manifest.json`，UTF-8）：

```json
{
  "format": "perknowbase.kb",
  "format_version": 1,
  "contract_version": "0.1.0",
  "schema_version": 1,
  "app_version": "0.1.0",
  "created_at": "2026-10-26T05:44:03.123Z",
  "kb": {"kb_id":"0199...","name":"光学笔记","created_at":"...","settings_snapshot":{}},
  "index": {"engine":"qdrant","collection":"kb_0199...","vector_model":"embeddinggemma-2-768","dim":768,"index_version":42,"tantivy":true},
  "files": [{"rel_path":"optics/a.md","size":1234,"content_hash":"<sha256>","mtime":"..."}],
  "counts": {"files": 120, "chunks": 3400, "vectors": 3400},
  "includes_cache": false,
  "checksums": {"algorithm":"sha256","file":"checksums.sha256"}
}
```

导入校验顺序：`format` → `format_version` → `contract_version`(≤当前) → `schema_version`(≤当前) → `checksums.sha256` 全量校验 → 文件数一致 → 才允许落盘。

### 2.13 历史

`HistoryRecord`：

| 字段 | 类型 | 说明 |
|---|---|---|
| `history_id` | string(UUIDv7) | |
| `kb_id` | string | |
| `query` | string | |
| `mode` | `SearchMode` | |
| `hybrid` | `HybridSpec` | 便于复现 |
| `top_k` | u32 | |
| `result_summary` | string | ≤200 字，纯文本摘要（不存 LLM 全文） |
| `hit_count` | u32 | |
| `degraded` | bool | |
| `hits` | `HistoryHit[]` | 轻量快照，≤20 条 |
| `created_at` | time | |

`HistoryHit`：`chunk_id`、`file_id`、`rel_path`、`file_type`、`source_location`、`preview`(≤240 字)、`score`、`rank`、`alive`(bool：文件当前是否仍存在且 hash 未变)。

**软引用规则**：快照永不随文件变更而改写；`alive=false` 时 UI 置灰且禁止跳转（立项报告 2.3）。历史保存在 KB 库内，无跨库历史。

### 2.14 设置

`AppSettings`（`app.sqlite3.app_settings`，单行 JSON）：

| 分组 | 字段 | 说明 |
|---|---|---|
| `storage` | `storage_root`(abs)、`history_limit`(`{max_items, max_bytes}`) | 修改 `storage_root` 需迁移提示，本期只允许“新建库生效” |
| `watch` | `interval_secs`(10..=3600, 默认 60)、`active_sync`(bool) | 默认监听周期 1 分钟（FR-08） |
| `search` | `default_hybrid`(`HybridSpec`)、`top_k`(默认 10)、`rrf_k`(默认 60) | 混合检索权重 |
| `display` | `top_k_options[]`、`preview_chars`(80..=2000, 默认 240) | 显示项数/预览大小 |
| `llm` | `LlmCfg` + `enabled`(bool) | 密钥不入 JSON，只存 `has_key` |
| `ui` | `max_tabs`(固定 5)、`locale`、`theme` | `max_tabs` 不可由用户改 |

`KbSettings`（`kb.sqlite3.settings`，单行 JSON）：

| 字段 | 说明 |
|---|---|
| `chunk` | `{strategy: recursive\|semantic\|parent_child\|ast, max_chars, overlap_chars, code_aware:bool}` |
| `embedding` | `{model_id, dim, batch_size, image_enabled:bool}` |
| `parse` | `{mineru_mode: flash\|basic\|standard\|advanced, cache_enabled:bool, keep_cache_on_export:bool}` |
| `search` | `{overrides: {default_hybrid?, rrf_k?}}`（库级覆盖应用级） |
| `created_from` | `{app_version, contract_version}` |

`UpdateKbSettingsResult`：`{settings: KbSettings, requires_rebuild: bool, plan: RebuildPlan|null}`。
变更 `chunk.strategy`/`chunk.max_chars`/`embedding.model_id`/`embedding.dim` 必然 `requires_rebuild=true`（DEC-20261026-05）。

### 2.15 分页与操作结果

`Page`：`{offset:u32, limit:u32}`。`ChildPage`：`{items: FileNode[], total:u32, has_more:bool}`。
`AddResult`：`{added:u32, skipped:u32, failed:[{source, code, message}]}`。
`RemoveResult`：`{removed:u32, failed:[{rel_path, code, message}]}`。
`MoveResult`：`{moved:u32, failed:[{from, code, message}]}`。
`MoveOp`：`{from:string, to_parent:string}`。

### 2.16 与立项报告 DR-08 的映射

| 立项报告 DR-08 | 本文类型 | 备注 |
|---|---|---|
| KnowledgeBase | `KnowledgeBase` | 原名 `KnowledgeBaseMeta` 废弃（DEC-20261026-12） |
| FileNode / DocumentChunk / EmbeddingRecord | 同名 | — |
| SearchRequest / SearchResult / Citation / PromptPackage | 同名 | — |
| SyncTask / ExportManifest | 同名 | — |
| （新增） | `SyncEvent`、`ParsedDocument`、`SourceLocation`、`HistoryRecord`、`MergeRequest`、`RebuildPlan`、`AppError`、`AppSettings`、`KbSettings`、`OperationHandle` | 实现必需，已纳入契约 |

### 2.17 命令 DTO 与端口类型（最小定义）

> 跨模块传递的类型必须在此登记；未登记的“匿名 JSON”禁止出现在 trait/命令边界。

| 类型 | 定义 | owner |
|---|---|---|
| `CreateKbRequest` | `{name, source_dir(abs), options?: {parse_mode?, exclude_globs?: string[]}}`（完整复制，不支持“原地建库”） | A |
| `ImportKbRequest` | `{zip_path(abs), target_name?, target_storage_root?}` | A |
| `ExportRequest` | `{zip_path(abs), include_cache?: bool=false, include_index?: bool=true}` | A |
| `AppSettingsPatch` | `AppSettings` 的**部分字段**；未提供即不变；**不接受** `llm.api_key`（走 `settings_set_llm_secret`） | A |
| `KbSettingsPatch` | `KbSettings` 的部分字段；返回 `UpdateKbSettingsResult` | A |
| `HistoryFilter` | `{query?: string, mode?: SearchMode, from?: time, to?: time}` | B |
| `HistoryPage` | `{items: HistoryRecord[], total: u32, has_more: bool}` | B |
| `FileRef`（内部） | `{kb_id, file_id, rel_path, abs_path, kind, size, content_hash?}` | C |
| `ParseOptions`（内部） | `{mineru_mode, cache_enabled: bool, ocr: bool=false(扩展), page_range?: {start,end}}` | C |
| `ChunkOptions`（内部） | `{strategy, max_chars, overlap_chars, code_aware: bool}` | C |
| `ImageRef`（内部） | `{kb_id, file_id, rel_path, abs_path}` | C |
| `IndexItem`（内部） | `{chunk_id, kb_id, file_id, rel_path, file_type, chunk_type, content, content_hash, model_id, seq, source_location, vector: f32[]}` | B |
| `VectorCfg`（内部） | `{collection, dim, distance:"cosine", on_disk: true}` | B |
| `VectorQuery` / `Bm25Query`（内部） | `{query_or_vector, top_k, filters}`（前者为向量，后者为查询串） | B |
| `ScoredChunk`（内部） | `{chunk_id, score, retriever}`（**不含预览文本**，见 `interface-contracts.md` §3.7） | B |
| `LlmProbeResult` | `{ok, model?, latency_ms, error?}` | B |
| `ParserHealth` | `{ok, protocol_version, mineru_version, inflight, supported_formats[]}` | C |
| `IndexHealth` | `{qdrant_ok, tantivy_ok, collections, error?}` | B |
| `HashRecord`（内部） | `{kb_id, rel_path, file_id, content_hash, size, mtime, hashed_at}` | A |
| `KbMetaRow`（内部） | `kb_meta` 表的一行（DB 映射，不直接暴露给 UI） | A |
| `MergeOpRow`（内部） | `merge_op` 表的一行：`{op_id, source_kb_id, target_kb_id, target_rel_dir, mode, conflict, status, entry_count, done_count, error, created_at, finished_at}` | A |
| `RebuildStateRow`（内部） | `rebuild_state` 表的一行：`{kb_id, op_id, reason, step, done, total, started_at, updated_at}` | A |
| `RepoTx`（内部） | 不透明事务句柄，仅 `MetadataRepo::tx` 内可见 | A |
| `AppInfo` | `{version, contract_version, schema_version, platform, parser_protocol_version}` | A |

### 2.18 `AppError`（统一错误载荷）

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `code` | `ErrorCode`（字符串枚举） | ✔ | 见 `interface-contracts.md` §2 的 30 个码；未知码按 `INTERNAL` 处理 |
| `message` | string | ✔ | 开发者可读；**不得**含文件正文/密钥/绝对路径 |
| `details` | map<string,any>\|null | ✔ | 结构化上下文（`kb_id`/`rel_path`/`op_id` 等） |
| `retryable` | bool | ✔ | 前端据此决定是否显示“重试” |
| `correlation_id` | string(UUIDv7) | ✔ | 与日志关联 |
| `user_message_key` | string | ✔ | i18n key（如 `error.kb.busy`），前端优先展示 |

---

## 3. 索引契约

### 3.1 Qdrant（B）

| 项 | 约定 |
|---|---|
| 进程 | 单进程，`127.0.0.1:<随机端口>`；数据目录 `<app_data>/qdrant/storage`（DEC-20261026-06） |
| collection | `kb_<kb_id 去掉连字符>`（例 `kb_0199a1f011117c3a...`）；**每库一个** |
| vector | 命名向量 `content`；`size = KbSettings.embedding.dim`；`distance = Cosine`；`on_disk = true` |
| point id | `chunk_id`（UUID 字符串） |
| HNSW | `m=16, ef_construct=100`；查询 `hnsw_ef=64`；可按 P-01 调优但需记录 |
| 写入 | 批量 `upsert`，批大小 64；`wait=true`（保证读己写） |

**payload**（字段名即契约；新增需 MINOR）：

| 字段 | 类型 | 索引 | 说明 |
|---|---|---|---|
| `kb_id` | keyword | ✔ | 冗余便于校验与【扩展】跨库 |
| `file_id` | keyword | ✔ | |
| `chunk_id` | keyword | — | 与 point id 一致 |
| `rel_path` | keyword | ✔ | 移动时只改此字段 |
| `file_type` | keyword | ✔ | `FileKind` 小写 |
| `chunk_type` | keyword | ✔ | |
| `source_location` | json→string | — | 序列化后的 `SourceLocation` |
| `content_hash` | keyword | — | 复用判定 |
| `model_id` | keyword | ✔ | 换模型时用于校验 |
| `seq` | integer | — | 文件内序号 |
| `created_at` | integer(epoch ms) | — | |

**禁止**把 `content` 全文放进 payload（体积与一致性风险）；预览从元数据库取。

### 3.2 Tantivy（B）

| 项 | 约定 |
|---|---|
| 位置 | `<app_data>/tantivy/<kb_id>/`；每库一套 |
| schema | 见下表；`kb_id` 保留（为【扩展】跨库合并预留） |
| 分词 | `content` 使用 `jieba` 中文分词（缺依赖时降级 `default` + ngram fallback，需在 `warnings` 标注）；`rel_path` 用 `raw` |
| BM25 | `k1=1.2, b=0.75` |
| commit | 每批 upsert 后 `commit()`；`reader.reload()` 保证可见 |

| 字段 | 类型 | 选项 |
|---|---|---|
| `chunk_id` | STRING | STORED, RAW/STRING |
| `kb_id` | STRING | STORED |
| `file_id` | STRING | STORED |
| `content` | TEXT | STORED, `jieba` |
| `rel_path` | STRING | STORED |
| `file_type` | STRING | STORED |
| `source_location` | STRING | STORED（JSON 串） |
| `seq` | u64 | STORED, FAST |

**语料铁律**：`content` 必须来自 `chunk.content`（元数据库已分块内容），**禁止**取解析原始产物或原文文件（立项报告 2.4 / 禁令 BAN-07）。

---

## 4. SQLite 表结构

> 每库一个 `kb.sqlite3`（`PRAGMA journal_mode=WAL; foreign_keys=ON; busy_timeout=5000`）；应用级一个 `app.sqlite3`。
> 所有表：`created_at`/`updated_at` 为 epoch 毫秒 `INTEGER NOT NULL`（除注明）。JSON 列以 `TEXT` 存，写入前必须通过 schema 校验。

### 4.1 应用级 `app.sqlite3`

**`app_settings`**

| 列 | 类型 | 约束 |
|---|---|---|
| `id` | INTEGER | PK, `CHECK (id = 1)` |
| `json` | TEXT | NOT NULL，`AppSettings` |
| `updated_at` | INTEGER | NOT NULL |

**`kb_registry`**

| 列 | 类型 | 约束 |
|---|---|---|
| `kb_id` | TEXT | PK |
| `name` | TEXT | NOT NULL |
| `root_dir` | TEXT | NOT NULL, UNIQUE |
| `status` | TEXT | NOT NULL（`KbStatus`） |
| `app_version` / `schema_version` / `contract_version` | TEXT/INTEGER/TEXT | NOT NULL |
| `created_at` / `last_opened_at` | INTEGER | NOT NULL / NULL |
| `stats_files` / `stats_chunks` / `size_bytes` | INTEGER | NOT NULL DEFAULT 0 |

**`app_schema_migrations`**：`version INTEGER PK, applied_at INTEGER NOT NULL, checksum TEXT NOT NULL`。

### 4.2 知识库库 `kb.sqlite3`

**`kb_meta`**（单行）

| 列 | 类型 | 约束 |
|---|---|---|
| `kb_id` | TEXT | PK |
| `name` | TEXT | NOT NULL |
| `root_dir` | TEXT | NOT NULL |
| `status` | TEXT | NOT NULL |
| `created_at` / `updated_at` | INTEGER | NOT NULL |
| `schema_version` | INTEGER | NOT NULL |
| `contract_version` | TEXT | NOT NULL |
| `app_version` | TEXT | NOT NULL |
| `index_version` | INTEGER | NOT NULL DEFAULT 0 |
| `vector_model` | TEXT | NOT NULL |
| `vector_dim` | INTEGER | NOT NULL |
| `settings_snapshot` | TEXT | NOT NULL（`KbSettings` JSON） |
| `stats_files` / `stats_chunks` / `stats_vectors` / `stats_failed` | INTEGER | NOT NULL DEFAULT 0 |

**`file_node`**

| 列 | 类型 | 约束 |
|---|---|---|
| `file_id` | TEXT | PK |
| `kb_id` | TEXT | NOT NULL |
| `parent_id` | TEXT | NULL, FK→`file_node.file_id` ON DELETE CASCADE |
| `rel_path` | TEXT | NOT NULL |
| `name` | TEXT | NOT NULL |
| `kind` | TEXT | NOT NULL（`FileKind`） |
| `ext` | TEXT | NULL |
| `size` | INTEGER | NOT NULL DEFAULT 0 |
| `mtime` | INTEGER | NOT NULL |
| `content_hash` | TEXT | NULL |
| `parse_status` | TEXT | NOT NULL DEFAULT `pending` |
| `index_status` | TEXT | NOT NULL DEFAULT `pending` |
| `error_code` | TEXT | NULL |
| `error_message` | TEXT | NULL |
| `metadata` | TEXT | NULL |
| `created_at` / `updated_at` | INTEGER | NOT NULL |

唯一/索引：`UNIQUE(kb_id, rel_path)`；`INDEX(kb_id, parent_id)`；`INDEX(kb_id, kind)`；`INDEX(kb_id, content_hash)`；`INDEX(kb_id, parse_status)`；`INDEX(kb_id, index_status)`。

**`file_hash`**（文件级哈希，独立表——立项报告 2.4）

| 列 | 类型 | 约束 |
|---|---|---|
| `file_id` | TEXT | PK, FK→`file_node` ON DELETE CASCADE |
| `kb_id` | TEXT | NOT NULL |
| `rel_path` | TEXT | NOT NULL |
| `content_hash` | TEXT | NOT NULL |
| `algorithm` | TEXT | NOT NULL DEFAULT `sha256` |
| `size` | INTEGER | NOT NULL |
| `mtime` | INTEGER | NOT NULL |
| `hashed_at` | INTEGER | NOT NULL |

唯一/索引：`UNIQUE(kb_id, rel_path)`；`INDEX(kb_id, content_hash)`。

**`chunk`**（块内容与块哈希**同表**——立项报告 2.4，禁止拆出 `chunk_hash` 表，DEC-20261026-09）

| 列 | 类型 | 约束 |
|---|---|---|
| `chunk_id` | TEXT | PK |
| `kb_id` | TEXT | NOT NULL |
| `file_id` | TEXT | NOT NULL, FK→`file_node` ON DELETE CASCADE |
| `seq` | INTEGER | NOT NULL |
| `content` | TEXT | NOT NULL |
| `content_hash` | TEXT | NOT NULL |
| `source_location` | TEXT | NOT NULL（JSON） |
| `chunk_type` | TEXT | NOT NULL |
| `parent_id` | TEXT | NULL, FK→`chunk.chunk_id` |
| `token_count` | INTEGER | NULL |
| `metadata` | TEXT | NULL |
| `created_at` | INTEGER | NOT NULL |

唯一/索引：`UNIQUE(kb_id, file_id, seq)`；`INDEX(kb_id, file_id)`；`INDEX(kb_id, content_hash)`；`INDEX(kb_id, chunk_type)`。

**`settings`**（库级设置，单行）

| 列 | 类型 | 约束 |
|---|---|---|
| `kb_id` | TEXT | PK |
| `json` | TEXT | NOT NULL（`KbSettings`） |
| `updated_at` | INTEGER | NOT NULL |

**`sync_task`**

| 列 | 类型 | 约束 |
|---|---|---|
| `task_id` | TEXT | PK |
| `kb_id` | TEXT | NOT NULL |
| `status` | TEXT | NOT NULL（`SyncTaskStatus`） |
| `trigger` | TEXT | NOT NULL（`manual/watch/active`） |
| `scope` | TEXT | NOT NULL（JSON） |
| `total` / `done` / `failed` | INTEGER | NOT NULL DEFAULT 0 |
| `errors` | TEXT | NULL（JSON 数组，≤100 条） |
| `created_at` | INTEGER | NOT NULL |
| `started_at` / `finished_at` | INTEGER | NULL |

索引：`INDEX(kb_id, created_at DESC)`；`INDEX(status)`。

**`history`**

| 列 | 类型 | 约束 |
|---|---|---|
| `history_id` | TEXT | PK |
| `kb_id` | TEXT | NOT NULL |
| `query` | TEXT | NOT NULL |
| `mode` | TEXT | NOT NULL（`SearchMode`） |
| `hybrid` | TEXT | NOT NULL（JSON） |
| `top_k` | INTEGER | NOT NULL |
| `result_summary` | TEXT | NOT NULL |
| `hit_count` | INTEGER | NOT NULL |
| `degraded` | INTEGER | NOT NULL DEFAULT 0 |
| `created_at` | INTEGER | NOT NULL |

索引：`INDEX(kb_id, created_at DESC)`；`INDEX(kb_id, mode)`。

**`history_hit`**（软引用轻量快照）

| 列 | 类型 | 约束 |
|---|---|---|
| `history_id` | TEXT | PK(复合1), FK→`history` ON DELETE CASCADE |
| `rank` | INTEGER | PK(复合2) |
| `chunk_id` / `file_id` | TEXT | NOT NULL |
| `rel_path` | TEXT | NOT NULL |
| `file_type` | TEXT | NOT NULL |
| `source_location` | TEXT | NOT NULL（JSON） |
| `preview` | TEXT | NOT NULL |
| `score` | REAL | NOT NULL |
| `alive` | INTEGER | NOT NULL DEFAULT 1 |

**`merge_op`**（一次融合操作的整体状态；条目级明细在 `merge_journal`）

| 列 | 类型 | 约束 |
|---|---|---|
| `op_id` | TEXT | PK |
| `source_kb_id` / `target_kb_id` | TEXT | NOT NULL |
| `target_rel_dir` | TEXT | NOT NULL |
| `mode` | TEXT | NOT NULL（`copy`/`move`） |
| `conflict` | TEXT | NOT NULL（`skip`/`overwrite`/`rename`） |
| `status` | TEXT | NOT NULL（`MergeJournalStatus`） |
| `entry_count` / `done_count` | INTEGER | NOT NULL DEFAULT 0 |
| `error` | TEXT | NULL（JSON） |
| `created_at` / `finished_at` | INTEGER | NOT NULL / NULL |

**`merge_journal`**

| 列 | 类型 | 约束 |
|---|---|---|
| `entry_id` | TEXT | PK |
| `op_id` | TEXT | NOT NULL, FK→`merge_op.op_id` ON DELETE CASCADE |
| `seq` | INTEGER | NOT NULL |
| `kb_id_src` / `kb_id_dst` | TEXT | NOT NULL |
| `action` | TEXT | NOT NULL |
| `source_path` / `target_path` | TEXT | NULL |
| `status` | TEXT | NOT NULL（`MergeJournalStatus`） |
| `created_at` | INTEGER | NOT NULL |

索引：`UNIQUE(op_id, seq)`。

**`rebuild_state`**

| 列 | 类型 | 约束 |
|---|---|---|
| `kb_id` | TEXT | PK |
| `op_id` | TEXT | NOT NULL |
| `reason` | TEXT | NOT NULL |
| `step` | TEXT | NOT NULL |
| `done` / `total` | INTEGER | NOT NULL DEFAULT 0 |
| `started_at` / `updated_at` | INTEGER | NOT NULL |

**`schema_migrations`**：`version INTEGER PK, applied_at INTEGER NOT NULL, checksum TEXT NOT NULL`。

### 4.3 一致性与事务规则

1. **单文件重解析**：`BEGIN` → 删旧 `chunk` → 插新 `chunk` → 更新 `file_hash`、`file_node.parse_status`、`file_node.index_status=indexing` → `COMMIT` → 索引 upsert → 更新 `index_status=indexed`（独立语句）。
2. **索引与元数据是最终一致**：检索时若 `chunk_id` 在元数据缺失 → 丢弃该命中（不报错）；若元数据有而索引缺失 → 由对账修复。**元数据库是权威，索引是派生物。**
3. **文件移动**：单事务更新 `file_node.rel_path/parent_id` + `chunk` 无需改（chunk 不含 rel_path）+ `file_hash.rel_path` → 再更新 Qdrant payload 与 Tantivy 存储字段；不重新解析、不重新嵌入。
4. **删除**：先删索引 → 再事务删 `file_node`（级联 `file_hash`/`chunk`）；软删除目录走 `trash` 暂存 24h 后清理（可配置，Phase-2）。
5. **计数**：`kb_meta.stats_*` 作为缓存值，允许最终一致；UI 以 `kb_list` 为准并在 `kb://changed` 后刷新。
6. **迁移**：只追加，不改已合入脚本；`checksum` 不符拒绝打开（`KB_CORRUPTED`）。

### 4.4 版本与迁移策略

| 版本 | 归属 | 规则 |
|---|---|---|
| `contract_version` | 契约 | 见 `contracts/README.md` §2 |
| `schema_version`（库） | A | 从 1 递增；应用只升不降；库版本 > 应用支持 → `KB_SCHEMA_TOO_NEW` |
| `index_version` | B | 每次“全库重建/索引恢复/融合”递增；检索结果携带；与 `kb_meta` 不一致 → 提示重建 |
| `format_version`（导出包） | A | 包格式，与 `schema_version` 同步递增 |

**触发重建的变更**（必须 `requires_rebuild=true`）：`chunk.strategy`、`chunk.max_chars`、`chunk.overlap_chars`、`embedding.model_id`、`embedding.dim`、`parse.mineru_mode`、向量引擎/分词器升级。

**不触发重建**：`display.*`、`search.*`、`watch.*`、`storage_root`（仅新建库生效）、`llm.*`、`history_limit`。

---

## 5. 枚举与状态机

### 5.1 `KbStatus`

`creating` → `ready` ⇄ `busy` → `ready`；`ready` → `missing`（目录被移走/删除）；`ready` → `corrupted`（迁移/校验失败）。
不变量：`busy` 时拒绝 `kb_merge/kb_rebuild/kb_delete/fs_refresh`（`KB_BUSY`）。

### 5.2 文件状态

`ParseStatus`：`pending → parsing → parsed | failed | unsupported`；目录恒 `skipped`；内容未变（hash 相同）从 `pending` 直接 `parsed`。

`IndexStatus`：`pending → indexing → indexed | stale | failed`；`indexed` 在路径/元数据变更后仍为 `indexed`（payload 已同步）；索引不可用时全局置 `stale`。

`FileKind`：`text | markdown | pdf | docx | html | csv | json | notebook | code | image | other`。

| 扩展名 | kind | 解析路径 |
|---|---|---|
| .txt/.log | text | native 直读 |
| .md/.markdown | markdown | native（保留标题层级） |
| .pdf | pdf | MinerU |
| .docx | docx | MinerU |
| .html/.htm | html | MinerU（抽正文） |
| .csv | csv | MinerU/表格读取 → `table` 块 |
| .json | json | native |
| .ipynb | notebook | native：code cells + markdown cells 分别解析（立项报告 2.4） |
| .c/.cpp/.rs/.py/.js/.ts/... | code | native，AST 感知分块 |
| .jpg/.jpeg/.png | image | 直接嵌入（不解析文本；OCR 为【扩展】） |
| 其他 | other | 标记 `unsupported`，UI 提示 |

### 5.3 检索枚举

`SearchMode`：`minimal | summary | test`（命令与 mode 必须一致）。
`SearchScope`：`local | global`（`global` 为【扩展】预留，本期返回 `SEARCH_INVALID_REQUEST`）。
`HybridKind`：`bm25_only | vector_only | hybrid`。
`ChunkType`：`text | heading | table | code | image`。
`Retriever`：`bm25 | vector | rrf`。

### 5.4 同步枚举

`SyncEventKind`：`create | modify | delete | move`；`SyncSource`：`active | passive`。
`SyncScope`：`{"kind":"all"}` 或 `{"kind":"paths","rel_paths":["a/b.md"]}`。
`SyncActionKind`：`parse | chunk | embed | index_upsert | index_delete | index_path_update | meta_update | meta_delete`。

| 事件 | 动作序列 | 备注 |
|---|---|---|
| `create` | parse → chunk → embed → index_upsert → meta_update | hash 已存在可复用嵌入 |
| `modify` | parse → chunk → embed → index_upsert（先删旧 point）→ meta_update | 块级 hash 相同时跳过嵌入 |
| `delete` | index_delete → meta_delete | 目录删除递归 |
| `move` | meta_update（rel_path）→ index_path_update | **不解析、不嵌入**（立项报告 2.4） |

`SyncTaskStatus`：`queued → running → succeeded | partial_failed | failed | cancelled`。
不变量：同库同时最多 1 个 `running` 任务（`SYNC_IN_PROGRESS`）；`partial_failed` 必须带 `errors[]`。

### 5.5 融合 / 长任务枚举

`MergeMode`：`copy | move`（`move` 成功后删除源库）。
`MergeJournalStatus`：`planned → copying → meta_updating → indexing → committed`；异常 → `rolling_back → rolled_back | failed`。
`OpKind`：`kb_create | kb_import | kb_export | kb_merge | kb_delete | kb_rebuild | fs_refresh`。
`OpStatus`：`running → succeeded | partial_failed | failed | cancelled`。
`RebuildReason`：`chunk_strategy | embedding_model | parse_mode | index_corrupt | manual`。
`KbChangeKind`（`kb://changed.what`）：`meta | settings | files | index`。

### 5.6 LLM 枚举

`LlmProvider`：`openai_compat | ollama | llama_cpp | custom`。
`FinishReason`：`stop | length | content_filter | error`。

---

## 6. 导出包布局

```
<name>-<kb_id>-<app_version>.zip
├─ manifest.json
├─ checksums.sha256              # "<sha256>  <相对路径>"
├─ kb/
│  ├─ kb.sqlite3                 # 导出前 checkpoint WAL
│  ├─ files/                     # 完整副本（含 mtime 元数据）
│  ├─ cache/                     # 可选：includes_cache=true 时
│  └─ index/
│     ├─ qdrant.snapshot         # collection 快照
│     └─ tantivy/                # 全文索引目录
```

导出要求：

1. 导出期间对 KB 持写锁（`busy`），UI 显示进度；导出中禁止 `fs_*` 写操作。
2. `files` 的 `content_hash` 必须与 `file_hash` 表一致，否则中止（数据已损坏）。
3. `includes_cache` 默认 `false`；`true` 时 `cache/` 参与 `checksums.sha256`。
4. 导入失败不得污染已有库：全部在 `.import-tmp` 内完成校验。

---

## 7. 解析缓存布局（C）

```
<kb_root>/cache/<file_id>/<content_hash>/
├─ parsed.json        # ParsedDocument 序列化（不含绝对路径）
├─ blocks.jsonl       # 可选：大文档分块流
├─ assets/            # 抽取的图片/表格截图（可被 chunk 引用）
└─ meta.json          # {parser, version, mode, created_at, source_hash}
```

规则：

1. 缓存键 = `(file_id, content_hash, parser_version, parse_options_hash)`；任一变化即未命中。
2. 缓存**只增不改**；删除文件时随 `file_node` 级联清理。
3. `parsed.json` 中的路径必须是 `rel_path`；禁止写入绝对路径（导出/迁移会失效）。
4. 缓存可安全删除：删除后行为等价于首次解析（`cache_enabled=false` 时不写缓存）。
5. 导出时按 `ExportRequest.include_cache` 决定是否打包（默认否）。

---

## 8. 数据边界与隐私

1. `files/` 副本是用户敏感数据，任何日志/事件/错误详情不得包含文件正文；`preview` 只出现在显式 API 响应中。
2. LLM API 请求只允许发送：`query` + 检索到的 `context_blocks` + 指令；**禁止**附带 `rel_path` 之外的绝对路径、用户目录结构、API 密钥。
3. `rel_path` 与文件名可以发送（作为引用标注），若用户开启“脱敏模式”（Phase-2），替换为 `文档1/文档2`。
4. 密钥存储：不得明文入 SQLite；使用系统凭据库（Windows Credential Manager / libsecret），不可用时退化为 AES-GCM + 主密钥文件（权限 0600）。
5. 删除知识库 = 删除 `root_dir` + Qdrant collection + Tantivy 目录 + 注册表行，四者必须全部成功或全部失败（失败保留注册表并提供“清理残留”）。
