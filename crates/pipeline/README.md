# perknow-pipeline（L2）

> owner：C　|　状态：Phase-0 空壳（只有 `lib.rs` 文档注释，无实现）　|　契约：见下

## 职责

`DocumentParser`（native + sidecar 客户端）、`Chunker`（纯函数）、`Embedder`（ONNX）、解析缓存读写。

## 边界

- 允许依赖：`perknow-contracts`、`perknow-core`、`perknow-storage`（仅端口）
- 禁止：`tauri`、`qdrant-client`、`tantivy`、任一 L3；不得调用 `perknow-index`（BAN-13）
- 依赖方向与禁令：`contracts/architecture-boundaries.md` §2/§3.2；违规由 `scripts/check-deps.ps1`、`scripts/check-boundaries.ps1` 拦截。

## 契约指针

- `contracts/interface-contracts.md` §3.5
- §6
- `contracts/data-model.md` §2.4/§2.5/§7
- 本 crate 依赖的外部库：（Phase-1 由 C 按 DEC 引入：ort/onnxruntime、image 等）

## 测试

`cargo test -p perknow-pipeline`（Phase-0 无测试；单测写在 `src/` 内的 `#[cfg(test)]`，集成用例放 `tests/integration`）。

## 约定

- 公共 API 只从 `src/lib.rs` `pub use` 导出，禁止外部访问私有模块（BAN-09）。
- 错误一律返回 `perknow_contracts::AppError`，不在库里 `unwrap`/`expect`。
- 日志不含正文、密钥、绝对路径（BAN-16）。
