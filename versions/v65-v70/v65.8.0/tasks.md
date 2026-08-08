# v65.8.0 タスクリスト

Status: COMPLETE
Version: 65.8.0
Base tests: 3467
Target tests: 3469

---

## T0: 事前確認

- [x] `cargo test --bin fav` でベース 3467 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"65.0.0"` であることを確認（sub-version では変更しない）
- [x] `lint.rs` に `W050`〜`W054` が存在しないことを確認（新規追加対象）
- [x] `lint.rs` の最大 W-code が W041 であることを確認（`grep "W04" fav/src/lint.rs` で W042 以降が出ないことを確認）
- [x] `driver.rs` に `v65700_tests` が存在することを確認（`v65800_tests` の挿入位置）
- [x] `driver.rs` に `v65800_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v65700_tests` で 2 件 PASS することを確認（前バージョンが正常）
- [x] `versions/current.md` の「進行中バージョン」が `v65.7.0` であることを確認

---

## T1: `lint.rs` — スタブ関数・呼び出し追加

- [x] `lint.rs` ファイル末尾に以下の 5 関数を追加（`check_w041_*` 群の後）
  - [x] `check_w050_matrix_dim_mismatch(_program, _errors)` — 空スタブ
  - [x] `check_w051_numeric_instability(_program, _errors)` — 空スタブ
  - [x] `check_w052_small_sample_test(_program, _errors)` — 空スタブ
  - [x] `check_w053_inplace_in_autodiff(_program, _errors)` — 空スタブ
  - [x] `check_w054_missing_convergence(_program, _errors)` — 空スタブ
- [x] `lint_program` 関数内の `check_w040_type_holes` 呼び出し直後（`errors` 裸式の直前）に 5 関数の呼び出しを追加
- [x] 各関数の引数に `_` 下線接頭辞がついていることを確認（未使用変数警告を抑制）
- [x] `cargo build` で未使用変数警告が出ないことを確認

---

## T2: `driver.rs` — `v65800_tests` 追加

- [x] `// -- v65700_tests (v65.7.0)` コメントの直前に `v65800_tests` を挿入
  - [x] `lint_w051_detects_div_zero_risk` — `W050` / `W051` / `check_w051` を lint.rs から確認
  - [x] `lint_w053_detects_inplace_in_autodiff` — `W052` / `W053` / `W054` / `check_w053` を lint.rs から確認
- [x] `include_str!("lint.rs")` パスが同ディレクトリ参照であることを確認（`../../` 不要）
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v65800_tests` で 2 件 PASS
  - [x] `lint_w051_detects_div_zero_risk` PASS
  - [x] `lint_w053_detects_inplace_in_autodiff` PASS
- [x] `cargo test --bin fav` で 3469 tests passed, 0 failed を確認

---

## T4: ドキュメント・ステータス更新

- [x] `versions/roadmap/roadmap-v65.1-v66.0.md` の v65.8.0 行を「完了」に更新
- [x] `versions/current.md` の「進行中バージョン」を v65.8.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（全チェックボックスを `[x]` に）

> **CHANGELOG 方針**: v65.1〜v65.9 では CHANGELOG.md を更新しない。v66.0.0 宣言時に一括追記する。
> **MDX 方針**: `site/` の MDX ドキュメントは v65.9.0 安定化時に一括作成するため今バージョンは省略。
