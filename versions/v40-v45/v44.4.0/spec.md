# v44.4.0 Spec — 型推論 x パイプライン lineage

## 概要

`fav explain --lineage` の出力に推論（または明示注釈）された型情報を付加する前段階として、ステージ（`TrfDef`）内の **型注釈付き `bind` 束縛を lineage エントリとして収集** できるようにする。

`LineageEntry` 構造体への `inferred_types` フィールド追加・`render_lineage_text` への型表示統合は将来版のスコープ。本バージョンは **「型注釈付き bind 束縛のリネージアノテーション収集」AST レベル MVP** とする。

ロードマップの「ウィンドウ・join の lineage も追跡対象に含める」も将来版のスコープ（現バージョンでは TrfDef トップレベルの `bind: Type <- expr` を対象とする）。

---

## AST 確認事項

- `TrfDef.body: Block` — stage 定義の本体
- `Stmt::Bind(BindStmt)` — bind 文
- `BindStmt.annotated_ty: Option<TypeExpr>` — 型注釈（`bind x: T <- expr` の `T`）
- `BindStmt.pattern: Pattern` — `Pattern::Bind(String, Span)` が変数名
- `BindStmt.span: Span` — 行番号
- `format_type_expr(te: &ast::TypeExpr) -> String` — 既存の型表示ヘルパー（`driver.rs` 行 15050 付近）

---

## 機能詳細

### 1. `collect_annotated_lineage_bindings` ヘルパー追加

`driver.rs` に以下の関数を追加:

```rust
pub fn collect_annotated_lineage_bindings(src: &str, filename: &str) -> Vec<String>
```

- ソースを parse して `TrfDef` の body.stmts を走査
- `Stmt::Bind(b)` かつ `b.annotated_ty.is_some()` の束縛を収集
- 返り値: `"<filename>:<line>: <stage_name>: <binding_name>: <type>"` 形式の文字列リスト
- 型表示には既存の `format_type_expr` を使用（全 TypeExpr バリアント対応）

---

## テスト

`v44400_tests` 2 件:

| テスト名 | 内容 |
|---|---|
| `cargo_toml_version_is_44_4_0` | `Cargo.toml` に `"44.4.0"` が含まれる |
| `annotated_lineage_bindings_detected` | `stage Validate: List<Float> -> List<Float> = \|events\| { bind valid: Stream<Float> <- events }` → `"valid: Stream<Float>"` が検出される |

---

## 完了条件

- `cargo test -j 8 -- --test-threads=8` で **2953 passed; 0 failed**（2951 + 2）
- `v44400_tests` 2 件 pass

---

## 注意事項

- `format_type_expr` は `driver.rs` 内のプライベート関数 — 同ファイル内で呼び出し可能
- `Pattern::Bind(name, _)` が変数束縛の正しいバリアント（`Pattern::Ident` は存在しない）
- MVP: TrfDef トップレベル stmts のみ走査（ネスト Block は将来版）
- `LineageEntry` 構造体拡張・`render_lineage_text` 統合は将来版のスコープ
- ロードマップ推定（2942）は旧見積もり。実績 2951 を基準とする
- `v44300_tests::cargo_toml_version_is_44_3_0` をスタブ化すること
