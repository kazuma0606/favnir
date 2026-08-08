# v65.3.0 タスクリスト

Status: COMPLETE
Version: 65.3.0
Base tests: 3457
Target tests: 3459

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3457 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"65.0.0"` であることを確認（sub-version では変更しない）
- [x] `runes/autodiff/` ディレクトリが存在しないことを確認（新規作成対象）
- [x] `driver.rs` に `v65200_tests` が存在することを確認（`v65300_tests` の挿入位置）
- [x] `driver.rs` に `v65300_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v65200_tests` で 2 件 PASS することを確認（前バージョンが正常）

---

## T1: Rune ファイル作成

- [x] `runes/autodiff/` ディレクトリ作成
- [x] `runes/autodiff/rune.toml` 作成（`entry` / `effects = []` / `[dependencies]` 形式）
- [x] `runes/autodiff/autodiff.fav` 作成（以下の関数・型をすべて定義）
  - **計算テープ**
  - [x] `tape` — Tape 型を返すコンストラクタ（`Tape` 型を戻り型に使うことで `contains("Tape")` を満たす）
  - **微分演算**
  - [x] `grad` — スカラー値関数の勾配
  - [x] `jacobian` — ベクトル値関数のヤコビアン
  - [x] `hessian` — 二階微分行列（ヘッシアン）
  - **勾配追跡制御**
  - [x] `no_grad` — 勾配追跡を無効化するコンテキスト
  - **プリミティブ（チェーンルール対応）**
  - [x] `prim_exp`
  - [x] `prim_log`
  - [x] `prim_sin`
  - [x] `prim_cos`
  - [x] `prim_tanh`
  - [x] `relu`
- [x] `autodiff.fav` 内に `let ` が含まれないことを確認
- [x] `autodiff.fav` 内に `bind.*=`（`<-` でない bind）が含まれないことを確認
- [x] `autodiff.fav` 内に `Float.from_int` / `Float.sqrt` が含まれないことを確認

---

## T2: `driver.rs` — `v65300_tests` 追加

- [x] `// -- v65200_tests (v65.2.0)` コメントの直前に `v65300_tests` を挿入
  - [x] `autodiff_rune_grad_scalar` — `fn grad(` / `fn jacobian(` / `fn hessian(` を含む
  - [x] `autodiff_rune_chain_rule` — `Tape` / `fn no_grad(` / `fn relu(` を含む
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v65300_tests` で 2 件 PASS
  - [x] `autodiff_rune_grad_scalar` PASS
  - [x] `autodiff_rune_chain_rule` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3459 tests passed, 0 failed を確認

---

## T4: ドキュメント・ステータス更新

- [x] `versions/roadmap/roadmap-v65.1-v66.0.md` の v65.3.0 行を「完了」に更新
- [x] `versions/current.md` の「進行中バージョン」を v65.3.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（全チェックボックスを `[x]` に）

> **CHANGELOG 方針**: v65.1〜v65.9 では CHANGELOG.md を更新しない。v66.0.0 宣言時に一括追記する。
> **MDX 方針**: `site/` の MDX ドキュメントは v65.9.0 安定化時に一括作成するため今バージョンは省略。

---

## コードレビュー対応

実装は spec.md の通り。注意点:
- `bind` は一切使用しない（全関数がスタブで中間値不要）
- `Float.from_int` / `Float.sqrt` は使用しない
- `contains("fn relu(")` 形式（`contains("relu")` より厳密）で検証
- `+/-/*/` 演算子のチェーンルールは組み込み演算子と重複のため対象外（非スコープ）
