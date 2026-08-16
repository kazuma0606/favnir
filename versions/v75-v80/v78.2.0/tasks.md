# v78.2.0 タスクリスト — キャッシュ戦略型

Date: 2026-08-16
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `78.1.0` であることを確認
- [x] `cargo test` が全 pass（3763 tests）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v78.2.0: キャッシュ戦略型 ---` コメントを追加する
- [x] `CacheStats` 構造体を追加する（`#[derive(Debug, Clone, PartialEq, Eq)]`、hits: u64, misses: u64, evictions: u64）
- [x] `simulate_lru_cache(accesses: &[&str], max_entries: usize) -> CacheStats` を追加する
  - LRU キューで各アクセスを処理（ヒット: 末尾へ移動 / ミス: 先頭エビクション + push_back）
  - `max_entries == 0` の場合は常にミス
- [x] `format_cache_stats_report(stats: &CacheStats) -> String` を追加する
  - `hit_rate(stats)` を使って `{:.1}%` でヒット率を表示
- [x] `hit_rate(stats: &CacheStats) -> f64` を追加する
  - `hits + misses == 0` → `0.0`; それ以外 → `hits / total * 100.0`
- [x] `cargo test` で既存 3763 tests が pass することを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v78.2.0 エントリを追加する（形式: `## [v78.2.0] — 2026-08-16 — キャッシュ戦略型`）
- [x] Added セクション（型 1 件・関数 3 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v782000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `lru_evicts_least_recently_used` テストを実装する
  - `simulate_lru_cache(&["A", "B", "C", "A"], 2)`
  - `stats.evictions == 2` / `stats.misses == 4` を assert
- [x] `cache_hit_rate_calculated` テストを実装する
  - `simulate_lru_cache(&["A", "B", "C", "A", "B"], 3)`
  - `stats.hits == 2` / `stats.misses == 3` を assert
  - `hit_rate(&stats)` が `40.0` に近いことを assert（誤差 < 0.01）
  - `format_cache_stats_report(&stats)` が `"Cache Stats:"` と `"40.0%"` を含むことを assert
- [x] `cargo test v782000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"78.1.0"` → `"78.2.0"` に変更する
- [x] driver.rs 内の `78.1.0` バージョン文字列アサーションを `78.2.0` に一括更新（`replace_all: true`）
- [x] **replace_all 後に** `grep "78.1.0" fav/src/driver.rs` を実行し、以下を確認する:
  - `// --- v78.1.0: !Cached エフェクト基盤 ---` が **1 件**残っていること
  - それ以外の `78.1.0` 文字列が 0 件であること（アサーションが書き換わっていないこと）

---

## T5: versions/current.md 更新

- [x] `## 進行中バージョン` 欄を `**v78.2.0**（キャッシュ戦略型）` に更新する
- [x] `## 次に切る版` 欄を `**v78.3.0**（!Adaptive エフェクト基盤）` に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3765 tests）
- [x] `cargo test v782000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `78.2.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v78.2.0]` であることを確認する
- [x] `versions/current.md` の「進行中バージョン」が v78.2.0 であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `lru_evicts_least_recently_used` が pass
- [x] `cache_hit_rate_calculated` が pass
- [x] テスト総数: 3766（+3: `lru_evicts_least_recently_used` / `cache_hit_rate_calculated` / `simulate_lru_max_entries_zero`）
- [x] site/ MDX 追加: 本バージョンでは対象外（型基盤のみ）
- [x] `changelog_has_v78_2_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）

---

## code-reviewer 指摘対応（適用済み）

- [x] [MED] `CacheStats` に `Hash` derive 追加（v78.1.0 の兄弟型との一貫性）
- [x] [MED] `simulate_lru_cache` に O(n) 計算量コメント追加（統計シミュレーション用途のため許容）
- [x] [LOW] `max_entries==0` 境界値テスト `simulate_lru_max_entries_zero` 追加（常にミス・エビクションなし）
