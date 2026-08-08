# v65.9.0 タスクリスト

Status: COMPLETE
Version: 65.9.0
Base tests: 3469
Target tests: 3471

---

## T0: 事前確認

- [x] `cargo test --bin fav` でベース 3469 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"65.0.0"` であることを確認（sub-version では変更しない）
- [x] `driver.rs` に `v65800_tests` が存在することを確認（`v65900_tests` の挿入位置）
- [x] `driver.rs` に `v65900_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v65800_tests` で 2 件 PASS することを確認（前バージョンが正常、`| grep "2 passed"` で件数も確認）
- [x] 7 Rune ファイルの存在確認（各パスが実際に存在すること）
  - [x] `runes/linalg/linalg.fav`
  - [x] `runes/stats/stats.fav`
  - [x] `runes/autodiff/autodiff.fav`
  - [x] `runes/optim/optim.fav`
  - [x] `runes/numeric/numeric.fav`
  - [x] `runes/timeseries/timeseries.fav`
  - [x] `runes/ml/ml.fav`
- [x] `versions/current.md` の「進行中バージョン」が `v65.8.0` であることを確認

---

## T1: MDX ドキュメント作成

- [x] `site/content/docs/runes/math-runes-overview.mdx` を新規作成
  - [x] ファイルが空でないことを確認
  - [x] `"Rune.linalg"` 文字列を含むことを確認
  - [x] v65.1〜v65.8 の 7 Rune（linalg / stats / autodiff / optim / numeric / timeseries / ml）を一覧で記載
  - [x] W050〜W054 Math Lint Rules の説明を含む
  - [x] MDX 構文が有効（コードブロック内に行頭 import/export を置かない）

---

## T2: `driver.rs` — `v65900_tests` 追加

- [x] `// -- v65800_tests (v65.8.0)` コメントの直前に `v65900_tests` を挿入
  - [x] `math_foundation_all_runes_stable` — 7 Rune ファイルすべてが存在し空でないことを確認
  - [x] `math_docs_complete` — `math-runes-overview.mdx` が存在し `"Rune.linalg"` を含む
- [x] `include_str!` パスがすべて `../../runes/...` または `../../site/...` の形式であることを確認
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v65900_tests` で 2 件 PASS
  - [x] `math_foundation_all_runes_stable` PASS
  - [x] `math_docs_complete` PASS
- [x] `cargo test --bin fav` で 3471 tests passed, 0 failed を確認

---

## T4: ドキュメント・ステータス更新

- [x] `versions/roadmap/roadmap-v65.1-v66.0.md` のサマリーテーブル（v65.9.0 行、`未着手` → `完了`）を更新
- [x] `versions/current.md` の「進行中バージョン」を v65.9.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（全チェックボックスを `[x]` に）

> **CHANGELOG 方針**: v65.1〜v65.9 では CHANGELOG.md を更新しない。v66.0.0 宣言時に一括追記する。
> **Cargo.toml 方針**: バージョン更新は v66.0.0 宣言時に `"66.0.0"` に更新する。
