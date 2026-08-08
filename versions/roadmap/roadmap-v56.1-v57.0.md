# Roadmap v56.1.0 〜 v57.0.0 — Language Power 2.0

Date: 2026-07-23
Status: COMPLETE

---

## 前提

- 直前完了: v56.0.0「Streaming Native 2.0」（tests ≥ 3228）
- マスターロードマップ: `roadmap-v55.1-v60.0.md`
- 本文書はマスターの v57.0 スプリント部分の詳細版
- **既存機能の扱い**: `where T: Interface`（`T with Ord` 形式）は v33.0 で実装済み。
  v56.1〜v56.2 は正式構文への統一・複数 constraint・coherence 強化が目的。
  行多相レコード（`R with { id: Int }` 形式）は v33.0 で実装済み。
  v56.3 は汎用関数での行変数 `<r>` 明示と LSP 表示の拡張が目的。
  エフェクト推論（`infer_effects_fn`）は v32.9 で実装済み。
  v56.4 は LSP inlay hints への統合が目的。
  `MatchArm.guard` は v0.5.0 から実装済み。v56.5 は OR パターン（`PatternOr`）の新規追加が目的。
  詳細はマスターロードマップ「既存機能との位置づけ」テーブルを参照。
- **エラーコード前提**: v55.5.0（E0421 `!State` エフェクトエラーを `error_catalog.rs` に追加）が完了していること。
  v56.1 で追加する E0422 は E0421 の直後のコードであるため、v55.5.0 完了を先に確認すること。

---

## 目標

v33.0「Language Power」で実装した型システム機能群を、
**より広い文脈・実用的なパターンで活用できる「Language Power 2.0」として完成させる**。

---

## バージョン計画

### v56.1.0 — 境界付きジェネリクス本番品質化（`where T: Interface` 拡張）

v33.0 実装済みの `where T: Interface`（`T with Ord` 形式）を、標準的な `where T: Interface`
構文として正式化。parser の `WhereClause` ノードを整理・統一し、checker の制約検証メッセージを
E0422 エラーコードとして正式カタログ登録（`error_catalog.rs` に新規エントリ追加）。
stdlib の各関数定義に `where` 節を適切に付与して型安全性を強化。

```favnir
interface Serializable {
  fn to_json(self: Self) -> String
}

fn serialize_all<T>(items: List<T>) -> List<String>
  where T: Serializable
{
  List.map(items, |x| x.to_json())
}
```

**完了条件**: Rust テスト 2 件（ベース 3227 + 2 = 3229 tests passed, 0 failed）
- `where_clause_stdlib_fn`
- `where_clause_e0422_emitted`

**実績**: COMPLETE — 3229 tests passed, 0 failed（2026-07-25）

---

### v56.2.0 — 境界付きジェネリクス Phase 2（複数 constraint・coherence 強化）

`T with Ord with Serialize` 形式の複数 `with` constraint の動作確認（既存 parser サポート済み）。
coherence ルール（同一型に対する重複 `impl` の禁止）の checker ロジックを強化し、
E0423 エラーコードで報告。

```favnir
fn pick<T with Ord with Serialize>(a: T, b: T) -> T {
    if a > b { a } else { b }
}
```

**完了条件**: Rust テスト 2 件（ベース 3229 + 2 = 3231 tests passed, 0 failed）
- `where_multiple_constraints`
- `impl_coherence_violation`

**実績**: COMPLETE — 3231 tests passed, 0 failed（2026-07-25）

---

### v56.3.0 — 行多相レコード活用拡張

v33.0 実装済みの `R with { id: Int }` 行多相を、関数の型パラメータ `<r>` として
明示的に扱えるよう拡張。`{ field: Type | r }` 記法を parser で受理し、
HM 型推論の `unify` で行変数を正しく扱う（既存 `unify_deep` の拡張）。
LSP ホバーで行変数の型を `{ name: String | ... }` 形式で表示。

```favnir
fn get_name<r>(record: { name: String | r }) -> String {
  record.name
}

let user_name = get_name({ name: "Alice", age: 30 })
let product_name = get_name({ name: "Widget", price: 9.99 })
```

**完了条件**: Rust テスト 2 件（ベース 3231 + 2 = 3233 tests passed, 0 failed）
- `row_poly_generic_fn`
- `row_poly_lsp_hover`

**実績**: **COMPLETE** — 3233 tests（2026-07-25）
- `TypeExpr::RecordType` に `Option<String>` row_var 追加、`TypeExpr::display()` 追加
- `{ field: Type | r }` parser 対応（`TokenKind::Pipe` 検出）
- 全 match arm 更新（emit_python / fmt / driver / lint / lsp/references / middle 各ファイル）
- `v56300_tests` 3 件追加（`cargo_toml_version_is_56_3_0` / `row_poly_generic_fn` / `row_poly_lsp_hover`）

---

### v56.4.0 — エフェクト推論 LSP 統合（inlay hints 表示）

v32.9 実装済みの `infer_effects_fn` の結果を LSP の `textDocument/inlayHint` に統合。
エフェクト注釈を省略した関数定義で、推論されたエフェクトセットをインラインに表示。
`fav check --show-types` の出力にも推論エフェクトを含めて一貫性を確保。

```favnir
// エフェクト注釈を省略
fn load_data() -> List<Row> {
  bind rows <- kafka.consume("orders")
  bind _ <- snowflake.insert(rows)
  rows
}
// エディタ inlay hint 表示: fn load_data() -> List<Row> /*!Kafka !Snowflake*/
```

**完了条件**: Rust テスト 2 件（ベース 3233 + 2 = 3235 tests passed, 0 failed）
- `effect_inference_inlay_hint`
- `effect_inference_check_output`

**実績**: — （未実施）

---

### v56.5.0 — OR パターン + パターンガード強化

`Pattern::Or` は v17.2.0 時点で実装済み（`ast.rs` L298）— AST ノード新規追加なし。
既存の `MatchArm.guard`（v0.5.0 実装済み）との組み合わせは checker / parser で対応済み。
W037 警告（到達不能パターン）を `lint.rs` に追加し、`lint_program` に統合。

```favnir
// OR パターン（新規追加）
match result {
  Ok(x) | Err("retry") -> retry(x)
  Err(e) -> fail(e)
}

// 既存のガード節との組み合わせ
match order.status {
  "pending" if order.amount > 1000.0 -> process_large(order)
  "pending" -> process_small(order)
  _ -> ignore(order)
}
```

**完了条件**: Rust テスト 3 件（ベース 3235 + 3 = 3238 tests passed, 0 failed）
- `match_or_pattern`
- `match_or_with_guard`
- `w037_unreachable_after_wildcard`

**実績**: 3235 + 3 = 3238 tests passed, 0 failed（2026-07-26）**COMPLETE**

---

### v56.6.0 — パターンエイリアス（as-patterns `@`）

`pattern @ sub-pattern` 構文（as-pattern）を parser に追加（`PatternAs` AST ノード — 新規追加）。
checker でバインディング変数のスコープを正しく管理。

```favnir
// @ でサブパターンに名前を付ける
match orders {
  [head @ { id, amount } | tail] -> {
    log("Processing order: " + id)
    process(head)
  }
  [] -> done()
}
```

**完了条件**: Rust テスト 2 件（ベース 3238 + 2 = 3240 tests passed, 0 failed）
- `pattern_alias_binds_whole`
- `pattern_alias_with_destructure`

**実績**: 3238 + 2 = 3240 tests passed, 0 failed（2026-07-26）**COMPLETE**

---

### v56.7.0 — モジュール名前空間（qualified imports）

`import "path" as alias.*` ワイルドカードインポートを追加（名前空間展開）。
`stages.validate.run` のような深い qualified アクセスを resolver で正式サポート。
W038 警告（ワイルドカードインポートによる名前衝突）を lint に追加。

```favnir
import "./stages" as stages

stages.validate.run(order)
stages.transform.apply(data)

// ワイルドカードインポート
import "./stages/validate" as validate.*
run(order)   // validate.run を直接参照
```

**完了条件**: Rust テスト 3 件（ベース 3240 + 3 = 3243 tests passed, 0 failed）
- `qualified_import_deep_access`
- `wildcard_import_expands`
- `w038_wildcard_import_collision_warning`

**実績**: 3240 + 3 = 3243 tests passed, 0 failed（2026-07-26）**COMPLETE**

---

### v56.8.0 — ドキュメントサイト Language Power 2.0 記事

`site/content/docs/language/bounded-generics.mdx` — `where T: Interface` 本番品質化・coherence ルール。
`site/content/docs/language/row-polymorphism.mdx` — 行多相レコードの実用拡張・LSP 表示。
`site/content/docs/language/effect-inference.mdx` — エフェクト推論 inlay hints・注釈省略の使い方。

**完了条件**: Rust テスト 3 件（ベース 3243 + 3 = 3246 tests passed, 0 failed）
- `docs_bounded_generics_page_exists`
- `docs_row_poly_page_exists`
- `docs_effect_inference_updated`

**実績**: 3243 + 3 = 3246 tests passed, 0 failed（2026-07-26）**COMPLETE**

---

### v56.9.0 — 安定化・コードフリーズ（Language Power 2.0 前調整）

全 lint / clippy クリーン確認。`site/content/docs/language-power2-overview.mdx` 骨子作成。
v56.1〜v56.8 の全テストが通過していることを確認して v57.0 へ。

**完了条件**: Rust テスト 2 件（ベース 3246 + 2 = 3248 tests passed, 0 failed）
- `cargo_toml_version_is_56_9_0`
- `language_power2_overview_exists`

**実績**: 3246 + 2 = 3248 tests passed, 0 failed（2026-07-26）**COMPLETE**

---

### v57.0.0 — Language Power 2.0 宣言 ★クリーンアップ

**宣言文**:

> 「ジェネリクスは制約で安全に縛られ、レコードは行変数で柔軟に合成され、
>  エフェクトは推論によって自然に表れる。
>  パターンはガード節と OR 構文で表現力を増し、モジュールは名前空間で整理される。
>  Favnir の型システムは開発者の意図を正確に表現できる。
>
>  これが Favnir v57.0 — Language Power 2.0 の姿である。」

**完了条件**:
- v56.1〜v56.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ **3250**）
- `v57000_tests` 4 件 pass（ベース 3248 + 4 = 3252 tests passed, 0 failed）:
  - `cargo_toml_version_is_57_0_0`
  - `changelog_has_v57_0_0`
  - `milestone_has_language_power2`
  - `readme_mentions_language_power2`
- `MILESTONE.md` に `"Language Power 2.0"` 宣言文エントリを追加する
- `★クリーンアップ`（`cargo clean`）完了

**実績**: 3248 + 4 = 3252 tests passed, 0 failed（2026-07-26）— **COMPLETE**

---

## 参考リンク

- マスターロードマップ: `versions/roadmap/roadmap-v55.1-v60.0.md`
- 前サブスプリント: `versions/roadmap/roadmap-v55.1-v56.0.md`
- 達成宣言: `MILESTONE.md`
