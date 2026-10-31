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
- [ ] B 签字：检索/索引/LLM/历史接口 + Qdrant payload/进程契约（DEC-20261031-25）
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

- [ ] `interface-contracts.md` §3.7 索引接口：`IndexService` 双写顺序、`ScoredChunk` 不含预览、快照/恢复语义
- [ ] §3.8 检索/LLM/历史：三模式流程、降级规则、`Usage`、`LlmProbeResult`
- [ ] `data-model.md` §3.1 Qdrant payload 字段与索引标记；§3.1.1 进程契约（DEC-20261031-25）
- [ ] §3.2 Tantivy schema 与分词（`jieba` 降级路径）
- [ ] `search.rs`/`llm.rs`/`index.rs`/`history.rs` 的字段名与默认值

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

## 5. 签字

| 角色 | 结论 | 日期 | 备注 |
|---|---|---|---|
| A | 同意冻结（待全体签字） | 2026-10-31 | 见 §3 自查 |
| B | 待签 | — | 需先确认 DEC-20261031-25 |
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
