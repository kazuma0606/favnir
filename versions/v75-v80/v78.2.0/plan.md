# v78.2.0 実装計画 — キャッシュ戦略型

---

## Step 1: 事前確認

- `fav/Cargo.toml` のバージョンが `78.1.0` であることを確認
- `cargo test` が全 pass（3763 tests）であることを確認
- `fav/tmp/hello.fav` が存在することを確認

---

## Step 2: driver.rs — 型・関数追加

`fav/src/driver.rs` の末尾（v78.1.0 ブロックの直後）に以下を追加する。

```
// --- v78.2.0: キャッシュ戦略型 ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheStats {
    pub hits:      u64,
    pub misses:    u64,
    pub evictions: u64,
}

pub fn simulate_lru_cache(accesses: &[&str], max_entries: usize) -> CacheStats { ... }

pub fn format_cache_stats_report(stats: &CacheStats) -> String { ... }

pub fn hit_rate(stats: &CacheStats) -> f64 { ... }
```

実装の詳細:

- `simulate_lru_cache`:
  - `use std::collections::VecDeque`（関数内ローカル）で LRU キューを管理
  - ヒット: キュー内に存在する場合 → 末尾へ移動（最近使用済みに昇格）、hits += 1
  - ミス: キュー内に存在しない場合 → misses += 1
    - `queue.len() >= max_entries` の場合: 先頭を pop（LRU エビクション）、evictions += 1
    - キューに push_back
  - `max_entries == 0`: 常にミス（push しない）
- `format_cache_stats_report`:
  - `hit_rate(stats)` を呼び出して `{:.1}%` フォーマット
  - 以下の形式の文字列を返す:
    ```
    Cache Stats:\n  hits:      {hits} ({rate:.1}%)\n  misses:    {misses}\n  evictions: {evictions}
    ```
- `hit_rate`:
  - `hits + misses == 0` の場合は `0.0`
  - それ以外: `hits as f64 / (hits + misses) as f64 * 100.0`

---

## Step 3: CHANGELOG.md 更新（テスト追加より先）

`CHANGELOG.md` の先頭（`[v78.1.0]` エントリの前）に v78.2.0 エントリを追加する。

```markdown
## [v78.2.0] — 2026-08-16 — キャッシュ戦略型

### Added
- `CacheStats` 構造体（hits: u64, misses: u64, evictions: u64）: ...
- `simulate_lru_cache(accesses: &[&str], max_entries: usize) -> CacheStats`: ...
- `format_cache_stats_report(stats: &CacheStats) -> String`: ...
- `hit_rate(stats: &CacheStats) -> f64`: ...

### Tests
- `lru_evicts_least_recently_used`: ...
- `cache_hit_rate_calculated`: ...
```

---

## Step 4: driver.rs — テストモジュール追加

```rust
#[cfg(test)]
mod v782000_tests {
    use super::*;

    #[test]
    fn lru_evicts_least_recently_used() {
        // max_entries=2、アクセス: [A, B, C, A]
        // 初期: miss A → {A}  / miss B → {A,B}
        // miss C → evict A → {B,C}  / miss A → evict B → {C,A}  -- A は最近参照されたがその後 C, A の順で追加
        // 実際の動作: A miss, B miss, C miss (evict A), A miss (evict B)
        // hits=0, misses=4, evictions=2
        let stats = simulate_lru_cache(&["A", "B", "C", "A"], 2);
        assert_eq!(stats.evictions, 2);
        assert_eq!(stats.misses, 4);
    }

    #[test]
    fn cache_hit_rate_calculated() {
        // max_entries=3、アクセス: [A, B, C, A, B]
        // A miss, B miss, C miss, A hit, B hit
        // hits=2, misses=3, evictions=0
        let stats = simulate_lru_cache(&["A", "B", "C", "A", "B"], 3);
        assert_eq!(stats.hits,   2);
        assert_eq!(stats.misses, 3);
        let rate = hit_rate(&stats);
        // 2/5 * 100 = 40.0
        assert!((rate - 40.0).abs() < 0.01);
        // format_cache_stats_report の出力が "Cache Stats:" と "40.0%" を含むことを確認
        let report = format_cache_stats_report(&stats);
        assert!(report.contains("Cache Stats:"));
        assert!(report.contains("40.0%"));
    }
}
```

テスト後に `cargo test v782000` で 2 件 pass を確認。

---

## Step 5: Cargo.toml バージョン更新

- `version` を `"78.1.0"` → `"78.2.0"` に変更
- driver.rs 内のバージョン文字列アサーション（`78.1.0`）を `78.2.0` に一括更新（`replace_all: true`）
- **replace_all 後に** `grep "v78.1.0" fav/src/driver.rs` を実行して確認:
  - `// --- v78.1.0: !Cached エフェクト基盤 ---` が残っていること
  - CHANGELOG/tasks.md 内テキストの `78.1.0` は書き換わっていないこと（`include_str!` は別ファイルなので無関係）

---

## Step 6: versions/current.md 更新

- `## 進行中バージョン` 欄を `**v78.2.0**（キャッシュ戦略型）` に更新
- `## 次に切る版` 欄を `**v78.3.0**（!Adaptive エフェクト基盤）` に更新

---

## Step 7: 最終確認

- `cargo test` が全 pass（3765 tests）であることを確認
- `cargo test v782000` で 2 件 pass を確認
- `fav/Cargo.toml` のバージョンが `78.2.0` であることを確認
- `CHANGELOG.md` の先頭が `[v78.2.0]` であることを確認
