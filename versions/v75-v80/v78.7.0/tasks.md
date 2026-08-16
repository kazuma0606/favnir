# v78.7.0 タスクリスト — Stream / Batch 統合実行モード

Date: 2026-08-16
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `78.6.0` であることを確認
- [x] `cargo test` が全 pass（3775 tests）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v78.7.0: Stream / Batch 統合実行モード ---` コメントを追加する
- [x] `ExecutionMode` enum を追加する（`#[derive(Debug, Clone, PartialEq, Eq, Hash)]`）
  - variants: `Batch`, `Streaming`, `Adaptive`
- [x] `ExecutionModeSelector` 構造体を追加する（`#[derive(Debug, Clone, PartialEq, Eq)]`、Hash なし）
  - `row_threshold: u64`, `latency_target_ms: u64`
- [x] `select_execution_mode(est_rows: u64, latency_target_ms_req: u64, selector: &ExecutionModeSelector) -> ExecutionMode` を追加する
  - 優先順位 1: `latency_target_ms_req < selector.latency_target_ms` → `Streaming`
  - 優先順位 2: `est_rows > selector.row_threshold` → `Batch`
  - 優先順位 3: それ以外 → `Adaptive`
- [x] `cargo build` でコンパイルエラーがないことを確認する
- [x] `cargo test` で既存 3775 tests が pass することを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v78.7.0 エントリを追加する（形式: `## [v78.7.0] — 2026-08-16 — Stream / Batch 統合実行モード`）
- [x] Added セクション（enum 1 件・struct 1 件・関数 1 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v787000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `make_selector(row_threshold, latency_ms)` ヘルパー関数を実装する
- [x] `mode_batch_for_large_data` テストを実装する
  - `select_execution_mode(10_000, 1_000, selector(5_000, 500))` → `Batch` を assert
  - 境界値: `latency_req == selector.latency_target_ms（500）` でも `Batch` を assert
- [x] `mode_streaming_for_low_latency` テストを実装する
  - `select_execution_mode(100, 50, selector(5_000, 500))` → `Streaming` を assert
  - 大量データでも latency 優先: `select_execution_mode(100_000, 50, selector)` → `Streaming` を assert
- [x] `cargo test v787000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"78.6.0"` → `"78.7.0"` に変更する
- [x] driver.rs 内の `78.6.0` バージョン文字列アサーションを `78.7.0` に一括更新（`replace_all: true`）
- [x] **replace_all 後に** `grep -c "78.6.0" fav/src/driver.rs` を実行し **出力が 1** であることを確認する
  - 残るのは `// --- v78.6.0: !Parallel エフェクト統合 ---` の 1 件のみ

---

## T5: versions/current.md 更新

- [x] `## 進行中バージョン` 欄を `**v78.7.0**（Stream / Batch 統合実行モード）` に更新する
- [x] `## 次に切る版` 欄を `**v78.8.0**（実行計画キャッシュ）` に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3777 tests）
- [x] `cargo test v787000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `78.7.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v78.7.0]` であることを確認する
- [x] `versions/current.md` の「進行中バージョン」が v78.7.0 であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `mode_batch_for_large_data` が pass
- [x] `mode_streaming_for_low_latency` が pass
- [x] テスト総数: 3778（+3、code-reviewer 対応で Adaptive 境界値テスト mode_adaptive_fallback +1）
- [x] site/ MDX 追加: 本バージョンでは対象外（型基盤のみ）
- [x] `changelog_has_v78_7_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）
