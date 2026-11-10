# 契约评审记录 · `contract_version 0.1.0`（PX-02）

> 目的：把“契约冻结”从口头约定变成可核对的签字流程。状态：**进行中**（A 自查完成，B/C/D 待签）。
> 依据：`AGENTS.md` §4.2（契约唯一路径）、`contracts/README.md` §2（版本化）、`任务分工建议.md` §四 W0（审核分工）。

## 1. 评审对象

| 文件 | 内容 | 主笔 | 审核 |
|---|---|---|---|
| `contracts/interface-contracts.md` | trait、39 条命令、7 个事件、30 个错误码、sidecar 协议、时序 | A | B（§3.7/§3.8）、C（§6）、D（§4/§5） |
| `contracts/data-model.md` | 磁盘布局、对象/枚举、SQLite 12+3 表、Qdrant/Tantivy、导出包、解析缓存 | A | B（§3）、C（§2.4–§2.6、§7） |
| `contracts/architecture-boundaries.md` | 分层、依赖矩阵、22 条 BAN、共享模块治理 | A | 全员 |
| `contracts/schemas/` | 五件套 + 示例（PA-03） | A | B/C/D 按字段 |
| `crates/contracts` | 类型的唯一代码形态（PA-05） | A | 同上 |

## 2. 冻结条件（全部满足才可标“完成”）

- [x] A 自查：五条 CI 闸门在本地/CI 全绿（见 §3）
- [x] 文档与代码同版本：`contracts/*.md` 三处 + `CHANGELOG` + `crates/contracts/src/version.rs`
- [x] B 签字：检索/索引/LLM/历史接口 + Qdrant payload/进程契约（DEC-20261031-25）——2026-11-09，附条件（见 §4.4）
- [ ] C 签字：sidecar 协议 + `ParsedDocument`/`FileNode`/`DocumentChunk`/`SyncEvent` + 缓存布局 + `BlockType` 大小写（DEC-20261031-26）
- [ ] D 签字：39 条命令 / 7 个事件的前端可用性、生成物与 mock 规范
- [ ] 无 `Disputed` 状态的 DEC；`Proposed` 项已转 `Accepted` 或明确推迟到 0.1.1

## 3. A 的自查结果（2026-10-31）

| 检查 | 命令 | 结果 |
|---|---|---|
| 契约版本四处一致 | `pwsh scripts/validate-contracts.ps1` | ok（`contracts/*.md` 三处 + CHANGELOG + `version.rs` 均为 0.1.0） |
| schema 与示例 | 同上（`validate-schemas.py`） | ok（5 个 schema + 6 个示例） |
| 命令表 39 条 | 同上 | ok（`interface-contracts.md` §4 ↔ `commands/table.rs`） |
| 错误码 30 个 | 同上 | ok（§2 ↔ `crates/contracts/src/error.rs`） |
| 事件名 7 个 | 同上 | ok（§5 ↔ `crates/contracts/src/events.rs`） |
| 依赖方向 | `pwsh scripts/check-deps.ps1` | ok（13 个 workspace 包） |
| 禁令扫描 | `pwsh scripts/check-boundaries.ps1` | ok（44 个 Rust 文件） |
| 迁移空库执行 | `pwsh scripts/check-migrations.ps1` | ok（app 3 表/1 索引；kb 12 表/17 索引；`SCHEMA_VERSION=1`） |
| 许可登记 | `pwsh scripts/check-licenses.ps1` | ok（12 个已声明依赖） |
| Rust 类型检查 | 直接调用 `rustc --emit=metadata` 编译 `crates/contracts` / `crates/core` / `apps/desktop/src-tauri`（本机沙箱内 `cargo` 无法 spawn `rustc`，用预构建依赖绕过） | ok（三 crate 均无错误） |
| Rust lint | `clippy-driver -W clippy::all -D warnings`（同上方式） | ok（无告警） |
| Rust 格式 | `rustfmt --check`（逐文件） | ok；CI 仍以 `cargo fmt --all --check` / `cargo clippy --workspace` 为准 |

## 4. 各 owner 审核清单

### 4.1 B（检索与 AI 链路）

- [x] `interface-contracts.md` §3.7 索引接口：`IndexService` 双写顺序、`ScoredChunk` 不含预览、快照/恢复语义
- [x] §3.8 检索/LLM/历史：三模式流程、降级规则、`Usage`、`LlmProbeResult`
- [x] `data-model.md` §3.1 Qdrant payload 字段与索引标记；§3.1.1 进程契约（DEC-20261031-25）
- [x] §3.2 Tantivy schema 与分词（`jieba` 降级路径）
- [x] `search.rs`/`llm.rs`/`index.rs`/`history.rs` 的字段名与默认值

> B 审核结论（2026-11-09）：以上 5 项逐条核对完毕，无异议项与待补缺口分别见 §4.4.1 / §4.4.2。

### 4.2 C（数据管线与同步）

- [ ] `interface-contracts.md` §6 sidecar：方法集、`-320xx` 错误映射、生命周期、`max_parallel`
- [ ] `data-model.md` §2.4 `ParsedDocument`/`Block`、§2.5 `DocumentChunk`、§2.10 `SyncEvent`、§7 缓存布局
- [ ] `sidecar.schema.json` 与 `ParsedDocument` 字段一致；DEC-20261031-26 的 `BlockType` 大写
- [ ] `parse.rs`/`sync.rs`/`file.rs` 的字段与枚举取值

### 4.3 D（前端基础设施与质量）

- [ ] `interface-contracts.md` §4 命令清单（39 条）与 §5 事件载荷是否满足前端需求（分页、`has_more`、`preview_truncated`、`tab_id`）
- [ ] 错误载荷 `AppError` 是否可直接驱动 UI（`retryable`、`user_message_key`）
- [ ] 生成链约束：`crates/contracts` 的 `ts` feature、`events.schema.json` 的可枚举性、mock 规范位置
- [ ] `schemas/app-settings.schema.json` 与设置页字段一致（`max_tabs` 固定 5、`preview_chars` 80..2000）

### 4.4 B 的审核意见（2026-11-09）

> 审核范围：`interface-contracts.md` §3.7 / §3.8；`data-model.md` §3.1 / §3.1.1 / §3.2，以及 §2.6 / §2.7 / §2.16 中 owner 为 B 的类型；`schemas/kb-settings.schema.json`、`schemas/events.schema.json`。
> 结论：**有条件同意冻结**。§3.1.1 七条均可在 `crates/index` 落地，其余段落无字段级异议；下列 5 条缺口不否定冻结，但会阻塞 W1–W3 的实现，须按标注时点补齐。

#### 4.4.1 已确认（无异议）

| # | 项 | 结论 |
|---|---|---|
| 1 | §3.7 双写顺序与失败语义 | 先 Qdrant `upsert`、再 Tantivy `commit`；任一失败标 `index_status=stale` 并回 `INDEX_UNAVAILABLE`（retryable），不写一半报成功。重试以 `chunk_id` 为幂等键，与 §3.1.1「已成功批次不重放」自洽，可实现。 |
| 2 | §3.7 `ScoredChunk` 不含预览 | 预览唯一来源 `chunk.content`，避免两套文本源，同意（须配套 F3 的批量端口，否则 P-01 不达）。 |
| 3 | §3.8 模式表与默认值 | `hybrid{bm25:0.5, vector:0.5}, rrf_k=60`（10..=200）、`top_k=10`（1..=50）、Summary 降级路径（`degraded=true` + 拼接上下文 + 原样引用 + `warnings`），按此实现。 |
| 4 | §3.1 Qdrant payload | 字段命名与索引标记合理；`content` 不入 payload、`source_location` 存 JSON 串、移动只改 `rel_path`，同意。**但缺 filters 下推所需字段，见 F6**。 |
| 5 | §3.1.1 Qdrant 进程契约（PX-01） | 按 DEC-20261031-25 的七条逐项核对：①二进制位置与 `config.yaml` 覆盖写、只用 HTTP REST 且绑回环；②端口 6333→6333..6342→系统分配，`runtime.json` 为唯一运行期来源；③`lock` 排他 + `runtime.json` 探活，残留锁可接管；④`/readyz` 200ms×30s、`/healthz` 10s×3；⑤重启退避 1/2/4s、5 分钟窗口上限 3 次；⑥关闭 SIGTERM（Windows `taskkill` 不带 `/F`）等 10s、空闲 30 分钟回收；⑦数据目录迁移 5 步 + 版本升级两条路径。均在 `crates/index` 实现范围内，**B 确认 DEC-20261031-25**，落地时在 PR 逐条勾选。 |
| 6 | §3.2 Tantivy schema | 8 字段 + `jieba`（缺依赖降级 ngram 且必须写 `warnings`）+ `k1=1.2, b=0.75` + 每批 `commit`/`reload`，同意。三项依赖在 `LICENSES/THIRD-PARTY.md` §3 已登记（状态「待确认」），仍需一条 DEC 覆盖引入并给出许可结论（BAN-20），B 在 W1 前补。 |
| 7 | `index://status` 事件 | 载荷 `{kb_id, indexed, total, failed, state, last_error?}`、`state ∈ idle\|indexing\|stale`、不含端口，满足前端展示且符合 BAN-16，同意。 |

#### 4.4.2 待补缺口（按截止时点排序）

| # | 缺口 | 证据 | 影响 | 建议处置 | 时点 |
|---|---|---|---|---|---|
| F1 | **查询向量无生产方** | §3.8 只收 `query: String`，`SearchRequest` 亦无 vector 字段；§3.7 `search_vector` 需 `VectorQuery.vector`；依赖矩阵禁止 `search` 依赖 `pipeline`，而 `Embedder`（§3.5）在 pipeline 职责内 | 三模式第一步即断链，Minimal 也跑不通；不是实现细节，是端口归属问题，须 A 裁决 | 三选一（成本递增）：①允许 `search` 依赖 `pipeline` 的 `Embedder` 端口并在 DEC 说明「BAN 只禁解析、不禁嵌入」；②端口 trait 下沉 `crates/contracts`（须同步放宽 contracts 的外部库限制，现仅 serde/uuid/ts-rs）；③新建 `crates/embed` 只放端口。无论哪种，`SearchService` 构造注入 `Arc<dyn Embedder>`，装配在 `apps/desktop` | W1 前（MINOR） |
| F2 | **`index_version` 的递增职责判给 B，但 B 无路径** | `data-model.md` §4.2 有该列、§4.4 判 owner=B「重建/恢复/融合递增」；而 §3.3 `MetadataRepo` 只暴露 `kb_meta_get`，BAN-03 禁止非 `storage` 的 crate 自建 SQLite 连接或拼 SQL | 责任与权限不匹配：`SearchResult.index_version` 与陈旧检测落不了地（D 的历史复现亦依赖） | **建议①**：A 在 `MetadataRepo` 增 `kb_meta_incr_index_version(kb_id) -> u64`（单事务 +1 返回新值）。理由：递增点有两个来源——A 的重建/融合、B 的索引恢复，只改 owner 解决不了 B 的那一半。备选②：`data-model.md` §4.4 该行 owner 改 A，B 只读 | W1 前（MINOR） |
| F3 | **预览补齐缺批量端口**（建议项，不阻塞） | §3.7 预览走 `MetadataRepo.chunk_get`；§3.3 仅单条 `chunk_get`；往返次数 = 返回命中数（默认 `top_k=10`，上限 50），候选 100 条只参与打分、不需要预览 | 单次检索 10～50 次点查，P-01 预算内可接受；但 P-04 五标签并发时按标签数线性放大 | 增 `chunk_get_many(kb_id, ids) -> Vec<DocumentChunk>`（单次 `IN` 查询） | W2 前（MINOR，可与 F1 合并一次补） |
| F4 | **历史列表返回类型自相矛盾** | §4 命令表 `history_list` 入参已是 `{kb_id, filter, offset, limit}`、返回 `HistoryPage`；而 §3.8 `HistoryService::list` 仍返回 `Vec<HistoryRecord>`，`HistoryPage{items,total,has_more}` 在 §2.16 注册（owner B）却无生产方 | 命令层要分页、服务层给不出 `total`/`has_more`；冻结后再改 trait 返回类型即 MAJOR | **冻结前**改为 `list(kb_id, f: HistoryFilter, offset: u32, limit: u32) -> Result<HistoryPage>`；同时 `trim(keep, max_bytes)` 与 `MetadataRepo::history_trim(keep)` 参数不一致，一并补齐 | **0.1.0 冻结前**（实现未写，成本为 0） |
| F5 | **`update_path` 缺 `content`，Tantivy 侧无法重建文档** | §3.7 签名 `update_path(kb_id, file_id, rel_path)`、注释「只改 payload/字段」；§3.2 `rel_path` 是 STORED 字段、Tantivy 无 update API（只能 delete + add，add 需 `content`）；依赖矩阵禁止 `index` 依赖 `storage`，`crates/index` 自己取不到 `chunk.content`（BAN-07 又限定语料只能来自它）；§5.4 又把 `move` 定义为「不解析、不嵌入」、只走 `meta_update → index_path_update` | 移动文件后 BM25 仍指向旧路径、跳转错文件；由于 move 明令不重新解析，现有签名下**没有任何合法实现路径** | 签名改为批量并带正文：`update_path(kb_id, items: &[PathUpdate{chunk_id, rel_path, content}])`，由调用方 `sync`（允许依赖 storage + index）从 `chunk_get` 取 `content` 传入；失败标 `stale` | **0.1.0 冻结前**（只改签名，实现未写，成本为 0） |
| F6 | **`SearchFilters` 有两个条件无法下推** | `SearchFilters` 含 `mtime_after` 与 `rel_path_prefix`（§2.7）；但 Qdrant payload 11 字段里**无 `mtime`**、Tantivy 8 字段里也无；`rel_path` 在 payload 侧是 keyword（仅精确匹配，无前缀语义；Tantivy 侧 raw 可做前缀查询，但与向量侧无法对齐）。契约全文未定义 filters 如何下推到索引 | `mtime_after` 只能事后回查元数据库（每命中一次点查，且过滤后可能不足 `top_k`，静默丢召回）；`rel_path_prefix` 若不定义策略，实现会各自发挥（全量后过滤 / 忽略） | ①payload 与 Tantivy 各加 `mtime`(integer/epoch ms)；②`rel_path_prefix` 二选一：加 `rel_dir` 字段（按目录精确匹配）或明确「由 `search` 取候选后本地前缀过滤 + 写 `warnings`」。`IndexItem` 需同步加 `mtime` 字段 | W1 前（MINOR，趁索引无数据，加了不用重建） |
| F7 | **检索侧 `warnings[].code` 无枚举**（建议项） | §2.7 `SearchResult.warnings: [{code,message}]`，但 §5.3 检索枚举里没有 `WarningCode`；对比 `ParseWarningCode` 与 30 个错误码都有大写枚举 | 前端无法按 code 分支渲染（降级、分词 fallback、过滤被忽略都只能靠 message 猜） | 在 §5.3 补 `WarningCode`：`BM25_TOKENIZER_FALLBACK`、`FILTER_POST_APPLIED`、`LLM_DEGRADED`、`HISTORY_TRUNCATED` | W1 前（PATCH/MINOR） |

> 签字附带条件：**F4、F5** 由 A 在 `contract_version 0.1.0` 冻结前修订（均为 trait 签名调整，实现未写，成本为 0）；**F1、F2、F6、F7** 归入 0.1.1，其中 F1/F2/F6 必须在 W1 动手前定（否则 `crates/index` 的 `IndexItem` 与 `SearchService` 装配要返工）；**F3** 为建议项，可与 F1 合并一次补。B 在 W1 动 `crates/index` 时按现有契约先行，不以此为阻塞。

## 5. 签字

| 角色 | 结论 | 日期 | 备注 |
|---|---|---|---|
| A | 同意冻结（待全体签字） | 2026-10-31 | 见 §3 自查 |
| B | 有条件同意冻结（F4/F5 冻结前改；F1/F2/F6 须 W1 前定；F3/F7 从容） | 2026-11-09 | 已确认 DEC-20261031-25；意见见 §4.4 |
| C | 待签 | — | 需先确认 DEC-20261031-26 |
| D | 待签 | — | 需确认命令/事件可用性 |

签字方式：在本表追加一行结论（或 PR approve 并在 `decisions.md` 的对应 DEC 的 `确认` 字段留言），然后在 `STATUS.md` §3.5 把 PX-02 置 `完成`，并在 `contracts/CHANGELOG.md` 顶部把 0.1.0 标为“已冻结”。

## 6. 已知遗留（不阻塞本轮签字，但必须记录）

| 事项 | 归属 | 计划 |
|---|---|---|
| `crates/contracts` 的 `ts` feature 未在 CI 编译（生成链是 D 的 PD-03） | D | PD-03 落地后由 `gen-diff` 作业覆盖 |
| `apps/ui` 尚未建立 → 前端侧禁令扫描与 `version.ts` 版本比对暂为 `[skip]` | D | PD-02 落地后自动生效 |
| `cargo build`/`clippy` 未在评审机验证（无 crates 缓存） | A | CI `rust` 作业（ubuntu + windows） |
| sidecar 黄金报文样例未落地（PC-02） | C | W0 结束前补到 `tests/mocks/` |
