# 架构边界与禁令（architecture-boundaries）

> 版本：`contract_version = 0.1.0`　|　owner：A 主持，全员评审　|　状态：Phase-0 冻结候选
> 本文件回答三件事：**什么层不能直接调用什么层**、**共享模块怎么改**、**哪些做法一律禁止**。
> 冲突时以 `立项报告草稿.md` 的四层架构（表现层 / 应用服务层 / 核心模块层 / 数据持久层）为准。

---

## 1. 分层与目录映射

| 层 | 立项报告对应 | 目录 | 职责 | 所有者 |
|---|---|---|---|---|
| **L0 契约** | — | `contracts/`、`crates/contracts` | 类型、枚举、错误码、schema；无业务逻辑 | A 主持 |
| **L1 基础** | — | `crates/core` | ID/时间/哈希/路径规范化、`AppError` 构造、`EventSink`/`CancelToken` 抽象 | A |
| **L2 引擎** | 核心模块层 | `crates/storage`、`crates/pipeline`、`crates/index`、`crates/llm` | 解析/分块/嵌入、SQLite、Qdrant/Tantivy、LLM 调用 | A/C/B/B |
| **L3 应用服务** | 应用服务层 | `crates/kb`、`crates/settings`、`crates/fs`、`crates/sync`、`crates/search`、`crates/history` | 用例编排、事务边界、跨引擎组合 | A/A/C/C/B/B |
| **L4 外壳** | 表现层 | `apps/desktop/src-tauri`、`apps/ui` | 命令注册与 DTO、React 页面与组件 | A/D |
| **P 外部进程** | — | `python/parsers`（sidecar）、Qdrant 进程 | 解析、向量存储；只能通过约定协议交互 | C（协议）/ B（Qdrant） |

依赖方向**自上而下**：L4 → L3 → L2 → L1 → L0。任何反向依赖都是架构违规。

```
apps/ui ──(generated TS only)──> apps/desktop/src-tauri ──> L3 services ──> L2 engines ──> L1 core ──> L0 contracts
                                                                  │
                                                                  └──(stdio JSON-RPC)──> python/parsers
                                                                  └──(HTTP 127.0.0.1)──> Qdrant
```

---

## 2. 依赖矩阵（crate 级，CI 强制）

| crate | 允许依赖（workspace 内） | 允许的外部库（关键） | 禁止依赖 |
|---|---|---|---|
| `contracts` | **无** | `serde`, `uuid`, `ts-rs` | 任何 workspace crate（防循环） |
| `core` | `contracts` | `serde`, `uuid`, `sha2`, `thiserror`, `tokio`(仅同步原语) | `tauri`, `rusqlite`, `qdrant-client`, `tantivy`, 任一 L2/L3 |
| `storage` | `contracts`, `core` | `rusqlite`/`sqlx`, `serde_json` | `tauri`, `qdrant-client`, `tantivy`, `ort`, 任一 L3 |
| `pipeline` | `contracts`, `core`, `storage`(仅 `MetadataRepo`/`CacheStore` 端口) | `ort`/`onnxruntime`, `image` | `tauri`, `qdrant-client`, `tantivy`, 任一 L3 |
| `index` | `contracts`, `core` | `qdrant-client`, `tantivy`, `tantivy-jieba` | `tauri`, `rusqlite`, `storage`, `pipeline`, 任一 L3 |
| `llm` | `contracts`, `core` | `reqwest`, `serde_json` | `tauri`, `rusqlite`, `qdrant-client`, `tantivy`, 任一 L3 |
| `fs` | `contracts`, `core`, `storage` | `walkdir`, `trash`(可选) | `tauri`, `pipeline`, `index`, `sync`, `search`, `kb` |
| `sync` | `contracts`, `core`, `storage`, `pipeline`, `index`, `fs` | `notify`, `sha2` | `tauri`, `kb`, `search`, `llm`, `history` |
| `search` | `contracts`, `core`, `storage`, `index`, `llm` | `serde_json` | `tauri`, `pipeline`(禁止解析), `kb`, `fs`, `sync` |
| `history` | `contracts`, `core`, `storage` | — | `tauri`, `index`, `llm`, `search`, `kb` |
| `kb` | `contracts`, `core`, `storage`, `pipeline`, `index`, `fs`, `settings`, `sync` | `zip`, `sha2` | `tauri`, `search`(禁止编排检索) |
| `settings` | `contracts`, `core`, `storage` | `keyring`/`aes-gcm` | `tauri`, `index`, `pipeline` |
| `apps/desktop` | 全部 L3 + `tauri` | `tauri`, `tauri-specta` | `qdrant-client`, `rusqlite`, `tantivy`, `ort`（直接依赖即违规） |
| `apps/ui` | 无 Rust | `react`, `@tauri-apps/api` | 手写 IPC 类型、直接 `fetch` 外部服务 |

**说明**：`apps/desktop` 允许 `tauri` 但**不得**直接依赖引擎库；它只能通过 L3 trait 工作。`index` 与 `pipeline` 互不依赖——两者由 L3 编排（`sync`/`kb`）。

---

## 3. 调用规则与禁令

### 3.1 允许的跨层调用（白名单）

| 调用方 → 被调方 | 允许形态 |
|---|---|
| UI → shell | 仅 `invoke(<命令名>)`，命令名必须存在于 `interface-contracts.md` §4 |
| shell → L3 | 仅调用 `contracts` 中登记的 trait；每命令 ≤ 20 行，无分支业务 |
| L3 → L2 | 通过 trait；L3 负责事务与顺序，L2 不感知业务语义 |
| L3/L2 → 事件 | 仅 `EventSink::emit()`（L1 定义），由 shell 实现为 Tauri emit |
| L3 → sidecar | 仅通过 `DocumentParser` trait（C 的 `SidecarParser` 实现），不直接 spawn |
| `index` → Qdrant | 仅 HTTP/gRPC API；禁止读写 Qdrant 数据文件格式 |
| `storage` → SQLite | 唯一入口；`PRAGMA` 与迁移也在此 crate |

### 3.2 禁令表（BAN）

| 编号 | 禁令 | 原因 | 反例 | 正确做法 |
|---|---|---|---|---|
| **BAN-01** | 表现层不得直接读 `kb.sqlite3`、`files/`、`cache/` | 绕过事务与状态机，破坏隔离 | React 侧 `open(kb.sqlite3)` | 调 `fs_*`/`search_*` |
| **BAN-02** | 表现层不得直接访问 Qdrant、LLM、sidecar | 破坏本地优先与密钥边界 | 前端 `fetch('http://127.0.0.1:6333')` | 调 `search_*` / `llm_probe` |
| **BAN-03** | 非 `crates/storage` 的任何 crate 不得建立 SQLite 连接或拼 SQL | 单写者原则、迁移可控 | `crates/search` 用 `rusqlite` 查 chunk | 扩展 `MetadataRepo`（PR） |
| **BAN-04** | 非 `crates/index` 不得 `use qdrant_client` / `tantivy` | 索引实现可替换（NFR-05） | `crates/kb` 直接 delete collection | `IndexService` |
| **BAN-05** | L1–L3 不得依赖 `tauri`、不得直接 emit 事件 | 可测试性、可迁移性 | `crates/sync` 里 `app.emit("sync://progress")` | `EventSink::emit` |
| **BAN-06** | shell 不得承载业务逻辑（解析/分块/拼接 prompt/算 hash/重试编排） | shell 只做适配 | `kb_create` 里写 while 解析循环 | 调 `KnowledgeBaseService::create` |
| **BAN-07** | BM25 与向量语料不得取自解析原始产物或原文文件，必须取自 `chunk.content` | 立项报告 2.4 明确以元数据库块内容为准 | Tantivy 直接索引 `ParsedDocument.blocks.text` | 从 `chunk` 表读 content |
| **BAN-08** | 前端不得手写/复制 IPC 类型或 DTO | 类型漂移 | 手写 `interface SearchResult` | 使用生成的 `lib/tauri-api` |
| **BAN-09** | 不得跨 crate 访问 `pub(crate)`/私有模块；公共 API 必须从 `lib.rs` re-export | 边界可审计 | `crate::internal::…` 直连 | 在 owner 处加公共 API（PR） |
| **BAN-10** | sidecar 不得写 `kb.sqlite3`、不得直连 Qdrant、不得修改 `files/` 原文件 | 单一写者；崩溃安全 | Python 写 chunk 表 | 只返回 `ParsedDocument` |
| **BAN-11** | 不得无界并发调用解析/嵌入/索引 | 内存与 CPU 打爆（P-05） | `join_all(10_000 files)` | 有界队列（默认 2/2/1） |
| **BAN-12** | sidecar 不得接受未校验路径；Rust 侧必须做 canonicalize + 前缀校验 | 路径逃逸 | 直接透传用户字符串 | 校验在 `fs`/`pipeline` 边界 |
| **BAN-13** | `pipeline` 不得调用 `index`；`index` 不得调用 `pipeline` | 防循环；编排归 L3 | 解析完直接写向量库 | 由 `sync`/`kb` 编排 |
| **BAN-14** | 增量同步不得绕过 `Chunker` 自行切分或直接改 `chunk.content` | 分块策略一致性、可重建 | 同步里手工 substring | 复用 `Chunker`（纯函数） |
| **BAN-15** | 不得修改已合入的 migration；迁移只追加 | 版本可复现 | 改 v3 脚本再发布 | 新增 v4 |
| **BAN-16** | 日志/事件/错误详情不得含文件正文、完整 prompt、API 密钥、绝对路径（`root_dir` 除外） | 隐私（NFR-06/DR-04） | `tracing::info!(body)` | 只记 `file_id`/长度/哈希 |
| **BAN-17** | 事件不得作为数据源；订阅方必须回拉权威状态 | 事件会丢、会重排 | UI 只靠事件渲染结果 | 事件→触发 `invoke` 刷新 |
| **BAN-18** | 未经 PR 与 `decisions.md` 记录，不得修改共享模块（§4 清单） | 协作基线 | 直接改 `components/` | 走 §4 流程 |
| **BAN-19** | 不得在代码/测试中硬编码绝对路径、用户名、固定端口 | 跨平台（C-01/02）、可移植 | `C:/Users/foo/...` | 用临时目录 + 相对路径 |
| **BAN-20** | 新增第三方依赖必须先在 `decisions.md` 记录并登记 `LICENSES/` | 许可合规（NFR-09） | 随手加 `pdfium` | DEC + LICENSES PR |
| **BAN-21** | 不得在前端多标签间共享可变检索状态（除显式 `tab_id` 隔离） | 并行检索互不干扰（P-04） | 全局 store 存单份结果 | 每 tab 独立状态 |
| **BAN-22** | 不得在应用服务层做**展示性**文案/格式化（单位换算、千分位、日期本地化、按界面宽度换行） | 关注点分离、便于多端复用 | 后端把 `elapsed_ms` 格式化成“1.2 秒” | 后端给原始值，前端渲染。**例外**：`SearchHit.preview` 截断属传输契约（`options.preview_chars`），必须同时返回 `preview_truncated` |

### 3.3 例外与豁免

- 确需突破禁令：在 `decisions.md` 提 `DEC`（状态 `Proposed`），写明**禁令编号、原因、影响范围、移除计划与截止周**，A + 受影响 owner 批准。
- 豁免必须有截止周；到期未移除 → CI 任务列表里标记为技术债，集成日优先处理。
- 禁止“口头豁免”和“临时改一下”。

---

## 4. 共享模块治理（"共享模块的修改必须走 PR 且通知对方"）

### 4.1 共享模块清单

| 共享模块 | 说明 | 主 owner | 共同 owner |
|---|---|---|---|
| `contracts/**`（含 `CHANGELOG.md`、`schemas/`） | 接口/数据/边界契约 | A | 全员（按段落） |
| `crates/contracts/**` | Rust 领域类型单一事实源 | A | B/C/D（各自字段） |
| `crates/core/**` | ID/时间/哈希/路径/`EventSink`/`AppError` | A | 全员 |
| `crates/storage/migrations/**` | 全库迁移脚本 | A | B/C 提需求 |
| `apps/ui/src/lib/tauri-api/**` | 生成的 IPC client（禁止手改） | D | A（生成器） |
| `apps/ui/src/components/**` | 前端共享组件 | D | A/C（复用方） |
| `tests/integration/**`、`tests/fixtures/**` | 集成测试与样例数据 | D | 全员 |
| `scripts/**`、`.github/workflows/**` | 生成/校验/CI | A | D |
| 根 `Cargo.toml`、`rust-toolchain.toml`、`package.json`、`pnpm-workspace.yaml` | 工作区与工具链 | A | 全员 |
| `AGENTS.md`、`decisions.md`、`contracts/README.md` | 协作规则本身 | A | 全员 |

### 4.2 变更流程（固定 6 步）

1. **记录意图**：在 `decisions.md` 追加 `DEC`（`Proposed`），写清：动机、影响模块、破坏性、迁移步骤、截止时间。
2. **开 PR**：标题 `[shared] <module>: <变更摘要>`；标签 `shared-module`；描述必填 `Affects:`、`DEC:`、`Contract-Version:`（若涉及契约）。
3. **通知对方**：PR 描述 `Notify: @<所有受影响 owner>`；同时在 `decisions.md` 条目的“通知”字段列出被通知人。**没有通知记录即视为未通知。**
4. **评审**：至少 2 个 approve —— 模块主 owner + A；涉及前端共享组件必须含 D；涉及存储必须含 A + 提出需求的 B/C；MAJOR 契约变更需所有受影响 owner 显式确认。
5. **异步 SLA**：非阻断性评审 24 小时内响应，48 小时无异议视为通过（MAJOR 与安全相关除外，必须显式确认）。
6. **合入后**：24 小时内更新 `contracts/CHANGELOG.md`（版本号）、必要的模块 README 与 `AGENTS.md` 指针；CI 必须全绿。

### 4.3 禁止行为

- 直接 push 到 `main`；对共享模块 force push；把共享模块改动混进“顺手重构”PR。
- 在未更新 `decisions.md` 的情况下修改共享模块语义。
- 手改 `apps/ui/src/lib/tauri-api/**`（生成物，改了下次生成即被覆盖）。
- 只改 `crates/contracts` 不改 `contracts/*.md`（或反之）——两者必须同一 PR/同一版本。

### 4.4 争议仲裁

1. 先回到 `立项报告草稿.md`（需求与验收优先）。
2. 再看 `contracts/`（已冻结约定）。
3. 仍无共识 → A 主持 15 分钟同步会议（或异步投票，48h），结论写入 `decisions.md`（`Accepted` 或 `Rejected`，写清理由）。
4. 不得以“先合了再说”绕过仲裁。

---

## 5. 数据所有权矩阵（唯一写者原则）

| 数据 | 唯一写者 | 读者 | 备注 |
|---|---|---|---|
| `app.sqlite3` 全部表 | `crates/settings`/`crates/kb`（经 `storage`） | 所有 L3 | 只有 `storage` 落地 SQL |
| `kb.sqlite3` 全部表 | `crates/storage`（按调用方语义） | 所有 L3 | 检索/历史只读或经服务 |
| `kb_meta`、`kb_registry` | A（`kb`/`settings`） | 全员 | — |
| `file_node`、`file_hash` | C（`fs`/`sync`），经 `storage` | A/B/D | D 只读 |
| `chunk` | C（`pipeline`+`sync` 编排） | B/A/D | 每文件单事务替换 |
| `history`/`history_hit` | B（`history`） | D | 库内，无跨库 |
| `sync_task`、`rebuild_state` | C（`sync`）/A（重构） | D | — |
| `merge_op`/`merge_journal` | A（`kb`） | A | 回滚依据 |
| Qdrant collection | B（`index`） | B | C 不得直连 |
| Tantivy 目录 | B（`index`） | B | 同上 |
| `files/` 副本 | C（`fs`） | 全员只读 | 只有 `fs` 可写 |
| `cache/` | C（`pipeline`） | C | 可安全删除 |
| LLM 密钥 | A（`settings`，系统凭据库） | B（只读使用） | 永不落 JSON/日志 |

---

## 6. 跨进程与外部边界

| 边界 | 规则 |
|---|---|
| Rust ↔ sidecar | 只走 NDJSON JSON-RPC（`interface-contracts.md` §6）；Rust 侧负责路径校验、超时、取消、重启；sidecar 无状态（重启后可继续处理新请求） |
| Rust ↔ Qdrant | 只走 HTTP/gRPC；单进程、`127.0.0.1` 随机端口；连接串由 `index` 独占管理；不得把端口/数据目录写进 `files/` |
| Rust ↔ LLM | 只经 `crates/llm`；超时/重试/降级集中在该 crate；禁止其他 crate 直接 `reqwest` 调外部 API |
| Rust ↔ 系统 | 打开文件/资源管理器只允许 `fs` 调 OS API；禁止 shell 直接 `Command::new("explorer")` |
| 前端 ↔ Rust | 只经 Tauri `invoke` + 事件；`capabilities` 白名单最小化，禁用 `fs`/`shell` 全量权限 |
| 网络 | 除 LLM API 外不得外联（NFR-10）；启动时不得自动检查更新（可显式触发） |

---

## 7. 异步、错误与幂等边界

1. **取消**：每个长任务持有 `CancelToken`；`cancel` 必须幂等，重复取消返回成功；取消后必须清理 `.tmp` 写入。
2. **超时**：sidecar 解析默认 300s/文件；LLM 默认 60s；索引操作 30s；超时映射为可重试错误并计入 `warnings`。
3. **重试**：只允许在 `index`、`llm`、sidecar 调用边界内重试（指数退避 + 抖动，≤3 次）；**禁止**在 shell/UI 层重试写操作（会产生重复数据）。
4. **幂等键**：`(kb_id, rel_path, content_hash)` 是文件级幂等键；`chunk_id` 复用规则见 `data-model.md` §2.5。
5. **部分失败**：批处理必须返回 `warnings[]` 且任务状态 `partial_failed`；UI 必须能展示失败清单并支持“仅重试失败项”。
6. **原子性**：任何跨库/跨引擎操作（融合、导入、导出、删除）必须先写日志表再执行，失败按日志回滚（`merge_journal`/`.import-tmp`/`.tmp` 目录）。
7. **顺序保证**：同一 KB 的写操作串行化（`kb.lock` + 内存互斥）；不同 KB 可并行。检索（只读）不持写锁。

---

## 8. 质量闸门与评审清单

### 8.1 CI 闸门（Phase-1 落地）

1. `cargo fmt --check`、`cargo clippy -D warnings`、`cargo test`。
2. `tsc --noEmit`、ESLint、前端单测。
3. 契约校验：`scripts/validate-contracts`（版本一致、schema 合法、示例过 schema）。
4. 生成物校验：重新生成 Rust→TS 与 command client，diff 必须为空。
5. 依赖方向：`scripts/check-deps` 对照 §2 矩阵。
6. 禁令扫描：`scripts/check-boundaries` 扫描 §3.2 关键 import（`tauri`、`rusqlite`、`qdrant_client`、`tantivy`、`ort`）。
7. 迁移校验：空库顺序执行 + `schema_version` 单调。
8. 集成冒烟：建库 → 解析样例 → 极简检索 → 命中可跳转。

### 8.2 评审清单（每个 PR 自检）

- [ ] 只改了自己 owner 的目录（或已走共享模块流程）
- [ ] 契约未变，或已更新 `contracts/` + `CHANGELOG` + `crates/contracts`
- [ ] 没有新增跨层调用（对照 §2/§3）
- [ ] 错误走 `AppError`，无 `unwrap` 泄漏到边界
- [ ] 长任务可取消、无界并发已限流
- [ ] 日志无正文/密钥/绝对路径
- [ ] 新增依赖已记 `decisions.md` + `LICENSES/`
- [ ] 影响对方的决策已写 `decisions.md` 并通知
- [ ] 测试覆盖新增分支；集成用例在 `tests/integration`
- [ ] 无手改生成物

---

## 9. 落地顺序（Phase-0 → Phase-1）

| 阶段 | 动作 | 产出 |
|---|---|---|
| Phase-0（第 0 周） | 冻结本文件 + 接口 + 数据模型；建 crate 骨架与空 trait；CI 空跑 | `contract_version=0.1.0` |
| Phase-1（第 1–2 周） | 落地 §8.1 的 1–7 号闸门；`scripts/check-deps`、`check-boundaries` 生效 | 架构违规即 CI 失败 |
| Phase-2（第 3 周起） | 每新增共享模块条目补进 §4.1；豁免清单纳入集成日 | 无“隐性共享模块” |

> 记住一句话：**契约是宪法，边界是海关，`decisions.md` 是海关的通关记录。** 三者缺一，4 人 + 4 个 Agent 并行就会退化成互相踩踏。
