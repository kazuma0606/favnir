# v66.1.0 タスクリスト

Status: COMPLETE
Version: 66.1.0
Base tests: 3475
Target tests: 3477
Actual tests: 3477

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3475 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"66.0.0"` であることを確認（sub-version では変更しない）
- [x] `runes/vec/` ディレクトリが存在しないことを確認（新規作成対象）
- [x] `driver.rs` に `v66000_tests` が存在することを確認（`v66100_tests` の挿入位置）
- [x] `driver.rs` に `v66100_tests` が存在しないことを確認（新規追加）
- [x] `cargo test --bin fav v66000_tests` で 4 件 PASS することを確認（前バージョンが正常）
- [x] `versions/current.md` の「最新安定版」が `v66.0.0` であることを確認

---

## T1: Rune ファイル作成

- [x] `runes/vec/` ディレクトリ作成
- [x] `runes/vec/rune.toml` 作成（`entry` / `effects = []` / `[dependencies]` 形式）
- [x] `runes/vec/vec.fav` 作成（以下の全 7 関数を定義）
  - **基本ベクトル演算**
  - [x] `normalize` — L2 正規化（入力をそのまま返すスタブ）
  - [x] `dot` — 内積（`0.0` を返すスタブ）
  - [x] `cosine_similarity` — コサイン類似度（`0.0` を返すスタブ）
  - [x] `euclidean_distance` — ユークリッド距離（`0.0` を返すスタブ）
  - **バッチ処理**
  - [x] `batch_embed` — バッチ埋め込み（`[]` を返すスタブ）
  - [x] `batch_cosine_matrix` — コサイン類似度行列（`[]` を返すスタブ）
  - **次元変換**
  - [x] `project` — 次元投影（`[]` を返すスタブ、コメントに `VecDimProjection` を含む）
- [x] `vec.fav` 内に `let ` が含まれないことを確認
- [x] `vec.fav` 内に `bind.*=`（`<-` でない bind）が含まれないことを確認
- [x] `vec.fav` 内に `Float.from_int` / `Float.sqrt` が含まれないことを確認
- [x] `grep -c 'public fn ' vec.fav` で 7 が出ることを確認

---

## T2: `driver.rs` — `v66100_tests` 追加

- [x] `// -- v66000_tests (v66.0.0)` コメントの直前に `v66100_tests` を挿入
  - [x] `vec_stage_dim_type_check` — `fn normalize(` / `fn dot(` / `fn cosine_similarity(` / `fn euclidean_distance(` を含む
  - [x] `vec_stage_batch_and_project` — `fn batch_embed(` / `fn batch_cosine_matrix(` / `fn project(` / `VecDimProjection` を含む
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v66100_tests` で 2 件 PASS
  - [x] `vec_stage_dim_type_check` PASS
  - [x] `vec_stage_batch_and_project` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3477 tests passed, 0 failed を確認

---

## T4: ドキュメント・ステータス更新

- [x] `versions/roadmap/roadmap-v66.1-v67.0.md` の v66.1.0 行を「完了」に更新
- [x] `versions/current.md` の「進行中バージョン」を v66.1.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（全チェックボックスを `[x]` に）

> **CHANGELOG 方針**: v66.1〜v66.9 では CHANGELOG.md を更新しない。v67.0.0 宣言時に一括追記する。
> **MDX 方針**: `site/` の MDX ドキュメントは v66.9.0 安定化時に一括作成するため今バージョンは省略。

---

## コードレビュー指摘と対応

<!-- 実装完了後に追記 -->
