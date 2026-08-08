# v65.4.0 タスクリスト

Status: COMPLETE
Version: 65.4.0
Base tests: 3459
Target tests: 3461

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3459 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"65.0.0"` であることを確認（sub-version では変更しない）
- [x] `runes/optim/` ディレクトリが存在しないことを確認（新規作成対象）
- [x] `driver.rs` に `v65300_tests` が存在することを確認（`v65400_tests` の挿入位置）
- [x] `driver.rs` に `v65400_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v65300_tests` で 2 件 PASS することを確認（前バージョンが正常）
- [x] `versions/current.md` の「進行中バージョン」が `v65.3.0` であることを確認

---

## T1: Rune ファイル作成

- [x] `runes/optim/` ディレクトリ作成
- [x] `runes/optim/rune.toml` 作成（`entry` / `effects = []` / `[dependencies]` 形式）
- [x] `runes/optim/optim.fav` 作成（以下の関数をすべて定義）
  - **ファーストオーダー最適化**
  - [x] `sgd` — 確率的勾配降下法
  - [x] `adam` — Adam オプティマイザ
  - [x] `adamw` — AdamW（weight decay 付き）
  - [x] `adagrad` — Adagrad
  - [x] `rmsprop` — RMSprop
  - **二次法**
  - [x] `l_bfgs` — L-BFGS 準ニュートン法
  - [x] `conjugate_gradient` — 共役勾配法
  - **汎用最適化**
  - [x] `minimize` — 損失関数を最小化する汎用エントリポイント
  - **学習率スケジューラ**
  - [x] `step_decay` — ステップ減衰スケジューラ
  - [x] `cosine_annealing` — コサインアニーリング
  - [x] `warmup_cosine` — ウォームアップ付きコサインスケジューラ
  - **制約付き最適化**
  - [x] `minimize_constrained` — 等式・不等式制約付き最適化
- [x] `optim.fav` 内に `let ` が含まれないことを確認
- [x] `optim.fav` 内に `bind.*=`（`<-` でない bind）が含まれないことを確認
- [x] `optim.fav` 内に `Float.from_int` / `Float.sqrt` が含まれないことを確認

---

## T2: `driver.rs` — `v65400_tests` 追加

- [x] `// -- v65300_tests (v65.3.0)` コメントの直前に `v65400_tests` を挿入
  - [x] `optim_rune_adam_converges` — `fn adam(` / `fn sgd(` / `fn minimize(` を含む
  - [x] `optim_rune_minimize_quadratic` — `fn l_bfgs(` / `fn conjugate_gradient(` / `fn step_decay(` を含む
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v65400_tests` で 2 件 PASS
  - [x] `optim_rune_adam_converges` PASS
  - [x] `optim_rune_minimize_quadratic` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3461 tests passed, 0 failed を確認

---

## T4: ドキュメント・ステータス更新

- [x] `versions/roadmap/roadmap-v65.1-v66.0.md` の v65.4.0 行を「完了」に更新
- [x] `versions/current.md` の「進行中バージョン」を v65.4.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（全チェックボックスを `[x]` に）

> **CHANGELOG 方針**: v65.1〜v65.9 では CHANGELOG.md を更新しない。v66.0.0 宣言時に一括追記する。
> **MDX 方針**: `site/` の MDX ドキュメントは v65.9.0 安定化時に一括作成するため今バージョンは省略。

---

## コードレビュー対応

実装は spec.md の通り。注意点:
- `bind` は一切使用しない（スタブで中間値不要）
- `Float.from_int` / `Float.sqrt` は使用しない
- スタブ本体でレコードコンストラクタ（`Optimizer { ... }`）を使用 — 将来の型登録フェーズで書き換え対象
- `fn minimize(` が `fn minimize_constrained(` に誤マッチしないことを確認（括弧で区別）
