# v84.1.0 タスクリスト

Status: COMPLETE

---

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,909 tests, 0 failures を確認する（前提: v84.0.0 完了済み）
- [x] `grep -m1 '^version' fav/Cargo.toml` の出力が `version = "84.0.0"` であることを確認する
  （v84.x マイナーバージョンは Cargo.toml を更新しない慣例。v85.0.0 宣言時に一括更新する）
- [x] `fav/src/driver.rs` に `mod v84000_tests` が存在することを確認する（v84.0.0 完了済みの証拠）

## T1: `infra/e2e-demo/favnir4-showcase/` 作成

- [x] `infra/e2e-demo/favnir4-showcase/` ディレクトリを作成する
- [x] `pipeline.fav` を作成する
  - `load_stage` / `transform_stage` / `quality_stage` / `observe_stage` の 4 関数（骨格）
  - 各関数は `Result<List<Row>, String>` を返す
  - Favnir 構文（`bind` を使用。`let` は使わない）
- [x] `fav.toml` を作成する
  - `[package]`・`[quality]`・`[contract]`・`[observe]` の 4 セクションを含める
- [x] `contract.fav` を作成する
  - `Favnir4ShowcaseContract` 型を宣言する（`input_fields`・`output_fields`・`sla_ms` フィールド）
- [x] `README.md` を作成する（概要・前提・実行方法を記述）

## T2: `fav/src/driver.rs` に `v84100_tests` を追加

- [x] `mod v84000_tests { ... }` の直後に `#[cfg(test)] mod v84100_tests { ... }` を追加する
  - `use` 文は不要（`std::path::Path` は絶対パス表記で `use` なし使用、`include_str!` はマクロのため）
  - `Path` は `std::path::Path::new(...)` と完全修飾で記述する（`use std::path::Path;` は不要）
- [x] `favnir4_showcase_structure_exists` テストを実装する
  - `../infra/e2e-demo/favnir4-showcase/pipeline.fav` の存在確認
  - `../infra/e2e-demo/favnir4-showcase/fav.toml` の存在確認
  - `../infra/e2e-demo/favnir4-showcase/contract.fav` の存在確認
  - `../infra/e2e-demo/favnir4-showcase/README.md` の存在確認
- [x] `favnir4_showcase_contract_valid` テストを実装する
  - `include_str!("../../infra/e2e-demo/favnir4-showcase/contract.fav")` に `"Favnir4ShowcaseContract"` が含まれることを確認
  - `include_str!("../../infra/e2e-demo/favnir4-showcase/fav.toml")` に `"[quality]"`・`"[contract]"`・`"[observe]"` が含まれることを確認

## T3: テスト通過確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,911 tests, 0 failures（+2）であることを確認する

## T4: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v84.1.0 エントリを追加する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
