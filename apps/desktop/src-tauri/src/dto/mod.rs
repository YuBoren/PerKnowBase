//! 命令入参 DTO。
//!
//! 规则（`interface-contracts.md` §1.8）：命令入参用独立 DTO 并 `deny_unknown_fields`，
//! 不直接收领域对象。DTO **定义在 `perknow-contracts`**（唯一类型事实源），本模块只做
//! 轻量包装/校验，避免出现两套字段。
//!
//! Phase-0：无 DTO（唯一命令 `app_info` 无入参）。

#![allow(dead_code)]
