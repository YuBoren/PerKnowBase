# 契约总览与治理规则（contracts/）

> **状态**：Phase-0 冻结中（仅文档，不含实现代码）。
> **适用对象**：A/B/C/D 四位成员及其智能体；任何跨模块改动必须先读本目录。
> **事实源层级**：`立项报告草稿.md`（需求与验收 SSOT）> `contracts/`（接口/数据/边界 SSOT）> `decisions.md`（取舍与例外）> `任务分工建议.md`（所有权与节奏）> 模块 README。

---

## 1. 文件清单与阅读顺序

| 顺序 | 文件 | 内容 | 主要 owner | 谁必须读 |
|---|---|---|---|---|
| 1 | `contracts/README.md`（本文） | 版本化、变更流程、命名与编码约定 | A | 全员 |
| 2 | `contracts/interface-contracts.md` | Rust trait、Tauri commands/events、sidecar JSON-RPC、错误码、调用时序 | A（sidecar 段 C，检索段 B） | 全员 |
| 3 | `contracts/data-model.md` | 磁盘布局、领域对象字段、枚举/状态机、SQLite 表、Qdrant payload、Tantivy schema、导出包 | A（存储）、B（索引）、C（解析缓存） | 全员 |
| 4 | `contracts/architecture-boundaries.md` | 分层、依赖矩阵、调用禁令、共享模块治理、质量闸门 | A 主持，全员评审 | 全员 |
| 5 | `contracts/CHANGELOG.md` | `contract_version` 变更记录 | A | 全员 |
| — | `contracts/schemas/`（Phase-0 待建） | JSON Schema：`app-settings`、`kb-settings`、`export-manifest`、`events`、`sidecar` | 各字段 owner | 生成与校验用 |

配套：`decisions.md`（为什么这么定）、`docs/project-structure.md`（工程目录与 crate 依赖）、`AGENTS.md`（智能体入口）。

---

## 2. 版本化规则

单个全局版本号 **`contract_version`**，语义化 `MAJOR.MINOR.PATCH`，同时写入：

- `contracts/README.md` 顶部（本文件）
- `contracts/CHANGELOG.md` 最新条目
- `crates/contracts/src/version.rs` 的 `pub const CONTRACT_VERSION: &str`（Phase-1 起）
- 生成的 `apps/ui/src/lib/tauri-api/version.ts`（Phase-1 起）
- SQLite 的 `schema_version` 是**独立**版本号，不跟随 `contract_version`（见 `data-model.md` §6）

| 级别 | 触发条件 | 审批 |
|---|---|---|
| MAJOR | 删除/重命名字段或命令、改字段类型或单位、改枚举取值、改错误码语义、改表结构、改 payload 字段名、改 sidecar 报文 | A + 所有受影响模块 owner 显式确认 |
| MINOR | 新增可选字段、新增命令/事件/枚举分支（带默认值）、新增表/索引 | A + 受影响 owner |
| PATCH | 仅文档澄清、注释、示例、排版 | 任意 owner，A 知会 |

**兼容性承诺**：同一 MAJOR 内，新增字段必须可选且有默认值；消费者必须忽略未知字段（`serde(default)` + `#[serde(deny_unknown_fields)]` 仅用于命令入参，不用于事件）。

**弃用**：字段/命令标记 `deprecated_since` 后至少保留 1 个 MINOR 周期，在 CHANGELOG 记录，下一 MAJOR 删除。

---

## 3. 变更流程（唯一合法路径）

```
发现契约不足
  └─ 在 decisions.md 追加一条 DEC（状态 Proposed，写清影响模块与迁移方案）
       └─ 提 PR：contracts/*.md + CHANGELOG（+ schemas/，+ 生成类型）
            └─ 涉及方 owner 评审（MAJOR 需全部显式同意）
                 └─ 合入；crates/contracts 与 TS 类型重新生成
                      └─ 各模块实现 + tests/integration 用例
                           └─ 集成日回归
```

规则：

1. **禁止先改代码再补契约**。代码与契约不一致时，以契约为准并回滚代码，或先走流程改契约。
2. 契约 PR 必须标注 `Affects: <模块列表>`、`Contract-Version: <新版本>`、`DEC: <编号>`。
3. 跨模块 PR 缺少受影响 owner 的 approve 不得合并（仓库分支保护打开）。
4. 每周三 24:00 后冻结 MINOR/MAJOR 变更至周五集成日结束；紧急变更需 A + 受影响 owner 在 `decisions.md` 记录并标注 `emergency`。
5. 契约变更与对应实现不得长期分叉：同一 MR 序列内完成，跨周分叉需在 `decisions.md` 写明完成时间点与临时兼容层。

---

## 4. 命名与编码约定（跨语言统一）

| 项 | 约定 | 反例 |
|---|---|---|
| 命名风格 | Rust/DB/JSON 一律 `snake_case`；生成的 TS 为 `camelCase`（ts-rs 自动转换） | `kbId` / `KbID` |
| 枚举 | Rust 变体 `CamelCase`（`SearchMode::Minimal`）；JSON/DB 中**取值一律小写 `snake_case`**（`minimal` / `pending` / `copy`）；TS 生成为字符串字面量联合。**错误码/警告码是"码"不是枚举值**，保持 `SCREAMING_SNAKE_CASE`（`KB_BUSY`） | 把 `PENDING`、`Copy` 当 JSON 值 |
| ID | UUIDv7 字符串，小写带连字符（`0199a1f0-...`）；可排序 | 自增整数、随机短码 |
| 时间 | DB 存 `INTEGER`（Unix epoch **毫秒**）；JSON 存 RFC 3339 UTC 毫秒（`2026-10-26T05:44:03.123Z`） | 本地时区字符串、秒级 |
| 路径 | 契约内一律 **POSIX 相对路径**，`/` 分隔，相对知识库副本根；只有 `root_dir`/`abs_path` 显式命名者为绝对路径 | `C:\...`、反斜杠 |
| 哈希 | SHA-256，小写 hex，64 字符；前缀不写 `sha256:` | MD5、大写 |
| 大小/时长 | 字节 `u64`；时长毫秒 `u64`；分页 `offset/limit` | 秒、浮点 |
| 向量 | `f32` little-endian；维度见 `KbSettings.embedding.dim`；Qdrant 命名向量 `content` | 动态维度、多 unnamed vector |
| 布尔 | 不用 `0/1` 语义重载；`Option<bool>` 表示 “未设置” | `del_flag` |
| 计数/进度 | `progress: {done, total}`，`total=0` 表示未知 | 百分比字符串 |

---

## 5. 校验与生成（Phase-1 落地，先立规则）

| 闸门 | 做法 | 失败即阻断 |
|---|---|---|
| 契约版本一致 | 脚本比对 README / CHANGELOG / `version.rs` / `version.ts` 四处 `contract_version` | 是 |
| Schema 合法 | `schemas/*.schema.json` 通过 JSON Schema meta-schema 校验；示例文件通过自身 schema | 是 |
| 生成代码最新 | CI 重新生成 Rust→TS 与 command client，工作区有 diff 则失败 | 是 |
| 依赖方向 | `scripts/check-deps` 依据 `architecture-boundaries.md` §2 矩阵检查 crate 依赖 | 是 |
| 禁令扫描 | `scripts/check-boundaries` 扫描禁止的 `use`/`import`（见 `architecture-boundaries.md` §3.2） | 是 |
| 迁移校验 | 所有 migration 可在空库顺序执行且 `schema_version` 单调递增；禁止修改已合入的 migration | 是 |

`contracts/*.md` 内的表格与 JSON 片段是**规范文本**，不是可执行配置；可执行配置在 `schemas/` 与代码中，二者必须文字一致。

---

## 6. Phase-0 待冻结清单（第 0 周）

| # | 交付 | owner | 完成判据 |
|---|---|---|---|
| 1 | 本文 + `interface-contracts.md` + `data-model.md` + `architecture-boundaries.md` + `CHANGELOG.md` | A | 全员评审通过，`contract_version=0.1.0` |
| 2 | `SearchRequest/SearchResult/SearchHit/PromptPackage` 字段冻结 | B | 与 `data-model.md` §2.7–§2.8 一致，含 `retriever_scores` |
| 3 | Qdrant payload、collection 命名、Tantivy schema | B | 字段名与 `data-model.md` §3.1–§3.2 一致 |
| 4 | sidecar JSON-RPC 报文 + 生命周期 | C | `initialize/health/parse_file/cancel/shutdown` 五个方法齐备 |
| 5 | `FileNode/DocumentChunk/SyncEvent/ParsedDocument` + 解析缓存布局 | C | 状态枚举闭合，可表达失败原因 |
| 6 | SQLite 全表 DDL 评审（只评审，不写实现） | A | 表/列/索引/外键齐备，`schema_version=1` |
| 7 | `AppError` 错误码表 | A | 每个 code 有触发条件与用户提示 key |
| 8 | JSON Schema 五件套 | 各 owner | 通过 meta-schema 校验 |

> 冻结后任何字段增删都按 §2/§3 执行；**冻结前允许大改，冻结后只允许流程内变更**。
