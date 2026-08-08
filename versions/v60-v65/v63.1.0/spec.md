# v63.1.0 Spec — 差分コンパイルキャッシュ（`.fav-cache/`）

Version: 63.1.0
Status: 未着手
Base tests: 3406
Target tests: 3408

---

## 概要

プロジェクトローカルな `.fav-cache/` ディレクトリにステージ単位のソースハッシュ（SHA-256）と
型シグネチャを JSON で保存する `IncrementalCache` 構造体を `fav/src/cache.rs` に新規実装する。
既存の `fav/src/incremental/cache.rs`（ファイルレベルキャッシュ）とは異なり、
ステージ単位（stage-level）の粒度で差分を管理する。
`driver.rs` に `cmd_incremental_cache_status(src: &str) -> String` を追加し、
パイプラインのキャッシュ状態を表示できるようにする。

---

## 前提確認（T0 で実施）

- `fav/src/cache.rs` が **存在しない** ことを確認
- `driver.rs` に `v63000_tests` が存在することを確認（挿入位置確認）
- `cargo test -j 8 -- --test-threads=8` でベース 3406 tests passed, 0 failed を確認
  （ロードマップ記載ベース 3396 より +10 — v62.8.0 / v62.9.0 / v63.0.0 の実績値）

---

## 実装スコープ

### 1. `fav/src/cache.rs` — `IncrementalCache` 新規作成

```rust
// ステージ単位差分キャッシュ（.fav-cache/ ディレクトリ）
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
    /// `.fav-cache/` をデフォルトルートとして初期化する。
    pub fn new(root: &Path) -> Self {
        std::fs::create_dir_all(root).ok();
        Self { root: root.to_path_buf() }
    }

    /// 指定ステージのハッシュがキャッシュと一致するか確認する。
    pub fn is_hit(&self, stage_name: &str, source_hash: &str) -> bool {
        match self.load_entry(stage_name) {
            Some(e) => e.source_hash == source_hash,
            None => false,
        }
    }

    /// ステージのキャッシュエントリを保存する。
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

    fn load_entry(&self, stage_name: &str) -> Option<StageEntry> {
        let bytes = std::fs::read(self.entry_path(stage_name)).ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    fn entry_path(&self, stage_name: &str) -> PathBuf {
        self.root.join(format!("{stage_name}.json"))
    }
}

/// バイト列の SHA-256 を小文字 hex 文字列で返す（stage ハッシュ計算用）。
pub fn stage_hash(src: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(src);
    format!("{:x}", h.finalize())
}
```

### 2. `fav/src/lib.rs` — `pub mod cache;` 追加

既存の `pub mod incremental;` の直後（またはアルファベット順の適切な位置）に追加する。

### 3. `driver.rs` — `cmd_incremental_cache_status` 追加

```rust
/// `.fav-cache/` ディレクトリのキャッシュ状態を表示する。
pub fn cmd_incremental_cache_status(cache_dir: &str) -> String {
    let root = std::path::Path::new(cache_dir);
    if !root.exists() {
        return "no cache directory found".to_string();
    }
    let entries: Vec<String> = std::fs::read_dir(root)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter_map(|e| {
                    e.path()
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_string())
                })
                .collect()
        })
        .unwrap_or_default();
    if entries.is_empty() {
        "cache is empty".to_string()
    } else {
        format!("cached stages: {}", entries.join(", "))
    }
}
```

### 4. `driver.rs` — `v63100_tests` 追加

`v63000_tests` の直前（ファイル先頭方向）に挿入する。

```rust
// -- v63100_tests (v63.1.0) -- 差分コンパイルキャッシュ --
#[cfg(test)]
mod v63100_tests {
    use crate::cache::{IncrementalCache, stage_hash};
    use tempfile::TempDir;

    #[test]
    fn incremental_cache_hit_unchanged() {
        let dir = TempDir::new().unwrap();
        let cache = IncrementalCache::new(dir.path());
        let src = b"stage LoadCsv: List<String> -> List<Row>";
        let hash = stage_hash(src);
        cache.store("LoadCsv", &hash, "List<String> -> List<Row>");
        assert!(cache.is_hit("LoadCsv", &hash),
            "cache should be a hit for unchanged source");
    }

    #[test]
    fn incremental_cache_miss_on_change() {
        let dir = TempDir::new().unwrap();
        let cache = IncrementalCache::new(dir.path());
        let src_v1 = b"stage LoadCsv: List<String> -> List<Row>";
        let src_v2 = b"stage LoadCsv: List<String> -> List<EnrichedRow>";
        let hash_v1 = stage_hash(src_v1);
        let hash_v2 = stage_hash(src_v2);
        cache.store("LoadCsv", &hash_v1, "List<String> -> List<Row>");
        assert!(!cache.is_hit("LoadCsv", &hash_v2),
            "cache should be a miss when source changes");
    }
}
```

---

## 完了条件

- `cargo build` エラーなし
- `cargo test v63100` で 2 件 PASS
- `cargo test -j 8 -- --test-threads=8` で 3408 tests passed, 0 failed

---

## 非スコープ

- `cmd_run` へのキャッシュ統合（実際の fav run 時のキャッシュ適用は v63.2.0 以降）
  ※ ロードマップ v63.1.0 は `cmd_run` 統合を含む記述だが、インフラ基盤整備を先行させ
    `cmd_run` 統合は v63.2.0 `fav watch` 改善と同時実装とする（意図的な後送り）
- ステージ単位の bytecode キャッシュ（現バージョンは型シグネチャ hash の管理のみ）
- `[fav watch]` との統合（v63.2.0）
- `.fav-cache/` を `.gitignore` に追加するツール機能
- `site/` MDX ドキュメント追加（v63.2.0 以降）
- `cmd_incremental_cache_status` の個別テスト（`cmd_run` 統合テストと合わせて v63.2.0 以降）

---

## 技術ノート

### 既存の `incremental/cache.rs` との関係

| 項目 | `incremental/cache.rs` | `cache.rs`（v63.1.0） |
|---|---|---|
| 単位 | ファイル全体 | ステージ単位 |
| キー | ファイルハッシュ | ステージ名 + ソースハッシュ |
| 保存先 | `~/.fav/cache/` | `.fav-cache/`（プロジェクトローカル） |
| 用途 | `fav build` のアーティファクト | `fav run` の差分スキップ |

### ベーステスト数の変更について

ロードマップ記載ベースは 3396 だが、v62.8.0 code-reviewer 対応 / v62.9.0 / v63.0.0 の
実績値の積み上げにより実際のベースは **3406**（T0 で `cargo test` 実測して確認すること）。
完了条件のターゲットは 実測ベース + 2 = **3408**（ベースが 3406 の場合）。

### `IncrementalCache` 命名の注意

`fav/src/incremental/cache.rs`（v19.3.0 実装）にも `IncrementalCache` が存在する。
v63.1.0 の `fav/src/cache.rs` の `IncrementalCache` はモジュールが異なるため
Rust 上の名前衝突はないが（`fav::cache::IncrementalCache` vs `fav::incremental::cache::IncrementalCache`）、
テストコードでは必ず `use crate::cache::IncrementalCache;` と明示すること。

### WASM ビルド影響

`cache.rs` は `std::fs::create_dir_all` / `std::fs::write` 等のファイルシステム操作を含む。
WASM ターゲット非対応のため、`lib.rs` での `pub mod cache;` 宣言は
`#[cfg(not(target_arch = "wasm32"))]` でガードする（または `cfg_attr` を利用）。

### テストの独立性

`v63100_tests` の 2 テストは `tempfile::TempDir` を使用して並列テスト競合を回避する。
`TempDir` は `[dev-dependencies]` 登録済み（`fav/Cargo.toml`）のため追加不要。
