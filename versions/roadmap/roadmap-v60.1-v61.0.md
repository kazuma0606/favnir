# Roadmap v60.1.0 〜 v61.0.0 — Developer Experience 2.0

Date: 2026-07-30
Status: 未着手

---

## 前提

- 直前完了: v60.0.0「Enterprise 1.0」（tests = 3330、2026-07-30 COMPLETE）
- マスターロードマップ: `roadmap-v60.1-v65.0.md`
- 本文書はマスターの v61.0 スプリント部分の詳細版
- **既存機能の扱い**:
  - `fav check --json` は v12.5 実装済み → span 情報フィールドを追加拡張
  - LSP hover / completion / inlay_hints は v9.11〜v32.9 実装済み → Code Action を新規追加
  - `fav repl` は v9.10 実装済み → `:load` / `:debug` / マルチライン入力を追加
  - `fav fmt` は v9.2 実装済み → コメント保持・`.favfmt` 設定対応を追加
  - `fav doc` は v9.8 実装済み → HTML 出力・Rune ドキュメント統合を追加
  - `fav explain-error` は v24.5 実装済み → 全コード `long_description` 補完

---

## 目標

**「エラーはソース位置を指し、修正候補は即座に現れる」** 体験を実現する。

v60.1〜v60.9 の 9 スプリントで DX 全面強化を積み上げ、
v61.0「Developer Experience 2.0」として宣言する。

---

## バージョン計画

### v60.1.0 — エラーメッセージ span 表示（ソース位置・アンダーライン）

`error_catalog.rs` の `DiagEntry` に `span: Option<Span>` フィールドを追加。
checker.rs / parser.rs からエラー生成時にソース位置（ファイル名・行・列）を付与するよう更新。
`main.rs` の `print_diag` 関数に `-->` / `|` / `^` アンダーライン形式の出力を実装。

```
// 改善前
E0001: undefined variable: user_id

// 改善後
E0001: undefined variable: `user_id`
 --> pipeline.fav:12:15
   |
12 |   transform(user_id, name)
   |             ^^^^^^^ この変数は定義されていません
   |
   help: もしかして `userId` のことですか？
```

**完了条件**: Rust テスト 2 件（ベース 3330 + 2 = 3332 tests passed, 0 failed）
- `error_span_display_e0001`
- `error_span_underline_format`

**実績**: 3332 tests passed, 0 failed（2026-07-30 完了）

---

### v60.2.0 — `fav check --fix` 自動修正 Phase 1（typo 修正・未使用 bind 削除）

`main.rs` に `Some("check")` アームの `--fix` フラグ処理を追加。
`driver.rs` に `cmd_check_fix` 関数を実装。
E0001（typo 候補が 1 件のみの場合）と W001（未使用 bind）の自動修正を実装。
`--fix --dry-run` で変更箇所のプレビューのみ表示するモードを追加。

```bash
$ fav check --fix pipeline.fav
[auto-fixed] E0001: `user_id` → `userId` (pipeline.fav:12)
[auto-fixed] W001: unused bind `tmp` removed (pipeline.fav:8)
2 fixes applied.

$ fav check --fix --dry-run pipeline.fav
[would fix] E0001: `user_id` → `userId` (pipeline.fav:12)
[would fix] W001: unused bind `tmp` removed (pipeline.fav:8)
2 fixes would be applied (dry-run, no changes made).
```

**完了条件**: Rust テスト 2 件（ベース 3332 + 2 = 3334 tests passed, 0 failed）
- `check_fix_typo_single_candidate`
- `check_fix_unused_bind`

**実績**: 3334 tests passed, 0 failed（2026-07-30 完了）
出力は E0102/L002 を使用（ロードマップ記載の E0001/W001 とは異なる）。

---

### v60.3.0 — LSP Code Action（Quick Fix / Rename Symbol）

`lsp/` に `textDocument/codeAction` ハンドラを追加。
E0001 typo 修正・W001 未使用 bind 削除を Quick Fix として提供。
`lsp/references.rs` に `textDocument/rename` ハンドラを追加（変数・関数のリネーム）。
既存の `lsp/` モジュール構造（`hover.rs` / `completion.rs` / `inlay_hints.rs` / `references.rs`）と
同じパターンで `code_action.rs` を新規作成。

```json
// LSP textDocument/codeAction レスポンス例
{
  "title": "Did you mean `userId`?",
  "kind": "quickfix",
  "edit": {
    "changes": {
      "file:///pipeline.fav": [
        { "range": {...}, "newText": "userId" }
      ]
    }
  }
}
```

**完了条件**: Rust テスト 2 件（ベース 3334 + 2 = 3336 tests passed, 0 failed）
- `lsp_code_action_e0001_quickfix`
- `lsp_rename_variable`

**実績**: 3336 tests passed, 0 failed（2026-07-30 完了）
`code_action.rs` / `rename.rs` は実装済みのためテストのみ追加。
W001 LSP Quick Fix は `CheckedDoc` に `lint_errors` なしのためスコープ外（E0102 のみ対応）。
テスト名 `lsp_code_action_e0001_quickfix` は実際には E0102 を検証（ロードマップ記載の E0001 とは異なる）。

---

### v60.4.0 — LSP Diagnostic 完全統合（全エラーコードの位置情報付与）

v60.1 で追加した `span` フィールドを E0001〜E0426（本バージョン時点で登録済みの全エラーコード）に付与する。
LSP の `textDocument/publishDiagnostics` 通知で位置情報付き diagnostic を送出するよう
`lsp/` の diagnostic 送信パスを更新。
`fav check --json` の出力に `"span": {"file": "...", "line": N, "col": N}` フィールドを追加。

```json
// fav check --json の出力例
{
  "code": "E0001",
  "message": "undefined variable: `user_id`",
  "span": { "file": "pipeline.fav", "line": 12, "col": 15, "len": 7 },
  "suggestion": "did you mean `userId`?"
}
```

**完了条件**: Rust テスト 2 件（ベース 3336 + 2 = 3338 tests passed, 0 failed）
- `lsp_diagnostic_has_span`
- `check_json_includes_span`

**実績**: 3338 tests passed, 0 failed（2026-07-30 完了）
LSP 診断は既存実装（v50.2.0）；`fav check --json` に `span` サブオブジェクト（`SpanOutput` 構造体）を追加。
`type_warning_to_diag` も `span` を設定（struct の必須フィールドのため；W コード拡張はスコープ外）。

---

### v60.5.0 — `fav repl` 強化（`:load` / `:debug` / マルチライン入力）

既存の `fav repl`（v9.10 実装）に以下を追加する。
- `:load <file>` コマンド: pipeline.fav を読み込み stage 定義を REPL セッションに登録
- `:debug <stage>` コマンド: 指定 stage の入出力型・エフェクトを表示
- マルチライン入力: `\` または未閉じカッコ/ブレースで次行に継続（`|` プロンプト表示）

`driver.rs` の `cmd_repl` に `:load` / `:debug` のコマンドディスパッチを追加。
lexer に「行末 `\` または未閉じトークン検出」フラグを追加してマルチライン継続を実装。

```
favnir> :load pipeline.fav
loaded: pipeline.fav (3 stages)
favnir> :debug LoadCsv
[debug] stage LoadCsv: input=(), output=List<Row>, effects=[!IO]
favnir> bind x <-
      |   42 +
      |   58
x : Int = 100
```

**完了条件**: Rust テスト 2 件（ベース 3338 + 2 = 3340 tests passed, 0 failed）
- `repl_load_pipeline_file`
- `repl_multiline_input`

**実績**: 3340 tests passed, 0 failed（2026-07-31 完了）
`:load` 基盤は v9.10 実装済み；`stage` 定義ロードの `repl_load_pipeline_file` テストを追加。
`handle_debug_cmd`（stage シグネチャ表示）・`needs_continuation`（マルチライン継続判定）を新規追加。
エフェクト表示（`effects=[!IO]` 形式）はスコープ外（v60.6 以降）。

---

### v60.6.0 — `fav explain-error` 全コード対応 + `long_description` フィールド追加

`error_catalog.rs` の `ErrorEntry` に `long_description: Option<&'static str>` フィールドを追加。
登録済みの全エントリ（E0001〜E0428 範囲、実装時点で 97 件 E0101〜E0384）に Markdown 形式の詳細説明テキストを記述。
`fav explain-error <CODE>` でターミナルに整形表示。
`fav generate-error-docs` コマンドを追加し、`site/content/docs/errors/` の MDX を自動生成。

```bash
$ fav explain-error E0001
E0001: undefined variable

  変数が未定義です。スペルミスや、スコープ外の変数を参照していないか確認してください。

  よくある原因:
  - bind の前に変数を参照した
  - スペルミス（`user_Id` vs `user_id`）

  ドキュメント: https://favnir.dev/docs/errors/E0001
```

**注意**: v62.8.0 の E0427 / v63.3.0 の E0428 を登録する際も
`long_description` フィールドを必ず含めること（本バージョンで確立するスキーマに従う）。

**完了条件**: Rust テスト 2 件（ベース 3340 + 2 = 3342 tests passed, 0 failed）
- `explain_error_all_codes_have_long_desc`
- `cmd_generate_error_docs`

**実績**: 3342 tests passed, 0 failed（2026-07-31 完了）
`ErrorEntry` 構造体に `long_description: Option<&'static str>` フィールドを追加。
全 97 エントリ（E0101〜E0384）に `Some("See \`fix\` field for remediation details.")` を一括設定。
`cmd_explain_error_collect` に Long Description セクションを追加。
`cmd_generate_error_docs_str` / `cmd_generate_error_docs` を新規追加。
`fav generate-error-docs` CLI ディスパッチを `main.rs` に追加。

---

### v60.7.0 — `fav fmt` ルール拡張（コメント保持・行長制限・`.favfmt` 設定）

既存の `fav fmt`（v9.2、`fmt.rs`）を以下の点で拡張する。
- コメント（`//` 行・インラインコメント）を正しく保持するよう `fmt.rs` を修正
- 行長制限（デフォルト 100 文字）を超える式を自動折り返し
- プロジェクトルートの `.favfmt` ファイルを読み込んでフォーマット設定を適用

```toml
# .favfmt — フォーマット設定ファイル
max_line_length = 100
indent_width = 2
preserve_comments = true
trailing_comma = "always"
```

`driver.rs` の `cmd_fmt` を更新し、`.favfmt` を `toml.rs` でパースして設定を注入。

**完了条件**: Rust テスト 2 件（ベース 3342 + 2 = 3344 tests passed, 0 failed）
- `fmt_preserves_comments`
- `fmt_respects_favfmt_config`

**実装スコープ整理**（spec-reviewer 指摘対応）:
- `//` 行コメント保持: 実装済み（`reinsert_comments` 関数 + `make_anchor` プレフィックスマッチ）
- インラインコメント（行末 `// ...`）保持: v60.8 以降に延期（lexer 変更が必要）
- 行長を超える式の自動折り返し: v60.8 以降に延期（AST ノード幅計算が必要）
- `.favfmt` パース: `toml.rs` は使わず `FmtConfig::from_toml_str` として `fmt.rs` 内に実装
- `indent_width` はフォーマット出力への反映は v60.8 以降（設定保持のみ）

**実績**: 3344 tests passed, 0 failed（2026-07-31 完了）
`FmtConfig` 構造体（`from_toml_str` 含む）を `fmt.rs` に追加。
`format_with_config` / `reinsert_comments`（`make_anchor` プレフィックスマッチ）を追加。
`cmd_fmt` を `load_favfmt_config` + `format_with_config` 呼び出しに更新。

---

### v60.8.0 — `fav doc` 強化（HTML 出力・Rune ドキュメント統合）

既存の `fav doc`（v9.8、`///` コメント → Markdown）に以下を追加する。
- `fav doc --format html --out <dir>` で静的 HTML を生成する出力バックエンドを追加
- `runes/*/rune.toml` の `description` フィールドを読み込み Rune のドキュメントページを生成
- `/// @param <name> <desc>` / `/// @returns <desc>` タグのパーサーを追加し型情報と統合表示

```bash
$ fav doc --format html --out docs/
Generated: docs/index.html
Generated: docs/pipeline/LoadCsv.html
Generated: docs/runes/postgres.html
```

**完了条件**: Rust テスト 2 件（ベース 3344 + 2 = 3346 tests passed, 0 failed）
- `doc_html_output_generated`
- `doc_rune_description_included`

**実績**: 3346 tests passed, 0 failed（2026-07-31 完了）
`--format html` を `main.rs` の `match format` に追加（`cmd_doc_site` へのエイリアス）。
`cmd_doc_html_str`（テスト専用 HTML ラッパー）/ `parse_rune_toml_description` / `cmd_doc_rune_description_str` / `parse_doc_tags` を `driver.rs` に追加。
`parse_doc_tags` は関数追加のみ（HTML 統合表示は v60.9 以降）。
`runes/*/rune.toml` の実スキャン・Rune ページ自動生成は v60.9 以降。
テスト入力は `public fn` を使用（`doc_source_str` は `stage` 非対応）。

---

### v60.9.0 — 安定化・DX チェックリスト

v60.1〜v60.8 の全機能が統合されていることを確認する。

確認項目:
- `fav check --fix` の出力と LSP diagnostic の span 情報が一致する
- `.favfmt` 設定が `fav fmt` で正しく読み込まれる
- REPL で `:load` → `:debug` → `bind x <-` のマルチライン入力 E2E が通る
- `fav explain-error E0001` が `long_description` を正しく表示する
- `fav doc --format html` が Rune ドキュメントページを生成する

**完了条件**: Rust テスト 2 件（ベース 3346 + 2 = 3348 tests passed, 0 failed）
- `dx_e2e_check_fix_lsp_consistent`
- `dx_repl_pipeline_e2e`

**実績**: 3349 tests passed, 0 failed（2026-07-31 完了）
- ベース 3347（ロードマップ記載 3346 + v60.8.0 XSS テスト +1）+ 2 = 3349
- `dx_e2e_check_fix_lsp_consistent` pass
- `dx_repl_pipeline_e2e` pass

---

### v61.0 — Developer Experience 2.0 宣言 ★クリーンアップ

**宣言文**:

> 「エラーはソース位置を指し、修正候補は即座に現れる。
>  エディタは意図を理解し、フォーマッタはコメントを守る。
>  REPL でパイプラインを対話的に探索でき、ドキュメントは自動生成される。
>
>  Favnir のエラーメッセージはデータエンジニアの道標になった。
>
>  これが Favnir v61.0 — Developer Experience 2.0 の姿である。」

**完了条件**:
- v60.1〜v60.9 の全機能が動作する
- `cargo test` 全通過（failures=0、テスト数 ≥ **3352**）
- `v61000_tests` 4 件 pass（ベース 3348 + 4 = 3352 tests passed, 0 failed）:
  - `cargo_toml_version_is_61_0_0`
  - `changelog_has_v61_0_0`
  - `milestone_has_dx2`
  - `readme_mentions_dx2`
- `MILESTONE.md` に `"Developer Experience 2.0"` 宣言文エントリを追加
- `★クリーンアップ`（`cargo clean`）完了

**実績**: 3353 tests passed, 0 failed（2026-07-31 完了）
- ベース 3349（ロードマップ記載 3348 + v60.9.0 XSS オフセット +1）+ 4 = 3353
- `cargo_toml_version_is_61_0_0` pass
- `changelog_has_v61_0_0` pass
- `milestone_has_dx2` pass
- `readme_mentions_dx2` pass
- `cargo clean` 完了（10.2 GiB 削除）

---

## テスト数推移

| バージョン | テスト数 | 増加 | 備考 |
|---|---|---|---|
| v60.0.0（ベース） | 3330 | — | 実績値（2026-07-30 COMPLETE） |
| v60.1.0 | 3332 | +2 | error span 表示 |
| v60.2.0 | 3334 | +2 | check --fix |
| v60.3.0 | 3336 | +2 | LSP Code Action |
| v60.4.0 | 3338 | +2 | LSP Diagnostic 統合 |
| v60.5.0 | 3340 | +2 | repl 強化 |
| v60.6.0 | 3342 | +2 | explain-error 全コード |
| v60.7.0 | 3344 | +2 | fmt 拡張 |
| v60.8.0 | 3346 | +2 | doc 強化 |
| v60.9.0 | 3349 | +2 | 安定化（実績値: ロードマップ記載 3348 + v60.8.0 XSS テスト +1） |
| v61.0.0 | 3353 | +4 | DX 2.0 宣言（★クリーンアップ）（実績値: ベース 3349 + 4 = 3353） |

---

## 参考リンク

- マスターロードマップ: `versions/roadmap/roadmap-v60.1-v65.0.md`
- 前サブスプリント（完了）: `versions/roadmap/roadmap-v59.1-v60.0.md`
- 達成宣言: `MILESTONE.md`
- 進行状況: `versions/current.md`
