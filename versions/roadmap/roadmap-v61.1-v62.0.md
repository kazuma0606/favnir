# Roadmap v61.1.0 〜 v62.0.0 — Language Polish

Date: 2026-07-30
Status: 未着手

---

## 前提

- 直前完了: v61.0.0「Developer Experience 2.0」（tests = 3352）
- マスターロードマップ: `roadmap-v60.1-v65.0.md`
- 本文書はマスターの v62.0 スプリント部分の詳細版
- **既存機能の扱い**:
  - `Pattern::Or(Vec<Pattern>, Span)` は v17.2.0 実装済み
    - v61.1.0 / v61.2.0 では **AST 変更なし**（型チェック強化・lint 統合のみ）
    - v61.3.0 では `Vec<Pattern>` → `Vec<(Pattern, Option<Expr>)>` に **AST 拡張**（OR パターン各アームへの個別ガード）
  - `Pattern::As(String, Box<Pattern>, Span)` は v56.6.0 実装済み → ネスト・LSP hover 拡張のみ（AST 変更なし）
  - `MatchArm.guard: Option<Expr>` は単一ガードとして実装済み → v61.3.0 で OR パターン各アームへの個別ガードに拡張
  - `FString` は実装済み → ネスト呼び出し・マルチライン `"""..."""` 対応追加
  - W037（到達不能パターン）は実装済み → OR パターンへの対応拡張

---

## 目標

**「型システムがデータエンジニアの思考を助ける」** 言語表現力を実現する。

v61.1〜v61.9 の 9 スプリントで言語機能を磨き上げ、
v62.0「Language Polish」として宣言する。

---

## バージョン計画

### v61.1.0 — OR パターン強化（ネスト・型チェック・lint 統合）

`Pattern::Or` は v17.2.0 実装済み（`ast.rs` L297-298）。AST 変更なし。
以下を拡張する：
- OR パターン内の各アームが同一型を持つことを checker.rs で検証（型不一致時 E0009 を発行）
- lint.rs の W037（到達不能パターン）を OR パターンに対応拡張
  （先行アームが後続アームを完全に包含する場合に警告）
- E2E テストで 3 段階ネスト OR パターンが正しく動作することを確認

```favnir
match status {
  "active" | "pending" => process(row)
  "deleted" | "archived" => skip(row)
  _ => error("unknown status")
}
```

**完了条件**: Rust テスト 2 件（ベース 3353 + 2 = 3355 tests passed, 0 failed）
- `pattern_or_type_check_arms_same`
- `pattern_or_lint_w037_integration`

**実績**: 3355 tests passed, 0 failed（2026-07-31 完了）
- ベース 3353（ロードマップ記載 3352 + v60.8.0 XSS テスト +1）+ 2 = 3355
- `pattern_or_type_check_arms_same` pass（3アーム OR パターン E2E 含む）
- `pattern_or_lint_w037_integration` pass

---

### v61.2.0 — as-pattern 拡張（ネストパターン・LSP hover 統合）

`Pattern::As` は v56.6.0 実装済み（`ast.rs` L305-307）。AST 変更なし。
以下を拡張する：
- as-pattern と Record パターンのネストを checker.rs で正しく型チェック
  （内部フィールドと全体の型を同時に束縛できることを保証）
- LSP の `lsp/inlay_hints.rs` を拡張して as-pattern の束縛変数の型を hover 表示
- W039 lint（名前衝突）と as-name の衝突チェックを追加
  （W038 は v56.7.0 で wildcard import collision として実装済みのため W039 を使用）

```favnir
match value {
  Some({ id, name }) as opt => { log(opt); transform(id, name) }
  None => default()
}
```

**完了条件**: Rust テスト 2 件（ベース 3355 + 2 = 3357 tests passed, 0 failed）
- `pattern_as_nested_record`
- `pattern_as_lsp_hover_type`

**実績**: 3358 tests passed, 0 failed（2026-07-31 完了）
- ベース 3355 + 3 = 3358（code-reviewer 指摘で W039 positive test 追加のため +3）
- `pattern_as_nested_record` pass（`whole @ { x, y }` 構文、既存 checker 動作保証）
- `pattern_as_lsp_hover_type` pass（`collect_as_pattern_hints` inlay hint 生成確認）
- `w039_as_name_shadows_inner_should_warn` pass（W039 positive test、code-reviewer 指摘対応）
- W039 `as-name shadows inner binding` を lint.rs に追加
- 実装上の補正: as-pattern 実際の構文は `name @ pattern`（`as` キーワードではなく `@`）
- `block.expr`（最終返り値式）も W039 検査対象に追加（code-reviewer 指摘対応）

---

### v61.3.0 — パターンガード拡張（OR パターン各アームへの個別ガード）

OR パターンの各アームに独立したガードを付与できる構文を追加する。

`ast.rs` の `Pattern::Or` を `Vec<Pattern>` から `Vec<(Pattern, Option<Expr>)>` に拡張。
parser.rs で `(pat if cond) | (pat if cond)` を解析。
codegen.rs でアーム別ガード評価ロジックを実装（左から評価、最初にマッチしたアームを採用）。
（vm.rs にはパターン処理コードが存在しないため codegen.rs に実装）
checker.rs でガード式の型が `Bool` であることを検証。

```favnir
match row {
  ("active" if score > 90) | ("pending" if score > 50) => process(row)
  _ => skip(row)
}
```

**完了条件**: Rust テスト 2 件（ベース 3358 + 2 = 3360 tests passed, 0 failed）
- `guard_or_pattern_per_arm`
- `guard_or_pattern_fallthrough`

**実績**: 3361 tests passed, 0 failed（2026-07-31 完了）
- ベース 3358 + 3 = 3361（code-reviewer 指摘で E0395 negative test 追加のため +3）
- `guard_or_pattern_per_arm` pass（`(y if y > 90) | (y if y > 50)` 構文の型チェック確認）
- `guard_or_pattern_fallthrough` pass（3 アーム + ワイルドカード組み合わせ確認）
- `ast.rs`: `Pattern::Or(Vec<Pattern>)` → `Pattern::Or(Vec<(Pattern, Option<Expr>)>)`
- `ir.rs`: `IRPattern::Or(Vec<IRPattern>)` → `IRPattern::Or(Vec<(IRPattern, Option<IRExpr>)>)`
- `parser.rs`: `parse_or_alternative` 新規追加、`parse_match_arm` を更新
- `checker.rs`: ガード式の型が Bool であることを E0395 で検証
- `codegen.rs`: per-arm ガード評価（JumpIfFalse で次アームへ分岐）
- `error_catalog.rs`: E0395 追加（`long_description` 含む）
- 実装上の補足: vm.rs にはパターン処理コードが存在しないため codegen.rs に実装

---

### v61.4.0 — record update 式（`{ base | field: new_value }`）

ETL で「既存レコードの一部フィールドだけ書き換えた新レコードを作る」操作を簡潔に記述。

`ast.rs` に `Expr::RecordUpdate { base: Box<Expr>, fields: Vec<(String, Expr)> }` を追加。
parser.rs で `{ expr | field: val, ... }` を解析
（`{` 直後が識別子 + `|` なら RecordUpdate、それ以外は RecordConstruct として判別）。
checker.rs で `base` の型から全フィールドを継承し、`fields` で上書きされたフィールドの型を検証。
新しい `Expr` バリアントを追加するため、exhaustive match が必要な以下の全ファイルを更新する：
`compiler.rs` / `checker.rs` / `vm.rs` / `fmt.rs` / `lint.rs` /
`backend/codegen.rs` / `emit_python.rs` / `middle/ast_lower_checker.rs`

```favnir
bind updated <- { row | status: "active", score: row.score + 10 }
bind enriched <- { order | total: order.price * order.qty, currency: "JPY" }
```

**完了条件**: Rust テスト 2 件（ベース 3361 + 2 = 3363 tests passed, 0 failed）
- `record_update_basic`
- `record_update_type_check`

**実績**: 3365 tests passed, 0 failed（2026-07-31 完了）
- `record_update_basic` pass
- `record_update_type_check` pass
- `record_update_unknown_field_e0397` pass（code-reviewer 指摘対応）
- `record_update_type_mismatch_e0396` pass（code-reviewer 指摘対応）
- デシュガー方式（compiler.rs → IRExpr::RecordSpread）、vm.rs / codegen.rs 変更なし
- E0396（型不一致）/ E0397（不存在フィールド）新設

---

### v61.5.0 — 文字列補間強化（ネスト呼び出し・マルチライン `"""..."""`）

既存の `FString` をネストした式（関数呼び出し・メソッドチェーン）に対応させる。
`"""..."""` 形式のマルチライン文字列補間を lexer / parser に追加する。

`lexer.rs` にトリプルクォート文字列トークンを追加。
`parser.rs` の FString パーサーを再帰的な式解析に対応させる。
`fmt.rs` でマルチライン f-string のインデントを保持する整形ルールを追加。

```favnir
bind msg <- f"user={user.name} score={Float.format(score, decimals: 2)}"
bind report <- f"""
  Summary for {user.name}:
  - Total: {total}
  - Avg:   {avg}
"""
```

**完了条件**: Rust テスト 2 件（ベース 3365 + 2 = 3367 tests passed, 0 failed）
- `fstring_nested_call`
- `fstring_multiline`

**実績**: 3369 tests passed, 0 failed（2026-08-01 完了）
- ベース 3365 + 2 = 3367、code-reviewer 対応 +2 = **3369**
- `fstring_nested_call` pass（`user.name` フィールドアクセス + `Int.to_string(x)` 関数呼び出しが `{...}` 内で正しく型チェック）
- `fstring_multiline` pass（`f"""..."""` マルチライン補間が parse + type-check 通過）
- `ast.rs`: `Expr::FString(Vec<FStringPart>, bool /* multiline */, Span)` に拡張
- `lexer.rs`: `FStringTripleRaw(String)` トークン追加、triple-quote を区別
- `parser.rs`: `FStringTripleRaw` アーム追加、`parse_fstring_parts(raw, span, multiline)` シグネチャ更新
- `fmt.rs`: multiline フラグで `f"""..."""` / `f"..."` 分岐出力（旧 `$"..."` → `f"..."` に正規化）
- exhaustive match 更新: lint.rs(9), lineage.rs(5), checker.rs(4), compiler.rs(2), ast_lower_checker.rs(1), emit_python.rs(1), driver.rs(2), lsp/references.rs(1), parser.rs(3) — 計 28 箇所

---

### v61.6.0 — 型エラーメッセージ品質（期待型 vs 実際型の差分表示）

checker.rs の `unify` 失敗時に構造的差分を計算して表示するロジックを追加。
- Record 型同士の場合: 不足フィールド・型が異なるフィールドを列挙
- List/Stream 型の場合: 要素型の差分を表示
- スカラー型 vs 複合型の場合: 「スカラーではなく構造体が必要」と案内

`error_catalog.rs` E0009 の `long_description` を差分表示対応テキストに更新する。
`suggestion` フィールドは静的テキストを維持し、動的な差分テキストは `type_error_h` の `hints` として実行時に生成する（`diff_types` の出力を hint に追加）。

```
E0009: type mismatch in stage output
  expected: List<Row>
  found:    List<String>
            ^^^^^^^^^^^^
  difference: Row has fields { id: Int, name: String }, but String is a scalar type.
  help: Did you forget to wrap the string in a Row record?
```

**完了条件**: Rust テスト 2 件（ベース 3369 + 2 = 3371 tests passed, 0 failed）
- `type_error_diff_display_record`
- `type_error_suggestion_e0009`

**実績**: 3371 tests passed, 0 failed（2026-08-01 完了）
- ベース 3369 + 2 = 3371
- `type_error_diff_display_record` pass（Row(Named) vs String(scalar) のパイプライン不一致で E0103 に差分 hint が付く）
- `type_error_suggestion_e0009` pass（E0009 fav explain テキストに Record 型差分ヒントの記述を追加）
- `diff_types(expected, found, record_fields) -> Option<String>` を checker.rs に追加（`unify` 直前）
- `Expr::Pipeline` の E0103 call site を `type_error_h` に更新（diff hints 付与）
- `error_catalog.rs` E0103 `long_description` を差分表示対応テキストに更新
- `driver.rs` E0009 fav explain テキストに Record 型差分ヒントの説明を追加

---

### v61.7.0 — `_` 型プレースホルダー（部分型注釈・推論ヒント）

`ast.rs` の `TypeExpr` に `Hole` バリアントを追加。
parser.rs で型注釈位置の `_` を `TypeExpr::Hole` として解析。
checker.rs で `Hole` を `Type::Unknown` として解決し、型チェックをブロックしない
（`resolve_type_expr_with_self` は `&self` のため `fresh_var` 呼び出し不可 → `Type::Unknown` で代替）。
`lsp/inlay_hints.rs` で `Hole` 位置に "inferred" inlay hint を追加。

W040 `type_hole_inferred` lint を lint.rs に追加。
（W039 は v61.2.0 で `as-name shadows inner binding` として使用済み）
**W040 は v61.7.0 で通常の `fav lint` に含める。`--strict` フラグによる有効化は v61.8.0 で実装。**

```favnir
fn process(rows: List<_>) -> _ {
  rows |> List.filter(|r| r.active)
}
// `_` は型推論が埋める → inlay hint: List<Row> -> List<Row>
```

**完了条件**: Rust テスト 2 件（ベース 3371 + 2 = 3373 tests passed, 0 failed）
- `type_hole_infers_correctly`
- `type_hole_inlay_hint`

**実績**: 3373 tests passed, 0 failed（2026-08-01 完了）
- ベース 3371 + 2 = 3373
- `TypeExpr::Underscore` ではなく `TokenKind::Underscore` として parse することを確認
- `type_hole_inlay_hint` → `type_hole_parsed_as_hole`（テスト名変更）

---

### v61.8.0 — `fav check --strict` モード（追加 lint の有効化）

`main.rs` に `--strict` フラグ処理を追加（`fav check` / `fav lint` 共通）。
lint.rs に `LintConfig { strict: bool, perf: bool }` を追加し、フラグに応じて
W040（`type_hole_inferred`）を有効化。
`fav.toml` の `[lint]` セクションに `strict = true` オプションを追加し
`toml.rs` の `FavConfig` に `LintConfig` フィールドを追加してパース。

```bash
$ fav check --strict pipeline.fav
W040: type hole `_` inferred as `Row` — consider making explicit (pipeline.fav:3) [strict]
```

```toml
# fav.toml
[lint]
strict = true
```

**完了条件**: Rust テスト 2 件（ベース 3374 + 2 = 3376 tests passed, 0 failed）
- `check_strict_mode_w040_tagged`（ロードマップ旧テスト名 `check_strict_mode_enables_w040` から変更）
- `fav_toml_lint_strict`

**実績**: 3376 tests passed, 0 failed（2026-08-01 完了）
- ベース 3374 + 2 = 3376
- `[lint]` セクション開始トリガーが `parse_fav_toml` に欠落していたため追加
- `cmd_check` 内に `lint_program` の直接呼び出しが存在しなかったため「新規追加」として実装

---

### v61.9.0 — 安定化・Language Polish チェックリスト

v61.1〜v61.8 の全機能が統合されていることを確認する。

確認項目:
- OR パターン・as-pattern・個別ガードが既存パイプラインと共存する
- record update 式と通常の `bind` が型チェックを通過する
- `--strict` で W040 が正しく発火し、通常モードでは発火しない
- `_` 型プレースホルダーが `fav check --json` の出力に推論型として含まれる
- マルチライン f-string が `fav fmt` で正しく整形される

**完了条件**: Rust テスト 2 件（ベース 3376 + 2 = 3378 tests passed, 0 failed）
- `pattern_all_forms_coexist`
- `record_update_bind_mixed`

**実績**: 3378 tests passed, 0 failed（2026-08-01 完了）
- ベース 3376 + 2 = 3378
- `pattern_all_forms_coexist`（OR パターン + per-arm ガード + as-pattern 共存確認）
- `record_update_bind_mixed`（record update + パターンバインド混在確認）

---

### v62.0 — Language Polish 宣言 ★クリーンアップ

**宣言文**:

> 「パターンは OR で分岐し、as で束縛される。
>  レコードは `{ base | field: value }` で一部だけ書き換えられる。
>  型注釈に `_` を置けば推論が答えを返す。
>  エラーは期待値と実際値の差分を語り、修正の道筋を示す。
>
>  Favnir の型システムはデータエンジニアの思考を助ける存在になった。
>
>  これが Favnir v62.0 — Language Polish の姿である。」

**完了条件**:
- v61.1〜v61.9 の全機能が動作する
- `cargo test` 全通過（failures=0、テスト数 ≥ **3382**）
- `v62000_tests` 4 件 pass（ベース 3378 + 4 = 3382 tests passed, 0 failed）:
  - `cargo_toml_version_is_62_0_0`
  - `changelog_has_v62_0_0`
  - `milestone_has_language_polish`
  - `readme_mentions_language_polish`
- `MILESTONE.md` に `"Language Polish"` 宣言文エントリを追加
- `★クリーンアップ`（`cargo clean`）完了

**実績**: 3382 tests passed, 0 failed（2026-08-01 完了）
- ベース 3378 + 4 = 3382（旧バージョンの `cargo_toml_version_is_X` 10 件を `62.0.0` に一括更新）
- `cargo_toml_version_is_62_0_0` / `changelog_has_v62_0_0` / `milestone_has_language_polish` / `readme_mentions_language_polish`
- `MILESTONE.md` v62.0.0 Language Polish 宣言エントリ追加
- `README.md` v62.0 Language Polish 言及追加
- `★クリーンアップ`（`cargo clean` 後 3382 PASS 確認）完了

---

## テスト数推移

| バージョン | テスト数 | 増加 | 備考 |
|---|---|---|---|
| v61.0.0（ベース） | 3353 | — | DX 2.0 宣言後（実績値: ロードマップ記載 3352 + v60.8.0 XSS テスト +1） |
| v61.1.0 | 3355 | +2 | OR パターン強化 |
| v61.2.0 | 3358 | +3 | as-pattern 拡張（code-reviewer 対応で +3） |
| v61.3.0 | 3361 | +3 | OR パターン個別ガード（code-reviewer 対応で +3） |
| v61.4.0 | 3365 | +4 | record update 式（code-reviewer 対応で +4） |
| v61.5.0 | 3369 | +4 | f-string 強化（code-reviewer 対応で +4） |
| v61.6.0 | 3371 | +2 | 型エラー差分表示 |
| v61.7.0 | 3374 | +3 | `_` 型プレースホルダー ✅（code-reviewer 対応で +3） |
| v61.8.0 | 3376 | +2 | check --strict |
| v61.9.0 | 3378 | +2 | 安定化 |
| v62.0.0 | 3382 | +4 | Language Polish 宣言（★クリーンアップ） |

---

## 参考リンク

- マスターロードマップ: `versions/roadmap/roadmap-v60.1-v65.0.md`
- 前サブスプリント: `versions/roadmap/roadmap-v60.1-v61.0.md`
- 達成宣言: `MILESTONE.md`
- 進行状況: `versions/current.md`
