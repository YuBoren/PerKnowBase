//! LLM 配置与报文（`data-model.md` §2.9/§2.14；网关见 `interface-contracts.md` §3.8）。
//!
//! 密钥永不进入本结构：`LlmCfg.has_key` 只表示“已配置”，明文由密钥库保存（BAN-16）。

use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// LLM 提供方（`data-model.md` §5.6）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum LlmProvider {
    /// OpenAI 兼容接口。
    OpenAiCompat,
    /// Ollama 本地服务。
    Ollama,
    /// llama.cpp server。
    LlamaCpp,
    /// 自定义兼容接口。
    Custom,
}

/// LLM 配置（存于设置；密钥单独加密，本结构只带 `has_key`）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct LlmCfg {
    /// 提供方。
    pub provider: LlmProvider,
    /// 接口基址（本地地址不允许外联，NFR-10）。
    pub base_url: String,
    /// 模型名。
    pub model: String,
    /// 单次超时毫秒（默认 60000）。
    pub timeout_ms: u64,
    /// 输出上限；`null` 表示用服务端默认。
    pub max_tokens: Option<u32>,
    /// 是否已配置密钥（只读，由 `settings_set_llm_secret` 维护）。
    pub has_key: bool,
}

/// 消息角色。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum LlmRole {
    /// 系统指令。
    System,
    /// 用户输入。
    User,
    /// 模型回复。
    Assistant,
}

/// 一条消息。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct LlmMessage {
    /// 角色。
    pub role: LlmRole,
    /// 内容。
    pub content: String,
}

/// 请求体。本期 `stream` 恒为 `false`。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct LlmRequest {
    /// 模型名。
    pub model: String,
    /// 消息序列。
    pub messages: Vec<LlmMessage>,
    /// 温度 0..=2。
    pub temperature: f32,
    /// 输出上限；`null` 用服务端默认。
    pub max_tokens: Option<u32>,
    /// 是否流式（本期恒 `false`）。
    pub stream: bool,
    /// 超时毫秒。
    pub timeout_ms: u64,
}

/// token 用量。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct Usage {
    /// 输入 token。
    pub prompt_tokens: u32,
    /// 输出 token。
    pub completion_tokens: u32,
    /// 合计。
    pub total_tokens: u32,
}

/// 结束原因（`data-model.md` §5.6）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub enum FinishReason {
    /// 正常结束。
    Stop,
    /// 触达长度上限。
    Length,
    /// 被内容过滤。
    ContentFilter,
    /// 协议错误。
    Error,
}

/// 响应体。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct LlmResponse {
    /// 模型名。
    pub model: String,
    /// 文本结果。
    pub content: String,
    /// 用量；提供方未返回时为 `null`。
    pub usage: Option<Usage>,
    /// 结束原因。
    pub finish_reason: FinishReason,
    /// 耗时毫秒。
    pub elapsed_ms: u64,
}

/// `llm_probe` 结果。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct LlmProbeResult {
    /// 是否连通且鉴权通过。
    pub ok: bool,
    /// 实际模型名。
    pub model: Option<String>,
    /// 往返毫秒。
    pub latency_ms: u64,
    /// 失败原因（已脱敏，不含密钥）。
    pub error: Option<AppError>,
}
