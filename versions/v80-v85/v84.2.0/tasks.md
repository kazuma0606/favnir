# v84.2.0 タスクリスト

Status: COMPLETE

---

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,911 tests, 0 failures を確認する（前提: v84.1.0 完了済み）
- [x] `grep -m1 '^version' fav/Cargo.toml` の出力が `version = "84.0.0"` であることを確認する
  （v84.x マイナーバージョンは Cargo.toml を更新しない慣例。v85.0.0 宣言時に一括更新する。
   この慣例は v84.0.0 宣言時から適用。v84.2.0 で独自に変更してはならない）
- [x] `fav/src/driver.rs` に `mod v84100_tests` が存在することを確認する（v84.1.0 完了済みの証拠）

## T1: `infra/e2e-demo/favnir4-showcase/pipeline.fav` にテスト統合セクションを追加

- [x] 既存の `pipeline.fav`（4 ステージ骨格）の末尾に以下の 3 関数を追加する
  - `showcase_stage_test` — `TestSuite.new` + `suite.add(StageTestCase {...})` + `TestSuite.run`
  - `showcase_golden_dataset` — `GoldenDataset.load` + `compare_golden_dataset`
  - `showcase_schema_snapshot` — `SchemaSnapshot.load` + `SchemaSnapshot.compare`
- [x] `bind` 構文を使用する（`let` は使わない）
- [x] コメント行 `-- ── テスト統合セクション（Sprint 1: Test-Driven Data 1.0）────────────` を先頭に追加する

## T2: `fav/src/driver.rs` に `v84200_tests` を追加

- [x] `mod v84100_tests { ... }` の直後に `#[cfg(test)] mod v84200_tests { ... }` を追加する
  - `use` 文は不要（`include_str!` はマクロのため）
  - `include_str!` は `"../../infra/..."` 形式を使用する（パス起点: `fav/src/`）
- [x] `showcase_test_suite_passes` テストを実装する
  - `include_str!("../../infra/e2e-demo/favnir4-showcase/pipeline.fav")` に `"TestSuite"` が含まれることを確認（メッセージ付き）
  - 同ファイルに `"StageTestCase"` が含まれることを確認（メッセージ付き）
- [x] `showcase_golden_dataset_comparison` テストを実装する
  - 同ファイルに `"GoldenDataset"` が含まれることを確認（メッセージ付き）
  - 同ファイルに `"SchemaSnapshot"` が含まれることを確認（メッセージ付き）

## T3: テスト通過確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,913 tests, 0 failures（+2）であることを確認する

## T4: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v84.2.0 エントリを追加する

> 注: 本バージョンは `pipeline.fav` 更新とテスト追加のみ。`site/` MDX 追加は v84.6.0 で実施する。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
