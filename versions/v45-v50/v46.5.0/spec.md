# Spec: v46.5.0 — LSP クイックフィックス強化

Date: 2026-07-17
Status: 計画中

---

## 概要

E0102（未定義変数）の did-you-mean 提案と E0101（引数数不一致）の診断アクションを
LSP `textDocument/codeAction` として提供する。

> **注意**: ロードマップは「E0007（引数数不一致）」と記載しているが、
> 実際の Rust checker のエラーコードは以下の通り:
> - 未定義変数: **E0102** (`checker.rs` line 4602)
> - 引数数不一致: **E0101** (`checker.rs` line 4758)
> ロードマップの E0007 / E0008 は checker.fav (Favnir セルフホスト側) の番号であり、
> Rust 実装とは異なる。本バージョンは Rust checker の実コード（E0102 / E0101）を対象とする。

---

## スコープ

### CA-4: `check_did_you_mean_fix`（E0102 — 未定義変数）

`fav/src/lsp/code_action.rs` に追加。

- `doc.errors` から `code == "E0102"` かつ `span.line - 1 == range.start.line` のエラーを検索
- `TypeError.hints` の各文字列から backtick 間の文字列を抽出（`"did you mean \`X\`?"` → `X`）
  - checker が発行する hints フォーマット: `"did you mean \`X\`?"` (小文字 `did`)
  - 生成する CodeAction タイトル: `"Did you mean \`X\`?"` (大文字 `Did` — UI 表示用)
- 候補ごとに `TextEdit` を生成：
  - range: start `(span.line-1, span.col-1)`, end `(span.line-1, span.col-1 + (span.end - span.start))`
  - new_text: 候補名 `X`
- `CodeAction { title: "Did you mean \`X\`?", kind: Some("quickfix"), edit: Some(...) }` を返す

### CA-5: `check_arg_count_fix`（E0101 — 引数数不一致）

- `doc.errors` から `code == "E0101"` かつ `span.line - 1 == range.start.line` のエラーを検索
- E0101 は型不一致にも使われるため、`message.contains("argument(s)")` でさらに絞り込む
- 診断情報アクション（edit: None）: 将来の TextEdit 対応を見越して `uri` も受け取るが現在は非使用
  - **note**: `edit: None` は意図的。引数の自動補完は v46.9.0 以降の課題とし、本バージョンは
    診断メッセージの表示（quickfix kind による視認性向上）のみを目的とする
- `CodeAction { title: format!("Fix: {}", err.message), kind: Some("quickfix"), edit: None }` を返す

### `handle_code_action` 更新

既存の CA-1〜CA-3 に加え、CA-4・CA-5 の呼び出しを追加。
CA-4/CA-5 は `Vec<CodeAction>` を返す（複数候補対応）。

ドキュメント（`site/content/docs/tools/lsp-quickfix.mdx`）は **v46.9.0 で一括追加**。

---

## 実装ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/lsp/code_action.rs` | `check_did_you_mean_fix` + `check_arg_count_fix` 追加、`handle_code_action` 更新 |
| `fav/src/driver.rs` | `v465000_tests` 追加（2 件） |
| `fav/Cargo.toml` | version → `46.5.0` |
| `CHANGELOG.md` | v46.5.0 エントリ追加 |
| `versions/current.md` | v46.5.0（3003 tests）に更新 |

---

## テスト

| テスト名 | 内容 |
|---|---|
| `lsp_quick_fix_undefined_var` | `"fn main() -> Int { totally_undefined_xyz }"` ソースで E0102 が発行され、`handle_code_action` が `kind == Some("quickfix")` の CodeAction を返すことを確認。hints が空の場合はアクションが 0 件であることを `assert!` で確認（パニックしないことの保証） |
| `lsp_quick_fix_arg_count` | `"fn add(a: Int, b: Int) -> Int { a + b }\nfn main() -> Int { add(1) }"` ソースで E0101 が発行され、`handle_code_action` が `"Fix: expected"` を title に含む `CodeAction` を返すことを確認 |

**テスト数**: 3001 + 2 = **3003 tests**

---

## 完了条件

- `cargo test` 3003 passed, 0 failed
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version = `46.5.0`
- `CHANGELOG.md` に v46.5.0 エントリ
- `versions/current.md` を v46.5.0（3003 tests）に更新
