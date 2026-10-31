# 项目结构规划（project-structure）

> 状态：**规划稿，不含实现代码**。第 0 周只建目录、空 crate 与文档指针；实现按第 1 周起的进度表推进。
> 依据：`立项报告草稿.md` 四层架构 + `contracts/architecture-boundaries.md` 依赖矩阵 + `任务分工建议.md` 所有权。
> 目标：四个人在各自分支上，**不读对方代码也能确定自己的文件该放哪**。
> **维护**：新增根目录文件、计划外目录或模块，必须在同一 PR 更新本文件（目录树 + 归属 + 建仓顺序），见 `AGENTS.md` §4.3。

---

## 1. 顶层目录

```
PerKnowBase/
├─ AGENTS.md                     # 智能体入口（只指路，不写规则正文）
├─ STATUS.md                     # 进度与交接（当前状态/下一步/阻塞）
├─ 立项报告草稿.md                # 需求/验收 SSOT
├─ 任务分工建议.md                # 所有权与节奏
├─ decisions.md                  # 异步决策日志（跨分支/跨 Agent 上下文）
├─ contracts/                    # 接口/数据/边界契约（宪法）
│  ├─ README.md
│  ├─ interface-contracts.md
│  ├─ data-model.md
│  ├─ architecture-boundaries.md
│  ├─ CHANGELOG.md
│  └─ schemas/                   # Phase-0 待建：*.schema.json + examples/
├─ docs/
│  ├─ project-structure.md       # 本文件
│  ├─ acceptance-traceability.md # FR→验收→责任人→契约/测试 追溯矩阵（立项报告“目标达成判定”第 7 条）
│  └─ decisions/                 # 决策日志按年归档（见 decisions.md §1.3）
├─ crates/                       # Rust workspace（后端全部逻辑）
├─ apps/
│  ├─ desktop/                   # Tauri 壳（A）
│  └─ ui/                        # React 前端（D 主，A/C 页面）
├─ python/
│  └─ parsers/                   # MinerU sidecar（C）
├─ tests/
│  ├─ integration/               # 跨模块集成（D 牵头）
│  └─ e2e/                       # 端到端与跨平台冒烟
├─ scripts/                      # 生成/校验脚本（A + D）
├─ LICENSES/                     # 第三方许可清单（A，Phase-0 建；至少含 THIRD-PARTY.md）
├─ Cargo.toml                    # workspace
├─ rust-toolchain.toml
├─ package.json / pnpm-workspace.yaml
└─ .github/workflows/            # CI（A）
```

**为什么 Rust 用 workspace 而不是单 crate**：四人需要物理隔离的编译单元与依赖边界；crate 边界即架构边界，CI 可直接检查依赖方向；编译可并行，避免“改前端等后端全量编译”。

---

## 2. crates/（12 个，对应契约分层）

| 目录 | package | 层 | owner | 职责 | 关键依赖（外部） |
|---|---|---|---|---|---|
| `crates/contracts` | `perknow-contracts` | L0 | A 主持 | 领域类型、枚举、错误码、`CONTRACT_VERSION`；`ts-rs` 导出 | `serde`, `ts-rs`, `uuid` |
| `crates/core` | `perknow-core` | L1 | A | `FileId/KbId` 生成、时间/哈希、路径规范化与校验、`EventSink`、`CancelToken`、日志初始化 | `uuid`, `sha2`, `tracing` |
| `crates/storage` | `perknow-storage` | L2 | A | SQLite 连接池、`MetadataRepo`/`AppRepo` 端口与实现、迁移执行、事务 | `rusqlite` 或 `sqlx`, `serde_json` |
| `crates/pipeline` | `perknow-pipeline` | L2 | C | `DocumentParser`（native + sidecar 客户端）、`Chunker`、`Embedder`（ONNX）、解析缓存 | `ort`, `image` |
| `crates/index` | `perknow-index` | L2 | B | Qdrant/Tantivy 适配、进程生命周期、快照、BM25 分词 | `qdrant-client`, `tantivy`, `tantivy-jieba` |
| `crates/llm` | `perknow-llm` | L2 | B | OpenAI 兼容客户端、重试/超时/降级、Prompt 渲染工具 | `reqwest` |
| `crates/kb` | `perknow-kb` | L3 | A | 建库/开关/删除/导入导出/融合/重构编排、`MergeJournal` | `zip` |
| `crates/settings` | `perknow-settings` | L3 | A | 应用级/库级设置、密钥加密、导入导出 | `keyring` 或 `aes-gcm` |
| `crates/fs` | `perknow-fs` | L3 | C | 知识依据浏览与文件操作、路径安全、发 `SyncEvent` | `walkdir` |
| `crates/sync` | `perknow-sync` | L3 | C | `notify` 监听、去抖、`SyncRouter`、增量任务执行 | `notify` |
| `crates/search` | `perknow-search` | L3 | B | `SearchService` 三模式编排、`Retriever`（BM25+向量+RRF）、`PromptBuilder` | — |
| `crates/history` | `perknow-history` | L3 | B | 历史写入/查询/删除/裁剪、软引用快照 | — |

**每个 crate 的固定内部结构**（Phase-1 起）：

```
crates/<name>/
├─ Cargo.toml
├─ README.md              # 本模块职责、边界、契约引用、测试怎么跑
└─ src/
   ├─ lib.rs              # 唯一公共导出面（pub use），禁止外部访问私有模块
   ├─ model.rs            # 本 crate 私有结构（可选）
   ├─ service.rs          # trait 实现（应用服务层）
   ├─ ports.rs            # trait 定义（需被上层消费时）
   └─ tests/              # 模块内单测（tests/ 或 #[cfg(test)]）
```

### 2.1 `migrations` 是什么（SQLite 表结构升级脚本）

**定义**：`crates/storage/migrations/` 存的是 **SQLite 的 DDL 脚本**，用来“建表/加列/加索引/重建表”，让**已有数据库**跟上新版本代码。它**不是**数据迁移（搬文件、搬知识库），**不涉及 Qdrant**（Qdrant 的结构变更是重建 collection，见 `contracts/data-model.md` §3.1）。

**两个库、两条脚本链**：

```
crates/storage/migrations/
├─ app/0001_init.sql      # 应用库 app.sqlite3（全局一份：注册表、应用设置）
│  └─ 0002_*.sql ...
└─ kb/0001_init.sql       # 每个知识库的 kb.sqlite3（各一份：元数据、块、历史…）
   └─ 0002_*.sql ...
```

**什么时候执行**：

- **新建知识库**：新建空 `kb.sqlite3`，从 `kb/0001` 顺序执行到最新，得到当前结构的库（相当于“从零建库”，复用同一批脚本）。
- **打开旧知识库**：读 `schema_migrations` 表里已执行的最高版本，只执行比它新的脚本。
- **导入旧导出包**：解压后按包里的 `schema_version` 补齐缺失脚本，再打开；包版本高于应用则拒绝（`KB_SCHEMA_TOO_NEW`）。
- **应用启动**：对 `app.sqlite3` 做同样的“只跑未执行脚本”。

**账本**：`schema_migrations(version, applied_at, checksum)` 记录每个脚本是否执行、执行时间、脚本哈希；`schema_version` 是“已执行的最高版本”（同时写入 `kb_meta` 与导出包 manifest）。checksum 不一致说明历史脚本被改过，拒绝打开（`KB_CORRUPTED`）。

**典型需要迁移的场景**：新增列（如 `chunk.token_count`）、新增表（如 `merge_op`）、新增索引、SQLite 不支持直接修改时“建新表→拷数据→删旧表→改名”、导入旧版本包。

**不需要迁移的场景**：用户创建/删除/融合知识库、修改设置、切换 LLM——这些是数据或文件操作，结构不变。

**硬规则**：只追加、不改已合入脚本；空库必须能顺序跑通（`scripts/check-migrations`）；只有 A 写脚本，B/C 提表需求（PR）。**当前状态**：Phase-0 尚未创建这些脚本。

---

## 3. 依赖图（与 architecture-boundaries §2 一致）

```
                 ┌─────────────┐
                 │  contracts  │◄────────────── 所有 crate
                 └─────────────┘
                        ▲
                     ┌──┴──┐
                     │core │◄──── 所有 L2/L3
                     └──┬──┘
        ┌───────────────┼───────────────┬──────────────┐
   ┌────┴────┐     ┌────┴────┐     ┌────┴────┐    ┌────┴────┐
   │ storage │     │pipeline │     │  index  │    │   llm   │   L2
   └────┬────┘     └────┬────┘     └────┬────┘    └────┬────┘
        │               │               │              │
   ┌────┴───────────────┴───────────────┴──────────────┴────┐
   │  L3: kb / settings / fs / sync / search / history      │
   └────────────────────────┬───────────────────────────────┘
                            │
                  ┌─────────┴─────────┐
                  │ apps/desktop (壳) │
                  └─────────┬─────────┘
                            │ Tauri IPC（生成 client）
                  ┌─────────┴─────────┐
                  │     apps/ui       │
                  └───────────────────┘
```

关键约束复述（详见边界文档）：

- `index` 与 `pipeline` **互不依赖**，由 `sync`/`kb` 编排。
- `storage` 是唯一 SQL 出口；`index` 是唯一 Qdrant/Tantivy 出口。
- 只有 `apps/desktop` 依赖 `tauri`；`apps/ui` 不依赖任何 Rust crate 源码，只用生成的 TS 类型。
- `contracts` 不依赖任何 workspace crate（防循环依赖）。

---

## 4. apps/

### 4.1 `apps/desktop`（Tauri 壳，A）

```
apps/desktop/
├─ src-tauri/
│  ├─ Cargo.toml
│  ├─ build.rs
│  ├─ tauri.conf.json
│  ├─ capabilities/default.json     # 最小权限：仅 core + 必要的 dialog/shell-open
│  ├─ icons/
│  └─ src/
│     ├─ main.rs                    # 启动、单实例、日志、崩溃钩子
│     ├─ lib.rs                      # AppState（打开中的 KB、服务句柄）、注册
│     ├─ commands/
│     │  ├─ mod.rs                  # 命令注册表（A 独占）
│     │  ├─ kb.rs                   # 调 KnowledgeBaseService
│     │  ├─ fs.rs                   # 调 FileSystemService
│     │  ├─ search.rs               # 调 SearchService
│     │  ├─ history.rs              # 调 HistoryService
│     │  ├─ settings.rs             # 调 SettingsService
│     │  └─ sync.rs                 # 调 SyncService
│     ├─ dto/                       # 命令入参 DTO（deny_unknown_fields）
│     ├─ error.rs                   # AppError → 前端结构 + user_message_key
│     ├─ events.rs                  # EventSink 实现（唯一 emit 点）
│     └─ state.rs                   # 打开的 KB、进行中的 Operation
└─ README.md                        # 命令清单指针、权限说明
```

规则：`commands/*.rs` 每个函数 ≤ 20 行；不允许出现循环、hash、解析、prompt 拼接；新增命令必须同步 `contracts/interface-contracts.md` §4 与生成 client。

### 4.2 `apps/ui`（React 19 + TS，D 主）

```
apps/ui/
├─ index.html
├─ vite.config.ts
├─ tsconfig.json
├─ package.json
├─ eslint.config.js
└─ src/
   ├─ main.tsx / App.tsx
   ├─ app/
   │  ├─ router.tsx              # 路由：启动/管理/浏览器/搜索/历史/设置
   │  ├─ providers.tsx           # 主题、i18n、通知
   │  └─ layout/                 # 侧边栏、状态栏（同步/索引状态）
   ├─ pages/
   │  ├─ kb-manage/              # A：创建/打开/切换/删除/导入导出/融合
   │  ├─ settings/               # A：通用/高级配置、LLM、导入导出
   │  ├─ knowledge-browser/      # C：目录树、右键菜单、拖拽移动
   │  ├─ search-workspace/       # D：多标签（≤5）、三模式、结果与跳转
   │  └─ history/                # D：列表、筛选、复现
   ├─ features/
   │  ├─ tabs/                   # D：标签生命周期、每 tab 独立状态切片
   │  ├─ search/                 # D：结果卡片、引用、复制提示词
   │  └─ sync-status/            # D：进度与索引状态（订阅事件）
   ├─ components/                # D：共享组件（禁止页面私藏可复用组件）
   ├─ lib/
   │  ├─ tauri-api/              # D 维护生成器；产物禁止手改
   │  ├─ store/                  # zustand 切片（按 tab 分域）
   │  └─ format/                 # 时间/大小/路径展示
   ├─ locales/                   # zh-CN（默认）/ en-US（Phase-2）
   └─ assets/
```

前端规则：

1. 只有 `lib/tauri-api/` 允许 `invoke`；页面与组件通过 hooks 调用。
2. 每 tab 独立状态；跨 tab 共享只允许“只读配置”。
3. 页面只能使用 `components/` 中的组件；若缺组件，向 D 提 PR 需求（不复制粘贴）。
4. 事件订阅集中在 `features/sync-status`（和其他域 hooks），页面不直接 `listen`。
5. 所有可跳转结果必须用统一 `useOpenAt()`，保证行/页/图片行为一致。

### 4.3 `python/parsers`（MinerU sidecar，C）

```
python/parsers/
├─ pyproject.toml
├─ README.md                    # 协议指针、本地跑法、依赖版本
├─ src/perknow_parsers/
│  ├─ __main__.py               # 入口：stdio 循环
│  ├─ protocol.py               # NDJSON JSON-RPC 编解码、错误码
│  ├─ server.py                 # 方法分发、并发上限、取消
│  ├─ mineru_adapter.py         # MinerU 4.x 调用、四档模式、定位符
│  ├─ native_parser.py          # txt/md/json/code/ipynb 直读（可回退 Rust 实现）
│  ├─ cache.py                   # cache/<file_id>/<content_hash>/ 读写
│  └─ models.py                 # 报文 dataclass（与契约字段一一对应）
└─ tests/                        # 协议黄金样例、缓存命中、取消
```

规则：sidecar 不写 `kb.sqlite3`、不连 Qdrant、不改 `files/`；只写 `cache_dir`；stdout 只能输出协议帧（BAN-10）。

---

## 5. tests/

```
tests/
├─ integration/                 # D 牵头（Rust 集成测试 + TS 契约测试）
│  ├─ scenarios/                # 建库→解析→检索→跳转 等端到端场景
│  ├─ fixtures/                 # 小体积样例：md/pdf/docx/csv/ipynb/png
│  └─ mocks/                    # mock LLM、mock sidecar（协议级）
└─ e2e/                         # Playwright/WebDriver：多标签、跳转、导入导出
```

约定：

- **禁止**把大文件（>1MB）提交到仓库；大样例放 `fixtures/remote/README.md` 说明获取方式。
- mock sidecar 必须复用 `interfaces-contracts.md` §6 的报文样例，避免“mock 与真实协议漂移”。
- 每个 FR 至少 1 个 integration 场景，映射写入 `docs/acceptance-traceability.md`。

---

## 6. scripts/ 与 CI

| 脚本 | 作用 | owner | 何时跑 |
|---|---|---|---|
| `gen-types.ps1` / `.sh` | Rust→TS 类型与 command client 生成 | D（A 评审） | 改契约后 |
| `validate-contracts.*` | 版本一致、schema 校验、示例校验 | A | 每次 PR |
| `check-deps.*` | 解析 `cargo metadata`，对照依赖矩阵 | A | 每次 PR |
| `check-boundaries.*` | 扫描禁用 import（tauri/rusqlite/qdrant_client/tantivy/ort） | A | 每次 PR |
| `check-migrations.*` | 空库顺序执行、版本单调 | A | 改 storage 时 |
| `dev.*` | 一键起 Tauri + Vite + sidecar（开发态） | D | 本地 |

CI 作业（`.github/workflows/ci.yml`）：`fmt+clippy+test` → `tsc+lint+unit` → `contracts` → `gen-diff` → `deps` → `boundaries` → `migrations` → `integration-smoke`。全部为 **required checks**。

---

## 7. 根配置文件（Phase-0 建）

| 文件 | 要点 |
|---|---|
| `Cargo.toml` | `[workspace] members = ["crates/*", "apps/desktop/src-tauri"]`；`resolver = "2"`；统一 `license = "MIT OR Apache-2.0"`；`[workspace.dependencies]` 锁版本 |
| `rust-toolchain.toml` | 固定 stable + 组件 `rustfmt, clippy`（跨平台一致） |
| `package.json` + `pnpm-workspace.yaml` | `apps/ui` 单包；scripts：`dev/build/typecheck/lint/test/gen` |
| `.editorconfig` / `.gitattributes` | LF、UTF-8；`*.png` 二进制；`*.sql` diff 友好 |
| `.gitignore` | `target/`、`node_modules/`、`.tmp`、`*.snapshot`、`fixtures/remote/*`、`python .venv` |
| `.github/PULL_REQUEST_TEMPLATE.md` | 必填 `Affects:` `DEC:` `Contract-Version:`；自检清单见 `contracts/architecture-boundaries.md` §8.2 |
| `LICENSES/THIRD-PARTY.md` | 依赖 → 版本 → 许可 → 用途；新增依赖必须登记 |

---

## 8. 建仓顺序（Phase-0，第 0 周，仍然“不写业务代码”）

1. **A**：建上述目录骨架、workspace `Cargo.toml`、空 crate（仅 `lib.rs` 含 `//! module doc` + 契约指针）、CI 空跑、`scripts/` 四个校验脚本骨架、`LICENSES/THIRD-PARTY.md`。
2. **A**：`crates/contracts` 落类型骨架（能编译、`ts-rs` 导出可跑），`crates/core` 落 `EventSink`/`CancelToken`/路径校验签名。
3. **B**：`contracts/data-model.md` §3 索引段与 `crates/index` 空 trait 实现骨架。
4. **C**：`python/parsers` 的 `protocol.py` + 黄金报文样例（先跑通 stdio echo，不做真实解析）。
5. **D**：`apps/ui` 壳 + 路由 + `gen-types` 产物接入 + 一个假数据页面（验证 IPC 端到端）。
6. **全员**：合入契约 PR（`contract_version=0.1.0`），再各自在分支上写实现。

**Phase-0 完成判据**：`cargo build` 通过、`pnpm build` 通过、CI 全绿、`app_info` 能返回 `contract_version`、四个校验脚本能在无违规时通过并在故意违规时报错。

---

## 9. 约定速查

| 项 | 约定 |
|---|---|
| 分支 | 各自 clone 后开分支；`main` 受保护；`feat/<owner>-<主题>`、`fix/…`、`contract/…`、`dec/…` |
| worktree | 仅同一人用多个 Agent 并行改多个功能时使用；跨机器协作不需要 |
| commit | Conventional Commits + trailer `Agent-Task:`、`Agent-Decision:`；一条 commit 一件事 |
| 文件放置 | 新公共组件先放 `components/`；新表先改 `data-model.md`；新命令先改 `interface-contracts.md` §4 |
| 命名 | Rust `snake_case` 模块/函数、`CamelCase` 类型；TS 组件 `PascalCase`、hooks `useXxx`；文件 `kebab-case` |
| 禁止 | 在 `apps/ui` 里手写 IPC 类型；在 `crates/*` 里 `unwrap`；提交 `.env`/密钥/大样例 |
| 依赖新增 | 先 `DEC` → 再 `LICENSES/` → 再 `Cargo.toml`/`package.json` |
