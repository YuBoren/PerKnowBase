//! `app_info`：返回应用版本与契约版本（Phase-0 唯一实现的命令）。
//!
//! 前端启动时比对 `contract_version` 与生成 client 的版本，不一致提示重装
//! （`interface-contracts.md` §8.2）。
//!
//! 版本来源：`env!("CARGO_PKG_VERSION")`（= workspace 版本）。它与 `tauri.conf.json` 的
//! `version` 必须一致；Phase-1 的发布脚本负责校验，避免两处漂移（现在都是 0.1.0）。

use perknow_contracts::{
    AppError, AppInfo, CONTRACT_VERSION, PARSER_PROTOCOL_VERSION, SCHEMA_VERSION,
};

/// 应用版本（与 `tauri.conf.json` 的 `version` 保持一致）。
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 返回版本信息。无入参：命令不需要打开中的知识库或应用句柄。
#[tauri::command]
pub async fn app_info() -> Result<AppInfo, AppError> {
    Ok(AppInfo {
        version: APP_VERSION.to_owned(),
        contract_version: CONTRACT_VERSION.to_owned(),
        schema_version: SCHEMA_VERSION,
        platform: std::env::consts::OS.to_owned(),
        parser_protocol_version: PARSER_PROTOCOL_VERSION.to_owned(),
    })
}
