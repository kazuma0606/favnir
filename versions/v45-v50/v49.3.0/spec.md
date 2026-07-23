# Spec: v49.3.0 — `fav check` インクリメンタル型チェック

## 概要

変更されたファイルのみ再チェックするインクリメンタル型チェック機能を実装する。
SHA-256 フィンガープリントを `.fav-cache/` に保存し、未変更ファイルをスキップすることで
大規模プロジェクトでの `fav check` 速度を改善する。

---

## 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `compute_file_fingerprint` / `file_needs_recheck` / `update_fingerprint_cache` ヘルパー追加 + `v493000_tests` 追加（2テスト）|
| `fav/Cargo.toml` | version → `"49.3.0"`（`sha2` は既存依存済み）|
| `CHANGELOG.md` | v49.3.0 エントリ追加 |

---

## 実装仕様

### ヘルパー関数（`driver.rs` に追加）

```rust
pub fn compute_file_fingerprint(path: &std::path::Path) -> Option<String> {
    use std::io::Read;
    use sha2::{Sha256, Digest};
    let mut file = std::fs::File::open(path).ok()?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).ok()?;
    let hash: [u8; 32] = Sha256::digest(&buf).into();
    Some(hash.iter().map(|b| format!("{:02x}", b)).collect::<String>())
}

pub fn file_needs_recheck(path: &std::path::Path, cache_dir: &std::path::Path) -> bool {
    let name = match path.file_name().and_then(|n| n.to_str()) {
        Some(n) => n,
        None => return true,
    };
    let cache_file = cache_dir.join(format!("{}.fp", name));
    let cached = std::fs::read_to_string(&cache_file).ok();
    let current = compute_file_fingerprint(path);
    match (cached, current) {
        (Some(c), Some(fp)) => c.trim() != fp.as_str(),
        _ => true,
    }
}

pub fn update_fingerprint_cache(path: &std::path::Path, cache_dir: &std::path::Path) {
    if let Some(fp) = compute_file_fingerprint(path) {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            let cache_file = cache_dir.join(format!("{}.fp", name));
            let _ = std::fs::create_dir_all(cache_dir);
            let _ = std::fs::write(cache_file, fp);
        }
    }
}
```

### キャッシュ形式

- ディレクトリ: `.fav-cache/`（プロジェクトルート直下）
- ファイル名: `<source_filename>.fp`（例: `main.fav.fp`）
- 内容: SHA-256 ハッシュ（hex 文字列）

---

## テスト（+2）

`v493000_tests` を `v492000_tests` の直前に追加:

```rust
#[cfg(test)]
mod v493000_tests {
    #[test]
    fn incremental_check_skips_unchanged() {
        let dir = tempfile::TempDir::new().unwrap();
        let fav_file = dir.path().join("main.fav");
        std::fs::write(&fav_file, "fn main() -> Int { 42 }").unwrap();
        let cache_dir = dir.path().join(".fav-cache");
        // 初回: キャッシュなし → 要再チェック
        assert!(super::file_needs_recheck(&fav_file, &cache_dir));
        // キャッシュ更新
        super::update_fingerprint_cache(&fav_file, &cache_dir);
        // 2回目: 未変更 → スキップ
        assert!(
            !super::file_needs_recheck(&fav_file, &cache_dir),
            "unchanged file should not need recheck"
        );
    }

    #[test]
    fn incremental_check_detects_change() {
        let dir = tempfile::TempDir::new().unwrap();
        let fav_file = dir.path().join("main.fav");
        std::fs::write(&fav_file, "fn main() -> Int { 42 }").unwrap();
        let cache_dir = dir.path().join(".fav-cache");
        // キャッシュ初期化
        super::update_fingerprint_cache(&fav_file, &cache_dir);
        // ファイル変更
        std::fs::write(&fav_file, "fn main() -> Int { 99 }").unwrap();
        // 変更検知 → 要再チェック
        assert!(
            super::file_needs_recheck(&fav_file, &cache_dir),
            "changed file should need recheck"
        );
    }
}
```

テスト数: 3073 → **3075**（+2）

---

## 注意事項

- `sha2` は `fav/Cargo.toml` に既存依存（`sha2 = "0.10"`）— 追加不要
- `tempfile` は `[dev-dependencies]` に既存登録済み — 追加不要
- `file_needs_recheck` / `update_fingerprint_cache` は `pub` で公開（`fav check` コマンドから呼び出し可能にする）
- `file_needs_recheck` は内部でファイルが開けない場合は `true`（要再チェック）を返す安全側フォールバック
- `update_fingerprint_cache` は `.fav-cache/` ディレクトリを自動作成する（`create_dir_all`）
- ロードマップの推定テスト数 3068 は旧推定値。v49.2.0 の実績が推定値（3068）を上回り 3073 となったため、本バージョン完了後は 3073 + 2 = **3075** になる
- `file_needs_recheck` のキャッシュキーはファイル名のみ（`<filename>.fp`）。同名ファイルが複数ディレクトリに存在するプロジェクトでは衝突する。v50.0 以降の hookup 時にパスハッシュをキーに変更すること（現バージョンは単一ファイルテストのみで顕在化しない）

---

## 完了条件

- `cargo test` 3075 passed, 0 failed（3073 + 2 件）
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"49.3.0"`
- `CHANGELOG.md` に v49.3.0 エントリ追加（インクリメンタルチェック・SHA-256・`.fav-cache/` を明記）
- `versions/current.md` を v49.3.0（3075 tests）に更新、進行中バージョンを `v49.4.0` に更新
- `versions/roadmap/roadmap-v49.1-v50.0.md` の v49.3.0 実績を記入
- `tasks.md` を COMPLETE に更新（T0〜T3 全 `[x]`）
