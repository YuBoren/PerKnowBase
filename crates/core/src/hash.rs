//! 哈希与内容归一化（`data-model.md` §2.5、`contracts/README.md` §4）。
//!
//! 哈希一律 **SHA-256 小写 hex，64 字符，不写 `sha256:` 前缀**。
//! `content_hash` 的归一化规则是契约：去首尾空白 + CRLF→LF——不一致会导致增量同步误判。

use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};

/// 字节序列的 SHA-256 小写 hex。
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex_lower(&hasher.finalize())
}

/// 文件的 SHA-256 小写 hex（流式读取，64 KiB 缓冲，避免大文件占内存）。
pub fn sha256_file(path: &Path) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buf)?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
    }
    Ok(hex_lower(&hasher.finalize()))
}

/// 块内容归一化：CRLF→LF，再去首尾空白（含 `\n`）。
pub fn normalize_content(text: &str) -> String {
    text.replace("\r\n", "\n").trim().to_owned()
}

/// 块内容哈希：`sha256(normalize_content(text))`。
pub fn content_hash(text: &str) -> String {
    sha256_hex(normalize_content(text).as_bytes())
}

/// 字节 → 小写 hex。
fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}
