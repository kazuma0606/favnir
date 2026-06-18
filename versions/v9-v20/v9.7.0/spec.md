# Favnir v9.7.0 Spec

Date: 2026-06-02
Theme: 名目型ラッパー + `where` バリデーション + `with` 自動合成 + `T?`/`T!`/`??`/`expr?` self-hosted 修正

---

## 概要

v9.7.0 では Favnir の型システムを強化する。2 つの独立した柱からなる。

**柱 1 — Bug fix: self-hosted pipeline の `T?` / `T!` / `??` / `expr?` 未対応を解消**

`fav run`（Favnir pipeline）は `compiler.fav` 自前の lexer/parser を使うため、
Rust パーサーが対応している `T?`（Option 型）・`T!`（Result 型）・`??`（null-coalesce 演算子）・
`expr?`（エラー伝播演算子）が `lexer.fav` / `parser.fav` に未実装。
これらを `lexer.fav` / `parser.fav` / `compiler.fav` に追加し、
Rust pipeline と Favnir pipeline の挙動を一致させる。

**柱 2 — 名目型ラッパー + `where` バリデーション + `with` 自動合成**

現在 `type UserId = Int` は型エイリアスであり、`UserId` と `Int` は型チェッカーで区別されない。
`type UserId(Int)` 構文（名目型ラッパー）を導入し、意味的に異なる値を型レベルで区別できるようにする。
`where |v| pred` でバリデーションを型定義に内包し、入口で一度だけ検証する保証を言語レベルで実現する。
`with Eq, Show, Serialize, Deserialize` でボイラープレートを自動合成する。

---

## 設計方針

### T? / T! / ?? / expr? の self-hosted 対応

#### lexer.fav

| トークン | スキャンルール | 先読み |
|---|---|---|
| `TkQuestion` | `?` のみ（次が `?` でない） | 1文字 |
| `TkQuestionQuestion` | `??` | 1文字 |

#### parser.fav — 型パース

| 構文 | 変換 |
|---|---|
| `T?` | `TeOption(T)` （型パース後の後置処理） |
| `T!` | `TeResult(T, TeSimple("String"))` |
| `??` | `BinOp(OpQuestionQuestion, lhs, rhs)` （優先順位は `\|\|` 相当） |

#### compiler.fav — expr?（エラー伝播演算子）

`parser.fav` が `expr?` を `EQuestion(expr)` に変換し、
`compiler.fav` の `compile_expr` で以下に脱糖する。

```
EQuestion(expr)
→ match expr { Ok(v) -> v  Err(e) -> return Err(e) }
```

**E0013 — QuestionOutsideResult**: `Result` を返さない関数内での `?` 使用を検出。

#### ?? 演算子のコード生成

```
a ?? b
→ match a { Some(v) -> v  None -> b }
```

---

### 名目型ラッパー

#### 構文

```favnir
// 基本（where なし）
type UserId(Int)

// バリデーション付き（where あり）
type Percent(Float)   where |v| v >= 0.0 && v <= 100.0
type Email(String)    where |v| String.contains(v, "@")
type NonEmpty(String) where |v| String.length(v) > 0

// with: インターフェース自動合成
type UserId(Int)   with Eq, Show
type Order with Eq, Show, Serialize = { id: Int  item: String  amount: Float }
type Email(String) with Eq, Show  where |v| String.contains(v, "@")
```

#### コンストラクタの型規則

| 定義 | コンストラクタ型 |
|---|---|
| `where` なし | `Inner -> Name`（直接 T を返す） |
| `where` あり | `Inner -> Result<Name, String>` |

```favnir
// where なし: 直接 T
let id = UserId(42)              // UserId

// where あり: bind で unwrap
bind pct <- Percent(50.0)        // OK: Percent
bind pct <- Percent(150.0)       // Err("Percent: validation failed")
bind em  <- Email("a@b.com")     // OK: Email
bind em  <- Email("invalid")     // Err("Email: validation failed")
```

#### パターンマッチでの分解

```favnir
match pct {
  Percent(v) -> v * 0.01
}
```

#### 内部表現

名目型ラッパーはバイトコードレベルで内部型と同一の値を保持（ボックス化なし）。
型チェッカーレベルでのみ区別される（構造的型と名目型の分離）。

---

### with 自動合成

| インターフェース | 合成される関数 |
|---|---|
| `Eq` | `fn eq(a: T, b: T) -> Bool` |
| `Show` | `fn show(t: T) -> String` |
| `Serialize` | `fn to_json(t: T) -> String` |
| `Deserialize` | `fn from_json(s: String) -> Result<T, String>` |

- `with` で自動合成された関数はユーザー定義関数より低優先度
- 未知のインターフェース名は **E0011 — UnknownInterface** でエラー

---

### エラーコード一覧（v9.7.0 追加分）

| コード | 名前 | 説明 |
|---|---|---|
| E0010 | WrapperTypeMismatch | `Name(x)` の `x` 型が内部型 `Inner` と不一致 |
| E0011 | UnknownInterface | `with` で指定したインターフェースが未定義 |
| E0013 | QuestionOutsideResult | `expr?` が `Result` を返さない関数内で使用された |

---

## ファイル構成

### 変更ファイル（Rust — パーサーのみ）

| ファイル | 変更内容 |
|---|---|
| `src/ast.rs` | `WrapperDef { name, inner_ty, validator, with_impls }` 追加、`Item::Wrapper` 追加 |
| `src/frontend/parser.rs` | `type Name(Inner)` / `where \|v\| pred` / `type T with ...` パース追加 |
| `src/fmt.rs` | `Item::Wrapper` の fmt 対応 |
| `src/middle/ast_lower_checker.rs` | `Item::Wrapper` の lower 対応 |

### 変更ファイル（Favnir self）

| ファイル | 変更内容 |
|---|---|
| `fav/self/lexer.fav` | `TkQuestion` / `TkQuestionQuestion` トークン追加、スキャンルール追加 |
| `fav/self/parser.fav` | `T?` / `T!` / `??` / `EQuestion` パース追加 |
| `fav/self/compiler.fav` | `EQuestion` 脱糖 / `??` コード生成 / 名目型コード生成 / `with` 自動合成 |
| `fav/self/checker.fav` | 名目型型チェック / E0010 / E0011 / E0013 追加 |

### 変更なし

- `src/backend/vm.rs` — 変更なし
- `src/backend/compiler.rs` — 変更なし
- `runes/` — 変更なし

---

## 完了条件

| 条件 | |
|---|---|
| `T?` / `T!` / `??` が `fav run`（Favnir pipeline）で正しく動作する | |
| `fav check` と `fav run` の挙動が `T?` に関して一致する | |
| `expr?` が `Result` を返す関数内で使える | |
| E0013 で `Result` 以外の関数内の `?` を検出できる | |
| `type Name(Inner)` がコンストラクタ・パターンマッチで使える | |
| `where` あり型のコンストラクタが `Result<T, String>` を返す | |
| `Percent(150.0)` が `Err("Percent: validation failed")` を返す | |
| `with Eq, Show, Serialize, Deserialize` の自動合成が動作する | |
| E0010 で型の取り違えをコンパイル時に検出できる | |
| E0011 で未知インターフェース名をコンパイル時に検出できる | |
| `checker.fav` self-check 通過 | |
| Bootstrap 検証（bytecode_A == bytecode_B）維持 | |
| 統合テスト 12 件以上通過 | |
| `cargo test` 全件通過（目標 1203 件以上） | |
