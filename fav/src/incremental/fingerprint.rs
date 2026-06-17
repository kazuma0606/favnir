//! v19.3.0: ファイルコンテンツのフィンガープリント（SHA-256）

use sha2::{Digest, Sha256};
use std::path::Path;

/// ファイル内容の SHA-256 を小文字 hex 文字列で返す。
pub fn file_hash(path: &Path) -> Result<String, String> {
    let bytes =
        std::fs::read(path).map_err(|e| format!("read error {}: {e}", path.display()))?;
    Ok(content_hash(&bytes))
}

/// バイト列の SHA-256 を小文字 hex 文字列で返す。
pub fn content_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
