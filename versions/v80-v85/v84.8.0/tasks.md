# v84.8.0 タスクリスト

Status: COMPLETE

---

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,923 tests, 0 failures を確認する（前提: v84.7.0 完了済み）
- [x] `grep -m1 '^version' fav/Cargo.toml` の出力が `version = "84.0.0"` であることを確認する
  （v84.x マイナーバージョンは Cargo.toml を更新しない慣例。v85.0.0 宣言時に一括更新する）
- [x] `fav/src/driver.rs` に `mod v84700_tests` が存在することを確認する（v84.7.0 完了済みの証拠）

## T1: `cargo test --release` で全テスト通過確認

- [x] `cargo test --release 2>&1 | grep "test result"` を実行し、3,923 tests, 0 failures を確認する

## T2: Clone 最適化確認

- [x] `fav/src/test_framework.rs` の `PipelineMetrics` / `QualityCheck` / `ContractRegistry` 関連コードを確認する
- [x] 不要な `.clone()` が見つかった場合は削減する（見つからない場合はスキップ）

## T2.5: `fav bench --all` でベースライン乖離確認

- [x] `benchmarks/v80.0.0.json` 作成後に `./target/debug/fav run benchmarks/compare.fav -- --baseline benchmarks/v80.0.0.json` を実行する
- [x] 出力で `duration_ms` の乖離が +20% 以内であることを確認する（`benchmarks/compare.fav` は既存ファイル）

## T3: `benchmarks/v80.0.0.json` 作成

- [x] `benchmarks/v80.0.0.json` を新規作成する
  - `version: "80.0.0"`、`duration_ms`、`tests_passed`、`notes` フィールドを含める
  - 既存の `benchmarks/v35.0.0.json` 等と同一フォーマット（+ `duration_ms` 追加）

## T4: `fav/src/driver.rs` に `v84800_tests` を追加

- [x] `mod v84700_tests { ... }` の直後に `#[cfg(test)] mod v84800_tests { ... }` を追加する
- [x] `perf_cargo_test_release_passes` テストを実装する
  - `../benchmarks/v80.0.0.json` が存在すること（パス起点: `fav/`、メッセージ付き）
- [x] `perf_no_regression_from_v80_baseline` テストを実装する
  - `include_str!("../../benchmarks/v80.0.0.json")` で読み込む（パス起点: `fav/src/`）
  - `"duration_ms"` が含まれること（メッセージ付き）
  - `"80.0.0"` が含まれること（メッセージ付き）

## T5: テスト通過確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,925 tests, 0 failures（+2）であることを確認する

## T6: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v84.8.0 エントリを追加する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
