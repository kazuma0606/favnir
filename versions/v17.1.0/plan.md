# v17.1.0 Plan — 境界付きジェネリクス（Bounded Generics）

Date: 2026-06-14

---

## Phase A — Cargo バージョン更新

`fav/Cargo.toml` の `version` を `"17.1.0"` に更新。
`cargo build` → コンパイルエラーなし確認。

---

## Phase B — AST 拡張（GenericParam）

`fav/src/ast.rs` を編集：

- `GenericParam` 構造体追加:
  ```rust
  pub struct GenericParam {
      pub name: String,
      pub bounds: Vec<String>,  // ["Ord", "Serialize"] など
  }
  ```
- 関数定義・stage 定義・`type` 定義の型パラメータを `Vec<String>` → `Vec<GenericParam>` に変更
- `GenericParam` を使っている全 exhaustive match を更新

変更対象フィールド:
- `FnDef.type_params: Vec<String>` → `Vec<GenericParam>`
- `StageDef.type_params: Vec<String>` → `Vec<GenericParam>`
- `TypeDef.type_params: Vec<String>` → `Vec<GenericParam>`（存在する場合）

---

## Phase C — パーサー拡張（`with` キーワード）

`fav/src/frontend/parser.rs` を編集：

- `with` を識別子（ソフトキーワード）として認識
- `parse_generic_params` を拡張:
  ```
  <T>                   → GenericParam { name: "T", bounds: [] }
  <T with Ord>          → GenericParam { name: "T", bounds: ["Ord"] }
  <T with Ord with Serialize> → GenericParam { name: "T", bounds: ["Ord", "Serialize"] }
  <A, B with Eq>        → [GenericParam { name: "A", bounds: [] }, GenericParam { name: "B", bounds: ["Eq"] }]
  ```
- 既存の `<T>` 構文との後方互換を維持

---

## Phase D — 型チェッカー拡張（bound 検査）

`fav/src/middle/checker.rs` を編集：

### 組み込み bound テーブル追加

型が Interface を実装しているかを判定するテーブル:

```rust
fn type_implements_bound(ty: &Type, bound: &str) -> bool {
    match bound {
        "Ord"       => matches!(ty, Type::Int | Type::Float | Type::String),
        "Eq"        => true,  // 全プリミティブ型 + レコード
        "Serialize" => true,  // 全レコード型（簡略実装）
        "Display"   => matches!(ty, Type::String | Type::Int | Type::Float | Type::Bool),
        "Hash"      => matches!(ty, Type::Int | Type::String),
        "Clone"     => true,  // 全値型
        _           => false, // カスタム interface は既存機構で確認
    }
}
```

### `check_bounded_call` 追加

ジェネリクス関数呼び出し時に型引数が各 bound を満たすか検査:
- 型引数を推論して `GenericParam.bounds` を順次チェック
- 違反した場合は E0325 を発行

### E0325 エラー追加

`E0325: Type does not implement Interface`

---

## Phase E — コンパイラ対応（GenericParam 参照箇所の更新）

`fav/src/middle/compiler.rs` を編集：

- `GenericParam` に変更した型パラメータ参照箇所を更新
- 型消去（type erasure）方式: bound は checker が検証済みのため、compiler は bound を無視して通常の generic 関数としてコンパイル

`self/checker.fav` を編集（Favnir 側）：
- `check_bounded_generics` 関数追加（bounds のチェックロジック）

---

## Phase F — exhaustive match 対応

`GenericParam` 変更に伴う exhaustive match 更新:
- `compiler.rs` / `checker.rs` / `driver.rs` / その他 `type_params` を参照している箇所すべて
- `cargo build` でエラーなしを確認

---

## Phase G — テスト追加（v171000_tests）

`fav/src/driver.rs` に `v171000_tests` モジュール追加（5 件）:

```rust
#[test]
fn version_is_17_1_0() {
    let cargo = include_str!("../Cargo.toml");
    assert!(cargo.contains("\"17.1.0\""), "...");
}

#[test]
fn bounded_generic_ord() { /* max<T with Ord> を Int/Float/String で動作確認 */ }

#[test]
fn bounded_generic_serialize() { /* serialize<T with Serialize> が動作 */ }

#[test]
fn bounded_generic_violation() { /* Ord を満たさない型で E0325 確認 */ }

#[test]
fn bounded_generic_multi() { /* T with Ord with Serialize の複数 bound */ }
```

`cargo test v171000` → 5/5 PASS 確認。

---

## Phase H — サイトドキュメント作成

`site/content/docs/language/generics.mdx` を新規作成:
- 境界付きジェネリクスの構文説明
- 組み込み Interface 一覧表
- カスタム `interface` との組み合わせ例
- Before / After 比較
- E0325 エラーの説明

---

## Phase I — 最終確認 + コミット

- `cargo test v171000` → 5/5 PASS 最終確認
- `cargo test` → 全件 PASS（リグレッションなし）
- コミット: `feat: v17.1.0 — 境界付きジェネリクス（Bounded Generics）`

---

## 依存関係

```
A（Cargo）→ G（テスト: version check）
B（AST）→ C（Parser）→ D（Checker）→ E（Compiler）→ F（exhaustive match）
F → G（テスト）
G → H（ドキュメント）→ I（コミット）
```

Phase A は独立して先行可能。
Phase B〜F は順次実施（後続が前段に依存）。
Phase H は Phase G 完了後。

---

## 技術メモ

- **`with` はソフトキーワード**: `Ident("with")` として解析し、型パラメータ文脈でのみ特別扱い。TokenKind::With は不要。
- **型消去方式**: v17.1 では bound を checker で検証し、compiler は bound を無視してコンパイル。実行時にディスパッチは発生しない（静的解決済み）。
- **後方互換**: 既存の `<T>` 構文は `GenericParam { name: "T", bounds: [] }` として表現。bounds が空の場合は従来と同じ動作。
- **カスタム interface**: `interface Scored { ... }` と `fn f<T with Scored>` の組み合わせは、checker の既存 interface 機構（v9.12.0 実装）と bounds チェックを連携させる。
- **`include_str!` パス**: driver.rs からの相対パス: `Cargo.toml` → `"../Cargo.toml"`。
