# AGENTS.md — 智能体入口

> **先读这里，再动手。** 本文件只负责“**去哪里找规则**”，不复制规则正文；规则以被指向的文件为准。
> 项目：**PerKnowBase** — 本地优先、离线运行的个人语义知识库桌面应用（Tauri 2.x + Rust + React 19）。
> 当前阶段：**Phase-0 契约冻结**（第 0 周）。只写骨架与文档，不写业务实现。

---

## 1. 五分钟上手（按顺序读）

| 顺序 | 文件 | 读它的目的 | 何时必读 |
|---|---|---|---|
| 1 | [`立项报告草稿.md`](立项报告草稿.md) | 需求、验收标准（F/P/C/DR/NFR）、技术选型。**冲突时的最高事实源** | 每次开工前 |
| 2 | [`contracts/README.md`](contracts/README.md) | 契约怎么版本化、怎么改、命名约定 | 每次开工前 |
| 3 | [`contracts/architecture-boundaries.md`](contracts/architecture-boundaries.md) | 分层、依赖矩阵、**22 条禁令**、共享模块治理 | 写任何 Rust/TS 代码前 |
| 4 | [`decisions.md`](decisions.md) | 影响你模块的既有决策（索引表看状态） | 每次开工前；改契约前 |
| 5 | [`任务分工建议.md`](任务分工建议.md) | 目录归属、验收项归属、进度节奏、冲突裁决记录（附录 A） | 接任务前 |
| 6 | [`docs/project-structure.md`](docs/project-structure.md) | 文件该放哪、crate 划分、建仓顺序 | 新建文件前 |
| 7 | `contracts/interface-contracts.md` / `contracts/data-model.md` | 你正在改的接口/字段的精确定义 | 按需（改接口/表/枚举/事件必读） |

**冲突裁决顺序**：`立项报告草稿.md` > `contracts/` > `decisions.md` > `任务分工建议.md` > 模块 `README.md`。
发现两处规则矛盾：**不要自行选边**，在 `decisions.md` 写一条 `DEC`（状态 `Proposed`），再按流程推进。

---

## 2. 事实源地图（“我要知道 X，去哪儿找”）

| 我想知道 | 去哪里 |
|---|---|
| 这个功能到底要不要做、验收怎么判定 | `立项报告草稿.md`（FR/F/P/C/NFR/DR、验收表、目标达成判定） |
| 这个字段叫什么、类型是什么、能不能为空 | `contracts/data-model.md` §2 |
| 表怎么建、索引怎么加、状态怎么流转 | `contracts/data-model.md` §4、§5 |
| 命令/事件/sidecar 报文长什么样 | `contracts/interface-contracts.md` §4–§6 |
| 我的 crate 能依赖谁、什么绝对不能调 | `contracts/architecture-boundaries.md` §2、§3 |
| 共享模块能不能改、怎么改、通知谁 | `contracts/architecture-boundaries.md` §4 |
| 这个目录归谁 | `任务分工建议.md` §1、§5；`docs/project-structure.md` §2 |
| 某个 FR 的验收归谁、测在哪 | `docs/acceptance-traceability.md` |
| 为什么这么设计（背景与备选） | `decisions.md` |
| 文件该放哪个目录 | `docs/project-structure.md` |
| 契约版本现在是多少 | `contracts/README.md` 顶部、`contracts/CHANGELOG.md` |
| 新增依赖要不要登记 | `contracts/architecture-boundaries.md` BAN-20 + `LICENSES/THIRD-PARTY.md` |

---

## 3. 你是哪条线？（确定所有权）

分工代号：**A** 平台与知识库生命周期 / **B** 检索与 AI 链路 / **C** 数据管线与同步 / **D** 前端基础设施与质量。

第一步：读 `任务分工建议.md` §1 “模块所有权规划”，找到你要改的目录 → 该行“所有者”就是唯一可直接改的人。

- **只改自己 owner 的目录。** 需要改别人目录：提 PR 或写 `DEC` 请求，不直接动手。
- **不确定归属**：以“谁产出这份数据/接口”为所有者（`contracts/architecture-boundaries.md` §5 数据所有权矩阵）；仍不确定 → 问 A。
- 每个 crate 的 `README.md`（Phase-1 起）应写：职责、契约指针、怎么跑测试、已知限制。**没有 README 的 crate 视为未就绪。**

---

## 4. 改动前的 6 步（每次任务都走）

1. **定位需求**：在 `立项报告草稿.md` 找到对应 FR/NFR 与验收项；找不到 → 先问，不要自行发明功能。
2. **查既有决策**：`decisions.md` §3 索引搜你的模块/字段；已有 `Accepted` 条目就照做，偏离需新 DEC。
3. **查契约**：字段/命令/表是否已定义？没定义 → 先补契约（§5），不要先写代码。
4. **确认边界**：对照 `architecture-boundaries.md` §2/§3，确认你要调的层是白名单内。
5. **动手**：只改 owner 目录；长任务必须可取消；错误走 `AppError`；日志无正文/密钥。
6. **收尾**：跑 §7 的验证命令；PR 填 `Affects:`/`DEC:`/`Contract-Version:`；影响对方 → §5 通知。

---

## 5. 契约、共享模块与决策（最容易踩的三件事）

### 5.1 什么时候必须写 `DEC`

见 `decisions.md` §1.1，速记：

- 改任何被多方消费的字段/命令/事件/表 → **必写**
- 与立项报告字面不一致的解释 → **必写**
- 新增依赖、升级会改变行为的版本 → **必写**
- 豁免禁令、改共享模块语义 → **必写**
- 纯自己模块内部实现、错别字 → 不必写

写法：复制 `decisions.md` §2 模板 → 追加到 §4 顶部 → 更新 §3 索引 → 提 `[dec]` PR。
**只追加，不改历史**；状态改 `Accepted` 才能合并契约/代码。

### 5.2 改契约的唯一路径

```
decisions.md (Proposed) → contracts/*.md + CHANGELOG.md → crates/contracts → 生成 TS → 实现 → tests/integration
```

- 禁止“先改代码再补契约”；禁止只改 `crates/contracts` 不改 `contracts/*.md`。
- 破坏性变更（删/改名/改类型/改枚举/改表）→ MAJOR，需 A + 所有受影响 owner 显式确认。
- 契约冻结窗口：每周三 24:00 至周五集成日结束；紧急变更标注 `emergency`。

### 5.3 共享模块（改前必须看 `architecture-boundaries.md` §4）

共享清单速记：`contracts/**`、`crates/contracts`、`crates/core`、`crates/storage/migrations`、`apps/ui/src/lib/tauri-api`（生成物，禁手改）、`apps/ui/src/components`、`tests/integration`、`scripts/**`、CI、根 `Cargo.toml`/`package.json`、`AGENTS.md`/`decisions.md` 本身。

流程：写 DEC → PR（标题 `[shared] <module>: <摘要>`，标签 `shared-module`）→ `Notify:` 列出受影响 owner → 至少 2 approve（主 owner + A）→ 更新 CHANGELOG → CI 全绿。
**48 小时内无异议视为通过（MAJOR 除外）。**

---

## 6. 禁令速查（全文见 `architecture-boundaries.md` §3.2）

| 不要做 | 应该做 |
|---|---|
| 前端直读 `kb.sqlite3`/`files/`/Qdrant/LLM | 走 `invoke` 命令（`interface-contracts.md` §4） |
| 非 `storage` 的 crate 连 SQLite、拼 SQL | 扩展 `MetadataRepo`（提 PR） |
| 非 `index` 的 crate `use qdrant_client`/`tantivy` | 调 `IndexService` |
| 在 crate 里 `use tauri` 或直接 emit 事件 | 用 `EventSink::emit`（L1 定义，壳实现） |
| shell 里写解析/分块/prompt/重试循环 | shell 只做 DTO + 错误映射，≤20 行/命令 |
| BM25/向量语料取原文或解析原始产物 | 必须取 `chunk.content`（BAN-07） |
| 前端手写 IPC 类型 | 用 `lib/tauri-api` 生成物（BAN-08） |
| sidecar 写 DB、连 Qdrant、改 `files/` | sidecar 只返回 `ParsedDocument`，只写 `cache_dir` |
| 无界并发解析/嵌入 | 有界队列（默认 2/2/1） |
| 日志/事件/错误里带正文、prompt、密钥、绝对路径 | 只记 id/长度/哈希（BAN-16） |
| 改已合入的 migration | 新增 migration |
| 硬编码绝对路径/端口/用户名 | 临时目录 + 相对路径 + 随机端口 |
| 用事件当数据源渲染 | 事件只触发回拉权威状态（BAN-17） |
| 未经 DEC 加依赖 | DEC + `LICENSES/THIRD-PARTY.md` |

任何例外：写 `DEC`（含禁令编号、原因、移除计划与截止周），A + 受影响 owner 批准；**禁止口头豁免**。

---

## 7. 常用任务路由（该动哪些文件）

| 任务 | 主要落点 | 必须同步 |
|---|---|---|
| 新增解析格式 | `python/parsers` + `crates/pipeline`（`FileKind` 映射） | `contracts/data-model.md` §5.2 格式矩阵、DEC |
| 改分块策略 | `crates/pipeline::Chunker` | `KbSettings.chunk`、是否 `requires_rebuild`（DEC-20261026-05） |
| 改嵌入模型/维度 | `crates/pipeline::Embedder` | `contracts/data-model.md` §3.1、`RebuildPlan`、DEC |
| 改检索排序/RRF | `crates/search::Retriever` | `SearchResult.retrieval`、`decisions.md`、基准脚本 |
| 加一个设置项 | `crates/settings` + `apps/ui/src/pages/settings` | `AppSettings`/`KbSettings` 契约、schema、生成类型 |
| 加一张表/一列 | `crates/storage/migrations`（新脚本） | `contracts/data-model.md` §4、`MetadataRepo`、DEC |
| 加一个 Tauri 命令 | `apps/desktop/src-tauri/src/commands` | `contracts/interface-contracts.md` §4、`crates/contracts`、生成 client |
| 加一个事件 | `crates/core::EventSink` 调用点 + 壳实现 | `contracts/interface-contracts.md` §5、前端订阅 hook |
| 改结果跳转行为 | `apps/ui/src/features/search` + `fs_open` | `SourceLocation` 语义（行/页/图片） |
| 改历史存储/清理 | `crates/history` | `history_limit` 设置、`contracts/data-model.md` §2.13 |
| 改融合/导入导出 | `crates/kb` | `ExportManifest`、`manifest.json` 校验顺序、DEC |
| 改 CI/脚本 | `scripts/`、`.github/workflows` | DEC（共享模块）、`AGENTS.md` 验证命令 |

---

## 8. 提交、分支、worktree

- 每人独立 worktree：`git worktree add ../wt-<代号> -b feat/<代号>-<主题>`；**不要在共享目录开工**。
- 分支：`feat/…`、`fix/…`、`contract/…`、`dec/…`；`main` 受保护。
- commit：Conventional Commits + trailer：

```
feat(search): 支持 RRF 权重配置

Agent-Task: B-003 混合检索权重
Agent-Decision: DEC-20261026-08
```

- 一条 commit 一件事；跨模块大改拆 Stacked PR；**禁止**把契约变更混进功能 PR。
- 提交信息里不要写“由某 AI 生成”之类无信息量的声明；写清意图与影响即可。

---

## 9. 验证命令（Phase-0 建，Phase-1 生效）

```bash
# Rust
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace

# 前端
pnpm --dir apps/ui typecheck && pnpm --dir apps/ui lint && pnpm --dir apps/ui test

# 契约与边界（改动契约/依赖/表结构后必跑）
pwsh scripts/validate-contracts.ps1
pwsh scripts/check-deps.ps1
pwsh scripts/check-boundaries.ps1
pwsh scripts/check-migrations.ps1

# 生成物必须与源一致
pwsh scripts/gen-types.ps1 && git diff --exit-code apps/ui/src/lib/tauri-api
```

不可用的脚本：先看 `scripts/README.md`（Phase-0 由 A 建）；**不得**用“跳过检查”代替修复。

---

## 10. 给别人留上下文（异步协作纪律）

1. **决策不进聊天**：口头/聊天结论一律回填 `decisions.md`，否则视为不存在。
2. **PR 描述必填**：`Affects:`（受影响模块）、`DEC:`（决策编号或 `none` + 理由）、`Contract-Version:`、`How tested:`。
3. **通知要在记录里可见**：DEC 的 `通知` 字段 + PR 的 `Notify:`，缺一即视为未通知。
4. **接手别人工作**：先读该模块 `README.md` + 相关 DEC + 最近 CHANGELOG 条目，再读代码。
5. **AI 上下文隔离**：不要假设对方 Agent 知道你的对话；把结论写进仓库文件。

---

## 11. 不要做（Do-not 清单）

- 不要修改 `立项报告草稿.md` 的需求/验收内容（文档 owner 才可；发现错误写 DEC 反馈）。
- 不要绕开 `contracts/` 直接和对方“口头对齐”字段。
- 不要手改 `apps/ui/src/lib/tauri-api/`、`target/`、生成产物。
- 不要在 Phase-0 写业务实现（本阶段只冻结契约与骨架）。
- 不要提交大样例、密钥、`.env`、本地绝对路径。
- 不要为本可复用 `IndexService`/`MetadataRepo` 的能力新建平行实现。
- 不要引入新的文档体系（不要新建 `RULES.md`、`ARCH.md` 之类平行事实源）；扩展就改现有文件并走共享模块流程。

---

## 12. 目录速查

```
AGENTS.md                   ← 你在读
立项报告草稿.md              ← 需求与验收（最高事实源）
任务分工建议.md              ← 所有权、验收归属、节奏
decisions.md                ← 异步决策日志（开工必读索引）
contracts/                  ← 接口 / 数据模型 / 边界与禁令 / CHANGELOG / schemas
docs/project-structure.md   ← 目录规划、crate 划分、建仓顺序
docs/acceptance-traceability.md ← FR→验收→责任人→测试 追溯矩阵
crates/                     ← contracts core storage pipeline index llm kb settings fs sync search history
apps/desktop, apps/ui       ← Tauri 壳（A）/ React 前端（D 主）
python/parsers              ← MinerU sidecar（C）
tests/integration, tests/e2e← 集成与端到端（D 牵头）
scripts/, LICENSES/         ← 校验生成（A/D）/ 第三方许可（A）
```

---

## 13. 本文件的变更

`AGENTS.md` 属于共享模块：改动需 PR + `[shared]` 标签 + A 与受影响 owner 确认，并在 `decisions.md` 留痕（见 `architecture-boundaries.md` §4.1）。保持它**短、只指路**；规则正文一律留在被指向的文件里。
