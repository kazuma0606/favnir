# v78.3.0 タスクリスト — `!Adaptive` エフェクト基盤

Date: 2026-08-16
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `78.2.0` であることを確認
- [x] `cargo test` が全 pass（3766 tests）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v78.3.0: !Adaptive エフェクト基盤 ---` コメントを追加する
- [x] `ExecutionStrategy` enum を追加する（`#[derive(Debug, Clone, PartialEq, Eq, Hash)]`、BroadcastJoin / HashJoin / SortMergeJoin / Auto）
- [x] `AdaptiveConfig` 構造体を追加する（`#[derive(Debug, Clone, PartialEq, Eq)]`、broadcast_threshold_rows: u64, default_parallelism: usize）
- [x] `select_join_strategy(left_rows: u64, right_rows: u64, config: &AdaptiveConfig) -> ExecutionStrategy` を追加する
  - `min(left_rows, right_rows) <= broadcast_threshold_rows` → `BroadcastJoin`
  - それ以外 → `HashJoin`
- [x] `format_strategy_selected(strategy: &ExecutionStrategy) -> String` を追加する
  - 4 variant すべてに対する文字列を返す
- [x] `cargo test` で既存 3766 tests が pass することを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v78.3.0 エントリを追加する（形式: `## [v78.3.0] — 2026-08-16 — !Adaptive エフェクト基盤`）
- [x] Added セクション（enum 1 件・構造体 1 件・関数 2 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v783000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `adaptive_selects_broadcast_for_small_table` テストを実装する
  - `AdaptiveConfig { broadcast_threshold_rows: 1000, default_parallelism: 4 }`
  - `select_join_strategy(100_000, 500, &config)` → `BroadcastJoin`（right=500 <= 1000）
  - `format_strategy_selected` の出力が "BroadcastJoin" を含むことを assert
- [x] `adaptive_selects_hash_for_large_table` テストを実装する
  - 同じ config
  - `select_join_strategy(50_000, 80_000, &config)` → `HashJoin`（min=50_000 > 1000）
  - `format_strategy_selected` の出力が "HashJoin" を含むことを assert
- [x] `cargo test v783000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"78.2.0"` → `"78.3.0"` に変更する
- [x] driver.rs 内の `78.2.0` バージョン文字列アサーションを `78.3.0` に一括更新（`replace_all: true`）
- [x] **replace_all 後に** `grep -c "78.2.0" fav/src/driver.rs` を実行し **出力が 1** であることを確認する
  - 残るのは `// --- v78.2.0: キャッシュ戦略型 ---` セクションコメントの 1 件のみ
  - 1 より多ければアサーション文字列が残っているため手動で `78.2.0` を `78.3.0` に修正する

---

## T5: versions/current.md 更新

- [x] `## 進行中バージョン` 欄を `**v78.3.0**（!Adaptive エフェクト基盤）` に更新する
- [x] `## 次に切る版` 欄を `**v78.4.0**（コスト推定モデル）` に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3768 tests）
- [x] `cargo test v783000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `78.3.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v78.3.0]` であることを確認する
- [x] `versions/current.md` の「進行中バージョン」が v78.3.0 であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `adaptive_selects_broadcast_for_small_table` が pass
- [x] `adaptive_selects_hash_for_large_table` が pass
- [x] テスト総数: 3768（+2）
- [x] site/ MDX 追加: 本バージョンでは対象外（型基盤のみ）
- [x] `changelog_has_v78_3_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）

---

## code-reviewer 指摘対応（適用済み）

- [x] [MED] 境界値アサーションを既存テストに追加
  - `adaptive_selects_broadcast_for_small_table`: min==threshold=1000 → BroadcastJoin（`<=` 境界値確認）
  - `adaptive_selects_hash_for_large_table`: min==threshold+1=1001 → HashJoin（閾値超え確認）
- [x] [LOW] `AdaptiveConfig` struct doc に `default_parallelism` 未使用注記追加（v78.4.0 から利用予定）
- [x] [LOW] `select_join_strategy` doc に「境界値含む（`<=`）」の明示追加
