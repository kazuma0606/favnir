# v65.1.0 タスクリスト

Status: COMPLETE
Version: 65.1.0
Base tests: 3453
Target tests: 3455

---

## T0: 事前確認

- [x] `cargo test -j 8 -- --test-threads=8` でベース 3453 tests passed, 0 failed を確認
- [x] `fav/Cargo.toml` の version が `"65.0.0"` であることを確認
- [x] `runes/linalg/` ディレクトリが存在しないことを確認（新規作成対象）
- [x] `driver.rs` に `v65000_tests` が存在することを確認（`v65100_tests` の挿入位置）
- [x] `driver.rs` に `v65100_tests` が存在しないことを確認（新規追加）
- [x] `fav/src/lint.rs` の最大 W コードが W041 以下であることを確認

---

## T1: Rune ファイル作成

- [x] `runes/linalg/` ディレクトリ作成
- [x] `runes/linalg/rune.toml` 作成（`[rune]` + `[exports]` セクション）
- [x] `runes/linalg/linalg.fav` 作成（以下の関数をすべて定義）
  - [x] `dot` — 内積
  - [x] `matmul` — 行列積
  - [x] `transpose` — 転置
  - [x] `inverse` — 逆行列
  - [x] `norm` — L2 ノルム
  - [x] `diag` — 対角成分
  - [x] `trace` — トレース
  - [x] `svd` — 特異値分解
  - [x] `lu` — LU 分解
  - [x] `qr` — QR 分解
  - [x] `cholesky` — Cholesky 分解
  - [x] `eig` — 固有値・固有ベクトル
  - [x] `eigh` — 対称行列固有値分解
  - [x] `cosine_similarity` — コサイン類似度
  - [x] `euclidean_distance` — ユークリッド距離
  - [x] `manhattan_distance` — マンハッタン距離
- [x] `linalg.fav` 内に `let ` が含まれないことを確認（`bind` 構文のみ使用）

---

## T2: `driver.rs` — `v65100_tests` 追加

- [x] `// -- v65000_tests (v65.0.0)` コメントの直前に `v65100_tests` を挿入
  - [x] `linalg_rune_matrix_ops` — `matmul` / `dot` / `transpose` / `cosine_similarity` を含む
  - [x] `linalg_rune_svd_decomposition` — `svd` / `eig` / `cholesky` / `euclidean_distance` を含む
- [x] `use super::*` は不要（`include_str!` のみ使用）
- [x] `cargo build` でエラーなし（warning 1件のみ、既存の private_interfaces warn）

---

## T3: ビルド・テスト

- [x] `cargo test --bin fav v65100_tests` で 2 件 PASS
  - [x] `linalg_rune_matrix_ops` PASS
  - [x] `linalg_rune_svd_decomposition` PASS
- [x] `cargo test -j 8 -- --test-threads=8` で 3455 tests passed, 0 failed を確認

---

## T4: ドキュメント・ステータス更新

- [x] `versions/roadmap/roadmap-v65.1-v66.0.md` の v65.1.0 行を「完了」に更新
- [x] `versions/current.md` の「進行中バージョン」を v65.1.0 に更新
- [x] 本 `tasks.md` を COMPLETE に更新（全チェックボックスを `[x]` に）

> **CHANGELOG 方針**: v65.1〜v65.9 では CHANGELOG.md を更新しない。v66.0.0 宣言時に一括追記する。

---

## コードレビュー対応

- [HIGH] `bind x = expr` → `bind x <- expr` に修正（Favnir の正式 bind 構文）
- [HIGH] `Float.sqrt` → `Math.sqrt` に修正（VM に Float.sqrt は存在しない）
- [HIGH] `List.zip_with(xs, ys, f)` → `List.zip_with(f, xs, ys)` に修正（VM の引数順）
- [MED]  `rune.toml` の `[exports] main =` → `entry =` + `effects = []` + `[dependencies]` に統一（既存 Rune 形式に合わせる）
- [LOW]  `contains("eig")` → `contains("fn eig(")` に変更（eigh への誤ヒットを防ぐ）
