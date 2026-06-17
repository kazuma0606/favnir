//! v19.3.0: インクリメンタルコンパイルキャッシュ
//!
//! キャッシュディレクトリ:
//!   FAV_CACHE_DIR 環境変数 → その値を使用
//!   未設定 → $HOME/.fav/cache/<project-hash>/
//!
//! キャッシュファイル: `{root}/{file_hash}.ir`（serde_json でシリアライズした FvcArtifact）

use crate::backend::artifact::FvcArtifact;
use std::path::{Path, PathBuf};

pub struct IncrementalCache {
    root: PathBuf,
}

impl IncrementalCache {
    /// `project_root` のパス文字列を project-hash としてキャッシュルートを決定する。
    pub fn new(project_root: &Path) -> Self {
        let root = if let Ok(dir) = std::env::var("FAV_CACHE_DIR") {
            PathBuf::from(dir)
        } else {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".to_string());
            let project_hash = super::fingerprint::content_hash(
                project_root.to_string_lossy().as_bytes(),
            );
            PathBuf::from(home)
                .join(".fav")
                .join("cache")
                .join(project_hash)
        };
        std::fs::create_dir_all(&root).ok();
        Self { root }
    }

    /// キャッシュエントリが存在するか確認する。
    pub fn is_hit(&self, file_hash: &str) -> bool {
        self.ir_path(file_hash).exists()
    }

    /// キャッシュから FvcArtifact を読み込む。
    pub fn read_artifact(&self, file_hash: &str) -> Result<FvcArtifact, String> {
        let bytes = std::fs::read(self.ir_path(file_hash))
            .map_err(|e| format!("cache read error: {e}"))?;
        serde_json::from_slice(&bytes).map_err(|e| format!("cache deserialize error: {e}"))
    }

    /// FvcArtifact をキャッシュに書き込む。
    pub fn write_artifact(
        &self,
        file_hash: &str,
        artifact: &FvcArtifact,
    ) -> Result<(), String> {
        let bytes =
            serde_json::to_vec(artifact).map_err(|e| format!("cache serialize error: {e}"))?;
        std::fs::write(self.ir_path(file_hash), bytes)
            .map_err(|e| format!("cache write error: {e}"))
    }

    /// キャッシュエントリを削除（無効化）する。
    pub fn invalidate(&self, file_hash: &str) {
        std::fs::remove_file(self.ir_path(file_hash)).ok();
    }

    fn ir_path(&self, file_hash: &str) -> PathBuf {
        self.root.join(format!("{file_hash}.ir"))
    }
}
