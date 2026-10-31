//! 应用信息（`app_info` 命令返回，`interface-contracts.md` §4）。

use serde::{Deserialize, Serialize};

/// `AppInfo`：前端启动时校验 `contract_version` 与生成 client 一致（§8.2）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct AppInfo {
    /// 应用版本。
    pub version: String,
    /// 契约版本，必须与生成 client 一致。
    pub contract_version: String,
    /// 支持的库表版本上限。
    pub schema_version: u32,
    /// 平台标识（`windows`/`linux`/`macos`）。
    pub platform: String,
    /// sidecar 协议版本。
    pub parser_protocol_version: String,
}
