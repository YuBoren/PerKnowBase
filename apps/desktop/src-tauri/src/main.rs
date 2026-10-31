//! 进程入口。逻辑都在 `lib.rs`，这里只负责启动（便于未来移动端复用）。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    perknow_desktop_lib::run();
}
