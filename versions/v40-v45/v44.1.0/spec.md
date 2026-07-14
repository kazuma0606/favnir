# v44.1.0 Spec — Refinement type x Streaming 統合

## 概要

Refinement type（`type PositiveFloat = Float where |v| v > 0.0`）がストリーム要素型として型注釈・伝播されることを、**AST レベルで検証・解析できる**ようにする。

checker.fav への完全統合（`List.filter` の絞り込みによる refinement 型の推論）は将来版のスコープ。本バージョンは **パーサー受容 + AST レベル MVP** とする。

---

## 機能詳細

### 1. パーサー受容確認

以下の構文が既存パーサーで正しく解析されることを確認する:

```favnir
type PositiveFloat = Float where |v| v > 0.0

stage Validate {
  bind valid: Stream<PositiveFloat> <- events
  bind filtered <- List.filter(events, |e| e.value > 0.0)
}
```

- `TypeDef` に `invariants: Vec<Expr>` が既に存在（`where` 節対応済み）
- `TypeExpr::Named("Stream", [Named("PositiveFloat", [], span)], span)` はパース済み
- `BindStmt.annotated_ty: Option<TypeExpr>` で型注釈を保持

### 2. `collect_refinement_stream_bindings` ヘルパー追加

`driver.rs` に以下の関数を追加:

```rust
pub fn collect_refinement_stream_bindings(src: &str, filename: &str) -> Vec<String>
```

- ソースを parse して `TypeDef` の refinement（`invariants` 非空）を収集
- `TrfDef`（stage）内の `BindStmt` を走査し、型注釈が `Stream<T>` または `List<T>` の形で T が refinement type 名に一致するものをリストアップ
- 返り値: `"<filename>:<line>: <binding_name>: Stream<T>"` 形式の文字列リスト

### 3. checker.fav 統合（スコープ外）

`List.filter` の述語から refinement type を推論し `Stream<PositiveFloat>` 型を自動付与する機能は将来版のスコープ（現バージョンでは型注釈 `bind valid: Stream<PositiveFloat>` の形式のみ認識）。

---

## テスト

`v44100_tests` 3 件:

| テスト名 | 内容 |
|---|---|
| `cargo_toml_version_is_44_1_0` | `Cargo.toml` に `"44.1.0"` が含まれる |
| `refinement_type_invariant_in_typedef_ast` | `type PositiveFloat = Float where |v| v > 0.0` の AST に `invariants` が非空であることを確認 |
| `collect_refinement_stream_bindings_detects_annotated_bind` | `bind valid: Stream<PositiveFloat>` が `collect_refinement_stream_bindings` に検出される |

---

## 完了条件

- `cargo test -j 8 -- --test-threads=8` で **2944 passed; 0 failed**（2941 + 3）
- `v44100_tests` 3 件 pass

---

## 注意事項

- ロードマップの推定テスト数（2934）は v43.x スプリント前の古い見積もり。実際の現行テスト数は 2941。
- `annotated_ty` フィールドが `BindStmt` に存在するかどうかを `ast.rs` で事前確認すること（確認済み）。
- `Pattern::Bind(String, Span)` が変数束縛の正しいバリアント名（`Pattern::Ident` は存在しない）。
- `bind x: Type <- expr` は `stage`（TrfDef）の body で動作確認する（`fn` body での受容は未確認のため使用しない）。
- ロードマップ（`roadmap-v44.1-v45.0.md`）の v44.1.0 説明には「checker.fav で検証」と記載されているが、本バージョンは **AST レベル MVP**（パーサー受容 + AST 解析のみ）。checker.fav 統合は将来版のスコープ。
