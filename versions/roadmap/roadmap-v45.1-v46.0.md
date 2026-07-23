# Roadmap v45.1.0 〜 v46.0.0 — Language Refinement

Date: 2026-07-15
Status: 計画中（v45.0 完了後に開始）

---

## 前提

- 直前完了: v45.0.0「Precision & Flow」（2026-07-15、2966 tests）
- マスターロードマップ: `roadmap-v45.1-v50.0.md`
- 本文書はマスターの v46.0 スプリント部分の詳細版

---

## 目標

`return` 構文・`match` 網羅性・型エイリアス完全化・エラーメッセージ改善・数値リテラル・examples 更新を実施し、**Favnir の構文を成熟させる**。

---

## バージョン計画

### v45.1.0 — `return` 構文 AST + parser

```favnir
stage ValidateOrder: Order -> Result<Order> = |order| {
  if order.amount <= 0.0 { return Err("invalid amount") }
  Ok(order)
}
```

`ReturnStmt` ノードを `ast.rs` に追加し、`parser.rs` で `return <expr>` を解析。

**適用スコープ**:
- `fn` ボディ: ✅
- `stage` ボディ: ✅
- `seq` パイプライン本体: ❌（stage の合成であり return の概念がない）

**単一式との関係**: 単一式ボディの暗黙 return は変更なし。`return` は複数行ボディの途中脱出専用。

**完了条件**: Rust テスト 2 件（実績推定 2968 tests passed, 0 failed）
- `return_stmt_parses`
- `single_expr_body_no_return_needed`

---

### v45.2.0 — `return` 型チェック + E0415

```favnir
fn bad() -> Int {
  return "hello"  // E0415: return type mismatch — expected Int, got String
}

fn ok() -> Int {
  if some_condition { return 0 }
  42
}
```

`checker.rs` で `ReturnStmt` の型を宣言戻り型と照合。不一致時 E0415 を発行。
stage の型注釈（`: Input -> Output`）も戻り型として参照する。

**完了条件**: Rust テスト 2 件（実績推定 2970 tests passed, 0 failed）
- `return_type_ok`
- `return_type_mismatch_e0415`

---

### v45.3.0 — `return` compiler + VM

`compiler.rs` で `ReturnStmt` → `Return` opcode を emit。
`vm.rs` で `Return` opcode 処理（現在のコールフレームを巻き戻し、値をスタックトップに積む）。

**完了条件**: Rust テスト 2 件（実績推定 2972 tests passed, 0 failed）
- `return_early_exit_executes`
- `return_in_stage_executes`

---

### v45.4.0 — `match` 網羅性改善 + W034 / E0416

```favnir
type Color = Red | Green | Blue

// W034: 値を返さない文脈（statement として使う場合）
match color {
  Red   -> process_red()
  Green -> process_green()
  // W034: non-exhaustive match — Blue is not covered
}

// E0416: 値を返す文脈（式として使う場合）は ハードエラー
let label = match color {
  Red   -> "red"
  Green -> "green"
  // E0416: non-exhaustive match in value context — Blue is not covered
}
```

`checker.rs` で全バリアントの網羅チェックを強化。
- 非網羅 + 文 → W034（`--deny-warnings` でエラー化可能）
- 非網羅 + 式 → E0416 ハードエラー

**完了条件**: Rust テスト 3 件（実績推定 2977 tests passed, 0 failed）
- `match_exhaustive_ok`
- `match_w034_missing_variant`
- `match_e0416_value_context`

---

### v45.5.0 — 型エイリアス完全化

```favnir
// 透過的エイリアス — UserId と Int は互換
type UserId = Int

fn process(id: UserId) -> Int { id + 1 }  // Int として扱える

// 不透過型 — OrderId と String は非互換
opaque type OrderId = String

fn get_order(id: OrderId) -> Order { ... }
// get_order("raw_string")  // E0413: type mismatch — expected OrderId, got String
```

`checker.rs` で `type A = B` の透過的互換性を型推論が正しく追う。
`collect_transparent_alias_chain` ヘルパーで alias チェーンを解決。

**完了条件**: Rust テスト 2 件（実績推定 2977 tests passed, 0 failed）
- `transparent_alias_compatible`
- `opaque_alias_incompatible`

---

### v45.6.0 — エラーメッセージ改善 Phase 1（E0001〜E0200）

```
// 改善前
E0001: undefined variable `ordr`

// 改善後
E0001: undefined variable `ordr`
  help: did you mean `order`?
  --> src/main.fav:12:5
```

`error_catalog.rs` に `suggestion: Option<&'static str>` フィールドを追加（静的カタログのため `&'static str`）。
E0101〜E0200 の主要エラーに did-you-mean / 修正提案テキストを付与。
編集距離（Levenshtein）を用いて候補変数名・関数名を提案（未定義識別子: E0102、引数数不一致: E0101）。

**完了条件**: Rust テスト 2 件（実績推定 2982 tests passed, 0 failed）
- `e0102_suggestion_similar_name`
- `e0101_suggestion_arg_count`

---

### v45.7.0 — エラーメッセージ改善 Phase 2（E0201〜E0414）+ 数値リテラル `_`

E0201〜E0414 に修正提案を追加。
あわせて `lexer.rs` の `lex_number` 関数で数値リテラル中の `_` をスキップ対応。

```favnir
stage FilterLarge: List<Int> -> List<Int> = |nums| {
  nums |> List.filter(|n| n > 1_000_000)
}

stage CalcRate: Float -> Float = |v| {
  v * 0.000_15
}
```

**完了条件**: Rust テスト 3 件（実績推定 2985 tests passed, 0 failed）
- `e0410_suggestion`
- `numeric_literal_underscore_int`
- `numeric_literal_underscore_float`

---

### v45.8.0 — examples 更新 Phase 1

`examples/` 以下の既存サンプルを最新構文に統一。
- 旧 `!Effect` 記法の完全除去確認
- `bind` / `ctx` 構文への統一
- `return` ガード節パターンを活用したサンプルへ書き換え
- import 構文の旧形式（`import rune "..."` 形式）は W035 として予告

**完了条件**: Rust テスト 1 件（実績推定 2986 tests passed, 0 failed）
- `examples_no_legacy_effect_syntax`

---

### v45.9.0 — examples 更新 Phase 2 + v46.0 前調整

examples 残件の更新・v46.0 前コードフリーズ。
`site/content/docs/language-refinement-overview.mdx` 新規作成。

**完了条件**: Rust テスト 2 件（実績推定 2988 tests passed, 0 failed）
- `examples_structure_valid`
- `language_refinement_overview_doc_exists`

---

### v46.0.0 — Language Refinement 宣言 ★クリーンアップ

**宣言文**:

> 「`return` によるガード節・`match` 完全網羅・型エイリアスの明確な境界・
>  改善されたエラーメッセージが揃い、Favnir の構文が成熟した。
>
>  これが Favnir v46.0 — Language Refinement の姿である。」

**完了条件**:
- v45.1〜v45.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ **2989**）
- `v46000_tests` 4 件 pass（`cargo_toml_version_is_46_0_0` / `changelog_has_v46_0_0` / `milestone_has_language_refinement` / `readme_mentions_language_refinement`）
- `MILESTONE.md` に `"Language Refinement"` が含まれる
- `★クリーンアップ`（`cargo clean`）完了

---

## 参考リンク

- マスターロードマップ: `versions/roadmap/roadmap-v45.1-v50.0.md`
- 前サブスプリント（完了）: `versions/roadmap/roadmap-v44.1-v45.0.md`
- 次サブスプリント（v46.0 完了後に作成）: `versions/roadmap/roadmap-v46.1-v47.0.md`
- 達成宣言: `MILESTONE.md`
