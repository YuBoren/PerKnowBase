# 第三方依赖许可清单

> 规则（BAN-20 / NFR-09）：**新增第三方依赖前**先在 `decisions.md` 写 DEC，再登记到本文件，最后才改 `Cargo.toml`/`package.json`。
> 本文件由 `scripts/check-licenses.ps1` 校验：清单里声明的依赖必须都能在这里找到对应行。
> 登记内容：包名 / 版本约束（与 workspace 锁定一致）/ 许可 / 用途 / 引入者。

## 1. Rust（当前已声明）

| 依赖 | 版本 | 许可 | 用途 | 引入者 |
|---|---|---|---|---|
| serde | 1.0 | MIT OR Apache-2.0 | 序列化/反序列化（全部契约类型） | A |
| serde_json | 1.0 | MIT OR Apache-2.0 | JSON、事件载荷、设置快照 | A |
| uuid | 1.10 | MIT OR Apache-2.0 | UUIDv7 主键与关联 id | A |
| thiserror | 2.0 | MIT OR Apache-2.0 | 错误类型派生（crate 内部错误） | A |
| sha2 | 0.10 | MIT OR Apache-2.0 | SHA-256（文件与块哈希） | A |
| tokio | 1.40 | MIT | `CancelToken` 的 `Notify`；后续异步运行时 | A |
| async-trait | 0.1 | MIT OR Apache-2.0 | 服务 trait 的 async 方法（P-1 起用） | A |
| tracing | 0.1 | MIT | 结构化日志（脱敏后） | A |
| ts-rs | 10.1 | MIT | Rust → TS 类型生成（`crates/contracts` 的 `ts` feature） | A |
| rusqlite | 0.32 | MIT | SQLite 访问（唯一出口 `crates/storage`，bundled 编译） | A |
| tauri | 2 | MIT OR Apache-2.0 | 桌面外壳与 IPC（仅 `apps/desktop`） | A |
| tauri-build | 2 | MIT OR Apache-2.0 | 外壳构建脚本与资源生成 | A |

## 2. 前端（尚未引入；D 的 PD-02/PD-03 落地时登记）

| 依赖 | 版本 | 许可 | 用途 | 引入者 |
|---|---|---|---|---|
| react | 19.x | MIT | UI 框架 | D |
| react-dom | 19.x | MIT | 渲染 | D |
| vite | 6.x | MIT | 构建与开发服务器 | D |
| typescript | 5.x | Apache-2.0 | 类型检查 | D |
| @tauri-apps/api | 2.x | MIT OR Apache-2.0 | `invoke`/`listen`（仅在 `lib/tauri-api`） | D |
| eslint | 9.x | MIT | 静态检查 | D |
| vitest | 3.x | MIT | 单元测试 | D |
| @playwright/test | 1.x | Apache-2.0 | E2E | D |

> 上表版本为规划值；实际引入时按 `pnpm` 锁定版本更新本表，并把许可核对结果写进 DEC。

## 3. Phase-1 计划引入（Rust，需在引入前确认许可并补 DEC）

| 依赖 | 计划用途 | 引入者 | 许可（待确认） |
|---|---|---|---|
| qdrant-client | Qdrant 访问（唯一出口 `crates/index`） | B | 待确认 |
| tantivy | BM25 全文索引（`crates/index`） | B | 待确认 |
| tantivy-jieba | 中文分词（DEC-20261026-13） | B | 待确认 |
| ort | ONNX Runtime 绑定（`crates/pipeline`） | C | 待确认 |
| notify | 文件监听（`crates/sync`） | C | 待确认 |
| walkdir | 目录遍历（`crates/fs`） | C | 待确认 |
| reqwest | LLM HTTP 客户端（`crates/llm`） | B | 待确认 |
| zip | 导入导出（`crates/kb`） | A | 待确认 |
| keyring 或 aes-gcm | 密钥存储（`crates/settings`） | A | 待确认 |
| image | 图片解码（`crates/pipeline`） | C | 待确认 |

## 4. 打包与运行时

| 组件 | 版本 | 许可 | 用途 |
|---|---|---|---|
| Qdrant | 随应用分发（版本由 B 定） | Apache-2.0 | 向量库进程 |
| MinerU | 随 sidecar 分发（版本由 C 定） | AGPL-3.0 | PDF/Office 解析（独立进程、只经 JSON-RPC 调用） |
| ONNX 嵌入模型 | 由 C 定 | 待确认 | 图片/文本向量化 |

> MinerU 为 AGPL：以**独立进程 + 进程间协议**方式使用，不链接进 Rust 二进制；如未来改为链接，
> 必须重开 DEC 评估对本项目许可（MIT OR Apache-2.0）的影响。
