# Favnir v1.2.0 仕様書 — `invariant` + `std.states` ルーン

作成日: 2026-05-06

> **テーマ**: 型にビジネスルールを埋め込み、バリデーションを型選択に変える
>
> **設計ドキュメント**: `dev/post-v1/roadmap/fav-standard-states.md`、`dev/post-v1/roadmap/fav-db-schema-integration.md`

---

## 1. スコープ概要

| Phase | テーマ | Done definition |
|---|---|---|
| 0 | バージョン更新 | `v1.2.0` がビルドされ、HELP テキストに反映される |
| 1 | `invariant` 構文 — パーサー拡張 | `type T { field: Type  invariant <expr> }` がパースできる |
| 2 | 型検査統合 | invariant 式が Bool 型か検査される（E045）|
| 3 | コンストラクタ自動生成 + VM | `TypeName.new(...)` が `T!` を返し、invariant を実行時チェックする |
| 4 | `std.states` ルーン | `PosInt` / `Email` など 8 型が `use std.states.*` で利用できる |
| 5 | `fav explain` invariant 表示 | `fav explain` で型の invariant 一覧が表示される |
| 6 | DB スキーマ CHECK 出力 | `fav explain --schema` で invariant が SQL `CHECK` 制約として出力される |
| 7 | テスト・ドキュメント | 新規テストが全通過、langspec.md 更新 |

---

## 2. Phase 0 — バージョン更新

### 変更内容

- `Cargo.toml`: `version = "1.2.0"`
- `main.rs`: HELP テキスト `v1.2.0`
- `versions/v1.2.0/spec.md`: 本ファイル

---

## 3. Phase 1 — `invariant` 構文 — パーサー拡張

### 新規構文

#### `invariant` in type body

```fav
type Email {
    value: String
    invariant String.contains(value, "@")
}

type PosInt {
    value: Int
    invariant value > 0
}

type UserAge {
    value: Int
    invariant value >= 0
    invariant value <= 150
}
```

- `invariant <expr>` は型ブロック内でフィールド宣言の後に記述する
- 複数の `invariant` を並べると全ての条件が AND で評価される
- `<expr>` 内でフィールド名を直接参照できる（スコープは型のフィールドセット）
- `invariant` はフィールド宣言の後にのみ記述可（フィールドより前は E046）

#### `bind` の型注釈によるコンストラクタ自動挿入

```fav
-- bind x: StateName <- value  は  chain x <- StateName.new(value)  に展開される
bind age: PosInt <- 25      -- chain age <- PosInt.new(25) と等価
bind email: Email <- input  -- chain email <- Email.new(input)  と等価
```

この展開は関数が `T!` を返す chain コンテキストで動作する。

### パーサー変更点（`frontend/parser.rs`）

- `invariant` を新規キーワードとして `lexer.rs` に追加
- `parse_type_def` で `Token::Invariant` を検出したとき `parse_invariant_expr` を呼ぶ
  - フィールド宣言の後にのみ受理（それ以前なら E046 相当の ParseError）
  - `invariant <expr>` の `<expr>` は既存の `parse_expr` で処理

### AST ノード（`ast.rs`）

`TypeDef` に `invariants: Vec<Expr>` フィールドを追加:

```rust
pub struct TypeDef {
    pub visibility:      Option<Visibility>,
    pub name:            String,
    pub type_params:     Vec<String>,
    pub with_interfaces: Vec<String>,   // v1.1.0 で追加済み
    pub body:            TypeBody,      // TypeBody::Record(Vec<Field>) or Sum(...)
    pub invariants:      Vec<Expr>,     // 追加: invariant 式のリスト
    pub span:            Span,
}
```

フィールドへのアクセスは `TypeBody::Record(fields)` へのパターンマッチで行う。
`invariant` は Record 型にのみ意味を持ち、Sum 型（ADT）には E046 相当のエラーとする（v1.2.0 では Record 型のみサポート）。

---

## 4. Phase 2 — 型検査統合

### invariant の型検査ルール

1. **Bool 型チェック**: `invariant <expr>` の式は必ず `Bool` 型でなければならない（E045）
2. **フィールド参照チェック**: `<expr>` 内で参照する識別子は同型のフィールド名であること（未定義なら E002）
3. **effect 制約**: `invariant` 式の中では effect を発生させる呼び出しは禁止（Pure のみ）

### 新規エラーコード

| コード | 内容 | 例 |
|---|---|---|
| E045 | `invariant` 式の型が `Bool` でない | `invariant value + 1`（Int 型） |

### 型検査実装方針

`check_type_def` の末尾で全 `invariants` を `check_expr` で型検査:

```rust
fn check_type_def(&mut self, def: &TypeDef) -> Vec<TypeError> {
    let mut errors = self.check_type_def_fields(def);  // 既存処理

    if def.invariants.is_empty() { return errors; }

    // Record 型のフィールドをスコープに追加して invariant 式を型検査
    let fields = match &def.body {
        TypeBody::Record(fields) => fields,
        TypeBody::Sum(_) => {
            errors.push(TypeError::new(E045_SCOPE, ...)); // Sum 型には invariant 不可
            return errors;
        }
    };
    self.with_field_scope(fields, |checker| {
        for inv_expr in &def.invariants {
            let ty = checker.check_expr(inv_expr);
            if ty != Type::Bool {
                errors.push(TypeError::new(E045, inv_expr.span()));
            }
        }
    });
    errors
}
```

---

## 5. Phase 3 — コンストラクタ自動生成 + VM

### `.new()` コンストラクタの自動生成

`invariants` が 1 つ以上ある `TypeDef` に対して、コンパイラが自動的に `TypeName.new` 関数を生成する。

#### 単一フィールドの場合（State 型パターン）

```fav
-- type PosInt { value: Int  invariant value > 0 } から自動生成:
fn PosInt.new(value: Int) -> PosInt! {
    bind t <- PosInt { value: value }
    if !(value > 0) {
        Result.err("InvariantViolation: PosInt.value must satisfy `value > 0`")
    } else {
        Result.ok(t)
    }
}
```

#### 複数フィールドの場合

```fav
-- type UserAge { value: Int  invariant value >= 0  invariant value <= 150 } から自動生成:
fn UserAge.new(value: Int) -> UserAge! {
    bind t <- UserAge { value: value }
    if !(value >= 0 && value <= 150) {
        Result.err("InvariantViolation: UserAge")
    } else {
        Result.ok(t)
    }
}
```

複数フィールドの型:
```fav
-- type Rect { w: Int  h: Int  invariant w > 0  invariant h > 0 } から:
fn Rect.new(w: Int, h: Int) -> Rect! { ... }
```

### `bind x: StateName <- value` の展開

チェッカーで `bind` の型注釈を検出し、注釈型が invariant 付き型なら:

```rust
// チェッカーでの展開ルール:
// bind x: T <- expr  (T has invariants)  →  chain x <- T.new(expr)
```

- これは chain コンテキスト（関数の return 型が `T!` の系列）でのみ有効
- chain コンテキスト外で使った場合は E024（chain statement outside Result/Option context）

### VM での `.new()` ディスパッチ

コンパイラが生成した IR 関数 `TypeName.new` はアーティファクトに通常の関数として収録される。
VM は `TypeName.new` を呼び出すとき、通常の関数呼び出しパスで処理する（特別なビルトインは不要）。

### 静的な invariant 検査（コンパイル時）

v1.2.0 では以下の場合のみ静的検査を行う（コンパイル時に invariant を評価）:

- RHS が整数・浮動小数・文字列リテラルのみで構成されるとき
- 例: `bind n: PosInt <- 42` → コンパイル時に `42 > 0` が True と確認 → `.new()` 呼び出しを省略可能

静的検査に失敗した場合はコンパイルエラー（E001 型不一致）として報告する。
静的に証明できない場合は常にランタイムチェックにフォールバックする。

---

## 6. Phase 4 — `std.states` ルーン

### 提供する標準 State 型

`use std.states.*` または個別に `use std.states.PosInt` でインポートする。

#### 数値型

| 型名 | 内部フィールド | invariant |
|---|---|---|
| `PosInt` | `value: Int` | `value > 0` |
| `NonNegInt` | `value: Int` | `value >= 0` |
| `Probability` | `value: Float` | `value >= 0.0 && value <= 1.0` |
| `PortNumber` | `value: Int` | `value >= 1 && value <= 65535` |

#### 文字列型

| 型名 | 内部フィールド | invariant |
|---|---|---|
| `NonEmptyString` | `value: String` | `String.length(value) > 0` |
| `Email` | `value: String` | `String.contains(value, "@") && String.length(value) > 3` |
| `Url` | `value: String` | `String.starts_with(value, "http://") \|\| String.starts_with(value, "https://")` |
| `Slug` | `value: String` | slug 文字列（英数字・ハイフン・アンダースコアのみ）|

### `std.states` の実装方針

`std.states` 型は Favnir ソースコードではなく、Rust でプリコンパイルされた型として `Checker::new()` / `compiler.rs` に内部登録する。外観は通常の `TypeDef` と同一（フィールドアクセス、`.new()` コンストラクタ）。

```rust
// checker.rs: register_stdlib_states()
fn register_stdlib_states(checker: &mut Checker) {
    checker.register_state_type("PosInt",  "Int",    vec!["value > 0"]);
    checker.register_state_type("NonNegInt", "Int",  vec!["value >= 0"]);
    // ... 以下同様
}
```

### `use std.states.*` の解決

`resolver.rs` で `std.states` を特別なビルトインモジュールとして認識し、
要求された型名を `Checker` のスコープに追加する。

---

## 7. Phase 5 — `fav explain` invariant 表示

### 変更内容

`fav explain <file>` の出力に、各型の invariant 一覧を追加する。

```
TYPE     FIELDS                      INVARIANTS
Email    value: String               value contains "@"; length > 3
PosInt   value: Int                  value > 0
UserAge  value: Int                  value >= 0; value <= 150
```

### 実装

`cmd_explain` in `driver.rs`:
- 既存の型一覧ループ内で `type_def.invariants` を確認
- 空でなければ `invariant` 列に `;` 区切りで式を文字列化して表示
- `std.states` 型にも同様に表示する（内部登録済みのものは `(stdlib)` ラベルを付ける）

---

## 8. Phase 6 — DB スキーマ CHECK 出力

### `fav explain --schema` オプション

型の invariant を SQL `CHECK` 制約として出力する。

```sh
fav explain main.fav --schema
```

出力例:

```sql
-- Email
CREATE TABLE emails (
    value TEXT NOT NULL,
    CHECK (value LIKE '%@%' AND length(value) > 3)
);

-- PosInt フィールドを持つ型
CREATE TABLE orders (
    id     INTEGER NOT NULL,
    amount INTEGER NOT NULL CHECK (amount > 0),  -- amount: PosInt
    ...
);
```

### 変換ルール

| Favnir invariant 式 | SQL CHECK 制約 |
|---|---|
| `value > N` | `value > N` |
| `value >= N` | `value >= N` |
| `value <= N` | `value <= N` |
| `String.contains(value, s)` | `value LIKE '%s%'` |
| `String.starts_with(value, s)` | `value LIKE 's%'` |
| `String.length(value) > N` | `length(value) > N` |
| `expr && expr` | `expr AND expr` |
| `expr \|\| expr` | `expr OR expr` |

変換不可能な式（ユーザー定義関数等）は `-- [unsupported invariant: <expr>]` としてコメント出力する。

---

## 9. Phase 7 — テスト・ドキュメント

### テスト要件

#### 既存テストの全通過

- v1.1.0 の全テストが通ること

#### 新規テスト

| テスト名 | 検証内容 |
|---|---|
| `invariant_parse_single` | `type PosInt { value: Int  invariant value > 0 }` がパースできる |
| `invariant_parse_multi` | 複数 `invariant` が `TypeDef.invariants` に複数格納される |
| `invariant_type_check_bool` | `invariant` 式が Bool 型なら通る |
| `invariant_type_check_e045` | `invariant value + 1`（非 Bool）で E045 が出る |
| `constructor_ok` | `PosInt.new(5)` が `Ok(PosInt { value: 5 })` を返す |
| `constructor_err` | `PosInt.new(-1)` が `Err(...)` を返す |
| `constructor_multi_field` | 複数フィールド型の `.new(...)` が全 invariant をチェックする |
| `bind_state_annotation` | `bind age: PosInt <- 25` が chain コンテキストで動く |
| `bind_state_annotation_fail` | `bind age: PosInt <- -1` が Err を伝播する |
| `std_states_pos_int` | `use std.states.PosInt` で `PosInt.new(1)` が動く |
| `std_states_email` | `use std.states.Email` で `Email.new("a@b.com")` が Ok を返す |
| `std_states_email_bad` | `Email.new("bad")` が Err を返す |
| `std_states_probability` | `Probability.new(0.5)` が Ok、`Probability.new(1.5)` が Err |
| `explain_invariant_display` | `fav explain` の出力に invariant が含まれる |
| `static_invariant_literal_ok` | `bind n: PosInt <- 42` がコンパイル時に静的検査される |
| `static_invariant_literal_fail` | `bind n: PosInt <- -5` がコンパイルエラーになる |

#### example ファイル

- `examples/invariant_basic.fav` — `type` + `invariant` + `.new()` の基本使用例
- `examples/std_states.fav` — `std.states` 型を用いたパイプライン例

### ドキュメント更新

- `versions/v1.2.0/langspec.md` を新規作成（v1.1.0 の langspec を起点に invariant 節を追加）
  - `invariant` 構文と例
  - `.new()` コンストラクタの挙動（`T!` を返す）
  - `std.states` 型一覧
  - E045 エラーコードの追記
- `README.md` に v1.2.0 セクションを追加

---

## 10. 実装の注意点

### コンストラクタの命名と既存コードとの共存

`TypeName.new` は既存のレコード構築 `TypeName { field: value }` とは別経路。
`invariant` なしの型は `.new()` を生成しない。
`invariant` ありの型でも `TypeName { field: value }` 構文は引き続き動作するが、
**invariant チェックは行われない**（直接構築は内部用途）。これはコンパイラ内部での
レコード生成（`.new()` 本体の中の `TypeName { ... }`）が再帰しないために必要。

### `std.states` 型のフィールドアクセス

```fav
bind email <- Email.new("user@example.com")  -- Result<Email, String>
chain e <- email
IO.println(e.value)                           -- "user@example.com"
```

`std.states` 型は通常の record 型と同様にフィールドアクセスが動く。

### `bind x: T <- expr` 展開のスコープ

型注釈による `chain` への自動展開は `bind` 文の型注釈がある場合 **のみ** 行われる。
型注釈なし (`bind x <- expr`) は従来通り。

---

## 11. 完了条件（Done Definition）

- [ ] `type Email { value: String  invariant String.contains(value, "@") }` が定義できる
- [ ] `Email.new("bad")` が `Err` を返す
- [ ] `Email.new("a@b.com")` が `Ok(Email { value: "a@b.com" })` を返す
- [ ] `use std.states.PosInt` で `bind age: PosInt <- 25` が chain コンテキストで動く
- [ ] `invariant value + 1`（非 Bool）で E045 が出る
- [ ] `fav explain` で `Email` の invariant 一覧が表示される
- [ ] v1.1.0 の全テストが通ること
- [ ] 新規 invariant テストが全て通ること
