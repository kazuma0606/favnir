# v78.1.0 実装計画 — `!Cached` エフェクト基盤

Date: 2026-08-16

---

## 実装順序

### Step 1: 事前確認
- `fav/Cargo.toml` のバージョンが `78.0.0` であることを確認
- `cargo test` が 3760 tests all pass であることを確認
- `fav/tmp/hello.fav` が存在することを確認

### Step 2: driver.rs — 型・関数追加
`fav/src/driver.rs` の末尾（`// --- v78.0.0` ブロックの後）に追加：

1. セクションコメント `// --- v78.1.0: !Cached エフェクト基盤 ---`
2. `CacheStrategy` enum（`#[derive(Debug, Clone, PartialEq, Eq)]`、Lru / Fifo / Lfu）
3. `CacheConfig` 構造体（`#[derive(Debug, Clone, PartialEq, Eq)]`）
4. `CacheEntry` 構造体（`#[derive(Debug, Clone, PartialEq, Eq)]`）
5. `check_cache_valid(entry: &CacheEntry, now: i64, config: &CacheConfig) -> bool`

### Step 3: CHANGELOG.md 更新（テスト追加より先）
先頭に v78.1.0 エントリを追加。

### Step 4: driver.rs — テストモジュール追加
`v781000_tests` モジュールを追加（`use super::*`）：
- `cache_entry_valid_within_ttl`
- `cache_entry_expired`

### Step 5: Cargo.toml バージョン更新
- `78.0.0` → `78.1.0` に変更
- driver.rs 内の `78.0.0` バージョン文字列アサーションを一括更新（`replace_all: true`）
- grep で `// --- v78.0.0: Verifiable Pipelines 宣言 ---` が維持されていることを確認
- `grep "78.0.0" fav/src/driver.rs` で v76/v77 の `// NOTE:` コメント内の `78.0.0` 記述を確認し、`78.1.0` に書き換わっていた場合は手動で `78.0.0` に戻す

### Step 6: versions/current.md 更新
- `## 進行中バージョン` 欄を `**v78.1.0**（!Cached エフェクト基盤）` に更新
- `## 次に切る版` 欄を `**v78.2.0**（キャッシュ戦略型）` に更新

### Step 7: 最終確認
- `cargo test` が 3762 tests all pass であることを確認
- `cargo test v781000` で 2 件が pass することを確認
