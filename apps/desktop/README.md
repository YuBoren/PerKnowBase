# apps/desktop（Tauri 壳，A）

> 目录结构与规则见 [`docs/project-structure.md`](../../../docs/project-structure.md) §4.1；命令清单见 [`contracts/interface-contracts.md`](../../../contracts/interface-contracts.md) §4。

## 1. 职责

只做四件事：**参数反序列化 → 调 L3 服务 trait → `AppError` 映射 → 返回 DTO**。
不写解析、分块、拼 prompt、算 hash、重试编排（BAN-06），也不直接依赖 `qdrant-client`/`rusqlite`/`tantivy`/`ort`。

## 2. 目录

```
src-tauri/
├─ Cargo.toml
├─ build.rs                 # tauri_build::build()
├─ tauri.conf.json          # 窗口、CSP、bundle
├─ capabilities/default.json# 最小权限：仅 core:default
├─ icons/                   # 占位图标（Phase-0 生成，发布前替换为正式素材）
└─ src/
   ├─ main.rs               # 进程入口（windows_subsystem 属性）
   ├─ lib.rs                # Builder、manage(AppState)、invoke_handler 注册、run
   ├─ commands/
   │  ├─ mod.rs             # 命令模块入口（A 独占）：注册说明与模块声明
   │  ├─ table.rs           # 39 条命令的清单（name/owner/status），供校验脚本比对
   │  └─ app_info.rs        # Phase-0 唯一实现：返回版本与契约版本
   ├─ dto/mod.rs            # 入参 DTO（Phase-1 从 perknow-contracts 引入，deny_unknown_fields）
   ├─ error.rs              # AppError → 前端结构（1:1，字段已在契约层定义）
   ├─ events.rs             # EventSink 实现：唯一 emit 点
   └─ state.rs              # 打开中的知识库与会话状态
```

## 3. 命令新增流程

1. 先改 `contracts/interface-contracts.md` §4 + `crates/contracts`（MINOR/MAJOR 按 `contracts/README.md` §2）；
2. 在 `commands/table.rs` 增一行 `CommandSpec`（`status: Pending`）；
3. 实现 `commands/<域>.rs`，在 `lib.rs` 的 `.invoke_handler(tauri::generate_handler![...])` 里加一行，状态改 `Implemented`；
4. 跑 `pwsh scripts/validate-contracts.ps1`（会比对文档表与 `table.rs`）；
5. 由 D 跑 `scripts/gen-types` 重新生成 `apps/ui/src/lib/tauri-api`（生成物禁止手改，BAN-08）。

## 4. 权限

`capabilities/default.json` 只给 `core:default`。文件打开/资源管理器不通过 `shell`/全量 `fs` 插件，
而是由 `crates/fs` 调 OS API（`contracts/architecture-boundaries.md` §6）。新增权限必须在 PR 说明原因并同步本文件。

## 5. 本地运行（Phase-1 可跑）

```powershell
pnpm --dir apps/ui install   # 前端依赖（D 维护）
pnpm dev                     # 起 Vite + Tauri（需要 apps/ui 壳就绪）
cargo run -p perknow-desktop # 只跑 Rust 侧（需要前端 dist 或 devUrl 可用）
```

Phase-0 状态：壳骨架，只有 `app_info` 一个命令可用；其余命令在 `table.rs` 中登记为 `Pending`。
