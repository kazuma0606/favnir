# v60.3.0 Spec — LSP Code Action（Quick Fix / Rename Symbol）

Date: 2026-07-30
Status: 計画中

---

## 概要

LSP `textDocument/codeAction`（E0102 Quick Fix）と `textDocument/rename`（変数リネーム）の
専用テストを追加する。

---

## ロードマップとの差分

| ロードマップ記述 | 実際のスコープ | 理由 |
|---|---|---|
| `E0001` typo Quick Fix | `E0102`（`check_did_you_mean_fix`） | E0001 は存在しない；未定義変数は E0102 |
| `W001` 未使用 bind Quick Fix | **v60.3.0 スコープ外** | `DocumentStore.CheckedDoc` に `lint_errors` フィールドがなく LSP 側で L002 を取得できないため |
| `code_action.rs` 新規作成 | 実装済みにつきテストのみ | v46.5.0 時点で実装完了 |

**W001 LSP Quick Fix について:**
`DocumentStore::open_or_change` は `Checker::check_with_self` のみ実行し、lint は実行しない。
L002 を LSP Code Action で提供するには `CheckedDoc` に `lint_errors` フィールドを追加する必要があり、
document_store.rs の変更が必要となる。これは v60.x 以降の課題とする。
v60.2.0 の CLI 版（`cmd_check_fix_src`）では L002 を処理済みだが、LSP 版は未対応。

**テスト名について:**
ロードマップ記載のテスト名は `lsp_code_action_e0001_quickfix` だが、
実際にテストするエラーコードは E0102（`check_did_you_mean_fix` が `err.code == "E0102"` でフィルタリング）。
ロードマップとの一貫性を保つためテスト名は変更せず、本文書に注記する。

---

## 既存実装の状況

| ファイル | 状態 | 備考 |
|---|---|---|
| `fav/src/lsp/code_action.rs` | **実装済み** | CA-4: `check_did_you_mean_fix`（E0102）・CA-5: `check_arg_count_fix`（E0101）含む |
| `fav/src/lsp/rename.rs` | **実装済み** | `handle_rename`（変数・関数・キーワード判定含む）|
| `fav/src/lsp/mod.rs` | **登録済み** | `handle_code_action` / `handle_rename` を LSP サーバーに接続済み |

ロードマップには「`code_action.rs` を新規作成」と記載されているが、
すでに実装済みのため v60.3.0 では**テストモジュールの追加のみ**を行う。
（v60.1.0 で `format_diagnostic` が既存だったのと同じ状況）

---

## v215000_tests との差分

`v215000_tests` はすでに以下をカバーしている:

| テスト名 | 内容 |
|---|---|
| `code_action_add_missing_import` | CA-1: use 自動追加 |
| `code_action_convert_to_fstring` | CA-2: f-string 変換 |
| `code_action_inline_binding` | CA-3: bind インライン化 |
| `code_action_no_actions_on_plain_code` | Code Action なしケース |
| `rename_symbol_returns_workspace_edit` | 関数名リネーム |
| `rename_keyword_returns_none` | キーワードリネーム禁止 |
| `rename_unknown_doc_returns_none` | ドキュメント不在 |

**未カバー（v60.3.0 で追加）:**

- `lsp_code_action_e0001_quickfix` — CA-4: E0102 did-you-mean Quick Fix の専用テスト
- `lsp_rename_variable` — bind 変数（関数名ではなく変数）のリネームテスト

---

## テスト実装方針

### `lsp_code_action_e0001_quickfix`（実際は E0102 Quick Fix）

> テスト名はロードマップ準拠。実際に検証するのは E0102 の did-you-mean Quick Fix。

```
ソース: "fn go(userId: Int) -> Int { user_id }"
         └─ userId を定義し user_id を参照 → E0102 + "did you mean `userId`?" hint
```

1. `DocumentStore::open_or_change` で上記ソースを登録（内部で checker が走り errors に E0102 が積まれる）
2. `handle_code_action(store, uri, range(0, 28, 0, 35))` — line 0、`user_id` のカラム範囲
3. 返却 `Vec<CodeAction>` の中に `title.contains("Did you mean")` のアクションがあることを assert
4. その action の `kind` が `"quickfix"` であることを assert

### `lsp_rename_variable`

```
ソース: "fn go(userId: Int) -> Int { userId }"
         └─ userId（引数）を rename → WorkspaceEdit が返る
```

1. `DocumentStore::open_or_change` で上記ソースを登録
2. `handle_rename(store, uri, pos(0, 6), "newName")` — line 0 の `userId`（char 6）
3. `WorkspaceEdit` が `Some(...)` であることを assert
4. edits の `new_text` がすべて `"newName"` であることを assert

---

## 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `v60300_tests` モジュール追加（`v60200_tests` の直前に挿入） |

`code_action.rs` / `rename.rs` への変更はなし。

---

## 完了条件

- `cargo test` 全通過（3334 → **3336** tests passed, 0 failed）
- 以下の 2 テストが pass:
  - `v60300_tests::lsp_code_action_e0001_quickfix`
  - `v60300_tests::lsp_rename_variable`

---

## 参照

- ロードマップ: `versions/roadmap/roadmap-v60.1-v61.0.md`（v60.3.0 セクション）
- 既存実装: `fav/src/lsp/code_action.rs`（L126-168: `check_did_you_mean_fix`）
- 既存実装: `fav/src/lsp/rename.rs`（L15-46: `handle_rename`）
- 次バージョン: v60.4.0 — LSP Diagnostic 完全統合
