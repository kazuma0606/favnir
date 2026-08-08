# Tasks — v56.9.0 — 安定化・コードフリーズ（Language Power 2.0 前調整）

## ステータス: COMPLETE

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v56.1-v57.0.md` の v56.9.0 セクションを確認
- [x] ベーステスト数 3246（v56.8.0 完了時点の実績値）を確認
- [x] `fav/Cargo.toml` が `56.8.0` であることを確認（更新前）
- [x] `site/content/docs/language-power2-overview.mdx` が存在しないことを確認（新規作成対象）
- [x] `v56900_tests` が `driver.rs` に存在しないことを確認（新規追加対象）
- [x] `v56300_tests::cargo_toml_version_is_56_3_0` が `"56.8.0"` を期待していることを確認（更新対象）
- [x] `site/content/docs/language-power2-overview.mdx` が `"Language Power 2.0"` を含まないことを確認（テスト偽陽性防止・ファイル未作成のため自明）
- [x] `site/content/docs/language-power2-overview.mdx` が `"bounded-generics"` / `"row-polymorphism"` / `"effect-inference"` を含まないことを確認（テスト偽陽性防止）

---

## 実装タスク

- [x] T1: `fav/Cargo.toml` version を `56.9.0` に更新
- [x] T2: `site/content/docs/language-power2-overview.mdx` を新規作成
  - [x] ページ概要セクション（Language Power 2.0 の位置づけ）
  - [x] 機能一覧テーブル（v56.1〜v56.8、バージョン / 機能 / 主な追加内容）
  - [x] 境界付きジェネリクス（v56.1〜v56.2）サマリー + `bounded-generics.mdx` リンク
  - [x] 行多相レコード（v56.3）サマリー + `row-polymorphism.mdx` リンク
  - [x] エフェクト推論 LSP（v56.4）サマリー + `effect-inference.mdx` リンク
  - [x] OR パターン + ガード強化（v56.5）サマリー + `pattern-matching.mdx` リンク
  - [x] as-パターン（v56.6）サマリー
  - [x] モジュール名前空間（v56.7）サマリー + W038
  - [x] 次のステップ（v57.0 宣言への案内）
  - [x] `"Language Power 2.0"` と `"bounded-generics"` の両キーワードを含める（テスト対象）
- [x] T3: `fav/src/driver.rs` — `v56900_tests` モジュールを `v56800_tests` の直前に追加
  - [x] `cargo_toml_version_is_56_9_0`: `Cargo.toml` version が `"56.9.0"` である
  - [x] `language_power2_overview_exists`: `language-power2-overview.mdx` が `"Language Power 2.0"` / `"bounded-generics"` / `"row-polymorphism"` / `"effect-inference"` を含む（4 アサート）
- [x] T4: `fav/src/driver.rs` — バージョンチェックテスト更新
  - [x] `v56300_tests::cargo_toml_version_is_56_3_0` の期待値を `"56.8.0"` → `"56.9.0"` に更新
  - [x] failure メッセージも `"should be 56.9.0"` に更新
  - [x] モジュール名 `v56300_tests` / 関数名は変更しない（慣例）

---

## テスト・検証

- [x] T5: `cargo build` でコンパイルエラーがないことを確認
- [x] T6: `cargo test` 全通過（**3248 tests passed, 0 failed**）
  - [x] `v56900_tests::cargo_toml_version_is_56_9_0` ok
  - [x] `v56900_tests::language_power2_overview_exists` ok
  - [x] 既存 3246 件全通過
- [x] T7: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x] T8: `CHANGELOG.md` に v56.9.0 エントリを追加
- [x] T9: `versions/current.md` を v56.9.0 / 3248 tests に更新
- [x] T10: `versions/roadmap/roadmap-v56.1-v57.0.md` の v56.9.0 実績を COMPLETE に更新
  - [x] `3246 + 2 = 3248 tests passed, 0 failed（2026-07-26）` を追記
- [x] T11: `versions/roadmap/roadmap-v55.1-v60.0.md` の v56.9.0 実績欄も COMPLETE に更新

---

## 完了確認

- [x] `cargo_toml_version_is_56_9_0` pass
- [x] `language_power2_overview_exists` pass
- [x] **3248 tests passed, 0 failed**
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `language-power2-overview.mdx` が新規作成されている
- [x] `language-power2-overview.mdx` に v56.1〜v56.8 全機能の記載がある
- [x] `language-power2-overview.mdx` に `"Language Power 2.0"` と `"bounded-generics"` が含まれている
- [x] `v56300_tests::cargo_toml_version_is_56_3_0` の期待値が `"56.9.0"` になっている
- [x] `CHANGELOG.md` に v56.9.0 エントリが追加されている
- [x] `versions/current.md` が v56.9.0 / 3248 tests を反映
- [x] T10 / T11 のロードマップ更新（実績 COMPLETE）が完了している

---

## 実装メモ

- `language-power2-overview.mdx` のパスは `site/content/docs/` 直下（`language/` サブディレクトリではない）
- `include_str!` パス: `driver.rs`（`fav/src/driver.rs`）から `../../site/content/docs/language-power2-overview.mdx`
- `v56900_tests` は `use super::*` 不要（`include_str!` のみ使用）
- `language_power2_overview_exists` の 2 つのアサート: `"Language Power 2.0"`（タイトル確認）+ `"bounded-generics"`（機能リンク確認）
- v56.9.0 は新機能なし — clippy / lint のクリーンのみを技術的成果物として検証する
