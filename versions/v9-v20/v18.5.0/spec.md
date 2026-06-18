# v18.5.0 仕様書 — 線形型（Linear Types）によるリソース安全性

## 概要

線形型（Linear Types）は「値がちょうど 1 回だけ使われる」ことをコンパイル時に保証する型システム機能。
接続・ファイルハンドル・トランザクションの「使い忘れ」「二重クローズ」をランタイムエラーではなくコンパイルエラーとして検出する。

---

## 構文

### 線形関数型 `T -o U`

```fav
// -o は「linear arrow」: 引数をちょうど 1 回消費する
fn with_connection<T>(f: Connection -o Result<T, String>) -> Result<T, String> !Db {
  bind conn <- Postgres.connect()
  f(conn)
}
```

通常の `->` との違い:

| Arrow | 意味 |
|---|---|
| `T -> U` | 通常の関数型（引数を何回でも使える） |
| `T -o U` | 線形関数型（引数をちょうど 1 回消費する） |

### 組み込み線形型

v18.5.0 では以下の 2 型を組み込み線形型として定義:

| 型名 | 意味 |
|---|---|
| `Connection` | Postgres 接続ハンドル |
| `Tx` | Postgres トランザクション |

### 使用例

```fav
// OK: conn を 1 回だけ使う
fn do_query() -> Result<Int, String> !Db {
  with_connection(|conn| {
    bind rows <- Postgres.query_with_conn(conn, "SELECT COUNT(*) FROM users", [])
    Result.ok(rows)
  })
}

// E0332: conn を 2 回使う（二重消費）
fn wrong_double() -> Result<Int, String> !Db {
  bind conn <- Postgres.connect()
  bind _    <- Postgres.query_with_conn(conn, "SELECT 1", [])
  bind _    <- Postgres.query_with_conn(conn, "SELECT 2", [])  // E0332
  Result.ok(0)
}

// E0333: conn を使わずに捨てる（未消費）
fn wrong_unused() -> Result<Int, String> !Db {
  bind conn <- Postgres.connect()
  Result.ok(0)  // E0333: conn が未消費のまま関数を終了
}

// トランザクション安全性
fn transact<T>(f: Tx -o Result<T, String>) -> Result<T, String> !Db {
  bind tx <- Postgres.begin()
  match f(tx) {
    ok(v)  => Result.ok(v)
    err(e) => Result.err(e)
  }
}
```

---

## エラーコード

| コード | 意味 | 検出タイミング |
|---|---|---|
| `E0332` | 線形変数の二重消費（2 回以上使おうとした） | コンパイル時 |
| `E0333` | 線形変数の未消費（関数終了時に未使用が残っている） | コンパイル時 |

---

## 線形型ルール

| 操作 | 通常型 | 線形型 |
|---|---|---|
| 同じ変数を 2 回参照 | OK | E0332（二重消費） |
| 使わずに捨てる | OK | E0333（未消費） |
| 関数に渡す | コピー渡し | 移動（以後使用不可） |
| 別の変数に bind | OK | 移動（元の変数は無効化） |

---

## AST 変更

### `TypeExpr::LinearArrow(Box<TypeExpr>, Box<TypeExpr>, Span)`

`T -o U` を表す新しい TypeExpr variant。

```rust
pub enum TypeExpr {
    // 既存
    Arrow(Box<TypeExpr>, Box<TypeExpr>, Span),   // T -> U
    // v18.5.0 新規
    LinearArrow(Box<TypeExpr>, Box<TypeExpr>, Span), // T -o U
}
```

### `Type::LinearFn(Box<Type>, Box<Type>)`

チェッカー内部の型表現。

```rust
pub enum Type {
    // 既存
    Arrow(Box<Type>, Box<Type>),
    // v18.5.0 新規
    LinearFn(Box<Type>, Box<Type>),  // T -o U
}
```

---

## チェッカー変更

### `LinearEnv`

線形変数の使用状況を追跡するマップ。

```
linear_env: HashMap<String, LinearState>

enum LinearState {
    Available,   // まだ消費されていない
    Consumed,    // 消費済み（以後の参照は E0332）
}
```

### 組み込み線形型の登録

```
linear_types: HashSet<String>  // { "Connection", "Tx" }
```

`bind conn <- Postgres.connect()` で `conn` が `Connection` 型と判明したとき、`linear_env` に `(conn, Available)` を追加。

### チェックのタイミング

1. **変数参照時** (`EVar`): `linear_env[name] == Consumed` なら E0332。`Available` なら `Consumed` に変更。
2. **関数終了時**: `linear_env` に `Available` 残りがあれば E0333。
3. **`-o` 関数適用時**: 引数が `Consumed` なら E0332。

---

## 実装範囲（v18.5.0）

- ✅ `T -o U` 構文のパース
- ✅ `TypeExpr::LinearArrow` / `Type::LinearFn` AST
- ✅ 組み込み線形型 `Connection` / `Tx`
- ✅ E0332（二重消費）の検出
- ✅ E0333（未消費）の検出
- ✅ `LinearFn` 型の表示（`fmt.rs` / `driver.rs`）
- ❌ ユーザー定義線形型（`#[linear]` アノテーション）→ v19.x 以降
- ❌ `LinearDrop` VM opcode → v19.x 以降（現在はチェックのみ）
- ❌ match 分岐での線形変数の分岐追跡 → v19.x 以降（複雑なフロー解析）

---

## 完了条件

1. `T -o U` 線形関数型がパースされる
2. 線形変数（`Connection` / `Tx`）を 1 回使うと正常にコンパイルされる
3. 線形変数を 2 回使うと E0332 が出る
4. 線形変数を使わずに関数が終わると E0333 が出る
5. `with_connection` パターンのような線形引数渡しが型チェックされる
