# v44.3.0 Spec — Stream join x Opaque type

## 概要

`opaque type` 同士の誤 join（同じ内部型を持つ複数の opaque type エイリアスが混在し、`Stream.join` キーとして誤用される可能性がある状態）を **AST レベルで検出・解析できる**ようにする。

`Stream.join` 呼び出し内での型レベル結合チェック（checker.fav 統合）は将来版のスコープ。本バージョンは **「同じ内部型を持つ opaque type エイリアスグループの検出」AST レベル MVP** とする。

ロードマップ（`roadmap-v44.1-v45.0.md`）の「Stream.join で OrderId != PaymentOrderId の誤 join を E0413 で検出」は完全な型チェック統合を意味するが、本バージョンでは AST レベルで潜在的な誤 join リスクを報告する前段階の解析を実装する。

---

## AST 確認事項

- `TypeDef.is_opaque: bool` — v43.11.0 追加（`opaque type` キーワード）
- `TypeBody::Alias(TypeExpr)` — エイリアス型の本体
- `TypeExpr::Named(String, Vec<TypeExpr>, Span)` — 3 フィールド（型名、型引数、スパン）
- opaque alias 収集パターン（`check_opaque_coerce_violations` と同様）:
  `TypeDef { is_opaque: true, body: TypeBody::Alias(TypeExpr::Named(inner, [], _)), span, ... }`
- `params.is_empty()` — `TypeExpr::Named` の型引数リストが空（`opaque type Foo = String` は対象、`opaque type Foo = List<String>` は対象外）。`TypeDef.type_params`（型定義パラメータ）とは別物

---

## 機能詳細

### 1. `collect_opaque_alias_groups` ヘルパー追加

`driver.rs` に以下の関数を追加:

```rust
pub fn collect_opaque_alias_groups(src: &str, filename: &str) -> Vec<String>
```

- ソースを parse して `opaque type Name = Inner`（型引数なし）の形式の TypeDef を収集
- 同じ `Inner` 型名を持つ opaque type が 2 件以上ある場合、グループをレポート
- 返り値: `"<filename>:<line>: <inner>: <Name1>, <Name2>[, ...]"` 形式の文字列リスト
  - `<line>` はグループ内最初の opaque type の行番号
  - 名前はアルファベット順にソート（安定した出力のため）

### 2. 検出の意図

`opaque type OrderId = String` と `opaque type PaymentOrderId = String` が同一ファイルに存在する場合、`Stream.join` で `OrderId` と `PaymentOrderId` を比較する誤 join が型システムで防がれるべき。本関数はその「同型 opaque エイリアスグループ」を列挙して開発者が注意できるよう通知する。

---

## テスト

`v44300_tests` 3 件:

| テスト名 | 内容 |
|---|---|
| `cargo_toml_version_is_44_3_0` | `Cargo.toml` に `"44.3.0"` が含まれる |
| `opaque_alias_group_detected` | `opaque type OrderId = String` + `opaque type PaymentOrderId = String` → グループ検出 |
| `non_opaque_type_excluded_from_groups` | `type X = String`（`opaque` なし）→ グループに含まれない |

---

## 完了条件

- `cargo test -j 8 -- --test-threads=8` で **2950 passed; 0 failed**（2947 + 3）
- `v44300_tests` 3 件 pass

---

## 注意事項

- `collect_opaque_alias_groups` は `check_opaque_coerce_violations` と同様の opaque alias 収集パターンを使用
- `params.is_empty()` チェック必須（`opaque type Box<T> = ...` のようなパラメータ付きは対象外）
- グループ内の名前はアルファベット順ソートで出力を安定させる
- `collect_cep_expr_refinement_refs` の直後（`bare_inner_literal_line` の直前）に配置
- ロードマップ推定（2940）は旧見積もり。実績 2947 を基準とする
- `v44200_tests::cargo_toml_version_is_44_2_0` をスタブ化すること
- checker.fav 統合（E0413 での誤 join 検出）は将来版のスコープ
