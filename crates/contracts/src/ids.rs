//! 标识类型别名。
//!
//! 取值统一为 **UUIDv7 小写带连字符**（`contracts/README.md` §4），因此线上类型就是
//! `String`；生成放在 `perknow-core`（`ids::new_id`），本 crate 只声明语义别名。

/// 知识库 id，同时是目录名。`data-model.md` §2.1
pub type KbId = String;
/// 文件/目录节点 id（目录也分配 id）。`data-model.md` §2.2
pub type FileId = String;
/// 块 id。`data-model.md` §2.5
pub type ChunkId = String;
/// 检索请求 id（前端生成）。`data-model.md` §2.7
pub type RequestId = String;
/// 长任务操作 id。`data-model.md` §2.11
pub type OpId = String;
/// 错误关联 id，贯穿 Rust/sidecar 日志。`data-model.md` §2.17
pub type CorrelationId = String;
/// 时间戳。JSON 为 RFC 3339 UTC 毫秒（`2026-10-31T09:09:05.123Z`），DB 为 epoch 毫秒。
pub type Timestamp = String;
