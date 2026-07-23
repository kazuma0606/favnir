# Spec: v45.5.0 — 型エイリアス完全化（E0413 opaque alias coerce）

Date: 2026-07-16
Status: TODO

---

## 概要

透明型エイリアス（transparent alias）と不透明型エイリアス（opaque alias）の型チェックを完全化する。

### 現状分析

**透明型エイリアス（`type UserId = Int`）**
- `type_aliases: HashMap<String, TypeExpr>` に登録済み
- `resolve_type_expr` でチェーン展開済み（後述 §1 で確認）
- 互換性チェック済み → **実装完了・テスト追加のみ**

**不透明型エイリアス（`opaque type Token = String`）**
- `register_item_signatures` で `TypeBody::Alias` を処理する際に `td.is_opaque` チェックが存在しない
- そのため `type_aliases` に登録されてしまい、透明型と同じように型解決される（**バグ**）
- `E0413` エントリは `error_catalog.rs` に存在するが、`checker.rs` から発行されていない

---

## §1 — 透明型エイリアスのチェーン解決（現状確認）

`checker.rs` の `resolve_type_expr` は以下のように動作する:
- `TypeExpr::Named(n, _)` → `type_aliases.get(n)` があれば再帰的に `resolve_type_expr` を呼ぶ
- これにより `type A = B`、`type B = Int` の2段チェーンも `Int` まで展開される

つまり `collect_transparent_alias_chain` という独立したヘルパーは不要であり、ロードマップに記載のある「`collect_transparent_alias_chain` ヘルパー」は `resolve_type_expr` の再帰処理で代替されている。

**このバージョンでは新しい `collect_transparent_alias_chain` 関数は実装しない。**
代わりにテストで `type A = B`、`type B = Int` の多段チェーンが正しく動作することを確認する。

---

## §2 — 実装スコープ

### §2.1 — `Checker` に `opaque_alias_inner` フィールド追加

```rust
opaque_alias_inner: HashMap<String, Type>,
```

初期化:
- `Checker::new()` の初期化リスト
- `Checker::new_with_resolver()` の初期化リスト（両方必須）

### §2.2 — `register_item_signatures` 修正

`TypeBody::Alias` 処理で `td.is_opaque` を判定:
- `is_opaque = false` → 従来通り `type_aliases` に登録
- `is_opaque = true` → `type_aliases` に登録**せず**、`resolve_type_expr(inner_te)` の結果を `opaque_alias_inner` に登録

`resolve_type_expr` の戻り値は `Type`（例: `String` inner → `Type::String`）であり、`Type::Named` ではないことに注意。

### §2.3 — `check_fn_def` に E0413 追加

戻り型チェック箇所（`check_fn_def` 内の E0101 発行前）に E0413 チェックを挿入:
1. 期待型が `Type::Named(n, _)` で `opaque_alias_inner` に `n` が存在する
2. 実際の型が inner type と一致する（`body_ty.is_compatible(&inner_ty)` — checker.rs 既存 `is_compatible` メソッドを使用）
3. → E0413 発行して return（E0101 は発行しない）

同様に `check_trf_def` にも追加。

### §2.4 — スコープ外

逆方向チェック（opaque を transparent として使う）は複雑度が高く v45.5.0 スコープ外とする。

---

## エラーコード

| コード | 条件 |
|---|---|
| `E0413` | opaque alias の inner type を直接返している（coerce 禁止） |

---

## テスト

| テスト名 | 内容 | 期待 |
|---|---|---|
| `transparent_alias_compatible` | `type UserId = Int`、`fn f() -> UserId { 42 }` | エラーなし |
| `opaque_alias_incompatible` | `opaque type Token = String`、`fn f() -> Token { "abc" }` | E0413 |

テスト数: 2977 → **2979**

---

## 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/middle/checker.rs` | `opaque_alias_inner` フィールド追加（new/new_with_resolver 両方）、`register_item_signatures` 修正、`check_fn_def` / `check_trf_def` E0413 追加 |
| `fav/Cargo.toml` | version `45.4.0` → `45.5.0` |
| `fav/src/driver.rs` | `v455000_tests` モジュール追加（2件） |
| `CHANGELOG.md` | v45.5.0 エントリ追加 |
| `versions/current.md` | v45.5.0（2979 tests）に更新 |

---

## 変更しないファイル

- `error_catalog.rs` — E0413 エントリ既存
- `ast.rs` — `TypeDef.is_opaque` 既存（parser も `opaque type` キーワード対応済み）
- `lint.rs` — 変更不要
- `compiler.fav` / `checker.fav` — 変更不要
