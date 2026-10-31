# STATUS — 项目进度与交接

> **用途**：新会话/新成员的第一站，回答“现在做到哪、下一步做什么、卡在哪”。只写**当前状态**，不写历史与理由（理由在 [`decisions.md`](decisions.md)）。
> **更新**：每个任务结束前由执行者花 2 分钟更新；只改状态、链接、下一步。
> **状态**：`未开始` / `进行中` / `待审核` / `完成` / `阻塞`。

## 1. 当前状态

| 项 | 值 |
|---|---|
| 阶段 | Phase-0：契约冻结 + 工程骨架（不写业务实现） |
| 契约版本 | `0.1.0`（预冻结补齐已完成；**等待 B/C/D 签字**） |
| 分支 | `ybr`（A 的骨架改动**已完成、待提交**：本机沙箱对 `.git` 只读） |
| 下一里程碑 | 契约签字（PX-02）→ B/C/D 各自骨架并行 |
| 更新日期 | 2026-10-31 |

## 2. 契约完成度

| 项 | 位置 | 状态 | 负责 | 备注 |
|---|---|---|---|---|
| 契约总览/命名/版本 | `contracts/README.md` | 完成 | A | 新增 §6 schema 使用说明 |
| 接口契约（trait/命令/事件/错误码） | `contracts/interface-contracts.md` | 完成 | A | 39 命令、7 事件、30 错误码、sidecar v1.0 |
| 数据模型（对象/枚举/表/索引/导出） | `contracts/data-model.md` | 完成 | A | 库级 12 表 + 应用级 3 表 |
| Qdrant 进程契约 | `data-model.md` §3.1.1 | **待审核** | A 起草（PX-01），B 审核 | 启动/端口回退/单实例/健康/重启/关闭/目录迁移/版本升级；DEC-20261031-25 |
| 架构边界与禁令 | `contracts/architecture-boundaries.md` | 完成 | A | 22 条禁令、依赖矩阵 |
| 变更记录 | `contracts/CHANGELOG.md` | 完成 | A | 0.1.0 + 预冻结补齐条目 |
| JSON Schema 五件套 | `contracts/schemas/` | 完成 | A（PA-03/PA-04） | 5 个 schema + 6 个示例；用法见 `contracts/README.md` §6 |
| Rust 类型实现 | `crates/contracts` | 完成 | A（PA-05） | 全部领域类型 + 版本常量；`ts` feature 待 D 的生成链接入 |
| TS 类型与命令 client 生成契约 | `apps/ui/src/lib/tauri-api` | 未开始 | D（PD-03） | 生成链未建立，`version.ts` 暂缺（校验脚本会 `[skip]`） |
| 集成 fixtures | `tests/integration/fixtures/` | 未开始 | C（PC-03） | 样例文件清单 |
| sidecar 黄金报文样例 | `tests/mocks/` 或 `python/parsers/tests/` | 未开始 | C（PC-02） | 契约测试用固定请求/响应 |
| 契约评审签字 | `docs/contract-review-0.1.0.md` | **进行中** | A 组织（PX-02），B/C/D 审核 | A 自查已过，待三方签字 |

## 3. 任务看板（Phase-0，按角色）

> 状态：`未开始 / 进行中 / 待审核 / 完成 / 阻塞`。编号按角色：`PA` 平台、`PB` 检索、`PC` 数据管线、`PD` 前端、`PX` 跨角色。

### 3.1 A — 平台与知识库生命周期

| ID | 任务 | 状态 | 交付物 |
|---|---|---|---|
| PA-01 | 契约初稿（接口 / 数据模型 / 架构边界 / CHANGELOG） | 完成 | `contracts/*.md` |
| PA-02 | 文档治理（AGENTS / STATUS / project-structure / migrations 说明） | 完成 | 各文档 |
| PA-03 | JSON Schema 五件套（settings / manifest / events / sidecar） | 完成 | `contracts/schemas/`（5 schema + 6 示例） |
| PA-04 | schema 使用说明（是什么 / 怎么用 / CI 如何校验 / 示例） | 完成 | `contracts/README.md` §6 |
| PA-05 | `crates/contracts` 类型骨架 + `CONTRACT_VERSION` | 完成 | `crates/contracts`（16 个模块、30 错误码、7 事件） |
| PA-06 | `crates/core` 基础（`EventSink`/`CancelToken`/路径校验/哈希/时间/错误构造） | 完成 | `crates/core` |
| PA-07 | Rust workspace + crates 空壳（`Cargo.toml`、toolchain、各 crate 骨架） | 完成 | 根 `Cargo.toml`、`rust-toolchain.toml`、12 个 crate、`package.json`/`pnpm-workspace.yaml` |
| PA-08 | Tauri 壳骨架 + 命令注册空表 + `app_info` | 完成 | `apps/desktop/src-tauri`（39 条命令登记；`app_info` 已实现） |
| PA-09 | SQLite migrations `0001_init.sql`（app / kb 两条链） | 完成 | `crates/storage/migrations/{app,kb}/0001_init.sql`（3 / 12 表） |
| PA-10 | `scripts/` 校验脚本 + CI | 完成 | `scripts/`（5 个脚本 + 2 个 Python 辅助）、`.github/workflows/ci.yml`、PR 模板 |
| PA-11 | `LICENSES/THIRD-PARTY.md` 依赖许可清单 | 完成 | `LICENSES/THIRD-PARTY.md` |

### 3.2 B — 检索与 AI 链路

| ID | 任务 | 状态 | 交付物 |
|---|---|---|---|
| PB-01 | 审核 B 段契约：索引/检索/LLM/历史接口（`interface-contracts.md` §3.7/§3.8）、Qdrant payload 与 Tantivy（`data-model.md` §3.1/§3.2）、Qdrant 进程契约（配合 PX-01） | 未开始 | 审核意见（PR）+ 在 `docs/contract-review-0.1.0.md` 签字 |

### 3.3 C — 数据管线与同步

| ID | 任务 | 状态 | 交付物 |
|---|---|---|---|
| PC-01 | 审核 C 段契约：sidecar 协议（`interface-contracts.md` §6）、`FileNode`/`ParsedDocument`/`DocumentChunk`/`SyncEvent`（`data-model.md` §2.2/§2.4/§2.5/§2.10）、缓存布局（§7） | 未开始 | 审核意见（PR）+ 签字 |
| PC-02 | `python/parsers` 协议 echo 骨架（`protocol.py` + NDJSON）+ 黄金报文样例 | 未开始 | `python/parsers`、`tests/mocks/` |
| PC-03 | 集成 fixtures（样例文件 + 期望结果清单） | 未开始 | `tests/integration/fixtures/` |

### 3.4 D — 前端基础设施与质量

| ID | 任务 | 状态 | 交付物 |
|---|---|---|---|
| PD-01 | 审核命令/事件与生成契约（`interface-contracts.md` §4/§5、生成规则、mock 规范） | 未开始 | 审核意见（PR）+ 签字 |
| PD-02 | React 壳 + 路由 + 状态骨架 | 未开始 | `apps/ui`（同时把 `tauri.conf.json` 的 `frontendDist` 改为 `../../ui/dist`，删除 `placeholder-dist`） |
| PD-03 | 命令 client 生成链（`ts-rs`/`tauri-specta` 接入并产出一次） | 未开始 | `scripts/gen-types.ps1`、`lib/tauri-api` |
| PD-04 | E2E 框架骨架 | 未开始 | `tests/e2e` |

### 3.5 跨角色

| ID | 任务 | 状态 | 交付物 |
|---|---|---|---|
| PX-01 | Qdrant 进程契约补全（A 起草，B 审核）：启动、端口回退、重启、单实例、数据目录迁移 | **待审核** | `data-model.md` §3.1.1 + DEC-20261031-25 |
| PX-02 | 契约评审与签字（A 组织，B/C/D 审核） | **进行中** | `docs/contract-review-0.1.0.md`（A 自查完成，B/C/D 待签） |

## 4. 下一步（按优先级）

1. **PX-02 签字**：B/C/D 先确认 DEC-20261031-25（Qdrant 进程）与 DEC-20261031-26（`BlockType` 大写），再按 `docs/contract-review-0.1.0.md` §4 逐项审。
2. **B**：`crates/index` 空 trait → Qdrant 进程实现骨架（按 §3.1.1 逐条落地）+ PB-01 审核。
3. **C**：PC-02（sidecar 协议 echo + 黄金报文）/ PC-03（fixtures）；顺带确认 `BlockType` 大写。
4. **D**：PD-02（React 壳）→ PD-03（生成链）；落地后 `version.ts` 版本比对、前端 `invoke`/`fetch` 扫描、`gen-diff` 作业自动生效。
5. **A**：Phase-1 起步补 `crates/storage` 的 `MetadataRepo` 端口与迁移 runner、`crates/kb`/`crates/settings` 服务骨架；命令按 `commands/table.rs` 的 `Pending` 列表逐个接上。
6. **CI 首跑**：在有网络的环境跑一次 `cargo build`/`clippy`/`test` 与 `pnpm` 作业，修正 manifest 层面的问题。

## 5. 阻塞与待决

- **提交待办**：本机沙箱禁止写 `.git`，改动留在工作区未提交；commit 文案已备好（`.git/dsh_commit_msg.txt`），在有写权限的环境执行 `git add -A && git commit -F .git/dsh_commit_msg.txt` 即可（完成后删除该临时文件）。
- **本机验证缺口**：评审环境无 crates 缓存与 `pnpm`，`cargo build`/`clippy`/`cargo test`/`pnpm build` 未跑过；已用 `rustfmt --check`、`cargo metadata`、五个校验脚本、空库迁移执行做替代验证。**CI 首跑是唯一闸门**。
- `apps/ui` 未建立：前端 `invoke`/`fetch` 扫描与 `version.ts` 版本比对暂为 `[skip]`（随 PD-02 自动生效）。
- 待 B/C/D 确认：DEC-20261031-25、DEC-20261031-26（Proposed）。
- 待用户确认：`LICENSES/THIRD-PARTY.md` §3 的 Phase-1 依赖（qdrant-client / tantivy / ort / notify / reqwest / zip / keyring 等）选型与许可；MinerU（AGPL）以独立进程使用的边界。
- 开放问题见 [`decisions.md`](decisions.md) §5（历史上限、回收站、移动端构造、OCR 预留）。

## 6. 交接约定

**新会话/新 Agent 的阅读顺序**：`AGENTS.md` §1 → 本文件 → `decisions.md` §3 索引 → 与你任务相关的契约章节 → 代码。

**更新规则**：

1. 只改状态、链接、下一步；不写过程、不贴日志、不写“本次修改了…”。
2. 一条任务一行；完成的移到 §2/§3 并标 `完成`（不删除，便于对照）。
3. 影响他人的取舍写 `decisions.md`，本文件只留一句“见 DEC-xxx”。
4. 阻塞要写清“缺什么、谁来解、下一步动作”，否则视为未阻塞。
5. 新增根目录文件、计划外目录或模块时，同步更新 `docs/project-structure.md`（见 `AGENTS.md` §4.3）。
