//! 路径规范化与越界校验（BAN-12：sidecar 只接受绝对路径，校验必须在 Rust 侧完成）。
//!
//! 规则（`data-model.md` §1.2 硬规则 1、`contracts/README.md` §4）：
//! - `rel_path` 一律相对 `files/`、POSIX 分隔、不含前导 `/`、不含 `..`。
//! - 解析后必须仍位于知识库根内：先 `canonicalize`（消解符号链接）再前缀比对。
//! - 错误消息不得带绝对路径（BAN-16）。

use std::path::{Path, PathBuf};

use perknow_contracts::{AppError, AppResult, ErrorCode};

/// `rel_path` 长度上限（字节）。超过视为异常数据，避免恶意超长路径。
pub const MAX_REL_PATH_LEN: usize = 4096;

/// 统一成 POSIX 分隔（Windows 传入 `\\`）。
pub fn to_posix(path: &str) -> String {
    path.replace('\\', "/")
}

/// 校验 `rel_path` 形态；空串表示知识库根目录，合法。
pub fn validate_rel_path(rel: &str) -> AppResult<()> {
    if rel.is_empty() {
        return Ok(());
    }
    if rel.len() > MAX_REL_PATH_LEN {
        return Err(outside("relative path is too long"));
    }
    if rel.contains('\0') {
        return Err(outside("relative path contains NUL"));
    }
    if rel.contains('\\') {
        return Err(outside("relative path must use POSIX separators"));
    }
    if rel.starts_with('/') {
        return Err(outside("relative path must not be absolute"));
    }
    for part in rel.split('/') {
        if part.is_empty() {
            return Err(outside("relative path contains an empty segment"));
        }
        if part == "." || part == ".." {
            return Err(outside("relative path contains a traversal segment"));
        }
        if part.contains(':') {
            return Err(outside(
                "relative path contains a drive or stream separator",
            ));
        }
    }
    Ok(())
}

/// 相对路径 → 绝对路径（目标必须已存在；会消解符号链接）。
pub fn resolve_existing(root: &Path, rel: &str) -> AppResult<PathBuf> {
    validate_rel_path(rel)?;
    let canonical_root = canonical_root(root)?;
    let candidate = if rel.is_empty() {
        canonical_root.clone()
    } else {
        canonical_root.join(rel)
    };
    let canonical = candidate
        .canonicalize()
        .map_err(|_| not_found("path does not exist"))?;
    if !canonical.starts_with(&canonical_root) {
        return Err(outside("path escapes the knowledge base root"));
    }
    Ok(canonical)
}

/// 相对路径 → 绝对路径（目标尚不存在，父目录必须存在且在根内）。
pub fn resolve_for_create(root: &Path, rel: &str) -> AppResult<PathBuf> {
    validate_rel_path(rel)?;
    if rel.is_empty() {
        return Err(outside("target must not be the knowledge base root"));
    }
    let canonical_root = canonical_root(root)?;
    let (parent_rel, file_name) = match rel.rsplit_once('/') {
        Some((parent, name)) => (parent, name),
        None => ("", rel),
    };
    let parent = if parent_rel.is_empty() {
        canonical_root.clone()
    } else {
        resolve_existing(root, parent_rel)?
    };
    if file_name.is_empty() || file_name == "." || file_name == ".." {
        return Err(outside("target name is invalid"));
    }
    Ok(parent.join(file_name))
}

/// 断言语义：已 canonicalize 的绝对路径必须位于根内。
pub fn ensure_within_root(root: &Path, candidate: &Path) -> AppResult<()> {
    let canonical_root = canonical_root(root)?;
    if candidate.starts_with(&canonical_root) {
        Ok(())
    } else {
        Err(outside("path escapes the knowledge base root"))
    }
}

/// 绝对路径 → 相对根的 POSIX `rel_path`。
pub fn rel_from_root(root: &Path, absolute: &Path) -> AppResult<String> {
    let canonical_root = canonical_root(root)?;
    let stripped = absolute
        .strip_prefix(&canonical_root)
        .map_err(|_| outside("path escapes the knowledge base root"))?;
    Ok(to_posix(&stripped.to_string_lossy()))
}

/// 根目录 canonicalize；失败按“路径不存在”处理（不泄露绝对路径）。
fn canonical_root(root: &Path) -> AppResult<PathBuf> {
    root.canonicalize()
        .map_err(|_| not_found("knowledge base root is not accessible"))
}

/// `PATH_OUTSIDE_KB`。
fn outside(message: &str) -> AppError {
    AppError::new(ErrorCode::PathOutsideKb, message)
}

/// `PATH_NOT_FOUND`。
fn not_found(message: &str) -> AppError {
    AppError::new(ErrorCode::PathNotFound, message)
}
