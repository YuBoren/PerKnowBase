# perknow-core（L1）

> owner：A　|　状态：Phase-0 空壳（只有 `lib.rs` 文档注释，无实现）　|　契约：见下

## 职责

ID/时间/哈希、路径规范化与越界校验、`CancelToken`、`EventSink` 抽象、`AppError` 构造。

## 边界

- 允许依赖：`perknow-contracts`
- 禁止：`tauri`（BAN-05）、`rusqlite`、`qdrant-client`、`tantivy`、`ort`、任一 L2/L3
- 依赖方向与禁令：`contracts/architecture-boundaries.md` §2/§3.2；违规由 `scripts/check-deps.ps1`、`scripts/check-boundaries.ps1` 拦截。

## 契约指针

- `contracts/architecture-boundaries.md` §3.2（BAN-05/12/16）
- `contracts/interface-contracts.md` §1/§5
- 本 crate 依赖的外部库：serde / serde_json / uuid / sha2 / thiserror / tokio（sync）/ tracing

## 测试

`cargo test -p perknow-core`（Phase-0 无测试；单测写在 `src/` 内的 `#[cfg(test)]`，集成用例放 `tests/integration`）。

## 约定

- 公共 API 只从 `src/lib.rs` `pub use` 导出，禁止外部访问私有模块（BAN-09）。
- 错误一律返回 `perknow_contracts::AppError`，不在库里 `unwrap`/`expect`。
- 日志不含正文、密钥、绝对路径（BAN-16）。
