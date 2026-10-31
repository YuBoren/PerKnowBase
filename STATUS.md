# STATUS — 项目进度与交接

> **用途**：新会话/新成员的第一站，回答“现在做到哪、下一步做什么、卡在哪”。只写**当前状态**，不写历史与理由（理由在 [`decisions.md`](decisions.md)）。
> **更新**：每个任务结束前由执行者花 2 分钟更新；只改状态、链接、下一步。
> **状态**：`未开始` / `进行中` / `待审核` / `完成` / `阻塞`。

## 1. 当前状态

| 项 | 值 |
|---|---|
| 阶段 | Phase-0：契约冻结（不写业务代码） |
| 契约版本 | `0.1.0`（文档已成型，**尚未评审签字**） |
| 分支 | `ybr`（本轮文档改动未提交） |
| 下一里程碑 | 契约评审签字 → 建 `crates/` 骨架 |
| 更新日期 | 2026-10-31 |

## 2. 契约完成度

| 项 | 位置 | 状态 | 负责 | 备注 |
|---|---|---|---|---|
| 契约总览/命名/版本 | `contracts/README.md` | 完成 | A | — |
| 接口契约（trait/命令/事件/错误码） | `contracts/interface-contracts.md` | 完成 | A | 39 命令、7 事件、30 错误码、sidecar v1.0 |
| 数据模型（对象/枚举/表/索引/导出） | `contracts/data-model.md` | 完成 | A | 库级 12 表 + 应用级 3 表 |
| Qdrant 进程契约 | `data-model.md` §3.1 + DEC-20261026-06 | **部分** | A 起草（PX-01），B 审核 | 缺启动参数、端口回退、重启策略、单实例、数据目录迁移 |
| 架构边界与禁令 | `contracts/architecture-boundaries.md` | 完成 | A | 22 条禁令、依赖矩阵 |
| 变更记录 | `contracts/CHANGELOG.md` | 完成 | A | — |
| JSON Schema 五件套 | `contracts/schemas/` | **未开始** | A（PA-03） | `app-settings`/`kb-settings`/`export-manifest`/`events`/`sidecar`；使用说明见 PA-04 |
| Rust 类型实现 | `crates/contracts` | **未开始** | A（PA-05） | 需进入写代码阶段 |
| TS 类型与命令 client 生成契约 | `contracts/`（缺专节） | **未开始** | D（PD-03） | 命名映射、错误类型、事件类型、mock 规范 |
| 集成 fixtures | `tests/integration/fixtures/` | **未开始** | C（PC-03） | 样例文件清单 |
| sidecar 黄金报文样例 | `tests/mocks/` 或 `python/parsers/tests/` | **未开始** | C（PC-02） | 契约测试用固定请求/响应 |
| 契约评审签字 | — | **未开始** | A 组织（PX-02），B/C/D 审核 | 签字后 `0.1.0` 冻结 |

## 3. 任务看板（Phase-0，按角色）

> 状态：`未开始 / 进行中 / 待审核 / 完成 / 阻塞`。编号按角色：`PA` 平台、`PB` 检索、`PC` 数据管线、`PD` 前端、`PX` 跨角色。

### 3.1 A — 平台与知识库生命周期

| ID | 任务 | 状态 | 交付物 |
|---|---|---|---|
| PA-01 | 契约初稿（接口 / 数据模型 / 架构边界 / CHANGELOG） | 完成 | `contracts/*.md` |
| PA-02 | 文档治理（AGENTS / STATUS / project-structure / migrations 说明） | 完成 | 各文档 |
| PA-03 | JSON Schema 五件套（settings / manifest / events / sidecar） | 未开始 | `contracts/schemas/` |
| PA-04 | schema 使用说明（是什么 / 怎么用 / CI 如何校验 / 示例） | 未开始（待 PA-03） | `contracts/README.md` 增补 |
| PA-05 | `crates/contracts` 类型骨架 + `CONTRACT_VERSION` | 未开始 | `crates/contracts` |
| PA-06 | `crates/core` 基础（`EventSink`/`CancelToken`/路径校验/错误构造） | 未开始 | `crates/core` |
| PA-07 | Rust workspace + crates 空壳（`Cargo.toml`、toolchain、各 crate 骨架） | 未开始 | 根 `Cargo.toml`、`crates/*` |
| PA-08 | Tauri 壳骨架 + 命令注册空表 + `app_info` | 未开始 | `apps/desktop/src-tauri` |
| PA-09 | SQLite migrations `0001_init.sql`（app / kb 两条链） | 未开始 | `crates/storage/migrations/` |
| PA-10 | `scripts/` 四个校验脚本骨架 + CI 空跑 | 未开始 | `scripts/`、`.github/workflows` |
| PA-11 | `LICENSES/THIRD-PARTY.md` 依赖许可清单 | 未开始 | `LICENSES/` |

### 3.2 B — 检索与 AI 链路

| ID | 任务 | 状态 | 交付物 |
|---|---|---|---|
| PB-01 | 审核 B 段契约：索引/检索/LLM/历史接口（`interface-contracts.md` §3.7/§3.8）、Qdrant payload 与 Tantivy（`data-model.md` §3.1/§3.2）、Qdrant 进程契约（配合 PX-01） | 未开始 | 审核意见（PR） |

### 3.3 C — 数据管线与同步

| ID | 任务 | 状态 | 交付物 |
|---|---|---|---|
| PC-01 | 审核 C 段契约：sidecar 协议（`interface-contracts.md` §6）、`FileNode`/`ParsedDocument`/`DocumentChunk`/`SyncEvent`（`data-model.md` §2.2/§2.4/§2.5/§2.10）、缓存布局（§7） | 未开始 | 审核意见（PR） |
| PC-02 | `python/parsers` 协议 echo 骨架（`protocol.py` + NDJSON）+ 黄金报文样例 | 未开始 | `python/parsers`、`tests/mocks/` |
| PC-03 | 集成 fixtures（样例文件 + 期望结果清单） | 未开始 | `tests/integration/fixtures/` |

### 3.4 D — 前端基础设施与质量

| ID | 任务 | 状态 | 交付物 |
|---|---|---|---|
| PD-01 | 审核命令/事件与生成契约（`interface-contracts.md` §4/§5、生成规则、mock 规范） | 未开始 | 审核意见（PR） |
| PD-02 | React 壳 + 路由 + 状态骨架 | 未开始 | `apps/ui` |
| PD-03 | 命令 client 生成链（`ts-rs`/`tauri-specta` 接入并产出一次） | 未开始 | `scripts/gen-types.*`、`lib/tauri-api` |
| PD-04 | E2E 框架骨架 | 未开始 | `tests/e2e` |

### 3.5 跨角色

| ID | 任务 | 状态 | 交付物 |
|---|---|---|---|
| PX-01 | Qdrant 进程契约补全（A 起草，B 审核）：启动、端口回退、重启、单实例、数据目录迁移 | 未开始 | `data-model.md` §3.1 |
| PX-02 | 契约评审与签字（A 组织，B/C/D 审核） | 未开始 | 评审记录 / `CHANGELOG` 冻结 |

## 4. 下一步（按优先级）

1. **PX-01** Qdrant 进程契约：卡住 B 的索引实现与 C 的写入联调。
2. **PA-03** schemas：让契约可被 CI 校验，也是设置导入导出的依据。
3. **PC-02 / PC-03** 契约测试资产：先定位置与样例清单，内容可渐进补。
4. **PD-03** client 生成链：PD-02 壳与前端页面依赖它。
5. **PA-05 ~ PA-11** 骨架类（契约类型 / core / workspace / Tauri / migrations / scripts / LICENSES）。
6. **PX-02** 评审签字：全部完成后的唯一闸门。
7. **PA-04** schema 使用说明：等 PA-03 落地后再补，不阻塞当前工作。

## 5. 阻塞与待决

- 无硬阻塞。
- 待用户确认：PA-03 与 PX-01 是否现在补；确认后更新本表状态。
- 延后：PA-04（schema 是什么、怎么用、CI 如何校验）待 schema 文件落地后补写。
- 开放问题见 [`decisions.md`](decisions.md) §5（历史上限、回收站、移动端构造、OCR 预留）。

## 6. 交接约定

**新会话/新 Agent 的阅读顺序**：`AGENTS.md` §1 → 本文件 → `decisions.md` §3 索引 → 与你任务相关的契约章节 → 代码。

**更新规则**：

1. 只改状态、链接、下一步；不写过程、不贴日志、不写“本次修改了…”。
2. 一条任务一行；完成的移到 §2/§3 并标 `完成`（不删除，便于对照）。
3. 影响他人的取舍写 `decisions.md`，本文件只留一句“见 DEC-xxx”。
4. 阻塞要写清“缺什么、谁来解、下一步动作”，否则视为未阻塞。
5. 新增根目录文件、计划外目录或模块时，同步更新 `docs/project-structure.md`（见 `AGENTS.md` §4.3）。
