//! 命令清单——`contracts/interface-contracts.md` §4 的代码镜像。
//!
//! 为什么需要这张表：
//! - 文档是权威，这里是**可被脚本比对**的副本：`scripts/validate-contracts.ps1` 会比较
//!   `interface-contracts.md` §4 表格里的命令名与本文件的 `COMMANDS`，两边不一致即失败；
//! - 新命令的顺序是：先改契约 → 再改本表 → 最后在 `commands::handler()` 里注册。
//!
//! 未实现的命令**不注册**（前端拿到“命令不存在”），避免出现假成功的空实现。

/// 命令 owner（与 `任务分工建议.md` §一 的所有权一致）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Owner {
    /// 平台与知识库生命周期。
    A,
    /// 检索与 AI 链路。
    B,
    /// 数据管线与同步。
    C,
}

/// 实现状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// 已在 `handler()` 注册。
    Implemented,
    /// 仅登记，尚未实现。
    Pending,
}

/// 一条命令的登记项。
#[derive(Debug, Clone, Copy)]
pub struct CommandSpec {
    /// 命令名（`snake_case`，即 `invoke` 的第一个参数）。
    pub name: &'static str,
    /// 实现归属。
    pub owner: Owner,
    /// 实现状态。
    pub status: Status,
}

/// 全部 39 条命令（顺序与契约表一致）。
pub const COMMANDS: [CommandSpec; 39] = [
    spec("kb_list", Owner::A, Status::Pending),
    spec("kb_create", Owner::A, Status::Pending),
    spec("kb_open", Owner::A, Status::Pending),
    spec("kb_close", Owner::A, Status::Pending),
    spec("kb_switch", Owner::A, Status::Pending),
    spec("kb_delete", Owner::A, Status::Pending),
    spec("kb_export", Owner::A, Status::Pending),
    spec("kb_import", Owner::A, Status::Pending),
    spec("kb_merge", Owner::A, Status::Pending),
    spec("kb_rebuild", Owner::A, Status::Pending),
    spec("kb_cancel_op", Owner::A, Status::Pending),
    spec("fs_list_children", Owner::C, Status::Pending),
    spec("fs_add_files", Owner::C, Status::Pending),
    spec("fs_add_folder", Owner::C, Status::Pending),
    spec("fs_remove", Owner::C, Status::Pending),
    spec("fs_move", Owner::C, Status::Pending),
    spec("fs_open", Owner::C, Status::Pending),
    spec("fs_reveal", Owner::C, Status::Pending),
    spec("fs_refresh", Owner::C, Status::Pending),
    spec("search_minimal", Owner::B, Status::Pending),
    spec("search_summary", Owner::B, Status::Pending),
    spec("search_test", Owner::B, Status::Pending),
    spec("search_cancel", Owner::B, Status::Pending),
    spec("history_list", Owner::B, Status::Pending),
    spec("history_delete", Owner::B, Status::Pending),
    spec("history_clear", Owner::B, Status::Pending),
    spec("history_get", Owner::B, Status::Pending),
    spec("settings_get_app", Owner::A, Status::Pending),
    spec("settings_update_app", Owner::A, Status::Pending),
    spec("settings_get_kb", Owner::A, Status::Pending),
    spec("settings_update_kb", Owner::A, Status::Pending),
    spec("settings_export", Owner::A, Status::Pending),
    spec("settings_import", Owner::A, Status::Pending),
    spec("settings_set_llm_secret", Owner::A, Status::Pending),
    spec("sync_trigger", Owner::C, Status::Pending),
    spec("sync_status", Owner::C, Status::Pending),
    spec("sync_set_interval", Owner::C, Status::Pending),
    spec("llm_probe", Owner::B, Status::Pending),
    spec("app_info", Owner::A, Status::Implemented),
];

/// 便于常量表使用。
const fn spec(name: &'static str, owner: Owner, status: Status) -> CommandSpec {
    CommandSpec {
        name,
        owner,
        status,
    }
}
