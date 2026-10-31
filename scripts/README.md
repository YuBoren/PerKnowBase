# scripts — 生成与校验脚本（A + D）

> 作用与何时跑见 [`docs/project-structure.md`](../docs/project-structure.md) §6；CI 作业见 [`.github/workflows/ci.yml`](../.github/workflows/ci.yml)。
> 约定：**任何一项失败都必须先修脚本或修代码，不得跳过检查**（`AGENTS.md` §8）。

## 1. 脚本清单

| 脚本 | owner | 检查什么 | 依赖 |
|---|---|---|---|
| `validate-contracts.ps1` | A | 契约版本四处一致、schemas 与示例、命令表/错误码/事件名三处一致 | PowerShell；深度校验需 `python` + `jsonschema` |
| `validate-schemas.py` | A | 被上一个脚本调用：schema 结构 + 示例符合 schema | `jsonschema`（缺失时退回内置最小校验器） |
| `check-deps.ps1` | A | 各 `Cargo.toml` 声明的依赖方向是否符合依赖矩阵 | PowerShell |
| `check-boundaries.ps1` | A | 源码里的禁用 API（tauri/rusqlite/qdrant_client/tantivy/ort/SQL）与前端 `invoke`/`fetch` | PowerShell |
| `check-migrations.ps1` | A | 迁移版本连续单调、空库顺序执行、表/索引建成、`SCHEMA_VERSION` 一致 | PowerShell + `python`（sqlite3） |
| `check-migrations.py` | A | 被上一个脚本调用：在临时库上执行两条链 | `python` |
| `check-licenses.ps1` | A | 声明的第三方依赖都已登记 `LICENSES/THIRD-PARTY.md` | PowerShell |
| `gen-types.ps1` | D | Rust → TS 类型与命令 client（产物禁止手改） | Phase-1 由 D 提供 |

## 2. 本地怎么跑

```powershell
# 一次跑全部门禁（与 CI 顺序一致）
pwsh -NoProfile -File scripts/validate-contracts.ps1
pwsh -NoProfile -File scripts/check-deps.ps1
pwsh -NoProfile -File scripts/check-boundaries.ps1
pwsh -NoProfile -File scripts/check-migrations.ps1
pwsh -NoProfile -File scripts/check-licenses.ps1
```

- Windows PowerShell 5.1 也能跑（脚本已按 UTF-8 BOM + 5.1 语法写）；跨平台建议用 `pwsh` 7。
- 退出码：`0` 通过，`1` 有失败项（CI 据此判红）。
- 输出里 `[ok]` 是通过项，`[skip]` 是当前环境缺少依赖或对象尚未建立（会写明原因），`[FAIL]` 是必须修的项。

## 3. 降级行为（不要当成“已通过”）

| 情形 | 行为 |
|---|---|
| 未安装 `jsonschema` | `validate-schemas.py` 用内置最小校验器（只覆盖本仓库用到的关键字），CI 会安装真库 |
| `apps/ui` 尚未建立 | `check-boundaries.ps1` 跳过前端扫描并打印 `[skip]` |
| `apps/ui/src/lib/tauri-api/version.ts` 尚未生成 | `validate-contracts.ps1` 跳过该处版本比对并打印 `[skip]` |
| 未安装 `python` | `check-migrations.ps1` 直接失败（迁移必须真跑一遍，不接受跳步） |

## 4. 改脚本的规矩

1. 新增检查项 → 同步更新本文件表格与 `docs/project-structure.md` §6。
2. 停止检查某项 → 必须写 `DEC` 并在 `decisions.md` 记录原因与恢复条件。
3. 脚本里不要硬编码绝对路径（BAN-19）；路径一律从 `$PSScriptRoot` 推导。
4. 修改 `.ps1` 后保持 **UTF-8 BOM**，否则 Windows PowerShell 5.1 会把中文读错。
