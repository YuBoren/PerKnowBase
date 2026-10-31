//! # perknow-desktop（L4 外壳）
//!
//! 只做四件事：参数反序列化 → 调 L3 服务 trait → `AppError` 映射 → 返回 DTO。
//! **不得**出现解析/分块/prompt 拼接/hash 计算/重试编排（BAN-06），也不得直接 `use` 引擎库。
//!
//! 命令清单与注册点在 `commands`；事件唯一 emit 点在 `events`；打开中的库在 `state`。

#![deny(unsafe_code)]

pub mod commands;
pub mod dto;
pub mod error;
pub mod events;
pub mod state;

use std::sync::Arc;

use perknow_core::EventSink;
use tauri::Manager;

/// 启动 Tauri 应用（被 `main.rs` 与集成测试调用）。
pub fn run() {
    tauri::Builder::default()
        .manage(state::AppState::default())
        .setup(|app| {
            // 事件出口只在这里构造一次：其他层拿 `Arc<dyn EventSink>`（BAN-05）。
            let sink: Arc<dyn EventSink> =
                Arc::new(events::TauriEventSink::new(app.handle().clone()));
            app.manage(sink);
            Ok(())
        })
        // 命令注册的唯一位置：新增命令必须先改 contracts/interface-contracts.md §4
        // 与 commands/table.rs，再在这里加一行。
        .invoke_handler(tauri::generate_handler![commands::app_info::app_info])
        .run(tauri::generate_context!())
        .expect("failed to run the PerKnowBase desktop shell");
}
