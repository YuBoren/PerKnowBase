# 契约总览与治理（contracts/）

> 状态：Phase-0 冻结中（只有文档，无实现代码）。任何跨模块改动先读本目录。
> 阅读顺序、事实源层级、PR 与分支规则见 [`AGENTS.md`](../AGENTS.md)，此处不重复。

## 1. 文件与 owner

> 契约由 **A 主笔初稿**，B/C/D 按模块审核各自段落；任何修改仍走 PR（见 AGENTS §4.3）。

| 文件 | 内容 | 撰写 | 审核 |
|---|---|---|---|
| `README.md` | 版本、变更流程、命名约定 | A | 全员 |
| `interface-contracts.md` | Rust trait、Tauri 命令/事件、sidecar 协议、错误码 | A | C（sidecar 段）、B（检索/索引段）、D（命令/事件前端可用性） |
| `data-model.md` | 磁盘布局、对象字段、枚举/状态、SQLite 表、Qdrant/Tantivy、导出包 | A | B（索引/检索段）、C（解析/对象段）、D（前端可见字段） |
| `architecture-boundaries.md` | 分层、依赖矩阵、禁令、共享模块规则 | A | 全员 |
| `CHANGELOG.md` | `contract_version` 记录 | A | 随契约 PR 评审 |
| `schemas/` | JSON Schema 五件套 + `examples/`（settings×2、manifest、events、sidecar） | A | B/C/D 按字段 |

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
| 命名 | Rust/DB/JSON `snake_case`；生成 TS 为 `camelCase`；**业务状态类**枚举取值小写（`minimal`/`pending`/`copy`）；**代码型**枚举大写（错误码 `KB_BUSY`、警告码 `OCR_REQUIRED`、块类型 `TEXT`），以对应章节字面为准（DEC-20261031-26） |
| ID | UUIDv7，小写带连字符，可排序 |
| 时间 | DB 存 epoch **毫秒**（INTEGER）；JSON 为 RFC 3339 UTC 毫秒 |
| 路径 | 契约内一律 POSIX 相对路径（相对知识库副本根）；仅 `root_dir`/`abs_path` 为绝对路径 |
| 哈希 | SHA-256 小写 hex，64 字符，不写 `sha256:` 前缀 |
| 数值 | 字节 `u64`；时长毫秒；分页 `offset/limit`（`limit <= 200`）；进度 `{done, total}`，`total=0` 表示未知 |
| 向量 | `f32` little-endian，维度见 `KbSettings.embedding.dim`；Qdrant 命名向量 `content` |
| 空值 | `null` 表示“无”，空串表示“空内容”；禁止用 `0`/`""` 重载“未设置” |

## 5. 校验（Phase-0 已可跑）

| 检查 | 脚本 | 说明 |
|---|---|---|
| 契约版本四处一致、schema/示例、命令表/错误码/事件名 | `scripts/validate-contracts.ps1`（+ `validate-schemas.py`） | 见 §6 |
| 依赖方向 | `scripts/check-deps.ps1` | 对照 `architecture-boundaries.md` §2 |
| 禁令扫描 | `scripts/check-boundaries.ps1` | 对照 §3.2 的 BAN |
| 迁移空库顺序执行、版本单调 | `scripts/check-migrations.ps1`（+ `.py`） | 两条链各自执行 |
| 第三方许可已登记 | `scripts/check-licenses.ps1` | 对照 `LICENSES/THIRD-PARTY.md` |
| 生成物 diff | `scripts/gen-types.ps1`（D，Phase-1） | diff 必须为空 |

任何一项失败即阻断合并；跳过检查视为未完成（`AGENTS.md` §8）。

---

## 6. JSON Schema 的使用（`contracts/schemas/`）

> 用途：把“字段名/类型/必填/取值”从散文变成**机器可校验**的约束；写设置、导出包、事件、sidecar 报文时先对照 schema，CI 会校验示例与 schema 是否一致。
> 文档仍以 `data-model.md`／`interface-contracts.md` 为准；schema 与文档冲突时**改 schema**，并走同一 PR。

### 6.1 五件套分别是什么

| 文件 | 对应契约 | 对应 Rust 类型 | 谁会用 |
|---|---|---|---|
| `app-settings.schema.json` | `data-model.md` §2.14 | `AppSettings` | A（settings）、D（设置页） |
| `kb-settings.schema.json` | `data-model.md` §2.14 | `KbSettings` | A（建库/重构判定）、B（检索参数覆盖） |
| `export-manifest.schema.json` | `data-model.md` §2.12/§6 | `ExportManifest` | A（导入导出校验） |
| `events.schema.json` | `interface-contracts.md` §5 | `Event` + 7 个载荷结构 | 全员（发布者/订阅者） |
| `sidecar.schema.json` | `interface-contracts.md` §6、`data-model.md` §2.4 | JSON-RPC 帧、`ParsedDocument` | C（sidecar）、A（进程管理） |

约定：

- `additionalProperties: false` 用于**我们写的**结构（设置、清单）——拼错字段名要立刻报错。
- 事件与 sidecar 的**载荷**允许未知字段（订阅方必须容忍，§5 规则 4）；新增字段仍要先改契约（MINOR）。
- 时间用 `format: date-time`（RFC 3339 UTC 毫秒），与 §4 一致；schema 不替代运行时校验，只做结构约束。

### 6.2 怎么写、怎么用

1. **写设置 JSON**（导入/导出、测试数据）：先看 §6.1 对应 schema 的 `required` 与 `enum`，不要凭记忆拼字段。
2. **实现校验**：Rust 侧以 `crates/contracts` 的 serde 类型为准（反序列化即校验，`deny_unknown_fields` 与 `additionalProperties: false` 对应）；Python/TS 脚本可用 schema 做前置校验。
3. **手工看示例**：`schemas/examples/` 下每个文件都是**能通过校验的最小完整样本**，可直接当测试 fixture 的蓝本（正式 fixtures 由 C 的 PC-03 落地）。
4. **报错信息**：校验失败时把 `instance_path` 与关键字（`required`/`enum`/`pattern`）一起给出，便于定位字段。

### 6.3 示例文件与命名约定

命名：`<schema 名>.<case>.json`，前缀必须等于某个 schema 去掉 `.schema.json` 后的名字。

| 示例 | 覆盖内容 |
|---|---|
| `examples/app-settings.default.json` | 默认应用设置（混合检索 0.5/0.5、60s 监听、240 字预览） |
| `examples/kb-settings.default.json` | 默认库级设置（recursive 分块、768 维嵌入、standard 解析） |
| `examples/export-manifest.full.json` | 完整 `manifest.json`（含 1 个文件条目与校验和描述） |
| `examples/events.op-done.json` | `kb://op-done` 事件（成功终态、无错误） |
| `examples/sidecar.initialize-request.json` | `initialize` 请求帧（协议版本、并发上限） |
| `examples/sidecar.parse-response.json` | `parse_file` 成功响应（`ParsedDocument` 两个块） |

### 6.4 CI 如何校验

```
scripts/validate-contracts.ps1
  ├─ 结构检查：五件套齐全、JSON 可解析、示例命名前缀正确、每个 schema 至少一个示例
  └─ 深度检查：scripts/validate-schemas.py
       ├─ 有 jsonschema：check_schema + Draft202012Validator 逐示例校验（CI 走这条）
       └─ 无 jsonschema：内置最小校验器（只覆盖本仓库用到的关键字），打印 engine=builtin
```

- CI 作业 `contracts` 会 `pip install jsonschema` 后执行，**不允许**拿内置校验器通过当作“已通过”。
- 失败即阻断合并；修 schema 或修示例，不许删检查。
- 新增 schema/示例：同一 PR 更新 §6.1 表与 `contracts/CHANGELOG.md`。

### 6.5 常见错误

| 现象 | 原因 | 处理 |
|---|---|---|
| `unexpected property "foo"` | 结构体已改但 schema 没改（或反之） | 同步 `crates/contracts` + schema，走 MINOR/PATCH |
| `oneOf matched 0 branches` | `hybrid.kind` 拼错，或 `weights` 不在 0..1 | 对照 `data-model.md` §2.7 的三种形态 |
| `string does not match pattern` | 哈希不是 64 位小写 hex；`rel_path` 不是 POSIX 相对路径 | 用 `crates/core` 的 `sha256_hex`／`validate_rel_path` 生成 |
| 示例通过但运行时崩 | schema 只查结构，不查语义（如 `rrf_k` 与模式组合） | 语义校验放 Rust（`crates/contracts` + 服务层） |

