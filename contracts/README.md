# 契约总览与治理（contracts/）

> 状态：Phase-0 冻结中（只有文档，无实现代码）。任何跨模块改动先读本目录。
> 阅读顺序、事实源层级、PR 与分支规则见 [`AGENTS.md`](../AGENTS.md)，此处不重复。

## 1. 文件与 owner

| 文件 | 内容 | owner |
|---|---|---|
| `README.md` | 版本、变更流程、命名约定 | A |
| `interface-contracts.md` | Rust trait、Tauri 命令/事件、sidecar 协议、错误码 | A（sidecar 段 C，检索段 B） |
| `data-model.md` | 磁盘布局、对象字段、枚举/状态、SQLite 表、Qdrant/Tantivy、导出包 | A（存储）、B（索引）、C（解析缓存） |
| `architecture-boundaries.md` | 分层、依赖矩阵、禁令、共享模块规则 | A 主持，全员评审 |
| `CHANGELOG.md` | `contract_version` 记录 | A |
| `schemas/`（待建） | JSON Schema：settings、manifest、events、sidecar | 各字段 owner |

## 2. 版本化

`contract_version`（`MAJOR.MINOR.PATCH`，与软件版本无关）同时写在 `README.md` 顶部、`CHANGELOG.md`、`crates/contracts/src/version.rs`、生成的 `lib/tauri-api/version.ts`。

| 级别 | 触发 | 审批 |
|---|---|---|
| MAJOR（不兼容） | 删/重命名字段或命令、改类型/单位/枚举取值、改表结构、改 payload | A + **所有**受影响 owner 显式同意，附迁移方案 |
| MINOR（兼容新增） | 加可选字段（有默认值）、加命令/事件/枚举分支、加表/索引 | A + 受影响 owner；48h 无异议可过 |
| PATCH（无行为变化） | 文档澄清、示例、注释 | 任意 owner，A 知会 |

同一 MAJOR 内旧客户端可用：新字段可选、消费者忽略未知字段；废弃项至少保留一个 MINOR。SQLite 的 `schema_version` 独立编号，不跟随契约版本。

## 3. 变更流程（唯一路径）

```
decisions.md (Proposed) → contracts/*.md + CHANGELOG.md → crates/contracts → 生成 TS → 实现 → tests/integration
```

- 禁止先改代码再补契约；禁止只改 `crates/contracts` 不改 `contracts/*.md`。
- 契约 PR 标注 `Affects:`、`DEC:`、`Contract-Version:`；MAJOR 需涉及方显式确认。
- 冻结窗口：周三 24:00 至周五集成日结束；紧急变更标 `emergency` 并写 DEC。
- 契约与实现不得长期分叉；跨周分叉需在 DEC 写明完成时间与临时兼容层。

## 4. 命名与编码约定

| 项 | 约定 |
|---|---|
| 命名 | Rust/DB/JSON `snake_case`；生成 TS 为 `camelCase`；枚举取值小写（`minimal`/`pending`/`copy`），错误码与警告码大写（`KB_BUSY`） |
| ID | UUIDv7，小写带连字符，可排序 |
| 时间 | DB 存 epoch **毫秒**（INTEGER）；JSON 为 RFC 3339 UTC 毫秒 |
| 路径 | 契约内一律 POSIX 相对路径（相对知识库副本根）；仅 `root_dir`/`abs_path` 为绝对路径 |
| 哈希 | SHA-256 小写 hex，64 字符，不写 `sha256:` 前缀 |
| 数值 | 字节 `u64`；时长毫秒；分页 `offset/limit`（`limit <= 200`）；进度 `{done, total}`，`total=0` 表示未知 |
| 向量 | `f32` little-endian，维度见 `KbSettings.embedding.dim`；Qdrant 命名向量 `content` |
| 空值 | `null` 表示“无”，空串表示“空内容”；禁止用 `0`/`""` 重载“未设置” |

## 5. 校验（Phase-1 落地）

`scripts/validate-contracts`（版本四处一致、schema 与示例合法）、生成物 diff 必须为空、migration 空库顺序执行且版本单调、依赖方向与禁令扫描（`check-deps`/`check-boundaries`）。任何一项失败即阻断合并。
