# 契约变更记录（CHANGELOG）

> 记录 `contract_version` 变化；每次变更只在顶部新增一条（版本、日期、级别、影响面、DEC），不在此重述字段。
> 版本号须与 `README.md` 顶部、`crates/contracts/src/version.rs`、生成的 `lib/tauri-api/version.ts` 一致。

## [0.1.0] - 2026-10-28 - 初始冻结

- 范围：接口契约、数据模型契约（磁盘布局、对象/枚举、表结构、Qdrant/Tantivy、导出包）、架构边界与禁令。
- 格式矩阵含 `.xlsx`（`FileKind.xlsx`）；全局搜索为【扩展】，仅保留 `SearchScope.global` 预留。
- 相关决策：见 `decisions.md` 索引（DEC-20261026-*、DEC-20261028-*）。
- 未完成：等待评审签字（PX-02）后冻结。

### 0.1.0 预冻结补齐（2026-10-31，不改变字段/命令/事件，版本号不变）

- **新增** `contracts/schemas/` 五件套 + `examples/` 6 个样本：`app-settings`、`kb-settings`、`export-manifest`、`events`、`sidecar`（PA-03）；使用说明见 `README.md` §6（PA-04）。
- **补充** `data-model.md` §3.1.1「Qdrant 进程契约」：启动参数、端口回退、单实例、就绪/健康、重启退避、关闭、数据目录迁移、版本升级（PX-01，DEC-20261031-25，待 B 确认）。
- **澄清** `data-model.md` §4.2 `merge_journal.status`：明确为**条目级**状态（`planned`/`done`/`rolled_back`），整体状态在 `merge_op.status`（PATCH，无行为变化）。
- **裁决** `BlockType` 取值：按 §2.4 字面用大写；`README.md` §4 补“代码型枚举报大写”例外（DEC-20261031-26，待 C 确认）。
- **代码** `crates/contracts` 落地对应类型与 `CONTRACT_VERSION`/`SCHEMA_VERSION`/`PARSER_PROTOCOL_VERSION`（PA-05）；`crates/core` 落地 `EventSink`/`CancelToken`/路径与哈希工具（PA-06）。
