# migrations — SQLite 表结构升级脚本（A 独占）

> 定义与执行时机见 [`docs/project-structure.md`](../../../docs/project-structure.md) §2.1；表结构契约见 [`contracts/data-model.md`](../../../contracts/data-model.md) §4。
> 本文件只写“怎么放、怎么写、怎么验”。

## 1. 两条链

```
crates/storage/migrations/
├─ app/0001_init.sql   # <app_data>/app.sqlite3（全局一份）：注册表、应用设置
└─ kb/0001_init.sql    # <kb_root>/kb.sqlite3（每库一份）：元数据、块、历史、融合、重构
```

- 文件名固定 `<4 位版本>_<snake_case 描述>.sql`，版本从 `0001` 起连续、单调，不跳号、不复用。
- 每个库各有 `schema_migrations` / `app_schema_migrations` 账本（`version, applied_at, checksum`）。
- **只追加、不改已合入脚本**（BAN-15）：脚本一旦发布，其 checksum 就是库的一部分；改了会让所有旧库报 `KB_CORRUPTED`。

## 2. 执行时机

| 场景 | 动作 |
|---|---|
| 新建知识库 | 空 `kb.sqlite3` → 从 `kb/0001` 顺序执行到最新 |
| 打开旧库 | 读账本最高版本，只执行更大的脚本 |
| 导入旧导出包 | 按包内 `schema_version` 补齐缺失脚本；包版本更高 → `KB_SCHEMA_TOO_NEW` |
| 应用启动 | 对 `app.sqlite3` 做同样的事 |

连接层负责 `PRAGMA journal_mode=WAL; foreign_keys=ON; busy_timeout=5000`（`data-model.md` §4 表头），
脚本内只在 `kb/0001` 显式开了 WAL（一次性、需在事务外执行）。

## 3. 刻意不做的两件事

1. **枚举列不加 `CHECK` 约束**：新增枚举取值属于契约 MINOR（`contracts/README.md` §2），加 `CHECK` 会迫使整表重建，与之冲突。取值校验在 Rust 侧（`crates/contracts` 的 serde 枚举）。
2. **不在脚本里搬数据**：需要“搬数据”的迁移另立 `DEC` 并写独立的数据迁移流程；`migrations/` 只做 DDL。

## 4. 校验

```powershell
pwsh scripts/check-migrations.ps1
```

校验项：文件名/版本连续单调、两条链都能在**空库**上顺序执行成功、脚本内不得出现
`DROP TABLE`／`ALTER TABLE ... RENAME` 之外的破坏性语句、每个脚本末尾语句可解析。

## 5. 新增一个脚本

1. 先改 `contracts/data-model.md` §4（表/列/索引）与 `crates/contracts` 的类型，走 `[shared]` PR；
2. 新增 `NNNN_<描述>.sql`（**不修改旧脚本**）；
3. 同步 `crates/contracts/src/version.rs` 的 `SCHEMA_VERSION`；
4. 跑 `pwsh scripts/check-migrations.ps1`，并在 PR 里写 `Affects:`／`DEC:`。
