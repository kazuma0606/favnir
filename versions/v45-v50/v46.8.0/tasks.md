# Tasks: v46.8.0 — `fav explain --types`

Status: COMPLETE
Date: 2026-07-17

---

## T0 — 事前確認

- [x] `cargo test` 3007 passed, 0 failed を確認

## T1 — `driver.rs`: `type_expr_str` + `format_stage_types` + `cmd_explain_types` + `v468000_tests`

- [x] `type_expr_str(ty: &ast::TypeExpr) -> String` ヘルパーを追加（`cmd_explain_lineage` 直後）
- [x] `format_stage_types(program: &ast::Program) -> String` を追加（private）
  - [x] `program.items` から `ast::Item::TrfDef` を収集
  - [x] 各 TrfDef を `stage Name<params>: InputType -> OutputType\n` 形式で出力
  - [x] ステージ 0 件のとき `"(no stages found)\n"` を出力
- [x] `cmd_explain_types(file: Option<&str>)` を追加（`format_stage_types` を呼んで print）
  - [x] ファイルパス解決（`file` が `None` の場合は `fav.toml` から収集）
- [x] `v468000_tests` モジュールを追加
  - [x] `explain_types_shows_stage_types`: 3 ステージの型出力を確認
  - [x] `explain_types_generic_instantiation`: ジェネリックステージの型出力を確認

## T2 — `main.rs`: `--types` フラグ追加

- [x] `Some("explain")` ブランチの `--lineage` チェック直前に `--types` ガードを追加
- [x] `use driver::{}` ブロックに `cmd_explain_types` を追加
- [x] `cmd_explain_types(file)` を呼び出す

## T3 — テスト＆完了

- [x] `cargo test` 3010 passed, 0 failed（3007 + 3 件 — code-reviewer 指摘対応で `explain_types_no_stages` テスト追加）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `fav/Cargo.toml` version → `"46.8.0"`
- [x] `CHANGELOG.md` に v46.8.0 エントリ追加
- [x] `versions/current.md` を v46.8.0（3009 tests）に更新
- [x] tasks.md を COMPLETE に更新（T0〜T3 全チェック）

---

## コードレビュー指摘と対応（spec-reviewer）

| 重大度 | 内容 | 対応 |
|---|---|---|
### spec-reviewer 指摘

| 重大度 | 内容 | 対応 |
|---|---|---|
| [HIGH] | ロードマップの「推論型/実体化」と spec の「宣言型のみ」の齟齬 | spec.md に「ロードマップ表記との対応」セクションを追加して明記 |
| [MED] | テストヘルパーの不要 `fs::write` | `format_stage_types` に切り出し、ファイル I/O を完全排除 |
| [MED] | `cmd_explain_types` とテストのロジック重複 | `format_stage_types` 純粋関数として抽出 |
| [LOW] | v46.9.0 MDX 追加の責務明示 | v46.9.0 tasks.md 作成時に対応 |

### code-reviewer 指摘

| 重大度 | 内容 | 対応 |
|---|---|---|
| [MED] | 空プログラム（ステージ 0 件）の `(no stages found)` テストが欠けている | `explain_types_no_stages` テストを追加 |
| [MED] | `type_expr_str` が `fmt_type_expr_simple` と重複 — 新 variant 追加時の同期漏れリスク | NOTE/TODO コメントを強化（`pub(crate)` 化は v47 以降で検討） |
| [LOW] | `format!("{}", n)` より `n.to_string()` が慣用的 | `n.to_string()` に修正 |

## 実装中に判明した修正（テスト失敗対応）

| 内容 | 修正 |
|---|---|
| `trf` キーワード廃止済み（v2.0.0 で `stage` に置換） | テストソースを `stage` 構文に修正 |
| `stage Name: In -> Out { body }` は構文エラー | 正しい構文 `stage Name: In -> Out = \|x\| { body }` に修正 |
