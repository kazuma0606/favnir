# v18.6.0 実装計画 — 共変・反変アノテーション（Variance）

Date: 2026-06-16

## 実装順序

```
T1: ast.rs — Variance 型追加・GenericParam 拡張・InterfaceDecl 拡張
    ↓
T2: 波及ファイル exhaustive match 修正（cargo build が通るまで）
    ↓
T3: parser.rs — +T / -T 解析
    ↓
T4: checker.rs — E0334 分散違反チェック + サブタイピング拡張
    ↓
T5: v186000_tests 追加（5件）
    ↓
T6: Cargo.toml バージョン更新（18.5.0 → 18.6.0）
    ↓
T7: site/content/docs/language/variance.mdx 作成
```

---

## T1: `fav/src/ast.rs` — 型追加・構造体拡張

### 追加内容

1. `Variance` enum を `GenericParam` の直前に追加:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Variance {
    Covariant,     // +T
    Contravariant, // -T
    Invariant,     // T（デフォルト）
}
```

2. `GenericParam` に `variance` フィールドを追加:

```rust
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<TypeConstraint>,
    pub variance: Variance,   // v18.6.0
}
```

`unbounded()` コンストラクタを更新:
```rust
pub fn unbounded(name: impl Into<String>) -> Self {
    Self { name: name.into(), bounds: vec![], variance: Variance::Invariant }
}
```

3. `InterfaceDecl` に `type_params` フィールドを追加:

```rust
pub struct InterfaceDecl {
    pub visibility: Option<Visibility>,
    pub name: String,
    pub super_interface: Option<String>,
    pub type_params: Vec<GenericParam>,  // v18.6.0 新規
    pub methods: Vec<InterfaceMethod>,
    pub span: Span,
}
```

---

## T2: 波及ファイル修正

`GenericParam` に `variance` フィールドを追加すると、`GenericParam { name, bounds }` の構造体リテラル全箇所でコンパイルエラーが起きる。

`InterfaceDecl` に `type_params` を追加すると、`InterfaceDecl { ... }` の構造体リテラル全箇所でコンパイルエラーが起きる。

### 対処方針

- `GenericParam { name, bounds }` のリテラル → `GenericParam { name, bounds, variance: Variance::Invariant }` に更新
- `InterfaceDecl { ... }` のリテラル → `type_params: vec![]` を追加
- Grep で全ファイルを特定してから一括修正

### 影響ファイル（予想）

- `fav/src/frontend/parser.rs` — `GenericParam { name, bounds }` が多数
- `fav/src/middle/checker.rs` — インターフェース関連処理
- `fav/src/middle/compiler.rs` — 型パラメータ処理
- `fav/src/middle/ast_lower_checker.rs` — AST ローワリング
- `fav/src/driver.rs` — テスト内の構造体リテラル

---

## T3: `fav/src/frontend/parser.rs` — `+T` / `-T` パース

### `parse_generic_params` の拡張

```
'<' → loop {
    variance = if peek('+') { advance; Covariant }
               else if peek('-') { advance; Contravariant }
               else { Invariant }
    name = parse identifier
    bounds = parse_type_bounds() (既存)
    push GenericParam { name, bounds, variance }
    ...
}
```

### `parse_interface_decl` の拡張

`interface Name` の後に `<...>` が続く場合、`parse_generic_params()` で型パラメータを解析して `InterfaceDecl.type_params` にセット。

---

## T4: `fav/src/middle/checker.rs` — 分散チェック実装

### 4-A: `check_interface_variance`

`Item::InterfaceDecl` の処理時に呼び出す:

```
for each type_param in interface.type_params:
  if variance == Covariant:
    for each method in interface.methods:
      if type_param.name appears in input position of method.ty:
        emit E0334
  if variance == Contravariant:
    for each method in interface.methods:
      if type_param.name appears in output position of method.ty:
        emit E0334
```

型シグネチャ中の「入力位置」/ 「出力位置」の判定:
- `TypeExpr::Arrow(a, b)`: `a` は入力、`b` は出力
- `TypeExpr::LinearArrow(a, b)`: `a` は入力、`b` は出力
- `TypeExpr::App(_, args)` / `TypeExpr::Named(_, args)` の中は不変として扱う

### 4-B: `is_subtype_with_variance` / `is_compatible` 拡張

型チェック時に `interface X<+T>` の型引数比較で分散を考慮する。
v18.6.0 ではまず E0334 チェックのみ実装し、サブタイピングは「型エラーを出さない」レベルで対応。
完全なサブタイピング推論は v19.x 以降。

---

## T5: `v186000_tests` 追加（5件）

`fav/src/driver.rs` の末尾に追加:

```rust
mod v186000_tests {
    // version_is_18_6_0
    // variance_covariant_parses
    // variance_contravariant_parses
    // variance_subtype_covariant (型チェックがエラーを出さない)
    // variance_violation_error (E0334)
}
```

---

## T6: バージョン更新

`fav/Cargo.toml`: `18.5.0` → `18.6.0`

`fav/src/driver.rs`: `version_is_18_5_0` に `#[ignore]` 追加

---

## T7: ドキュメント作成

`site/content/docs/language/variance.mdx`:
- 共変・反変の概念説明
- `+T` / `-T` 構文
- Source / Sink パターンの例
- E0334 エラーの説明

---

## 注意事項

### `GenericParam.variance` の後方互換

既存の `parse_type_params()` は `GenericParam { name, bounds }` を返す。
`variance: Variance::Invariant` をデフォルト値として追加するだけなので、
既存の型チェック挙動は変わらない。

### `InterfaceDecl.type_params` の後方互換

既存の `InterfaceDecl` 構文（`interface Foo { ... }` 型パラメータなし）は
`type_params: vec![]` で従来通り動作する。

### E0334 の検出粒度

v18.6.0 では「同じ名前の型変数が入力・出力のどちらに現れるか」のみチェック。
ネストした `interface X<+T>` の `App(X, [T])` 内での共変・反変の伝播は v19.x 以降。

### `-T` と `Minus` トークンの競合

型パラメータ位置（`<` の直後、カンマの直後）の `-` は反変アノテーションとして扱う。
式の位置の `-` は引き続き `Minus`（減算）として扱う。
パーサーが文脈（generic params parse vs expression parse）で自動的に判別する。
