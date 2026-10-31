<!-- 模板字段见 AGENTS.md §7；共享模块自检清单见 contracts/architecture-boundaries.md §8.2 -->

## 改了什么

<!-- 一句话说明；关联任务看板编号（PA-xx / PB-xx / PC-xx / PD-xx / PX-xx） -->

## 必填字段

- `Affects:` <!-- 受影响模块，如 crates/contracts、crates/storage、apps/ui -->
- `DEC:` <!-- 决策编号，或 none + 一句话理由 -->
- `Contract-Version:` <!-- 如 0.1.0；未改契约写 none -->
- `How tested:` <!-- 实际跑过的命令与结果；没跑的写“未跑 + 原因” -->
- `Notify:` <!-- @受影响 owner；无则 none -->

## 自检（勾选）

- [ ] 只改了自己 owner 的目录，或已走共享模块流程（`contracts/architecture-boundaries.md` §4.2）
- [ ] 契约改动已同步 `contracts/*.md` + `CHANGELOG.md` + `crates/contracts`
- [ ] 没有新增跨层调用（对照 `architecture-boundaries.md` §2）
- [ ] 错误走 `AppError`，无 `unwrap` 泄漏到命令边界
- [ ] 新增依赖已写 `decisions.md` 并登记 `LICENSES/THIRD-PARTY.md`
- [ ] 无手改生成物（`apps/ui/src/lib/tauri-api/**`）
