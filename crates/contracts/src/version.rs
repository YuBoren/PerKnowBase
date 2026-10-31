//! 版本常量。
//!
//! 契约版本四处一致（`contracts/README.md` §2）：`contracts/*.md` 顶部、
//! `contracts/CHANGELOG.md`、本文件、生成的 `apps/ui/src/lib/tauri-api/version.ts`。
//! `scripts/validate-contracts.ps1` 校验前三处（生成物由 D 的 `scripts/gen-types` 校验）。

/// 契约版本 `MAJOR.MINOR.PATCH`（与软件版本无关）。
pub const CONTRACT_VERSION: &str = "0.1.0";

/// SQLite `schema_version`：`crates/storage/migrations/` 已执行脚本的最高版本。
pub const SCHEMA_VERSION: u32 = 1;

/// sidecar 协议版本 `MAJOR.MINOR`（独立于契约版本，见 `interface-contracts.md` §6.5）。
pub const PARSER_PROTOCOL_VERSION: &str = "1.0";

/// 导出包 `format_version`（`data-model.md` §2.12）。
pub const EXPORT_FORMAT_VERSION: u32 = 1;

/// 导出包 `format` 标识。
pub const EXPORT_FORMAT_ID: &str = "perknowbase.kb";
