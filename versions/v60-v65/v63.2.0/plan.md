# v63.2.0 Plan — `fav watch` 改善・IncrementalCache 統合

Version: 63.2.0
Status: 未着手

---

## 実装順序

### Step 1: `driver.rs` — `cmd_run_with_cache` 追加

`cmd_incremental_cache_status` の直後（または近傍）に以下を追加する:

```rust
pub fn cmd_run_with_cache(src: &str, cache_dir: &str) -> String {
    use crate::cache::{IncrementalCache, stage_hash};
    let hash = stage_hash(src.as_bytes());
    let root = std::path::Path::new(cache_dir);
    let cache = IncrementalCache::new(root);
    if cache.is_hit("__pipeline__", &hash) {
        return "cache hit: (skipped recompile)".to_string();
    }
    let result = match crate::frontend::parser::Parser::parse_str(src, "<cache>") {
        Ok(_) => "ok".to_string(),
        Err(e) => return format!("parse error: {e}"),
    };
    cache.store("__pipeline__", &hash, "pipeline");
    result
}
```

`cargo build` でエラーなしを確認。

### Step 2: `driver.rs` — `v63200_tests` 追加

`v63100_tests` の直前（ファイル先頭方向）に挿入する:

```rust
// -- v63200_tests (v63.2.0) -- fav watch 改善・IncrementalCache 統合 --
#[cfg(test)]
mod v63200_tests {
    use crate::cache::{IncrementalCache, stage_hash};
    use tempfile::TempDir;

    #[test]
    fn watch_incremental_recompile() {
        let dir = TempDir::new().unwrap();
        let cache = IncrementalCache::new(dir.path());
        let src = "fn main() -> Bool { true }";
        let hash = stage_hash(src.as_bytes());
        assert!(!cache.is_hit("__pipeline__", &hash), "first run: expected cache miss");
        cache.store("__pipeline__", &hash, "pipeline");
        assert!(cache.is_hit("__pipeline__", &hash), "second run: expected cache hit");
    }

    #[test]
    fn watch_notify_integration() {
        use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
        use std::sync::mpsc;
        let (tx, _rx) = mpsc::channel();
        let watcher = RecommendedWatcher::new(
            move |res| { let _ = tx.send(res); },
            Config::default(),
        );
        assert!(watcher.is_ok(), "RecommendedWatcher should be constructable");
        let mut w = watcher.unwrap();
        let dir = TempDir::new().unwrap();
        let result = w.watch(dir.path(), RecursiveMode::NonRecursive);
        assert!(result.is_ok(), "watching a valid directory should succeed");
    }
}
```

`cargo test v63200` で 2 件 PASS を確認。

### Step 3: 全テスト

`cargo test -j 8 -- --test-threads=8` で 3410 tests passed, 0 failed を確認。

### Step 4: ドキュメント更新

tasks.md T4 の各項目に従って更新する:
1. `CHANGELOG.md` 先頭に v63.2.0 エントリを追加
2. `versions/roadmap/roadmap-v63.1-v64.0.md` v63.2.0 セクションに実績追記
3. `versions/current.md` の「進行中」を v63.2.0（3410 tests）に更新
4. 最終ステップとして `tasks.md` を COMPLETE に更新（全チェックボックス `[x]`）

---

## 設計メモ

### `__pipeline__` キーの意味

`IncrementalCache::entry_path` のサニタイズにより `__pipeline__` は
`__pipeline__.json` というファイル名になる（`__` は `_` として許容される）。
アンダースコア 2 つは `[a-zA-Z0-9_-]` の範囲内なので変換されない。

### `watch_notify_integration` の構造

`RecommendedWatcher::new` → `w.watch(dir, NonRecursive)` → `drop(w)` の流れで
`notify` クレートが正常に動作することを確認する。
イベント受信は行わない（`_rx` で無視）。

### `cmd_run_with_cache` の位置

`cmd_incremental_cache_status` の直後に配置することで、
キャッシュ関連の関数が局所化され後から読みやすくなる。
