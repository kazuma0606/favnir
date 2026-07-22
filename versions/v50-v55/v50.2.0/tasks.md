# Tasks: v50.2.0 — エラー診断統一 Phase 2（JSON / LSP / CLI 出力の一貫化）

Status: COMPLETE
Date: 2026-07-18

---

## T0 — 事前確認

- [x] `cargo test` 3093 passed, 0 failed を確認（ベース確認）
- [x] `lsp/protocol.rs` の `Diagnostic` struct に `data` フィールドがないことを確認
- [x] `lsp/diagnostics.rs` の `errors_to_diagnostics` が suggestion を設定していないことを確認
- [x] `cargo clippy -- -D warnings` クリーンであることを確認（ベース）

## T1 — `lsp/protocol.rs` — `DiagnosticData` + `Diagnostic.data` 追加

- [x] `DiagnosticData { suggestion: String }` struct を `Diagnostic` 直前に追加
  - [x] `#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]` 付与
  - [x] `pub` 可視性
- [x] `Diagnostic` struct に `data: Option<DiagnosticData>` フィールドを追加
  - [x] `#[serde(skip_serializing_if = "Option::is_none")]` 付与
  - [x] `/// v50.2.0:` コメント付与

## T2 — `lsp/diagnostics.rs` — `errors_to_diagnostics` 更新

- [x] `use crate::error_catalog;` を追加
- [x] `use crate::lsp::protocol::DiagnosticData;` を追加
- [x] `errors_to_diagnostics` の各 error に対して:
  - [x] `error_catalog::lookup(err.code).and_then(|e| e.suggestion).unwrap_or("")` で suggestion 取得
  - [x] suggestion が空でなければ `Some(DiagnosticData { suggestion })` を設定
  - [x] `Diagnostic { ..., data }` に変更
- [x] `lsp/diagnostics.rs` の既存テストはフィールドアクセス形式のため修正不要（data フィールド追加の影響なし）

## T3 — `v502000_tests` モジュール追加

- [x] `v502000_tests` モジュールを `driver.rs` の `v501000_tests` 直前に追加（機能 2 件 + バージョン確認 1 件 = 合計 3テスト）
  - [x] `cargo_toml_version_is_50_2_0`: version = "50.2.0" を assert
  - [x] `check_json_includes_suggestion`: E0213/E0380 の suggestion が非空であることを assert
  - [x] `lsp_diagnostic_includes_suggestion`: LSP 出力に `"suggestion"` キーが含まれることを assert

## T4 — バージョン更新・完了

- [x] `fav/Cargo.toml` version → `"50.2.0"`
- [x] `v501000_tests::cargo_toml_version_is_50_1_0` を削除（Cargo.toml が 50.2.0 になると 50.1.0 アサーションが FAIL するため）
- [x] `cargo test` 3095 passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `CHANGELOG.md` に v50.2.0 エントリ追加
- [x] `versions/current.md` を v50.2.0（3095 tests）に更新
- [x] `versions/roadmap/roadmap-v50.1-v51.0.md` の v50.2.0 実績を記入
- [x] tasks.md を COMPLETE に更新（T0〜T4 全 `[x]`）
