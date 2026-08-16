# v71.3.0 Spec — Phantom Types（型タグによる誤使用防止）

Date: 2026-08-09
Status: 計画中

---

## Background

v71.2.0 で Refined Types（`where self` 制約）を実装した。v71.3.0 では Type System 2.0 フェーズの第 3 弾として、同じ内部型を持つ異なる意味の値を型レベルで区別する **Phantom Types** を実装する。

`type UserId = phantom String` のように `phantom` キーワードを使うことで、`UserId` と `OrderId` は両方 `String` を内部に持つが、互いに代入・混用できない別型として扱われる。

**実装概要**:
- パーサー: `type Name = phantom InnerType` 構文を追加（`is_phantom: bool` フィールドを AST に追加）
- チェッカー: phantom 型のコンストラクタ `UserId: Fn([String], UserId)` を env に登録（type_aliases には追加しない）
- 型の混用エラーは既存の E0218（cannot unify）が自然に発火する

---

## Goals

1. **`phantom_type_prevents_id_confusion`** — `UserId` を要求する関数に `OrderId` を渡すとコンパイルエラーになることを確認
2. **`phantom_type_explicit_cast`** — `UserId("u-123")` が typecheck で通り、`UserId` 型の値として使えることを確認
3. テスト 2 件追加（3589 → 3591）

---

## Syntax / API Examples

```favnir
// 型タグ宣言
type UserId  = phantom String
type OrderId = phantom String

// コンストラクタで生成
fn get_user(id: UserId) -> Bool { true }

// OK: UserId コンストラクタで明示的に構築
fn good() -> Bool { get_user(UserId("u-123")) }

// コンパイルエラー: OrderId ≠ UserId
fn bad() -> Bool { get_user(OrderId("x")) }
```

---

## 実装スコープ

v71.3.0 は最小実装（2 テスト）。

### 1. AST — `is_phantom: bool` フィールド追加

`fav/src/ast.rs` の `TypeDef` 構造体に:
```rust
pub is_phantom: bool,  // v71.3.0: phantom type キーワード（デフォルト false）
```

### 2. パーサー — `phantom` 文脈キーワード

`parse_type_def` の alias body 解析部分（`self.expect(&TokenKind::Eq)?` 後）に追加:

```rust
// v71.3.0: phantom type: `type Name = phantom InnerType`
if matches!(self.peek(), TokenKind::Ident(n) if n == "phantom") {
    self.advance(); // consume "phantom"
    let inner = self.parse_type_expr()?;
    return Ok(TypeDef {
        visibility, name, type_params, with_interfaces,
        invariants: vec![], is_opaque: false, is_phantom: true,
        body: TypeBody::Alias(inner),
        span: self.span_from(&start),
    });
}
```

全 `TypeDef` 初期化箇所に `is_phantom: false` を追加する。

### 3. チェッカー — コンストラクタ登録

`register_item_signatures` の `TypeBody::Alias` ブランチで、`is_phantom` の場合:
- `type_aliases` に追加しない（透過解決させない）
- コンストラクタを env に登録: `env.define(name, Fn([inner_ty], Named(name, [])))`
- pre-pass（alias invariants 登録）でも `is_phantom` を除外する

### 4. fmt.rs — phantom 型のフォーマット

`TypeBody::Alias` で `td.is_phantom == true` の場合、`type Name = phantom Inner` 形式で出力する。

---

## Error Codes

新しいエラーコードは不要。型の混用（`OrderId` を `UserId` に渡す）は既存の E0218（cannot unify）で検出される。

---

## Success Criteria

- [ ] `phantom_type_prevents_id_confusion`: OrderId を UserId 引数に渡すとエラーになる
- [ ] `phantom_type_explicit_cast`: `UserId("u-123")` が typecheck で通る（errors.is_empty()）
- [ ] `cargo test v713000` で 2 件 pass
- [ ] `cargo test` 全体で 3591 tests pass（0 failures）

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/ast.rs` | `TypeDef` に `is_phantom: bool` 追加 |
| `fav/src/frontend/parser.rs` | `phantom` 文脈キーワード + 全 TypeDef 初期化に `is_phantom: false` |
| `fav/src/middle/checker.rs` | `register_item_signatures` に phantom コンストラクタ登録（+ pre-pass から除外） |
| `fav/src/fmt.rs` | phantom 型の `type Name = phantom Inner` 出力 |
| `fav/src/driver.rs` | `v713000_tests` モジュール + version 文字列更新 |
| `fav/Cargo.toml` | `version` を `"71.2.0"` → `"71.3.0"` |
| `CHANGELOG.md` | v71.3.0 エントリ追加 |
| `versions/current.md` | 進行中バージョンを v71.3.0 に更新 |

### 事前確認事項

- `phantom` がレキサー/パーサーで未定義であることを確認（識別子として扱われている）
- `TypeDef.is_opaque` と同じパターンで `is_phantom` を追加できること
- `type UserId(String)` 形式（Wrapper）とは別実装であること（Wrapper は Rust チェッカーでコンストラクタ未登録）

### スコープ外として許容する事項

- **`ast_lower_checker.rs`**: `lower_type_def` の `TypeBody::Alias` ブランチは `is_phantom` を認識しない。セルフホストパス（compiler.fav / checker.fav）では phantom 型の型安全が強制されない。v71.3.0 は Rust チェッカー（`Checker::check_program`）でのテストのみを対象とし、セルフホスト対応は将来バージョンで行う。
- **`emit_python.rs`**: `TypeBody::Alias` は `Sum | Alias | Wrapper` アームで `# TODO: ...` コメントになる。phantom 型も同じ扱い。Python トランスパイル時に phantom 型が消えるのは許容範囲（v71.3.0 スコープ外）。
- **テスト数**: ロードマップ記載の `3585 + 2 = 3587` は v71.2.0 完了前の古い値。現行テスト数は 3589 であり、`3589 → 3591` が正しい。
