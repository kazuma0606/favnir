# v78.4.0 タスクリスト — コスト推定モデル

Date: 2026-08-16
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `78.3.0` であることを確認
- [x] `cargo test` が全 pass（3768 tests）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v78.4.0: コスト推定モデル ---` コメントを追加する
- [x] `CostEstimate` 構造体を追加する（`#[derive(Debug, Clone, PartialEq)]` のみ、Eq / Hash は f64 のため付与しない）
  - `cpu_units: f64`, `memory_mb: f64`, `io_ops: u64`
- [x] `estimate_broadcast_cost(right_rows: u64) -> CostEstimate` を追加する
  - `cpu_units = right_rows as f64 * 0.01`
  - `memory_mb = right_rows as f64 * 0.1`
  - `io_ops = right_rows`
- [x] `estimate_hash_cost(left_rows: u64, right_rows: u64) -> CostEstimate` を追加する
  - `cpu_units = 5.0 + (left_rows + right_rows) as f64 * 0.0001`
  - `memory_mb = (left_rows + right_rows) as f64 * 0.01`
  - `io_ops = (left_rows + right_rows) / 2`
- [x] `select_min_cost_strategy(estimates: &[(ExecutionStrategy, CostEstimate)]) -> ExecutionStrategy` を追加する
  - `cpu_units` 最小のエントリのストラテジーを返す
  - 空スライスの場合は `ExecutionStrategy::Auto` を返す
- [x] `cargo build` でコンパイルエラーがないことを確認する
- [x] `cargo test` で既存 3768 tests が pass することを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v78.4.0 エントリを追加する（形式: `## [v78.4.0] — 2026-08-16 — コスト推定モデル`）
- [x] Added セクション（構造体 1 件・関数 3 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v784000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `cost_estimate_broadcast_cheaper_for_small` テストを実装する
  - `estimate_broadcast_cost(100)` vs `estimate_hash_cost(10_000, 100)`
  - `select_min_cost_strategy` → `BroadcastJoin` を assert
- [x] `cost_estimate_hash_wins_for_large` テストを実装する
  - `estimate_broadcast_cost(50_000)` vs `estimate_hash_cost(10_000, 50_000)`
  - `select_min_cost_strategy` → `HashJoin` を assert
- [x] `cargo test v784000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"78.3.0"` → `"78.4.0"` に変更する
- [x] driver.rs 内の `78.3.0` バージョン文字列アサーションを `78.4.0` に一括更新（`replace_all: true`）
- [x] **replace_all 後に** `grep -c "78.3.0" fav/src/driver.rs` を実行し **出力が 1** であることを確認する
  - 残るのは `// --- v78.3.0: !Adaptive エフェクト基盤 ---` の 1 件のみ

---

## T5: versions/current.md 更新

- [x] `## 進行中バージョン` 欄を `**v78.4.0**（コスト推定モデル）` に更新する
- [x] `## 次に切る版` 欄を `**v78.5.0**（fav explain plan 可視化）` に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3770 tests）
- [x] `cargo test v784000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `78.4.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v78.4.0]` であることを確認する
- [x] `versions/current.md` の「進行中バージョン」が v78.4.0 であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `cost_estimate_broadcast_cheaper_for_small` が pass
- [x] `cost_estimate_hash_wins_for_large` が pass
- [x] テスト総数: 3770（+2）
- [x] site/ MDX 追加: 本バージョンでは対象外（型基盤のみ）
- [x] `changelog_has_v78_4_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）

---

## code-reviewer 指摘対応（適用済み）

- [x] [HIGH] `estimate_hash_cost` の u64 オーバーフロー修正: `(left + right) as f64` → `left as f64 + right as f64`、io_ops に `saturating_add` 適用
- [x] [HIGH] `select_min_cost_strategy` の NaN 対応: `partial_cmp + unwrap_or(Equal)` → `total_cmp`（NaN を最大コスト扱いに確定）
- [x] [MED] `estimate_hash_cost` の `io_ops` にハッシュ分散の根拠コメント追加（"ハッシュパーティション分散により有効 IO は合計の半分"）
- [x] [LOW] 空スライス → Auto フォールバックのアサーションを `cost_estimate_hash_wins_for_large` 末尾に追加
- [x] [LOW] `select_min_cost_strategy` に `// TODO(v78.5.0): memory_mb / io_ops を加重スコアに組み込む予定` コメント追加
