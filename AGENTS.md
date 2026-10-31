# AGENTS.md — 智能体入口

> **先读这里，再动手。** 本文件只负责“**去哪儿找规则**”，不复制规则正文；规则以被指向的文件为准。你的角色在 §3 确认。
> 项目：**PerKnowBase** — 本地优先、离线运行的个人语义知识库桌面应用（Tauri 2.x + Rust + React 19）。
> 当前阶段：**Phase-0 契约冻结**（第 0 周）。只写骨架和文档，不写业务实现。

---

## 1. 事实源与读取顺序

**冲突裁决顺序（唯一权威定义在本节）**：

```
立项报告草稿.md  >  contracts/  >  decisions.md  >  任务分工建议.md  >  模块 README
```

| 顺序 | 文件 | 读它的目的 | 何时必读 |
|---|---|---|---|
| 0 | [`STATUS.md`](STATUS.md) | 当前进度、下一步、阻塞 | 每次接手/交接 |
| 1 | [`立项报告草稿.md`](立项报告草稿.md) | 需求、验收（FR/F/P/C/DR/NFR）、技术选型 | 每次开工前 |
| 2 | [`contracts/README.md`](contracts/README.md) | 契约版本、变更流程、命名约定 | 每次开工前 |
| 3 | [`contracts/architecture-boundaries.md`](contracts/architecture-boundaries.md) | 分层、依赖矩阵、禁令、共享模块规则 | 写 Rust/TS 前 |
| 4 | [`decisions.md`](decisions.md) | 影响你模块的既有决策 | 每次开工前；改契约前 |
| 5 | [`任务分工建议.md`](任务分工建议.md) | 目录与验收项归属、进度 | 接任务前 |
| 6 | [`docs/project-structure.md`](docs/project-structure.md) | 文件放哪、crate 划分 | 新建文件前 |
| 7 | [`contracts/interface-contracts.md`](contracts/interface-contracts.md) / [`contracts/data-model.md`](contracts/data-model.md) | 接口、字段、表、枚举的精确定义 | 改接口/表时 |

两处规则矛盾时不要自行选边：在 `decisions.md` 写一条 `DEC`（`Proposed`），按流程推进。

### 1.1 契约版本：MAJOR / MINOR / PATCH 是什么

`contract_version` 是**契约**的版本号（不是软件版本），语义化三段 `MAJOR.MINOR.PATCH`：

| 段 | 含义 | 典型改动 | 处理 |
|---|---|---|---|
| **MAJOR** | 不兼容变更 | 删/重命名字段或命令、改类型/单位、改枚举取值、改表结构、改 payload 字段 | A + 所有受影响 owner **显式同意**，并写迁移方案 |
| **MINOR** | 向后兼容的新增 | 加可选字段（有默认值）、加命令/事件/枚举分支、加表或索引 | A + 受影响 owner 确认，48h 无异议可过 |
| **PATCH** | 不影响行为 | 文档澄清、示例、注释 | 任意 owner，A 知会即可 |

同一 MAJOR 内旧客户端必须可用：新增字段可选、消费者忽略未知字段；废弃字段至少保留一个 MINOR 周期。

### 1.2 `contracts/` 与 `crates/contracts` 是什么关系

同一份契约的**两种形态**，不是两份契约：

| | `contracts/`（目录） | `crates/contracts`（Rust crate） |
|---|---|---|
| 形态 | Markdown 规范 | Rust 代码：结构体、枚举、错误码、`CONTRACT_VERSION` |
| 读者 | 人（前端、Python、评审） | 编译器、所有 Rust crate、TS 生成器 |
| 内容 | 字段语义、约束、状态机、示例 | 类型的真实定义与序列化行为 |
| 谁维护 | A 主笔初稿，B/C/D 按模块审核 | A 主笔，随契约 PR 同步修改 |

- 文档保持语言中立（Python/前端都要读），代码保持可编译（workspace 成员）；两者由“同 PR 修改 + 版本号四处一致 + CI 生成物 diff”保持一致。
- 改契约时两边都必须改：只改文档或只改代码都算契约违约。

---

## 2. 事实源地图

| 我想知道 | 去哪里 |
|---|---|
| 现在做到哪、下一步是什么 | `STATUS.md` |
| 我的角色与可改目录 | 本文 §3；`任务分工建议.md` §1 |
| 功能要不要做、验收怎么判定 | `立项报告草稿.md` |
| 字段名/类型/是否必填 | `contracts/data-model.md` §2 |
| 表、索引、状态流转 | `contracts/data-model.md` §4、§5 |
| 命令/事件/sidecar 报文 | `contracts/interface-contracts.md` §4–§6 |
| 我的 crate 能依赖谁、什么不能调 | `contracts/architecture-boundaries.md` §2、§3.2 |
| 共享模块/工程结构怎么改、通知谁 | 本文 §4.3、§4.4；`contracts/architecture-boundaries.md` §4 |
| 目录/验收项归谁 | `任务分工建议.md` §1、§3 |
| 某个 FR 的验收归谁、测在哪 | `docs/acceptance-traceability.md` |
| 为什么这么设计 | `decisions.md` |
| 文件/目录该放哪 | `docs/project-structure.md` |
| 契约版本现在是多少 | `contracts/README.md` 顶部、`contracts/CHANGELOG.md` |
| 新增依赖怎么登记 | `contracts/architecture-boundaries.md` §3.2 的 BAN-20；登记到 `LICENSES/`（清单格式见 `docs/project-structure.md` §7） |

---

## 3. 你的角色与任务归属

**开工前先确认角色**：优先采用用户在任务里指定的身份（例如“你是 A”）；没有则看`STATUS.md` §1；都未指定时，按任务涉及的目录查 `任务分工建议.md` §1，取该目录的“所有者”；仍不确定就问用户，不要自选。

| 角色 | 负责方向 | 典型目录 |
|---|---|---|
| **A** | 平台与知识库生命周期 | `apps/desktop/src-tauri`、`crates/kb`、`crates/settings`、`crates/storage`、`contracts/` 主笔、CI、打包 |
| **B** | 检索与 AI 链路 | `crates/search`、`crates/index`、`crates/llm`、`crates/history` |
| **C** | 数据管线与同步 | `crates/pipeline`、`crates/sync`、`crates/fs`、`python/parsers`、`pages/knowledge-browser` |
| **D** | 前端基础设施与质量 | `apps/ui`（壳/组件/API client/搜索工作区/历史）、`tests/` |

规则：

- 只改自己 owner 的目录；需要改别人目录 → 提 PR 或写 `DEC`，不直接动手。
- 归属不清 → 按“谁产出这份数据/接口”判断（`contracts/architecture-boundaries.md` §5 数据所有权矩阵）；仍不清问 A。
- 每个 crate 的 `README.md` 写职责、契约指针、测试方式；**没有 README 视作未就绪**。

**接到任务后按这 6 步走**（第 1–4 步只是读，不动手改任何文件）：

| 步 | 读 / 做 | 要弄清的问题 |
|---|---|---|
| 1 | `STATUS.md` §1、§3、§5 | 现在做到哪一步；这个任务是否已在看板；有没有前置任务、依赖或阻塞 |
| 2 | 立项报告对应 FR/NFR 与验收条目（F/P/C 等编号） | **要做什么**、做到什么程度算完成、哪些明确不做 |
| 3 | `decisions.md` §3 索引及相关条目 | **已经做过什么决定**、我的做法是否与既有决策冲突 |
| 4 | 相关契约章节（接口 / 数据模型）+ `contracts/architecture-boundaries.md` §2、§3 | **必须遵守什么约定**：字段、命令、表怎么定义；能调谁、绝对不能调谁 |
| 5 | 开工 | 只改自己 owner 的目录；需要改别人 → 先提 PR 或写 DEC |
| 6 | 收尾 | 按 §8 验证 → 按 §7 提交/开 PR → 更新 `STATUS.md` |


---

## 4. 契约、共享模块、决策

### 4.1 何时必须写 DEC

改多方消费的字段/命令/事件/表；与立项报告字面不一致的解释；新增依赖；豁免禁令；改共享模块或工程结构语义。纯内部实现与错别字不必写。

流程：复制 `decisions.md` 模板 → 追加到详情区顶部 → 更新索引 → 提 `[dec]` PR；状态 `Accepted` 后才能合并契约/代码。

### 4.2 改契约的唯一路径

```
1. decisions.md 写 DEC（Proposed）(具体参考 §4.1)
2. 改 contracts/*.md + contracts/CHANGELOG.md（版本按 §1.1）
3. 改 crates/contracts 的 Rust 类型
4. 生成 TS 类型与命令 client（ts-rs / tauri-specta）
5. 改各调用方实现（Rust 侧 trait 实现、前端 hooks）
6. 补 tests/integration 用例
```
这里的“**生成 TS**”指：用 `ts-rs` / `tauri-specta` 从 `crates/contracts` 的 Rust 类型自动生成 `apps/ui/src/lib/tauri-api/` 下的 TS 类型与命令 client。**禁止手写或手改生成物**，生成后 diff 必须为空。


禁止先改代码再补契约；禁止只改 `crates/contracts` 不改 `contracts/*.md`。提交与评审按 §7。紧急变更标 `emergency`。

### 4.3 共享模块清单
`contracts/**`、`crates/contracts`、`crates/core`、`crates/storage/migrations`、`apps/ui/src/lib/tauri-api`（生成物）、`apps/ui/src/components`、`tests/integration`、`scripts/**`、CI、根 `Cargo.toml`/`package.json`、`AGENTS.md`/`decisions.md`/`STATUS.md`。

### 4.4 工程结构变更
新增根目录文件、计划外目录或模块
流程：必须同一 PR 更新 [`docs/project-structure.md`](docs/project-structure.md)（目录树 + 归属 + 建仓顺序），否则视为未完成；新增目录若影响他人，另写 DEC。

---

## 5. 禁令与不要做

**完整禁令**（BAN-01 ~ BAN-22）见 [`contracts/architecture-boundaries.md`](contracts/architecture-boundaries.md) §3.2；以下只列最易踩的：

| 不要做 | 应该做 |
|---|---|
| 前端直读 DB/文件/Qdrant/LLM | 走命令（`contracts/interface-contracts.md` §4） |
| 非 `storage` 的 crate 连 SQLite、拼 SQL | 扩展 `MetadataRepo` |
| 非 `index` 的 crate 用 `qdrant_client`/`tantivy` | 调 `IndexService` |
| crate 里 `use tauri` 或直接 emit 事件 | `EventSink::emit` |
| shell 写解析/分块/prompt/重试 | shell 只做 DTO + 错误映射 |
| BM25/向量语料取原文 | 必须取 `chunk.content` |
| 前端手写 IPC 类型 | 用 `lib/tauri-api` 生成物 |
| sidecar 写 DB/连 Qdrant/改 `files/` | 只返回 `ParsedDocument` |
| 无界并发 | 有界队列（2/2/1） |
| 日志/事件带正文、密钥、绝对路径 | 只记 id/长度/哈希 |
| 改已合入的 migration | 新增 migration |
| 未经 DEC 加依赖 | DEC + 登记 `LICENSES/` |

**另外不要做**：

- 改 `立项报告草稿.md` 的需求/验收（文档 owner 才可；发现问题写 DEC 反馈）。
- 新建平行事实源（`RULES.md`、`ARCH.md` 之类）；扩展现有文件并走共享模块流程。
- 在 Phase-0 写业务实现。
- 提交大样例、密钥、`.env`、本地绝对路径。
- 在文档里保留修订痕迹/变更摘要（只留最新结果，差异写 `decisions.md`）。

例外须写 DEC（禁令编号、原因、移除计划、截止周），A + 受影响 owner 批准。

---

## 6. 常用任务路由

| 任务 | 落点 | 同步 |
|---|---|---|
| 新增解析格式 | `python/parsers` + `crates/pipeline` | `contracts/data-model.md` §5.2 格式矩阵、DEC |
| 改分块/嵌入 | `crates/pipeline` | `KbSettings`、是否需重建（DEC-20261026-05） |
| 改检索/RRF | `crates/search` | `SearchResult.retrieval`、基准脚本 |
| 加设置项 | `crates/settings` + `pages/settings` | `AppSettings`/`KbSettings`、生成类型 |
| 加表/列 | `crates/storage/migrations`（新脚本） | `contracts/data-model.md` §4、`MetadataRepo`、DEC |
| 加命令/事件 | `apps/desktop/.../commands`、`EventSink` 调用点 | `contracts/interface-contracts.md` §4/§5、`crates/contracts`、生成 client |
| 改跳转 | `features/search` + `fs_open` | `SourceLocation` 语义 |
| 改融合/导入导出 | `crates/kb` | `ExportManifest`、DEC |
| 改 CI/脚本 | `scripts/`、`.github/workflows` | DEC、本文 §7 |

---

## 7. 分支、提交与 PR

**分支模型（多人协作用分支，不用 worktree）**：每人本地 clone 一个仓库、开自己的分支即可，开发机之间天然隔离。
`worktree` 只在**同一个人同时用多个 Agent 并行改多个功能**时使用（避免同一工作区互相覆盖）。

- `main` 受保护；工作分支可用你的名字（如 `dev`），或 `feat/<名字>-<主题>`、`fix/…`、`contract/…`、`dec/…`。
- 提交：Conventional Commits + trailer `Agent-Task:`、`Agent-Decision:`。
- 一条 commit 一件事；契约变更不混入功能 PR；大改拆 Stacked PR。

**什么是 PR**：把分支推到 GitHub 远端（本仓库 `origin` = `github.com:YuBoren/PerKnowBase.git`），在网页发起的 Pull Request（合并请求）。它是“异步评审 + 合并”的载体，也是通知与留痕的地方。

```bash
git switch -c dev                      # 1. 开分支（dev 可换成你的名字）
# ... 改代码/文档 ...
git add -A && git commit               # 2. 提交（带 trailer）
git push -u origin dev                 # 3. 推到远端
# 4. GitHub 网页点 “Compare & pull request”，填模板，指派 reviewer
# 5. CI 绿 + 至少 1 个 approve（契约/共享模块见 §4.3）→ Squash merge → 删分支
```

PR 描述必填：`Affects:`（受影响模块）、`DEC:`（决策编号或 `none` + 理由）、`Contract-Version:`、`How tested:`、`Notify:`（受影响 owner）。

无远端时的替代（需 DEC 记录）：`git format-patch` 或 `git bundle` 发给 owner，由对方 review 后合入；等价流程，只是没有网页评审界面。

---

## 8. 验证命令（Phase-0 建，Phase-1 生效）

```bash
cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace
pnpm --dir apps/ui typecheck && pnpm --dir apps/ui lint && pnpm --dir apps/ui test
pwsh scripts/validate-contracts.ps1 && pwsh scripts/check-deps.ps1
pwsh scripts/check-boundaries.ps1 && pwsh scripts/check-migrations.ps1
pwsh scripts/gen-types.ps1 && git diff --exit-code apps/ui/src/lib/tauri-api
```

脚本不可用先修脚本，不得跳过检查。

---

## 9. 异步协作纪律

1. 决策不进聊天：结论回填 `decisions.md`，否则视为不存在。
2. 通知要留痕：DEC 的 `通知` 字段 + PR 的 `Notify:`，缺一即视为未通知。
3. 接手他人工作：先读 `STATUS.md` + 模块 README + 相关 DEC + CHANGELOG，再读代码。
4. 不假设对方的 Agent 知道你的对话；把结论写进仓库文件。
5. 每次任务结束前更新 `STATUS.md`（状态、下一步、阻塞），让新会话能直接续做；任务细节留在 PR 与 DEC。

---

## 10. 目录速查

```
STATUS.md              当前进度、下一步、阻塞（新会话先看）
立项报告草稿.md        需求与验收（最高事实源）
contracts/             接口 / 数据模型 / 边界与禁令 / CHANGELOG / schemas
decisions.md           异步决策日志（开工先看索引）
任务分工建议.md         所有权、验收归属、进度
docs/project-structure.md        目录规划与建仓顺序
docs/acceptance-traceability.md  FR→验收→责任人→测试 追溯
crates/                contracts core storage pipeline index llm kb settings fs sync search history
apps/desktop, apps/ui  Tauri 壳（A）/ React 前端（D 主）
python/parsers         MinerU sidecar（C）
tests/integration, tests/e2e     集成 / 端到端
scripts/, LICENSES/    校验生成（A/D）/ 第三方许可（A）
```

本文属共享模块，改动走 PR + `[shared]` 并在 `decisions.md` 留痕（见 DEC-20261030-23）；保持简短，规则正文留在被指文件。
