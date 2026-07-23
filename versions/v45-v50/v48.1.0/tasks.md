# Tasks: v48.1.0 — import 構文刷新 AST + parser（パッケージ）

Status: COMPLETE
Date: 2026-07-18

---

## T0 — 事前確認

- [x] `cargo test` 3045 passed, 0 failed を確認
- [x] `ast.rs` に `ImportKind` が存在しないことを確認（新規追加対象）
- [x] `import kafka`（引用符なし bare ident）が現状 ParseError になることを確認

## T1 — AST + Parser 変更

- [x] `ast.rs`: `ImportKind` enum 追加（`Package` / `Local` / `Legacy`）
- [x] `ast.rs`: `ImportDecl` バリアントに `kind: ImportKind` フィールド追加
- [x] `parser.rs`: bare Ident（スラッシュなし）ブランチを `ImportKind::Package` に変更
- [x] `parser.rs`: 文字列パス（`Str` ブランチ）に `kind: ImportKind::Legacy` を設定
- [x] `parser.rs`: `ImportDecl` 構築に `kind` フィールドを追加
- [x] `cargo build` でコンパイルエラー確認 → 全パターンマッチに `kind: _` 追記
  - [x] `checker.rs:1267` `process_imports` の `ImportDecl` パターンに `kind: _` 追記
  - [x] `parser.rs` 内 `parse_simple_import` テストは `..` 使用のため変更不要

## T2 — `driver.rs` テスト追加・バージョン更新・完了

- [x] `v481000_tests` モジュールを `v48000_tests` の直前に追加（2テスト）
  - [x] `import_package_parses`: `import kafka` → `ImportKind::Package`、`path == "kafka"`
  - [x] `import_package_with_alias`: `import postgres as db` → `ImportKind::Package`、`alias == Some("db")`
- [x] `v48000_tests::cargo_toml_version_is_48_0_0` をスタブ化（v48.1.0 への version bump で失敗するため）
- [x] `fav/Cargo.toml` version → `"48.1.0"`
- [x] `CHANGELOG.md` に v48.1.0 エントリ追加
- [x] `cargo test` 3047 passed, 0 failed（3045 + 2 件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `versions/current.md` を v48.1.0（3047 tests）に更新、進行中バージョンを `v48.2.0` に更新
- [x] `versions/roadmap/roadmap-v48.1-v49.0.md` の v48.1.0 完了条件テスト数（3047）を実績で確認
- [x] tasks.md を COMPLETE に更新（T0〜T2 全 `[x]`）

> **注記**: マスターロードマップ（`roadmap-v45.1-v50.0.md`）への反映は v49.0.0 マイルストーン宣言時に実施

---

## コードレビュー指摘と対応（spec-reviewer）

| 重大度 | 内容 | 対応 |
|---|---|---|
| [MED] | plan.md Step 2 の `kind` 擬似コードで `started_as_ident_no_slash` が未定義変数 | `let mut kind = ImportKind::Legacy;` を match 前に宣言し各ブランチで設定する具体的なコードに書き直し |
| [LOW] | `runes/kafka`（Ident+Slash）ケースの `kind = Legacy` が plan 本文に未記載 | Step 2 の実装コードに明示 |

## 実装時の追加発見

| 内容 | 対応 |
|---|---|
| `v48000_tests::cargo_toml_version_is_48_0_0` が version bump で失敗 | v47.0.0 / v46.0.0 と同様にスタブ化（`// Stubbed: version bumped to 48.1.0`） |
| コンパイルエラーは `checker.rs:1267` の 1 箇所のみ（`parse_simple_import` は `..` 使用で変更不要） | `kind: _` 追記で解決 |
