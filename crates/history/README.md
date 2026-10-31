# perknow-history（L3）

> owner：B　|　状态：Phase-0 空壳（只有 `lib.rs` 文档注释，无实现）　|　契约：见下

## 职责

`HistoryService`：历史写入/查询/删除/裁剪、软引用轻量快照。

## 边界

- 允许依赖：`contracts`、`core`、`storage`
- 禁止：`tauri`、`index`、`llm`、`search`、`kb`
- 依赖方向与禁令：`contracts/architecture-boundaries.md` §2/§3.2；违规由 `scripts/check-deps.ps1`、`scripts/check-boundaries.ps1` 拦截。

## 契约指针

- `contracts/interface-contracts.md` §3.8
- `contracts/data-model.md` §2.13
- 本 crate 依赖的外部库：（Phase-1 按 DEC 引入）

## 测试

`cargo test -p perknow-history`（Phase-0 无测试；单测写在 `src/` 内的 `#[cfg(test)]`，集成用例放 `tests/integration`）。

## 约定

- 公共 API 只从 `src/lib.rs` `pub use` 导出，禁止外部访问私有模块（BAN-09）。
- 错误一律返回 `perknow_contracts::AppError`，不在库里 `unwrap`/`expect`。
- 日志不含正文、密钥、绝对路径（BAN-16）。
