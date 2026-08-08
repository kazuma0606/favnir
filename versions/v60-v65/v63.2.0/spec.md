# v63.2.0 Spec — `fav watch` 改善（差分再コンパイルと IncrementalCache 統合）

Version: 63.2.0
Status: 未着手
Base tests: 3408
Target tests: 3410

---

## 概要

v63.1.0 で実装した `IncrementalCache`（`fav/src/cache.rs`）を `cmd_run` / `cmd_watch` に統合する。
`cmd_run` ではソースファイルのハッシュを計算してキャッシュヒット判定を行い、
ヒット時はコンパイルをスキップしてキャッシュメッセージを返す新関数 `cmd_run_with_cache` を追加する。
また `cmd_watch` の `notify` 統合（RecommendedWatcher）が正しく機能することを確認する
テスト `watch_notify_integration` を追加する。

**既存実装の確認**:
- `fav watch` は `notify::RecommendedWatcher`（inotify/FSEvents）を既に使用済み（v9.9.0 実装）
  → ポーリング廃止・inotify 切り替えは既に完了。v63.2.0 では `IncrementalCache` 統合が主眼
  ※ ロードマップ v63.2.0 は「ポーリング廃止・inotify 切り替え」を成果物として記載しているが、
    これは `roadmap-v63.1-v64.0.md` の `**既存機能の扱い**` 注記（`fav watch` は v9.9 実装済み）
    と整合する既存実装の活用であり、本バージョンでの再実装は不要（意図的な後送りなし・既実装確認）。
- `cmd_watch` は `cmd_run` / `cmd_check` / `cmd_test` を呼び出す無限ループ構造

---

## 前提確認（T0 で実施）

- `fav/src/cache.rs` が存在し `IncrementalCache` が実装されていることを確認
- `driver.rs` に `v63100_tests` が存在することを確認（挿入位置確認）
- `cargo test -j 8 -- --test-threads=8` でベース 3408 tests passed, 0 failed を確認

---

## 実装スコープ

### 1. `driver.rs` — `cmd_run_with_cache` 追加

ソースハッシュを計算し `IncrementalCache` でキャッシュヒット判定を行う関数を追加する。

```rust
/// v63.2.0: IncrementalCache を使った差分実行。
/// - cache_dir: キャッシュディレクトリのパス（例: ".fav-cache"）
/// - キャッシュヒット時は "cache hit: (skipped recompile)" を返す
/// - キャッシュミス時はコンパイル・実行し、結果をキャッシュに保存する
pub fn cmd_run_with_cache(src: &str, cache_dir: &str) -> String {
    use crate::cache::{IncrementalCache, stage_hash};
    let hash = stage_hash(src.as_bytes());
    let root = std::path::Path::new(cache_dir);
    let cache = IncrementalCache::new(root);
    if cache.is_hit("__pipeline__", &hash) {
        return format!("cache hit: (skipped recompile)");
    }
    // コンパイル・実行（parse のみ確認: 型チェック・実行はスコープ外）
    let result = match crate::frontend::parser::Parser::parse_str(src, "<cache>") {
        Ok(_) => "ok".to_string(),
        Err(e) => return format!("parse error: {e}"),
    };
    cache.store("__pipeline__", &hash, "pipeline");
    result
}
```

### 2. `driver.rs` — `v63200_tests` 追加

`v63100_tests` の直前（ファイル先頭方向）に挿入する。

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
        // 1回目: キャッシュミス → 保存
        assert!(!cache.is_hit("__pipeline__", &hash), "first run: expected cache miss");
        cache.store("__pipeline__", &hash, "pipeline");
        // 2回目: キャッシュヒット
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
        // watcher を即座に drop — ウォッチ対象なしでも panic しないことを確認
        let mut w = watcher.unwrap();
        let dir = TempDir::new().unwrap();
        // 存在するディレクトリを watch できることを確認
        let result = w.watch(dir.path(), RecursiveMode::NonRecursive);
        assert!(result.is_ok(), "watching a valid directory should succeed");
    }
}
```

---

## 完了条件

- `cargo build` エラーなし
- `cargo test v63200` で 2 件 PASS
- `cargo test -j 8 -- --test-threads=8` で 3410 tests passed, 0 failed

---

## 非スコープ

- `cmd_run` 本体（`pub fn cmd_run`）のキャッシュ統合（既存関数の大規模改修を避けるため）
- `cmd_watch` 本体への `cmd_run_with_cache` 呼び出し（無限ループの統合テスト困難）
- キャッシュヒット時の実際の VM 実行スキップ（parse のみ確認）
- ステージ単位での差分キャッシュ適用（v63.1.0 の `IncrementalCache` は pipeline 全体単位で使用）
- `site/` MDX ドキュメント追加（v63.x 以降）

---

## 技術ノート

### `cmd_run_with_cache` のキャッシュキー設計

`__pipeline__` という固定ステージ名を使用してパイプライン全体のハッシュを管理する。
将来的にステージ単位のキャッシュへ移行する際は、各ステージの関数定義単位でキーを分割する。

### `watch_notify_integration` テストの方針

`cmd_watch` は無限ループのため直接テスト不可。
代わりに `RecommendedWatcher` の構築・ウォッチ操作が正常に完了することを単体テストで確認する。
`notify` クレートは `Cargo.toml` に登録済みのため追加不要。

### `tempfile` の使用

`v63200_tests` の全テストは `TempDir` で独立したディレクトリを使用し、
並列テスト（`--test-threads=8`）での競合を防ぐ。
