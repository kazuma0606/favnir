# v78.2.0 仕様書 — キャッシュ戦略型

Date: 2026-08-16

---

## Background

v78.1.0 で `CacheStrategy` / `CacheConfig` / `CacheEntry` / `check_cache_valid` の基盤型を導入した。
v78.2.0 では LRU シミュレーションとヒット率計算を通じて、`CacheStrategy` を実際の動作型として活用する。
`CacheStats` 構造体はキャッシュ統計レポートの基盤となり、v78.x スプリント後半の Adaptive/実行計画キャッシュで参照される。

ロードマップ見出しには「LRU / FIFO / LFU」と記載されているが、v78.2.0 のスコープは **LRU シミュレーションのみ**。
FIFO / LFU のシミュレーション関数は v78.x 後半バージョン（v78.4.0 以降）で追加予定。

---

## Goals

1. `CacheStats` 構造体を追加する（hits / misses / evictions）
2. `simulate_lru_cache` 関数を追加する（LRU アルゴリズムのシミュレーション）
3. `format_cache_stats_report` 関数を追加する（統計の可読テキスト表現）
4. `hit_rate` 関数を追加する（ヒット率 f64 計算）
5. テスト 2 件を追加する（3763 → 3765）

---

## API 仕様

### `CacheStats`

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheStats {
    pub hits:      u64,
    pub misses:    u64,
    pub evictions: u64,
}
```

### `simulate_lru_cache`

```rust
pub fn simulate_lru_cache(accesses: &[&str], max_entries: usize) -> CacheStats
```

- `accesses` のキーを順番にアクセスし、LRU ポリシーでキャッシュをシミュレートする
- キャッシュヒット: `hits += 1`
- キャッシュミス（新規エントリ）: `misses += 1`
- キャッシュ容量超過時の LRU エビクション: `evictions += 1`
- `max_entries == 0` の場合は常にミス（エントリ保持不可）: ヒット率 0
- `accesses.is_empty()` の場合は `CacheStats { hits: 0, misses: 0, evictions: 0 }`

### `format_cache_stats_report`

```rust
pub fn format_cache_stats_report(stats: &CacheStats) -> String
```

出力例:

```
Cache Stats:
  hits:      8432 (84.3%)
  misses:    1568
  evictions: 204
```

- `hit_rate` を使ってヒット率を計算し、1 桁小数（`{:.1}`）で表示
- total アクセス 0 の場合はヒット率 `0.0%`

### `hit_rate`

```rust
pub fn hit_rate(stats: &CacheStats) -> f64
```

- `stats.hits as f64 / (stats.hits + stats.misses) as f64 * 100.0`
- `hits + misses == 0` の場合は `0.0` を返す

---

## 出力例（`fav cache stats`）

```
$ fav cache stats --pipeline pipeline.fav
Cache Stats:
  hits:      8432 (84.3%)
  misses:    1568
  evictions: 204
  strategy:  LRU (max=1000 entries)
```

※ `strategy` フィールドの出力は将来バージョン（v78.4〜）で追加予定。v78.2.0 では `format_cache_stats_report` は hits/misses/evictions のみ出力する。

---

## Success Criteria

- `CacheStats` 構造体が `Debug / Clone / PartialEq / Eq` を持つ
- `simulate_lru_cache` が正しく LRU エビクションをシミュレートする
- `format_cache_stats_report` がヒット率を 1 桁小数で表示する
- `hit_rate` が ゼロ除算に対し `0.0` を返す
- テスト 2 件（`lru_evicts_least_recently_used` / `cache_hit_rate_calculated`）が pass する
- `cargo test` 全体が 3765 tests pass する

---

## Files to Modify

- `fav/src/driver.rs` — 型・関数・テストモジュール追加
- `CHANGELOG.md` — v78.2.0 エントリ追加
- `fav/Cargo.toml` — version を `78.1.0` → `78.2.0` に更新
- `versions/current.md` — 進行中バージョン更新

---

## Error Codes

新規エラーコードなし。
