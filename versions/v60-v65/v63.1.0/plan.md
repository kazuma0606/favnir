# v63.1.0 Plan — 差分コンパイルキャッシュ（`.fav-cache/`）

Version: 63.1.0
Status: 未着手

---

## 実装順序

### Step 1: `fav/src/cache.rs` — `IncrementalCache` 新規作成

`fav/src/cache.rs` を新規作成する:
- `StageEntry { stage_name, source_hash, type_sig }` （serde Serialize/Deserialize）
- `IncrementalCache { root: PathBuf }` with `new / is_hit / store / invalidate / load_entry / entry_path`
- `pub fn stage_hash(src: &[u8]) -> String`（SHA-256、`sha2` クレートを使用）

（この時点では `lib.rs` に未登録のため `cargo build` は実施しない）

### Step 2: `fav/src/lib.rs` — モジュール登録

既存の `pub mod incremental;` の直後に
`#[cfg(not(target_arch = "wasm32"))] pub mod cache;` を追加する。

`cargo build` でエラーなしを確認（Step 1 + Step 2 合わせて初めてビルド検証）。

### Step 3: `driver.rs` — `cmd_incremental_cache_status` 追加

`driver.rs` の適切な位置に `pub fn cmd_incremental_cache_status(cache_dir: &str) -> String`
を追加する。

`cargo build` でエラーなしを確認。

### Step 4: `driver.rs` — `v63100_tests` 追加

`v63000_tests` の直前（ファイル先頭方向）に挿入する:

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

`cargo test v63100` で 2 件 PASS を確認。

### Step 5: 全テスト

`cargo test -j 8 -- --test-threads=8` で 3408 tests passed, 0 failed を確認（実測ベース + 2）。

### Step 6: ドキュメント更新

以下を更新する（tasks.md T6 の各項目に対応）:
1. `CHANGELOG.md` 先頭に v63.1.0 エントリを追加
2. `versions/roadmap/roadmap-v63.1-v64.0.md` v63.1.0 セクションに実績を追記・テスト数補正
3. `versions/roadmap/roadmap-v60.1-v65.0.md` テスト数推移表の v63.1.0 行を確認・更新
4. `versions/current.md` の「進行中」を v63.1.0（3408 tests）に更新
5. `tasks.md` を COMPLETE に更新（全チェックボックス `[x]`）

---

## 設計メモ

### `stage_hash` の命名

`fav/src/incremental/fingerprint.rs` に `content_hash` / `file_hash` が既にあるが、
それらは `fav::incremental::fingerprint` 名前空間にある。
`cache.rs` の `stage_hash` は `fav::cache::stage_hash` として独立させ、
将来的なシグネチャ変更（ステージ名を含んだハッシュ計算等）の柔軟性を確保する。

### `IncrementalCache::store` の設計

`store` は Result を返さず、エラーを握り潰す（`.ok()`）設計とする。
キャッシュ書き込み失敗はコンパイル・実行を止めるべきではないため。

### テストの独立性

並列テスト（`--test-threads=8`）での競合を避けるため `TempDir` を使用。
テスト終了時に自動削除されるためファイルシステムを汚染しない。
