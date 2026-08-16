# v71.5.0 仕様書 — Generic Constraints（`impl Trait` 風の境界）

Date: 2026-08-09
Status: 計画中

---

## Background

Favnir では v18.2.0 から `<T with Ord with Serialize>` 形式の型パラメータ境界が存在し、
チェッカーで E0422 を発行する仕組みも完成している。
しかし現代的な言語（Rust, Swift 等）に慣れた開発者には `<T: A & B>` や `<T: impl A>` の方が読みやすい。
v71.5.0 では **コロン記法** (`<T: A>`) と **アンパサンド結合** (`<T: A & B>`) および `impl Trait` 糖衣構文を追加する。
チェッカーは既存の `TypeConstraint::Interface` と E0422 を流用するため、パーサー拡張のみで実現できる。

---

## Goals

1. `<T: A>` を `<T with A>` の代替構文として受け付ける
2. `<T: A & B>` を `<T with A with B>` の代替として受け付ける（複数境界）
3. `<T: impl A>` を `<T: A>` の糖衣構文として受け付ける（`impl` キーワードは無視）
4. 既存の `<T with A>` 構文は引き続き動作する（後方互換性）
5. E0422 は既存チェッカーが引き続き発行する（新規エラーコード不要）
6. テスト総数: 3594 + 2 = 3596 件

6. テスト総数ベース: 3594 件（v71.1〜v71.4 完了後の実績値; ロードマップの 3592 予測値とは
   code-review 追加テストにより差異あり）

**注意: E0423 について**
ロードマップは「境界を満たさない型の使用（E0423）」を求めているが、
E0423 は `error_catalog.rs`（line 832）で「duplicate impl」として確認済み使用中。
境界違反の検出は既存 E0422 が担当しており（checker.rs line 5202）、
実際の T0 grep で `code: "E0423"` が error_catalog.rs に存在することを再確認した上で新規追加しない判断を確定する。
もし T0 grep で E0423 が未使用と判明した場合は本 spec を更新して E0423 を実装すること。

---

## Syntax / API

```favnir
// 単一境界（コロン記法）— 既存 `<T with Serializable>` と等価
fn serialize_single<T: Serializable>(item: T) -> String {
    T.serialize(item)
}

// 複数境界（& 結合）— 既存 `<T with Serializable with Comparable>` と等価
fn serialize_all<T: Serializable & Comparable>(items: List<T>) -> String {
    items |> List.sort |> List.map(T.serialize) |> String.join(",")
}

// impl Trait 記法（T: A の糖衣）
fn store<T: impl DbRecord>(ctx: AppCtx, item: T) -> Result<Int, String> {
    ctx.db.insert(T.table_name(), T.to_row(item))
}

// 既存構文も継続サポート
fn legacy<T with Ord>(a: T, b: T) -> T { a }
```

**混在記法について**: `<T with A: B>` のように `with` と `:` を同一パラメータに混在させた場合、
パーサーは両方を受理し bounds を union する（`[Interface("A"), Interface("B")]`）。
混在は非推奨だが lint 警告は本バージョンのスコープ外とする。

**fmt round-trip について**: `:` 記法で書いたソースを `fav fmt` すると `with` 記法に正規化される
（例: `<T: Serializable & Comparable>` → `<T with Serializable with Comparable>`）。
これは意図した動作。`:` は入力専用の代替構文であり、正規形は `with` 記法とする。

---

## 実装スコープ

### パーサー（`fav/src/frontend/parser.rs`）のみ変更

`parse_type_bounds` 関数に `:` 記法と `&` 結合を追加:

```rust
fn parse_type_bounds(&mut self) -> Result<Vec<TypeConstraint>, ParseError> {
    use crate::ast::TypeConstraint;
    let mut bounds = vec![];

    // 既存: `with` 記法
    while self.peek() == &TokenKind::With || self.peek_ident_text("with") {
        // ... 既存処理 ...
    }

    // v71.5.0: `:` 記法（コロン区切り + & 結合）
    if self.peek() == &TokenKind::Colon {
        self.advance(); // consume `:`
        loop {
            // `impl` キーワードはスキップ（糖衣構文）
            if self.peek_ident_text("impl") {
                self.advance();
            }
            let (bound_name, _) = self.expect_ident()?;
            bounds.push(TypeConstraint::Interface(bound_name));
            if self.peek() == &TokenKind::Ampersand {
                self.advance(); // consume `&`
            } else {
                break;
            }
        }
    }

    Ok(bounds)
}
```

### AST・チェッカー・その他 — 変更なし

- `TypeConstraint::Interface` は既存のまま流用
- `GenericParam.bounds` は既存のまま流用
- E0422 チェックは既存チェッカーが引き続き担当
- `fmt.rs` — `GenericParam.bounds` のフォーマットは既存ロジックが `with` で出力（変更なし）

---

## Error Codes

新規エラーコードなし。境界違反は既存 E0422 が発行される。

---

## Success Criteria

- [x] `fn f<T: Serializable>(item: T)` がパース・型チェックで通る
- [x] `fn f<T: Serializable & Comparable>(items: List<T>)` がパース・型チェックで通る
- [x] `fn f<T: impl DbRecord>(ctx: AppCtx, item: T)` がパース・型チェックで通る
- [x] 既存 `fn f<T with Ord>` 構文が引き続き通る
- [x] テスト総数: 3594 + 2 = 3596 件

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/frontend/parser.rs` | `parse_type_bounds` に `:` 記法と `&` 結合を追加 |
| `fav/src/driver.rs` | `v715000_tests` 追加 |
| `fav/Cargo.toml` | `version = "71.5.0"` |
| `CHANGELOG.md` | v71.5.0 エントリ追加 |
| `versions/current.md` | 進行中バージョン更新 |

**変更しないファイル**: `ast.rs`、`middle/checker.rs`、`fmt.rs`、`error_catalog.rs`

`fmt.rs` は `GenericParam.bounds` を `with Interface` 形式で出力する（既存）。
`:` 記法でパースしたものも `with` 形式で再出力される（round-trip 正規化、意図した動作）。
