# Tasks — v55.9.0 — 安定化・コードフリーズ（Streaming Native 2.0 前調整）

## ステータス: COMPLETE（2026-07-24）

---

## 事前確認（T0）

- [x] `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.9.0 セクションを確認
- [x] ベーステスト数 3222（v55.8.0 完了時点の実績値）を確認
- [x] `fav/Cargo.toml` が現在 `55.8.0` であることを確認（更新前）
- [x] `site/content/docs/streaming-native2-overview.mdx` が存在しないことを確認（新規追加）
- [x] `site/content/docs/streaming-native2-overview.mdx` の配置先が `docs/` 直下（`runtime/` サブディレクトリではない）であることを確認
- [x] `fav/src/driver.rs` の `v55800_tests` モジュール位置を確認（直前に `v55900_tests` を挿入）
- [x] `include_str!` パス確認: `../Cargo.toml`（`fav/Cargo.toml`）/ `../../site/content/docs/streaming-native2-overview.mdx`
- [x] CI self-lint 対象（`self/compiler.fav` / `self/checker.fav`）に今回の変更が影響しないことを確認（MDX / driver.rs テスト追加・Cargo.toml バージョン更新のみ）

---

## 実装タスク

- [x] T1: `fav/Cargo.toml` version を `55.9.0` に更新
- [x] T2: `site/content/docs/streaming-native2-overview.mdx` を新規作成
  - [x] ロードマップ記載の宣言文（引用ブロック）を含む
  - [x] v55.1〜v55.8 の機能一覧テーブルを含む
  - [x] fav.toml `[stream]` 設定例を含む
  - [x] Stateful stage コード例を含む
  - [x] CEP stage コード例を含む（`fn is_start` / `fn is_end` を事前定義）
  - [x] チェックポイント CLI 操作例を含む
  - [x] 詳細ドキュメントリンクを含む
  - [x] `"Streaming Native 2.0"` / `"Exactly-once"` / `"CEP"` / `"Stateful"` キーワードを含む
- [x] T3: `fav/src/driver.rs` に `v55900_tests` モジュールを追加（`v55800_tests` の直前）
  - [x] `cargo_toml_version_is_55_9_0`（Cargo.toml バージョン検証）
  - [x] `streaming_native2_overview_exists`（MDX キーワード 4 件検証）

---

## テスト・検証

- [x] T4: `cargo build` でコンパイルエラーがないことを確認
- [x] T5: `cargo test` 全通過（3224 tests passed, 0 failed）
- [x] T6: `cargo clippy -- -D warnings` クリーン

---

## ポスト処理

- [x] T7: `CHANGELOG.md` に v55.9.0 エントリ追加
- [x] T8: `versions/current.md` を v55.9.0 / 3224 tests に更新
- [x] T9: `versions/roadmap/roadmap-v55.1-v56.0.md` の v55.9.0 実績を COMPLETE に更新
- [x] T10: `versions/roadmap/roadmap-v55.1-v60.0.md` の v55.9.0 実績欄も COMPLETE に更新

---

## コードレビュー

- [x] コードレビュー実施（`/review code`）
- [x] 指摘事項対応
  - [LOW] `streaming-native2-overview.mdx` の CEP 例で `is_start` / `is_end` が未定義 → `fn is_start` / `fn is_end` を事前定義して修正

---

## 完了確認

- [x] `cargo_toml_version_is_55_9_0` pass
- [x] `streaming_native2_overview_exists` pass
- [x] 3224 tests passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `streaming-native2-overview.mdx` に宣言文・機能一覧・CEP 述語定義付きコード例を含む
- [x] `CHANGELOG.md` に v55.9.0 エントリが追加されている
- [x] `versions/current.md` が v55.9.0 / 3224 tests を反映
- [x] T9 / T10 のロードマップ更新が完了している
