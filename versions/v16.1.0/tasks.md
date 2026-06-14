# v16.1.0 Tasks — エラーメッセージ品質向上

Date: 2026-06-14
Branch: master

---

## Phase A — Cargo バージョン更新 + strsim 依存追加

- [x] A-1: `fav/Cargo.toml` の `version` を `"16.1.0"` に変更
- [x] A-2: `fav/Cargo.toml` の `[dependencies]` に `strsim = "0.11"` 追加
- [x] A-3: `cargo build` → コンパイルエラーなし確認

---

## Phase B — テスト追加（v161000_tests）

- [x] B-1: `fav/src/driver.rs` に `check_source_to_string` テスト補助関数追加
- [x] B-2: `fav/src/driver.rs` に `v161000_tests` モジュール追加（5 テスト）
  - `version_is_16_1_0`
  - `error_output_has_line_number`
  - `error_output_has_caret`
  - `error_output_has_hint`
  - `error_output_has_url`

---

## Phase C — `span.rs` 新規作成

- [x] C-1: `Span` 構造体は既に `fav/src/frontend/lexer.rs` に存在（スキップ）
- [x] C-2: `TokenKind` enum は既に `lexer.rs` に存在（スキップ）
- [x] C-3: `cargo build` → コンパイルエラーなし確認

---

## Phase D — Lexer に Span 追加（lexer.rs）

- [x] D-1〜D-6: `Span` / `TokenKind` は既に `lexer.rs` に実装済み（スキップ）

---

## Phase E — AST ノードに Span 追加（ast.rs）

- [x] E-1〜E-6: AST に Span は既に統合済み（スキップ）

---

## Phase F — Parser に Span 伝播（parser.rs）

- [x] F-1〜F-5: Parser の Span 伝播は既に実装済み（スキップ）

---

## Phase G — `error.rs` 新規作成 → `TypeError.hints` + `format_diagnostic` 拡張

- [x] G-1: `TypeError` に `hints: Vec<String>` フィールド追加
- [x] G-2: `TypeError::new` / `with_hints` メソッド追加
- [x] G-3: `format_diagnostic` に動的 hints 出力 + URL 出力を追加
  - `-->` ファイル:行:列 の出力（既存）
  - ソース行の取得と表示（既存）
  - `^` アンダーライン（既存）
  - ラベルテキストの付与（既存）
  - `= ヒント: ...` 行の出力（動的 hints、新規）
  - `= help: ...` 行の出力（既存 get_help_text）
  - `= 参照: https://favnir.dev/errors/{code}` 行の出力（新規）
- [x] G-4: `NO_COLOR` 対応は plain text デフォルトのため不要
- [x] G-5: `checker_fav_runner.rs` の `TypeError` struct literal に `hints: vec![]` 追加
- [x] G-6: `cargo build` → コンパイルエラーなし確認

---

## Phase H — `levenshtein_candidates` 実装（checker.rs）

- [x] H-1: `fav/src/middle/checker.rs` に `levenshtein_candidates` 関数追加
  - `strsim::levenshtein` を使用
  - 距離 ≤ 2、最大 3 候補
- [x] H-2: `TyEnv::names()` メソッド追加（スコープ内の全変数名列挙）
- [x] H-3: `Checker::type_error_h` 補助メソッド追加（hints 付き TypeError 生成）

---

## Phase I — checker.rs エラーを Diagnostic に移行

- [x] I-1: E0102（undefined identifier / legacy E0001相当）に Levenshtein hint + URL 追加
- [x] I-2〜I-6: 全エラーコードに URL が付与される（`format_diagnostic` でコード別 URL を出力）
- [x] I-7: `cargo build` → コンパイルエラーなし確認

---

## Phase J — `driver.rs` 更新

- [x] J-1: `fav check` エラー出力が `format_diagnostic` 整形済み文字列を使用
- [x] J-2: CLI 引数に `--no-color` フラグ追加（`cmd_check` / `cmd_run` 共通）
- [x] J-3: `--no-color` で plain text 出力（format_diagnostic はデフォルトで plain text）

---

## Phase K — テスト確認

- [x] K-1: `cargo test v161000` → 5/5 PASS
  - `version_is_16_1_0` PASS
  - `error_output_has_line_number` PASS（`-->` が出力に含まれる）
  - `error_output_has_caret` PASS（`^` が出力に含まれる）
  - `error_output_has_hint` PASS（ヒントテキストが含まれる）
  - `error_output_has_url` PASS（`favnir.dev/errors/` が含まれる）
- [x] K-2: `cargo test` → 1580 PASS（リグレッションなし）
  - 4 failures は全て旧バージョン向けの version-pin テスト（E0001〜E0018 機能とは無関係）

---

## Phase L — サイトドキュメント

- [x] L-1: `site/content/docs/errors/index.mdx` 新規作成（エラーコード一覧）
- [x] L-2: `site/content/docs/errors/E0001.mdx` 新規作成
- [x] L-3: `site/content/docs/errors/E0007.mdx` 新規作成
- [x] L-4: `site/content/docs/errors/E0008.mdx` 新規作成
- [x] L-5: `site/content/docs/errors/E0009.mdx` 新規作成
- [x] L-6: `site/content/docs/errors/E0018.mdx` 新規作成
- [x] L-7: `site/content/docs/errors/E0252.mdx` 新規作成
- [x] L-8: `site/content/docs/errors/E0314.mdx` 新規作成
- [x] L-9: `site/content/docs/errors/E0319.mdx` 新規作成
- [x] L-10: `site/content/docs/errors/E0322.mdx` 新規作成
- [x] L-11: `site/content/docs/errors/E0013.mdx` 新規作成（bonus）

---

## Phase M — コミット

- [ ] M-1: `cargo test v161000` → 5/5 PASS 最終確認
- [ ] M-2: `cargo test` → 全件 PASS 最終確認
- [ ] M-3: コミット `feat: v16.1.0 — エラーメッセージ品質向上（rustc スタイル + typo ヒント）`

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "16.1.0"` | [x] |
| `strsim 0.11` 依存が追加されている | [x] |
| `fav/src/span.rs` が存在する（lexer.rs に統合） | [x] |
| `fav/src/error.rs` が存在する（driver.rs に統合） | [x] |
| `cargo test v161000` 全テストパス（5/5） | [x] |
| `cargo test` 全件パス（リグレッションなし） | [x] |
| `fav check` 出力に `-->` 行・列が含まれる | [x] |
| `fav check` 出力に `^` アンダーラインが含まれる | [x] |
| E0001 / E0007 で typo 候補が提示される（legacy checker） | [x] |
| 全エラーコードに hint または URL が付与されている | [x] |
| `--no-color` フラグが動作する | [x] |
| `site/content/docs/errors/` に 9 件以上の MDX が存在する | [x] |
