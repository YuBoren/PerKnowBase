# perknow-contracts（L0）

> owner：A（主笔）　|　状态：Phase-0 空壳（只有 `lib.rs` 文档注释，无实现）　|　契约：见下

## 职责

领域类型、枚举、错误码、事件名、版本常量（`CONTRACT_VERSION` / `SCHEMA_VERSION` / `PARSER_PROTOCOL_VERSION`）；`ts` feature 下用 `ts-rs` 导出 TS 类型。

## 边界

- 允许依赖：（无 workspace 内部依赖）
- 禁止：任何 workspace crate（防循环）；`tauri`、`rusqlite`、`qdrant-client`、`tantivy`、`ort` 等引擎库
- 依赖方向与禁令：`contracts/architecture-boundaries.md` §2/§3.2；违规由 `scripts/check-deps.ps1`、`scripts/check-boundaries.ps1` 拦截。

## 契约指针

- `contracts/data-model.md` §2/§5（字段与枚举）
- `contracts/interface-contracts.md` §2/§4/§5（错误码/命令/事件）
- 本 crate 依赖的外部库：serde / serde_json / uuid（可选 ts-rs：`--features ts`）

## 测试

`cargo test -p perknow-contracts`（Phase-0 无测试；单测写在 `src/` 内的 `#[cfg(test)]`，集成用例放 `tests/integration`）。

## 约定

- 公共 API 只从 `src/lib.rs` `pub use` 导出，禁止外部访问私有模块（BAN-09）。
- 错误一律返回 `perknow_contracts::AppError`，不在库里 `unwrap`/`expect`。
- 日志不含正文、密钥、绝对路径（BAN-16）。
