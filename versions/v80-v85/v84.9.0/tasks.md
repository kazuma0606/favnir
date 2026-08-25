# v84.9.0 タスクリスト

Status: COMPLETE

---

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,925 tests, 0 failures を確認する（前提: v84.8.0 完了済み）
- [x] `grep -m1 '^version' fav/Cargo.toml` の出力が `version = "84.0.0"` であることを確認する
  （v84.x マイナーバージョンは Cargo.toml を更新しない慣例。v85.0.0 宣言時に一括更新する）
- [x] `fav/src/driver.rs` に `mod v84800_tests` が存在することを確認する（v84.8.0 完了済みの証拠）

## T1: E2E ショーケース統合確認

- [x] `infra/e2e-demo/favnir4-showcase/pipeline.fav` に Sprint 1〜4 全識別子が含まれることを確認する
  - `TestSuite`（Sprint 1）/ `QualityCheck`（Sprint 2）/ `ContractRegistry`（Sprint 3）/ `PipelineMetrics`（Sprint 4）
- [x] `infra/e2e-demo/favnir4-showcase/fav.toml` に `[quality]`・`[contract]`・`[observe]` が含まれることを確認する

## T2: v84.1〜v84.8 全テストモジュール存在確認

- [x] `fav/src/driver.rs` に以下のテストモジュールが存在することを確認する:
  `v84100_tests`, `v84200_tests`, `v84300_tests`, `v84400_tests`,
  `v84500_tests`, `v84600_tests`, `v84700_tests`, `v84800_tests`

> ⚠️ 本バージョンはバグ修正のみ受け入れる。新機能追加は行わない。

## T3: `fav/src/driver.rs` に `v84900_tests` を追加

- [x] `mod v84800_tests { ... }` の直後に `#[cfg(test)] mod v84900_tests { ... }` を追加する
  - `include_str!` パス起点: `fav/src/`（`../../infra/e2e-demo/favnir4-showcase/...`）
- [x] `favnir4_full_sprint_all_stable` テストを実装する
  - `TestSuite` が含まれること（Sprint 1 メッセージ付き）
  - `QualityCheck` が含まれること（Sprint 2 メッセージ付き）
  - `ContractRegistry` が含まれること（Sprint 3 メッセージ付き）
  - `PipelineMetrics` が含まれること（Sprint 4 メッセージ付き）
- [x] `favnir4_e2e_showcase_runs` テストを実装する
  - `fav.toml` に `[quality]` が含まれること（メッセージ付き）
  - `fav.toml` に `[contract]` が含まれること（メッセージ付き）
  - `fav.toml` に `[observe]` が含まれること（メッセージ付き）

## T4: テスト通過確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,927 tests, 0 failures（+2）であることを確認する

## T5: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v84.9.0 エントリを追加する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
