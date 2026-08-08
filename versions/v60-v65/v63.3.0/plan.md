# v63.3.0 Plan — キャッシュ型シグネチャ不整合検出 E0428

Version: 63.3.0
Status: 未着手

---

## 実装順序

### Step 1: `error_catalog.rs` — E0428 エントリ追加

`E0397` エントリの直後（ファイル末尾の `];` の前）に挿入する:

```rust
// ── E0428: キャッシュ型シグネチャ不整合 (v63.3.0) ──────────────────────
ErrorEntry {
    code: "E0428",
    title: "incremental_cache_conflict",
    category: "cache",
    description: "The cached type signature for a stage does not match the current compilation result.",
    example: "// stage Transform: cached Row -> Row, now Row -> EnrichedRow\n// E0428: incremental cache signature mismatch",
    fix: "The cache entry has been automatically invalidated. Re-run to recompile.",
    long_description: Some("Favnir's incremental cache stores the type signature of each stage alongside its source hash. If the source hash matches but the type signature has changed (e.g., after refactoring a return type), E0428 is emitted as a warning. The cache entry is automatically invalidated and the stage will be recompiled on the next run."),
    suggestion: Some("This is a non-fatal warning. The cache has been cleared for the affected stage."),
},
```

`cargo build` でエラーなしを確認。

### Step 2: `cache.rs` — `check_type_sig` メソッド追加

`invalidate` メソッドの直後に追加する:

```rust
/// ソースハッシュと型シグネチャの両方を検証する。
/// - ハッシュ + シグ両方一致 → true（キャッシュヒット、再コンパイル不要）
/// - ハッシュ一致・シグ不一致 → E0428 警告を eprintln!、キャッシュを自動無効化、false を返す
/// - ハッシュ不一致またはエントリなし → false（通常のキャッシュミス）
pub fn check_type_sig(&self, stage_name: &str, source_hash: &str, current_sig: &str) -> bool {
    match self.load_entry(stage_name) {
        Some(e) if e.source_hash == source_hash && e.type_sig == current_sig => true,
        Some(e) if e.source_hash == source_hash && e.type_sig != current_sig => {
            eprintln!(
                "E0428: incremental cache signature mismatch\n  stage `{}` の型シグネチャがキャッシュと一致しません。\n  cached:  {}\n  current: {}\n  キャッシュを無効化して再コンパイルします。",
                stage_name, e.type_sig, current_sig
            );
            self.invalidate(stage_name);
            false
        }
        _ => false,
    }
}
```

`cargo build` でエラーなしを確認。

### Step 3: `driver.rs` — `v63300_tests` 追加

`v63200_tests` の直前（ファイル先頭方向）に挿入する:

```rust
// -- v63300_tests (v63.3.0) -- E0428 キャッシュ型シグネチャ不整合検出 --
#[cfg(test)]
mod v63300_tests {
    use crate::cache::{IncrementalCache, stage_hash};
    use tempfile::TempDir;

    #[test]
    fn incremental_e0428_signature_mismatch() {
        let dir = TempDir::new().unwrap();
        let cache = IncrementalCache::new(dir.path());
        let hash = stage_hash(b"fn transform(r: Row) -> Row { r }");
        cache.store("Transform", &hash, "Row -> Row");
        let result = cache.check_type_sig("Transform", &hash, "Row -> EnrichedRow");
        assert!(!result, "signature mismatch should return false");
    }

    #[test]
    fn cache_auto_invalidated() {
        let dir = TempDir::new().unwrap();
        let cache = IncrementalCache::new(dir.path());
        let hash = stage_hash(b"fn transform(r: Row) -> Row { r }");
        cache.store("Transform", &hash, "Row -> Row");
        cache.check_type_sig("Transform", &hash, "Row -> EnrichedRow");
        assert!(
            !cache.is_hit("Transform", &hash),
            "cache should be invalidated after E0428"
        );
    }
}
```

`cargo test v63300` で 2 件 PASS を確認。

### Step 4: 全テスト

`cargo test -j 8 -- --test-threads=8` で 3412 tests passed, 0 failed を確認。

### Step 5: ドキュメント更新

tasks.md T4 の各項目に従って更新する:
1. `CHANGELOG.md` 先頭に v63.3.0 エントリを追加
2. `versions/roadmap/roadmap-v63.1-v64.0.md` v63.3.0 セクションに実績追記
3. `versions/current.md` の「進行中」を v63.3.0（3412 tests）に更新
4. 最終ステップとして `tasks.md` を COMPLETE に更新（全チェックボックス `[x]`）

---

## 設計メモ

### 変更対象ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/error_catalog.rs` | E0428 エントリを追加（`ERROR_CATALOG` 配列末尾） |
| `fav/src/cache.rs` | `check_type_sig` メソッドを追加 |
| `fav/src/driver.rs` | `v63300_tests` を追加 |

### `check_type_sig` の位置

`invalidate` の直後に配置することで、無効化→シグネチャチェックの流れが
コードの近傍で読めるようにする。

### `match` パターンのガード順序

Rust の `match` はガードを上から評価するため、
`Some(e) if ... && type_sig == ...` を `Some(e) if ... && type_sig != ...` より先に置く。
順序を入れ替えると型不一致ケースが先にマッチして誤動作する。

### `v63300_tests` の挿入位置

`v63200_tests` の直前（ファイル先頭方向）に挿入する。
バージョン番号の降順で `v632 > v633` ではなく、
慣例として新しいテストモジュールを古いものの直前に挿入している（`v63100_tests` 前に `v63200_tests` を挿入した前例に倣う）。
