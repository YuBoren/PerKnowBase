# 验收追溯矩阵（acceptance-traceability）

> 目的：落实立项报告“目标达成判定”第 7 条——**“研究目标—功能需求—验收标准—责任人—契约落点—测试落点”一一对应**。
> 状态：Phase-0 骨架（责任人/契约落点已定，测试用例列在 Phase-1 起逐项填充）。
> 维护：A（结构）、各 owner（自己行）、D（测试落点汇总）。新增/修改必须同步 `立项报告草稿.md` 与本表。

---

## 1. 功能需求追溯

| FR | 需求摘要 | 验收 | 主责 | 协作 | 契约落点 | 测试落点 | 状态 |
|---|---|---|---|---|---|---|---|
| FR-01 | 知识库创建与副本隔离 | F-01 | A | C | `CreateKbRequest`、`KnowledgeBase`、`files/` 布局（data-model §1.2） | `tests/integration/scenarios/kb-create` | 待实现 |
| FR-02 | 知识依据目录浏览/增删/打开/定位 | F-02 | C | A、D | `FileNode`、`fs_*` 命令、`FileSystemService` | `scenarios/fs-browse` | 待实现 |
| FR-03 | 多格式解析、分块、索引 | F-12 | C | B | `ParsedDocument`、`Chunker`、`FileKind` 矩阵（data-model §5.2） | `scenarios/parse-matrix` | 待实现 |
| FR-04 | 图片语义解析与以文搜图 | F-11 | C | B | `EmbeddingRecord`、`image` chunk/块、`SourceLocation.image` | `scenarios/image-search` | 待实现 |
| FR-05 | 极简模式 | F-03 | B | D | `SearchRequest/Result/Hit`、`SourceLocation` | `scenarios/search-minimal` | 待实现 |
| FR-06 | 总结模式（含降级） | F-04 | B | D | `SummaryResponse`、`Citation`、`degraded` | `scenarios/search-summary` | 待实现 |
| FR-07 | 测试模式 | F-05 | B | D | `PromptPackage.rendered_prompt` | `scenarios/search-test` | 待实现 |
| FR-08 | 增量同步（手动/自动） | F-06、P-03 | C | B | `SyncEvent/SyncTask`、`SyncActionKind` 映射（data-model §5.4） | `scenarios/sync-incremental` | 待实现 |
| FR-09 | 导入导出 | F-07 | A | C、B | `ExportManifest`、ZIP 布局（data-model §6） | `scenarios/export-import` | 待实现 |
| FR-10 | 知识库融合 | F-08 | A | C、B | `MergeRequest`、`merge_op`/`merge_journal` | `scenarios/merge` | 待实现 |
| FR-11 | 多知识库切换 | F-14（待回填） | A | D | `kb_switch`、`KbStatus` | `scenarios/kb-switch` | 待验收条目 |
| FR-12 | 多标签并行检索 | F-09、P-04 | D | B | tab 隔离、`tab_id`、并发预算 | `e2e/multi-tab` | 待实现 |
| FR-13 | 检索历史 | F-10 | B | D | `HistoryRecord`、软引用快照（data-model §2.13） | `scenarios/history` | 待实现 |
| FR-14 | 全局搜索（【扩展】） | F-13 | B | D | `SearchScope.global`（本期拒绝） | `scenarios/global-搜索`（跳过） | 扩展 |
| FR-15 | 设置面板 | F-15（待回填） | A | D | `AppSettings`、`KbSettings`、`UpdateKbSettingsResult` | `scenarios/settings` | 待验收条目 |
| FR-16 | LLM API 网关 | F-16（待回填） | B | A | `LlmGateway`、`LlmCfg`、`LLM_*` 错误码 | `scenarios/llm-gateway` | 待验收条目 |
| FR-17 | 多模态与插件化（【扩展】） | 扩展 | C | B | `ParsedDocument`/`FileKind`/`IndexService` 的扩展点 | 不测 | 扩展 |

## 2. 非功能与验收

| 编号 | 指标 | 主责 | 契约/实现落点 | 测试落点 |
|---|---|---|---|---|
| P-01 | 极简 p95 ≤ 500ms | B | `SearchResult.elapsed_ms`、索引参数（data-model §3.1） | `bench/search_p95` |
| P-02 | 嵌入 ≥ 50 页/分钟 | C | `Embedder`、批量参数 | `bench/embed_throughput` |
| P-03 | 单文件增量 ≤ 30s | C | `SyncTask` 计时 | `bench/sync_single` |
| P-04 | 5 标签并发 ≤ 1.5× | B、D | 并发限流（默认 2/2/1）、tab 隔离 | `e2e/multi-tab-perf` |
| P-05 | 空闲内存 ≤ 500MB | A、D | 进程模型（DEC-20261026-02）、sidecar 空闲回收 | `bench/idle_mem` |
| P-06 | 冷启动 ≤ 5s | A | `main.rs` 启动路径、索引懒加载 | `bench/cold_start` |
| C-01/C-02 | Win10+/Ubuntu22.04+ | D、A | 打包与 CI 矩阵 | `e2e/platform-smoke` |
| C-03 | 移动端（【扩展】） | D | Tauri 2.x 构建可编译 | `ci/mobile-build`（仅编译） |
| C-04 | 断网可用（除 LLM） | A | 无启动期外联（architecture-boundaries §6） | `e2e/offline` |
| NFR-03 | 崩溃不损坏、RTO/RPO | A | WAL、`.tmp` 原子 rename、`merge_journal` 回滚 | `scenarios/crash-recovery` |
| NFR-06 | 密钥加密、不外传 | A、B | `settings_set_llm_secret`、BAN-16 | `scenarios/secret-handling` |
| NFR-09 | 许可证完整 | A | `LICENSES/THIRD-PARTY.md`、CI 检查 | CI `licenses` |

## 3. 使用规则

1. 每个 FR 至少有 1 个 integration 场景；否则该行状态不得标“已验收”。
2. 验收条目标“待回填/待验收条目”（F-14/F-15/F-16）时，按立项报告 FR 描述的量化要求验收，并在验收记录注明依据（见 `decisions.md` DEC-20261026-11）。
3. 契约变更导致本表落点变化时，同一 PR 更新本表。
4. 本表不替代立项报告的验收标准原文；冲突以立项报告为准。
