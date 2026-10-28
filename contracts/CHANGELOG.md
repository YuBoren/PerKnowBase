# 契约变更记录（CHANGELOG）

> 记录 `contract_version` 变化；每次变更只在顶部新增一条（版本、日期、级别、影响面、DEC），不在此重述字段。
> 版本号须与 `README.md` 顶部、`crates/contracts/src/version.rs`、生成的 `lib/tauri-api/version.ts` 一致。

## [0.1.0] - 2026-10-28 - 初始冻结

- 范围：接口契约、数据模型契约（磁盘布局、对象/枚举、表结构、Qdrant/Tantivy、导出包）、架构边界与禁令。
- 格式矩阵含 `.xlsx`（`FileKind.xlsx`）；全局搜索为【扩展】，仅保留 `SearchScope.global` 预留。
- 相关决策：见 `decisions.md` 索引（DEC-20261026-*、DEC-20261028-*）。
- 未完成：`contracts/schemas/*.schema.json`（settings / manifest / events / sidecar）待第 1 周补齐。
