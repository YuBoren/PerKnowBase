# perknow-llm（L2）

> owner：B　|　状态：Phase-0 空壳（只有 `lib.rs` 文档注释，无实现）　|　契约：见下

## 职责

OpenAI 兼容客户端、超时/重试/降级、Prompt 渲染工具。

## 边界

- 允许依赖：`perknow-contracts`、`perknow-core`
- 禁止：`tauri`、`rusqlite`、`qdrant-client`、`tantivy`、任一 L3
- 依赖方向与禁令：`contracts/architecture-boundaries.md` §2/§3.2；违规由 `scripts/check-deps.ps1`、`scripts/check-boundaries.ps1` 拦截。

## 契约指针

- `contracts/interface-contracts.md` §3.8
- `contracts/data-model.md` §2.9
- 本 crate 依赖的外部库：（Phase-1 由 B 按 DEC 引入：reqwest 等）

## 测试

`cargo test -p perknow-llm`（Phase-0 无测试；单测写在 `src/` 内的 `#[cfg(test)]`，集成用例放 `tests/integration`）。

## 约定

- 公共 API 只从 `src/lib.rs` `pub use` 导出，禁止外部访问私有模块（BAN-09）。
- 错误一律返回 `perknow_contracts::AppError`，不在库里 `unwrap`/`expect`。
- 日志不含正文、密钥、绝对路径（BAN-16）。
