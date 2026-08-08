//! v63.1.0: ステージ単位差分コンパイルキャッシュ（`.fav-cache/` ディレクトリ）
//!
//! プロジェクトローカルな `.fav-cache/` にステージ単位のソースハッシュ（SHA-256）と
//! 型シグネチャを JSON で保存する。
//!
//! 既存の `fav::incremental::cache::IncrementalCache`（ファイルレベル、`~/.fav/cache/`）とは
//! 異なり、ステージ単位（stage-level）の粒度で差分を管理する。

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct StageEntry {
    pub stage_name: String,
    pub source_hash: String,
    pub type_sig: String,
}

pub struct IncrementalCache {
    root: PathBuf,
}

impl IncrementalCache {
    /// 指定ディレクトリをキャッシュルートとして初期化する。
    /// `.fav-cache/` をルートとして渡すことを想定。
    pub fn new(root: &Path) -> Self {
        if let Err(e) = std::fs::create_dir_all(root) {
            eprintln!("[fav-cache] warning: could not create cache directory {}: {e}", root.display());
        }
        Self {
            root: root.to_path_buf(),
        }
    }

    /// 指定ステージのソースハッシュがキャッシュと一致するか確認する。
    pub fn is_hit(&self, stage_name: &str, source_hash: &str) -> bool {
        match self.load_entry(stage_name) {
            Some(e) => e.source_hash == source_hash,
            None => false,
        }
    }

    /// ステージのキャッシュエントリを保存する。
    /// 書き込み失敗はコンパイル・実行を止めるべきでないため Result を返さず握り潰す。
    pub fn store(&self, stage_name: &str, source_hash: &str, type_sig: &str) {
        let entry = StageEntry {
            stage_name: stage_name.to_string(),
            source_hash: source_hash.to_string(),
            type_sig: type_sig.to_string(),
        };
        if let Ok(bytes) = serde_json::to_vec(&entry) {
            std::fs::write(self.entry_path(stage_name), bytes).ok();
        }
    }

    /// ステージのキャッシュエントリを削除（無効化）する。
    pub fn invalidate(&self, stage_name: &str) {
        std::fs::remove_file(self.entry_path(stage_name)).ok();
    }

    /// v63.3.0: ソースハッシュと型シグネチャの両方を検証する。
    ///
    /// - ハッシュ + シグ両方一致 → `true`（キャッシュヒット、再コンパイル不要）
    /// - ハッシュ一致・シグ不一致 → E0428 を `eprintln!` で警告し、キャッシュを自動無効化、`false` を返す
    /// - ハッシュ不一致またはエントリなし → `false`（通常のキャッシュミス）
    ///
    /// `invalidate` は `&self` で動作するため（`std::fs::remove_file` は内部状態不変）、
    /// 本メソッドも `&self` で実装できる。
    pub fn check_type_sig(&self, stage_name: &str, source_hash: &str, current_sig: &str) -> bool {
        // ネスト構造で「ハッシュ一致」を外側に明示し、
        // 内側で「シグ一致」と「シグ不一致」を分岐する。
        // 2アーム方式と比べてガード条件の誤削除リスクを排除している。
        match self.load_entry(stage_name) {
            Some(e) if e.source_hash == source_hash => {
                if e.type_sig == current_sig {
                    true
                } else {
                    eprintln!(
                        "E0428: incremental cache signature mismatch\n  stage `{}` の型シグネチャがキャッシュと一致しません。\n  cached:  {}\n  current: {}\n  キャッシュを無効化して再コンパイルします。",
                        stage_name, e.type_sig, current_sig
                    );
                    self.invalidate(stage_name);
                    false
                }
            }
            _ => false,
        }
    }

    fn load_entry(&self, stage_name: &str) -> Option<StageEntry> {
        let bytes = std::fs::read(self.entry_path(stage_name)).ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    /// ステージ名を安全なファイル名に変換してパスを返す。
    /// `[a-zA-Z0-9_-]` 以外の文字（`..` / スラッシュ等）を `_` に置換し
    /// パストラバーサルを防止する。
    fn entry_path(&self, stage_name: &str) -> PathBuf {
        let safe_name: String = stage_name
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
            .collect();
        self.root.join(format!("{safe_name}.json"))
    }
}

/// ステージソースのバイト列から SHA-256 を小文字 hex 文字列（64文字）で返す。
/// 各バイトを `{:02x}` でゼロパディングして出力する。
pub fn stage_hash(src: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(src);
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}
