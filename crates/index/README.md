# perknow-index（L2）

> owner：B　|　状态：Phase-0 空壳（只有 `lib.rs` 文档注释，无实现）　|　契约：见下

## 职责

Qdrant/Tantivy 适配、Qdrant 进程生命周期与端口回退、快照、BM25 分词（唯一索引出口）。

## 边界

- 允许依赖：`perknow-contracts`、`perknow-core`
- 禁止：`tauri`、`rusqlite`、`perknow-storage`、`perknow-pipeline`（BAN-04/BAN-13）
- 依赖方向与禁令：`contracts/architecture-boundaries.md` §2/§3.2；违规由 `scripts/check-deps.ps1`、`scripts/check-boundaries.ps1` 拦截。

## 契约指针

- `contracts/data-model.md` §3
- `contracts/interface-contracts.md` §3.7
- 本 crate 依赖的外部库：（Phase-1 由 B 按 DEC 引入：qdrant-client、tantivy、tantivy-jieba）

## 测试

`cargo test -p perknow-index`（Phase-0 无测试；单测写在 `src/` 内的 `#[cfg(test)]`，集成用例放 `tests/integration`）。

## 约定

- 公共 API 只从 `src/lib.rs` `pub use` 导出，禁止外部访问私有模块（BAN-09）。
- 错误一律返回 `perknow_contracts::AppError`，不在库里 `unwrap`/`expect`。
- 日志不含正文、密钥、绝对路径（BAN-16）。
