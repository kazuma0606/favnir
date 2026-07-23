# Roadmap v46.1.0 〜 v47.0.0 — Developer Experience

Date: 2026-07-15
Status: 計画中（v46.0 完了後に開始）

---

## 前提

- 直前完了: v46.0.0「Language Refinement」（v46.0 宣言後、tests ≥ 2989）
- マスターロードマップ: `roadmap-v45.1-v50.0.md`
- 本文書はマスターの v47.0 スプリント部分の詳細版

---

## 目標

`fav test` インラインテスト・LSP クイックフィックス・型情報可視化（`fav explain` 2.0）を実装し、
**Favnir の開発体験を実用水準に引き上げる**。

---

## バージョン計画

### v46.1.0 — `#[test]` ブロック AST + parser

```favnir
stage Add: (Int, Int) -> Int = |(a, b)| { a + b }

#[test]
fn test_add() {
  assert_eq(Add((1, 2)), 3)
  assert_eq(Add((0, 0)), 0)
}
```

`#[test]` アトリビュートを AST に追加。`parser.rs` で `#[test] fn ...` を解析し
`FnDef.is_test = true` として収集（`TestDef`/`TestGroup` との名前衝突を避けるため
独立 AST ノード `TestBlock` ではなく既存 `FnDef` のフィールド追加で実装）。
既存の `test "description" { ... }` 構文との共存を確認。

**完了条件**: Rust テスト 2 件（実績推定 2994 tests passed, 0 failed、実際の v46.0 完了は 2992）
- `test_block_parses`
- `test_fn_collected`

---

### v46.2.0 — `fav test` コマンド実装

```bash
fav test main.fav         # すべての #[test] fn を実行
fav test main.fav --filter test_add   # 名前でフィルタ
```

`fav test <file.fav>` で `#[test]` 付き `fn` を検出・実行。pass/fail 件数を報告。
`driver.rs` の `cmd_test` に `#[test]` 収集と VM 実行ループを追加。

**完了条件**: Rust テスト 2 件（実績推定 2996 tests passed, 0 failed、実際の v46.1 完了は 2994）
- `fav_test_discovers_tests`
- `fav_test_reports_results`

---

### v46.3.0 — assertion 拡充

```favnir
#[test]
fn test_validate() {
  assert_eq(ValidateOrder(bad_order), Err("invalid amount"))
  assert_ok(ValidateOrder(good_order))
  assert_err(ValidateOrder(zero_amount))
  assert_ne(1, 2)
}
```

`assert_eq` / `assert_ok` / `assert_err` / `assert_ne` を VM primitive として追加。
失敗時の diff メッセージも表示。

**完了条件**: Rust テスト 2 件（実績推定 2999 tests passed, 0 failed、起点 2997 実績）
- `assert_ok_passes`
- `assert_err_passes`

---

### v46.4.0 — LSP inlay hints 強化

型推論結果をエディタ上にインライン表示。`fav check --show-inference` の結果を
LSP `textDocument/inlayHint` として提供。パイプライン各ステージの推論型・
`bind` 変数の型を行末に表示。

**完了条件**: Rust テスト 2 件（実績推定 3001 tests passed, 0 failed、起点 2999 実績）
- `lsp_inlay_hints_type_annotation`
- `lsp_inlay_hints_pipeline`

---

### v46.5.0 — LSP クイックフィックス強化

E0102（未定義変数）に did-you-mean `quickFix` アクション。
E0101（引数数不一致）に引数追加提案アクション。
`lsp/` モジュールに `code_action` ハンドラーを追加。

**完了条件**: Rust テスト 2 件（実績推定 3003 tests passed, 0 failed）
- `lsp_quick_fix_undefined_var`
- `lsp_quick_fix_arg_count`

---

### v46.6.0 — `fav explain` 2.0 Phase 1（パイプライン図改善）

`fav explain --format mermaid` の出力品質向上。
`return` 早期脱出パスを dead path（点線）として図示。
`Err(...)` 返却パスをエラーパス（赤）として区別。

**完了条件**: Rust テスト 2 件（実績推定 3005 tests passed, 0 failed）
- `explain_mermaid_includes_dead_path`
- `explain_pipeline_v2`

---

### v46.7.0 — `fav explain --lineage` 2.0

`return` 早期脱出パスを lineage グラフから dead path としてマーク。
lineage エントリに `is_dead: bool` フラグ追加（`lineage.rs`）。
`fav explain --lineage --show-dead` で表示切替。

**完了条件**: Rust テスト 2 件（実績推定 3007 tests passed, 0 failed）
- `lineage_return_path_is_dead`
- `lineage_happy_path_active`

---

### v46.8.0 — `fav explain --types`

パイプライン各ステージの推論型を `fav explain --types` で表示。
ジェネリック型の実体化結果（例: `List<Row>` の具体型）も表示。

```
stage ParseCsv:   String -> List<Row>
stage FilterRows: List<Row> -> List<Row>
stage SaveToDb:   List<Row> -> Result<Int>
```

**完了条件**: Rust テスト 2 件（実績推定 3009 tests passed, 0 failed）
- `explain_types_shows_stage_types`
- `explain_types_generic_instantiation`

---

### v46.9.0 — Developer Experience ドキュメント + v47.0 前調整

`fav test` / LSP クイックフィックス / `fav explain` 2.0 の
`site/content/docs/` MDX ドキュメント追加。
v47.0 前コードフリーズ。

**完了条件**: Rust テスト 2 件（実績推定 3012 tests passed, 0 failed — v46.8 実績 3010 + 2）
- `developer_experience_doc_exists`
- `fav_test_doc_exists`

---

### v47.0.0 — Developer Experience 宣言 ★クリーンアップ

**宣言文**:

> 「インラインテスト・LSP クイックフィックス・型情報可視化が揃い、
>  Favnir の開発体験が実用水準に達した。
>
>  これが Favnir v47.0 — Developer Experience の姿である。」

**完了条件**:
- v46.1〜v46.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ **3012**）
- `v47000_tests` 4 件 pass:
  - `cargo_toml_version_is_47_0_0`
  - `changelog_has_v47_0_0`
  - `milestone_has_developer_experience` — MILESTONE.md に `"Developer Experience"` が含まれる
  - `readme_mentions_developer_experience`
- `MILESTONE.md` に `"Developer Experience"` が含まれる
- `★クリーンアップ`（`cargo clean`）完了

---

## 参考リンク

- マスターロードマップ: `versions/roadmap/roadmap-v45.1-v50.0.md`
- 前サブスプリント（アクティブ）: `versions/roadmap/roadmap-v45.1-v46.0.md`
- 次サブスプリント（v47.0 完了後に開始）: `versions/roadmap/roadmap-v47.1-v48.0.md`
- 達成宣言: `MILESTONE.md`
