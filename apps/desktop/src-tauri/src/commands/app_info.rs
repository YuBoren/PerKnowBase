//! `app_info`：返回应用版本与契约版本（Phase-0 唯一实现的命令）。
//!
//! 前端启动时比对 `contract_version` 与生成 client 的版本，不一致提示重装
//! （`interface-contracts.md` §8.2）。

use perknow_contracts::{
    AppError, AppInfo, CONTRACT_VERSION, PARSER_PROTOCOL_VERSION, SCHEMA_VERSION,
};
use tauri::Manager;

/// 返回版本信息。
#[tauri::command]
pub async fn app_info(app: tauri::AppHandle) -> Result<AppInfo, AppError> {
    Ok(AppInfo {
        version: app.package_info().version.to_string(),
        contract_version: CONTRACT_VERSION.to_owned(),
        schema_version: SCHEMA_VERSION,
        platform: std::env::consts::OS.to_owned(),
        parser_protocol_version: PARSER_PROTOCOL_VERSION.to_owned(),
    })
}
