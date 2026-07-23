# Roadmap v45.1.0 〜 v50.0.0 — Language Maturity

Date: 2026-07-15
Status: 計画中（v45.0 完了後に詳細確定）

---

## 前提（版体系の継承）

v41.0〜v45.0 は `roadmap-v40.1-v45.0.md`（マスター）と個別サブスプリントロードマップ
（`roadmap-v41.1-v42.0.md` 〜 `roadmap-v44.1-v45.0.md`）によって管理された。
v45.0「Precision & Flow」を 2026-07-15 に宣言し、そのフェーズは完了している。

本文書（`roadmap-v45.1-v50.0.md`）は上記を **supersede** し、v46.0〜v50.0 の新マスターとなる。
各マイルストーン開始時に対応するサブスプリントロードマップを作成する運用は継続する：

| サブスプリント文書 | カバー範囲 | 状態 |
|---|---|---|
| `roadmap-v45.1-v46.0.md` | v45.1〜v45.9 + v46.0 | 作成済み（アクティブ） |
| `roadmap-v46.1-v47.0.md` | v46.1〜v46.9 + v47.0 | 作成済み |
| `roadmap-v47.1-v48.0.md` | v47.1〜v47.9 + v48.0 | 作成済み |
| `roadmap-v48.1-v49.0.md` | v48.1〜v48.9 + v49.0 | 作成済み |
| `roadmap-v49.1-v50.0.md` | v49.1〜v49.9 + v50.0 | 作成済み |

---

## 目標

v45.0「Precision & Flow」で型安全なリアルタイムパイプラインを宣言した。
このフェーズは **「言語構文・開発者体験・標準ライブラリ・モジュールシステムを成熟させ、Favnir を実務で迷わず使える言語にする」** を実現する。

---

## バージョン計画

---

## v46.0 — Language Refinement（v45.1〜v45.9）

### v45.1.0 — `return` 構文 AST + parser

```favnir
stage ValidateOrder: Order -> Result<Order> = |order| {
  if order.amount <= 0.0 { return Err("invalid amount") }
  Ok(order)
}
```

`ReturnStmt` ノードを `ast.rs` に追加し、`parser.rs` で `return <expr>` を解析。
**適用スコープ**: `fn` ボディ・`stage` ボディのみ（`seq` 本体には不可）。
単一式ボディでは暗黙 return が継続して機能するため `return` は不要。複数行ボディの途中脱出専用。

**完了条件**: Rust テスト 2 件（`return_stmt_parses` / `single_expr_body_no_return_needed`）

---

### v45.2.0 — `return` 型チェック + E0415

```favnir
fn bad() -> Int {
  return "hello"  // E0415: return type mismatch — expected Int, got String
}
```

`checker.rs` で `ReturnStmt` の型を宣言戻り型と照合。不一致時 E0415 を発行。

**完了条件**: Rust テスト 2 件（`return_type_ok` / `return_type_mismatch_e0415`）

---

### v45.3.0 — `return` compiler + VM

`compiler.rs` で `ReturnStmt` → `Return` opcode を emit。`vm.rs` で `Return` opcode 処理（コールスタック巻き戻し）。

**完了条件**: Rust テスト 2 件（`return_early_exit_executes` / `return_in_stage_executes`）

---

### v45.4.0 — `match` 網羅性改善 + W034

```favnir
type Color = Red | Green | Blue

match color {
  Red   -> "red"
  Green -> "green"
  // W034: non-exhaustive match — Blue is not covered
}
```

`checker.rs` で全バリアントの網羅チェックを強化。未カバー時 W034 警告（`--deny-warnings` でエラー化可能）。E0416 は `match` 式が値を返す文脈で未網羅の場合のハードエラー。

**完了条件**: Rust テスト 3 件（`match_exhaustive_ok` / `match_w034_missing_variant` / `match_e0416_value_context`）

---

### v45.5.0 — 型エイリアス完全化

```favnir
// 透過的エイリアス（既存の type alias）
type UserId = Int        // UserId と Int は互換

// 不透過型（既存の opaque type）
opaque type OrderId = String   // OrderId と String は非互換
```

透過的 alias と opaque type の境界を checker.rs で完全に区別。
`type A = B` の文脈で `A` が `B` と互換であることを型推論が正しく追う。`collect_transparent_alias_chain` ヘルパー追加。

**完了条件**: Rust テスト 2 件（`transparent_alias_compatible` / `opaque_alias_incompatible`）

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

`error_catalog.rs` に `suggestion: Option<String>` フィールドを追加。
E0001〜E0200 の主要エラーに did-you-mean / 修正提案テキストを付与。

**完了条件**: Rust テスト 2 件（`e0001_suggestion_similar_name` / `e0007_suggestion_arg_count`）

---

### v45.7.0 — エラーメッセージ改善 Phase 2（E0201〜E0414）+ 数値リテラル `_`

E0201〜E0414 に修正提案を追加。
あわせて `parser.rs` のレキサーで数値リテラル中の `_` をスキップ対応。

```favnir
stage FilterLarge: List<Int> -> List<Int> = |nums| {
  nums |> List.filter(|n| n > 1_000_000)
}
```

**完了条件**: Rust テスト 3 件（`e0410_suggestion` / `numeric_literal_underscore_int` / `numeric_literal_underscore_float`）

---

### v45.8.0 — examples 更新 Phase 1

`examples/` 以下の既存サンプルを最新構文に統一。
- `!Effect` 記法の完全除去確認
- `bind` / `ctx` 構文への統一
- `return` ガード節パターンを活用したサンプルへ書き換え

**完了条件**: Rust テスト 1 件（`examples_no_legacy_effect_syntax`）

---

### v45.9.0 — examples 更新 Phase 2 + v46.0 前調整

examples 残件の更新・v46.0 前コードフリーズ。

**完了条件**: Rust テスト 2 件（`examples_structure_valid` / `language_refinement_overview_doc_exists`）

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
---

## v47.0 — Developer Experience（v46.1〜v46.9）

### v46.1.0 — `#[test]` ブロック AST + parser

```favnir
stage Add: (Int, Int) -> Int = |(a, b)| { a + b }

#[test]
fn test_add() {
  assert_eq(Add((1, 2)), 3)
  assert_eq(Add((0, 0)), 0)
}
```

`#[test]` アトリビュートを AST に追加。`parser.rs` で `#[test] fn ...` を解析し `TestBlock` ノードとして収集。

**完了条件**: Rust テスト 2 件（`test_block_parses` / `test_fn_collected`）

---

### v46.2.0 — `fav test` コマンド実装

`fav test <file.fav>` で `#[test]` ブロックを検出・実行。pass/fail 件数を報告。

**完了条件**: Rust テスト 2 件（`fav_test_discovers_tests` / `fav_test_reports_results`）

---

### v46.3.0 — assertion 拡充

```favnir
#[test]
fn test_validate() {
  assert_eq(ValidateOrder(bad_order), Err("invalid amount"))
  assert_ok(ValidateOrder(good_order))
  assert_err(ValidateOrder(zero_amount))
}
```

`assert_eq` / `assert_ok` / `assert_err` / `assert_ne` を VM primitive として追加。

**完了条件**: Rust テスト 2 件（`assert_ok_passes` / `assert_err_passes`）

---

### v46.4.0 — LSP inlay hints 強化

型推論結果をエディタ上にインライン表示。`fav check --show-inference` の結果を LSP inlayHints として提供。

**完了条件**: Rust テスト 2 件（`lsp_inlay_hints_type_annotation` / `lsp_inlay_hints_pipeline`）

---

### v46.5.0 — LSP クイックフィックス強化

E0001（未定義変数）に did-you-mean quickFix。E0007（引数数不一致）に引数追加提案。

**完了条件**: Rust テスト 2 件（`lsp_quick_fix_undefined_var` / `lsp_quick_fix_arg_count`）

---

### v46.6.0 — `fav explain` 2.0 Phase 1（パイプライン図改善）

`fav explain --format mermaid` の出力品質向上。`return` パスを dead path として図示。

**完了条件**: Rust テスト 2 件（`explain_mermaid_includes_dead_path` / `explain_pipeline_v2`）

---

### v46.7.0 — `fav explain --lineage` 2.0

`return` 早期脱出パスを lineage から除外（dead path マーク）。lineage エントリに `is_dead: bool` フラグ追加。

**完了条件**: Rust テスト 2 件（`lineage_return_path_is_dead` / `lineage_happy_path_active`）

---

### v46.8.0 — `fav explain --types`

パイプライン各ステージの推論型を `fav explain --types` で表示。

**完了条件**: Rust テスト 2 件（`explain_types_shows_stage_types` / `explain_types_generic_instantiation`）

---

### v46.9.0 — Developer Experience ドキュメント + v47.0 前調整

`fav test` / LSP / `fav explain` 2.0 の MDX ドキュメント追加。

**完了条件**: Rust テスト 2 件（`developer_experience_doc_exists` / `fav_test_doc_exists`）

---

### v47.0.0 — Developer Experience 宣言 ★クリーンアップ

**宣言文**:

> 「インラインテスト・LSP クイックフィックス・型情報可視化が揃い、
>  Favnir の開発体験が実用水準に達した。
>
>  これが Favnir v47.0 — Developer Experience の姿である。」

**完了条件**:
- v46.1〜v46.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ **3011**）
- `v47000_tests` 4 件 pass（`cargo_toml_version_is_47_0_0` / `changelog_has_v47_0_0` / `milestone_has_developer_experience` / `readme_mentions_developer_experience`）
- `MILESTONE.md` に `"Developer Experience"` が含まれる
- `★クリーンアップ`（`cargo clean`）完了

---
---

## v48.0 — Standard Library 2.0（v47.1〜v47.9）

### v47.1.0 — `List.zip` / `List.zip_with` / `List.chunk`

```favnir
bind pairs  <- List.zip(names, scores)          // [(String, Int)]
bind batched <- data |> List.chunk(100)          // List<List<Row>>
bind totals  <- List.zip_with(|a, b| a + b, xs, ys)
```

**完了条件**: Rust テスト 2 件（`list_zip_pairs` / `list_chunk_batches`）

---

### v47.2.0 — `List.flat_map` / `List.group_by` / `List.dedupe`

```favnir
bind expanded <- orders |> List.flat_map(|o| o.items)
bind by_region <- orders |> List.group_by(|o| o.region)  // Map<String, List<Order>>
bind unique   <- tags |> List.dedupe
```

**完了条件**: Rust テスト 3 件（`list_flat_map` / `list_group_by` / `list_dedupe`）

---

### v47.3.0 — `List.scan` / `List.take_while` / `List.drop_while`

```favnir
bind running_total <- prices |> List.scan(0, |acc, p| acc + p)
bind valid         <- rows   |> List.take_while(|r| r.status == "ok")
bind rest          <- rows   |> List.drop_while(|r| r.status == "ok")
```

**完了条件**: Rust テスト 3 件（`list_scan_cumulative` / `list_take_while` / `list_drop_while`）

---

### v47.4.0 — `String` 拡充

```favnir
bind padded  <- "42" |> String.pad_left(6, "0")   // "000042"
bind trimmed <- "  hello  " |> String.trim_start
bind rep     <- "ab" |> String.repeat(3)           // "ababab"
```

`String.trim_start` / `trim_end` / `repeat(n)` / `pad_left(n, ch)` / `pad_right(n, ch)` 追加。

**完了条件**: Rust テスト 3 件（`string_pad_left` / `string_trim_start` / `string_repeat`）

---

### v47.5.0 — `Float` / `Int` 拡充

```favnir
bind rounded <- 3.14159 |> Float.round(2)     // 3.14
bind clamped <- score   |> Float.clamp(0.0, 100.0)
bind digits  <- 255     |> Int.to_hex         // "ff"
```

`Float.round(n)` / `Float.clamp(min, max)` / `Int.to_hex` / `Int.abs` / `Float.abs` 追加。

**完了条件**: Rust テスト 3 件（`float_round` / `float_clamp` / `int_to_hex`）

---

### v47.6.0 — `Option` 拡充

```favnir
bind doubled <- maybe_int |> Option.map(|n| n * 2)
bind value   <- maybe_str |> Option.unwrap_or("default")
bind chained <- maybe_user |> Option.and_then(|u| lookup(u.id))
```

`Option.map` / `Option.unwrap_or` / `Option.and_then` / `Option.is_some` / `Option.is_none` 追加。

**完了条件**: Rust テスト 3 件（`option_map` / `option_unwrap_or` / `option_and_then`）

---

### v47.7.0 — `Result` 拡充

```favnir
bind doubled <- result_int |> Result.map(|n| n * 2)
bind handled <- result_val |> Result.map_err(|e| "wrapped: " ++ e)
bind chained <- parse_int(s) |> Result.and_then(|n| validate(n))
```

`Result.map` / `Result.map_err` / `Result.and_then` / `Result.is_ok` / `Result.is_err` 追加。

**完了条件**: Rust テスト 3 件（`result_map` / `result_map_err` / `result_and_then`）

---

### v47.8.0 — `Map` 拡充

```favnir
bind merged   <- Map.merge(defaults, overrides)
bind filtered <- config |> Map.filter_values(|v| v != "")
bind mapped   <- scores |> Map.map_values(|v| v * 2)
```

`Map.merge` / `Map.filter_values` / `Map.map_values` / `Map.keys` / `Map.values` 追加。

**完了条件**: Rust テスト 3 件（`map_merge` / `map_filter_values` / `map_map_values`）

---

### v47.9.0 — stdlib ドキュメント + v48.0 前調整

stdlib 全関数の MDX ドキュメント・cookbook サンプル更新。

**完了条件**: Rust テスト 2 件（`stdlib_v2_doc_exists` / `stdlib_v2_overview_exists`）

---

### v48.0.0 — Standard Library 2.0 宣言 ★クリーンアップ

**宣言文**:

> 「List・String・Float・Option・Result・Map の主要操作が揃い、
>  外部ライブラリなしに実務的なデータ変換が書ける。
>
>  これが Favnir v48.0 — Standard Library 2.0 の姿である。」

**完了条件**:
- v47.1〜v47.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ **3040**）
- `v48000_tests` 4 件 pass（`cargo_toml_version_is_48_0_0` / `changelog_has_v48_0_0` / `milestone_has_stdlib_v2` / `readme_mentions_stdlib_v2`）
- `MILESTONE.md` に `"Standard Library 2.0"` が含まれる
- `★クリーンアップ`（`cargo clean`）完了

**実績**: 3045 tests passed, 0 failed（2026-07-18 完了）✅

---
---

## v49.0 — Module & Package 2.0（v48.1〜v48.9）

### v48.1.0 — import 構文刷新 AST + parser（パッケージ）

```favnir
// fav.toml [runes] に宣言されたパッケージ → 引用符なし
import kafka
import postgres as db
```

`ImportStmt` ノードに `kind: ImportKind` フィールド追加（`Package` / `Local`）。
`parser.rs` で引用符なし import をパッケージ import として解析。

**完了条件**: Rust テスト 2 件（`import_package_parses` / `import_package_with_alias`）

---

### v48.2.0 — import 構文刷新（ローカルファイル）

```favnir
// ./ から始まる → ローカル .fav ファイル
import "./src/helpers" as helpers
import "./stages/validate" as validate
```

`parser.rs` で `"./"` prefix を持つ import を Local import として解析。

**完了条件**: Rust テスト 2 件（`import_local_parses` / `import_local_relative_path`）

---

### v48.3.0 — `fav.toml [runes]` 解決ロジック

```toml
[runes]
kafka    = "2.1.0"
postgres = "1.0.0"
```

`import kafka` → `fav.toml [runes]` を参照 → `runes/kafka/` から読み込み。
バージョン未登録時 E0417 エラー。

**完了条件**: Rust テスト 2 件（`rune_resolution_from_toml` / `e0417_rune_not_in_toml`）

---

### v48.4.0 — `fav install` コマンド

```bash
fav install kafka       # runes/kafka/ にローカル展開
fav install             # fav.toml [runes] 全件インストール
```

`fav.toml [runes]` を読んで `runes/<name>/` にダウンロード・展開する MVP 実装。

**完了条件**: Rust テスト 2 件（`fav_install_creates_rune_dir` / `fav_install_all_from_toml`）

---

### v48.5.0 — import エイリアス完全化 + 旧構文 deprecation

旧 `import rune "kafka"` 構文を W035 警告（非推奨）化。
`import kafka as k` の完全サポート確認。

**完了条件**: Rust テスト 2 件（`import_alias_resolves` / `legacy_import_rune_w035`）

---

### v48.6.0 — 循環 import 検出 + E0418

```
E0418: circular import detected
  a.fav → b.fav → a.fav
```

import グラフをトポロジカルソートし、循環時 E0418 を発行。

**完了条件**: Rust テスト 2 件（`circular_import_e0418` / `non_circular_import_ok`）

---

### v48.7.0 — rune.toml 標準化

全公式 rune の `rune.toml` を統一フォーマットに更新。
`[rune]` セクション必須・`[connection]` 非標準セクション除去確認。

**完了条件**: Rust テスト 2 件（`rune_toml_standard_format` / `rune_toml_no_connection_section`）

---

### v48.8.0 — `fav rune` コマンド群

```bash
fav rune list           # インストール済み rune 一覧
fav rune info kafka     # rune の詳細（バージョン・関数一覧）
fav rune remove kafka   # rune 削除
```

**完了条件**: Rust テスト 2 件（`fav_rune_list_shows_installed` / `fav_rune_info_shows_version`）

---

### v48.9.0 — Module ドキュメント + migration guide + v49.0 前調整

import 構文移行ガイド MDX・旧 `import rune` からの自動変換スクリプト案。

**完了条件**: Rust テスト 2 件（`module_system_doc_exists` / `import_migration_guide_exists`）

---

### v49.0.0 — Module & Package 2.0 宣言 ★クリーンアップ

**宣言文**:

> 「パッケージ import とローカル import が構文で明確に分離され、
>  `fav.toml` が依存関係の唯一の真実となった。
>
>  これが Favnir v49.0 — Module & Package 2.0 の姿である。」

**完了条件**:
- v48.1〜v48.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ **3065**）
- `v49000_tests` 4 件 pass（`cargo_toml_version_is_49_0_0` / `changelog_has_v49_0_0` / `milestone_has_module_package_v2` / `readme_mentions_module_package_v2`）
- `MILESTONE.md` に `"Module & Package 2.0"` が含まれる
- `★クリーンアップ`（`cargo clean`）完了

**実績**: 3069 tests passed, 0 failed（2026-07-18 完了）✅

---
---

## v50.0 — Production 2.0（v49.1〜v49.9）

### v49.1.0 — 全機能統合テスト + E2E デモ更新

v46〜v49 の全機能を使った E2E デモ更新。`return` / 新 import / stdlib 2.0 / `fav test` ブロックを統合したパイプラインデモ。

**完了条件**: Rust テスト 2 件（`e2e_demo_v50_structure` / `e2e_demo_uses_new_import`）

---

### v49.2.0 — パフォーマンス計測 + ボトルネック修正

`fav bench --all` で v46〜v49 機能追加後の速度計測。checker.rs / compiler.rs のホットパスを特定し改善。

**完了条件**: Rust テスト 2 件（`bench_all_result_recorded` / `checker_perf_regression_none`）

---

### v49.3.0 — `fav check` インクリメンタル型チェック

変更ファイルのみ再チェック（SHA-256 フィンガープリント）。大規模プロジェクトでの `fav check` 速度改善。

**完了条件**: Rust テスト 2 件（`incremental_check_skips_unchanged` / `incremental_check_detects_change`）

---

### v49.4.0 — ドキュメントサイト全面更新 Phase 1

v46〜v48 機能の docs MDX 更新（`return` 構文・stdlib 2.0・import 2.0）。

**完了条件**: Rust テスト 2 件（`docs_return_syntax_exists` / `docs_import_v2_exists`）

---

### v49.5.0 — cookbook 更新

新機能を活用したレシピ追加（`return` ガード節パターン / `fav test` 活用 / 新 import 構文）。

**完了条件**: Rust テスト 2 件（`cookbook_return_guard_exists` / `cookbook_fav_test_exists`）

---

### v49.6.0 — WASM / Python transpiler 互換確認

v46〜v49 の新構文が WASM ビルドと Python transpiler で正しく動作することを確認。`return` / 新リテラル / 新 import が各ターゲットで処理されること。

**完了条件**: Rust テスト 2 件（`wasm_compat_return_stmt` / `python_emit_return_stmt`）

---

### v49.7.0 — セキュリティ審査 2.0

import 2.0 でのパストラバーサル検証（`"../../etc/passwd"` 等の拒否）。`fav install` でのパッケージ名バリデーション。

**完了条件**: Rust テスト 2 件（`import_path_traversal_rejected` / `install_invalid_name_rejected`）

---

### v49.8.0 — ドキュメントサイト全面更新 Phase 2 + CHANGELOG 整理

v49〜全体の CHANGELOG / MILESTONE 整理。`site/content/docs/` 全ページの最終チェック。

**完了条件**: Rust テスト 2 件（`docs_site_v50_overview_exists` / `milestone_has_language_maturity`）

---

### v49.9.0 — v50.0 前調整・安定化

コードフリーズ。`site/content/docs/language-maturity-overview.mdx` 作成。

**完了条件**: Rust テスト 2 件（`cargo_toml_version_is_49_9_0` / `language_maturity_overview_doc_exists`）

---

### v50.0.0 — Production 2.0 宣言 ★クリーンアップ

**宣言文**:

> 「`return` による安全なガード節、成熟した標準ライブラリ、
>  明確なモジュールシステム、インラインテストが揃い、
>  Favnir は迷わず使える実用言語になった。
>
>  これが Favnir v50.0 — Production 2.0 の姿である。」

**完了条件**:
- v49.1〜v49.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ **3084**）
- `v50000_tests` 4 件 pass（`cargo_toml_version_is_50_0_0` / `changelog_has_v50_0_0` / `milestone_has_language_maturity` / `readme_mentions_language_maturity`）
- `MILESTONE.md` に `"Language Maturity"` が含まれる
- `★クリーンアップ`（`cargo clean`）完了

---

## 参考リンク

- 本文書（マスター）: `versions/roadmap/roadmap-v45.1-v50.0.md`
- 前マスター（完了）: `versions/roadmap/roadmap-v40.1-v45.0.md`
- 前サブスプリント（完了）: `versions/roadmap/roadmap-v44.1-v45.0.md`
- 現行サブスプリント（アクティブ）: `versions/roadmap/roadmap-v45.1-v46.0.md`
- 達成宣言: `MILESTONE.md`
- 現行状況: `versions/current.md`
