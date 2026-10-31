//! ID 生成（UUIDv7，小写带连字符、可排序，`contracts/README.md` §4）。
//!
//! 为什么是 v7：按时间排序的主键，SQLite 顺序写友好，日志按 id 排序即时间顺序。

use perknow_contracts::{ChunkId, CorrelationId, FileId, KbId, OpId, RequestId};

/// 生成一个 UUIDv7 字符串。
pub fn new_id() -> String {
    uuid::Uuid::now_v7().to_string()
}

/// 新知识库 id。
pub fn new_kb_id() -> KbId {
    new_id()
}

/// 新文件节点 id（目录也分配）。
pub fn new_file_id() -> FileId {
    new_id()
}

/// 新块 id。
pub fn new_chunk_id() -> ChunkId {
    new_id()
}

/// 新长任务 id。
pub fn new_op_id() -> OpId {
    new_id()
}

/// 新检索请求 id。
pub fn new_request_id() -> RequestId {
    new_id()
}

/// 新错误关联 id。
pub fn new_correlation_id() -> CorrelationId {
    new_id()
}
