# v16.5.0 Spec — 型エイリアス（Type Alias）

Date: 2026-06-14

---

## 概要

`alias Email = String` で「ドキュメント目的の型別名」を宣言できるようにする。
型チェック時にエイリアスを元の型に展開するため、エイリアス型は元の型と完全に交換可能。
実行時コストゼロ（コンパイル後の IR には型情報が残らない）。

---

## `alias` vs `type` の違い

| | `alias Email = String` | `type Email(String)` |
|---|---|---|
| 互換性 | `String` と交換可能 | 区別される（`Email.unwrap()` が必要） |
| 目的 | ドキュメント・可読性 | ビジネスルール強制 |
| `where` バリデーター | 不可 | 可能 |
| 実行時コスト | ゼロ | ゼロ（ラッパー展開）|

---

## 構文

### 基本

```fav
alias Email      = String
alias Timestamp  = String
alias RowId      = Int
alias JsonStr    = String
```

### ジェネリック

```fav
alias Result2<T>     = Result<T, String>
alias Rows<T>        = List<T>
```

### 使用例

```fav
-- シグネチャが読みやすくなる
fn parse_email(s: Email) -> Result2<Email> {
    if String.contains(s, "@") {
        Result.ok(s)
    } else {
        Result.err("invalid email")
    }
}

-- エイリアス同士は元の型と交換可能
fn send(addr: Email) -> Result<Unit, String> { Result.ok(()) }
bind e: String <- "alice@example.com"
send(e)   -- OK: Email = String は同じ型
```

---

## 実装仕様

### 1. Lexer / Parser（parser.rs）

`alias` キーワードの追加:
- `alias Name = TypeExpr` を `TopLevel::AliasDecl` としてパース
- `alias Name<T, U> = TypeExpr<T, U>` のジェネリックパラメータ対応
- `TypeParam` は既存の型パラメータパース (`parse_type_params`) を流用

```
AliasDecl ::= "alias" Ident TypeParams? "=" TypeExpr
TypeParams ::= "<" Ident ("," Ident)* ">"
```

### 2. AST（ast.rs）

`TopLevel` enum に追加:

```rust
AliasDecl {
    name: String,
    params: Vec<String>,  // ジェネリックパラメータ名
    ty: TypeExpr,
    span: Span,
}
```

### 3. 型チェッカー（checker.rs）

チェック時の処理:

1. **収集フェーズ**: プログラム先頭で全 `AliasDecl` を走査し `alias_env: HashMap<String, (Vec<String>, Type)>` に登録
2. **展開フェーズ** (`resolve_alias`): 型解決時に `Named(name, args)` が `alias_env` にあれば展開
   - 非ジェネリック: `alias Email = String` → `Named("Email", [])` → `Type::String`
   - ジェネリック: `alias Result2<T> = Result<T, String>` + 呼び出し `Result2<Int>` → `Result<Int, String>` に展開
3. **型チェック**: 展開後の型で通常の型チェックを行う（型エラーはエイリアス名ではなく展開後の型で報告）

**注意事項:**
- 循環エイリアス（`alias A = B, alias B = A`）は展開深度制限（10回）で検出しエラー
- `type Name(Inner)` との共存: `AliasDecl` と `TypeDecl` は別テーブルで管理

### 4. Compiler（compiler.rs）

`AliasDecl` はコンパイル時に完全に無視（IRGlobal への登録なし）。
型情報はチェック完了後に捨てられるため、バイトコードへの影響ゼロ。

### 5. その他のファイル

- `lineage.rs`: `Expr::*` に新ノードなし → 変更不要
- `lint.rs`: 同上
- `wasm_codegen.rs`: 同上
- `driver.rs`: `format_expr_compact` / `expr_to_sql` → 変更不要

---

## エラーコード

| コード | 説明 |
|---|---|
| E0329 | 循環エイリアス定義 |

---

## テスト（v165000_tests — 5 件）

| # | テスト名 | 確認内容 |
|---|---|---|
| 1 | `version_is_16_5_0` | Cargo.toml バージョンが 16.5.0 |
| 2 | `alias_basic` | `alias Email = String` を使った関数がコンパイル・実行される |
| 3 | `alias_interchangeable` | `alias UserId = Int` — `Int` 引数として渡せる |
| 4 | `alias_generic` | `alias Result2<T> = Result<T, String>` を使った関数が動作する |
| 5 | `alias_in_signature` | エイリアスを引数型・戻り型に使った fn が正常に動作する |

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "16.5.0"` | [ ] |
| `alias Email = String` が解析・型解決される | [ ] |
| エイリアス型は元の型と交換可能（型エラーなし） | [ ] |
| ジェネリックエイリアス `alias Result2<T> = Result<T, String>` が動作する | [ ] |
| エイリアスを引数型・戻り型に使った関数が正常に動作する | [ ] |
| `type Name(Inner)` との共存に問題がない | [ ] |
| `cargo test v165000` 全テストパス（5/5） | [ ] |
| `cargo test` 全件パス（リグレッションなし） | [ ] |
