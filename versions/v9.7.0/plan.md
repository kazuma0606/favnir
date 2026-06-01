# Favnir v9.7.0 Implementation Plan

Date: 2026-06-02
Theme: 名目型ラッパー + `where` バリデーション + `with` 自動合成 + `T?`/`T!`/`??`/`expr?` self-hosted 修正

---

## Phase A: Rust パーサー — 名目型ラッパー構文 AST 追加

v9.7.0 の Rust 変更はパーサーのみ。vm.rs / compiler.rs / checker.rs は触らない。

### A-1: `src/ast.rs`

```rust
// 新規追加
pub struct WrapperDef {
    pub name: String,
    pub inner_ty: TypeExpr,
    pub validator: Option<Expr>,   // where |v| pred
    pub with_impls: Vec<String>,   // ["Eq", "Show", ...]
}

// Item に追加
pub enum Item {
    // ...既存...
    Wrapper(WrapperDef),
}
```

### A-2: `src/frontend/parser.rs`

`parse_type_def` で `type Name` の後のトークンを確認する分岐を追加。

```
type <Name>         → 既存の TypeAlias / RecordDef 分岐
type <Name>(        → WrapperDef パース
```

**`type Name(InnerType)` パース**:
1. `(` を消費
2. `parse_type_expr` で `InnerType` をパース
3. `)` を消費
4. `with` キーワードがあれば `Vec<String>` をパース（カンマ区切り識別子）
5. `where` キーワードがあれば `parse_expr` でラムダ式をパース
6. `WrapperDef` を構築

**`type T with Iface1, Iface2 = { ... }` パース**:
既存のレコード型定義パスに `with` 節オプションを追加。
`=` の前に `with <ident, ...>` が続く場合を処理。

### A-3: `src/fmt.rs`

```rust
Item::Wrapper(def) => {
    // "type Name(Inner)" / "type Name(Inner) with Eq, Show" / "type Name(Inner) where |v| ..." の fmt
}
```

### A-4: `src/middle/ast_lower_checker.rs`

`Item::Wrapper` を型環境に登録するスタブを追加（内容は checker.rs 側で今後対応）。
今バージョンは `exhaustive match` エラーを解消するだけでよい。

### A-5: `cargo build` 確認

exhaustive match エラーがすべて解消されていること。

---

## Phase B: Bug fix — lexer.fav / parser.fav の T? / ?? / expr? 対応

### B-1〜B-3: lexer.fav

**新規トークン追加**:
```favnir
type Token =
  // ... 既存 ...
  | TkQuestion
  | TkQuestionQuestion
```

**scan_token の分岐追加**:
```favnir
'?' ->
  if peek() == '?' {
    advance()
    TkQuestionQuestion
  } else {
    TkQuestion
  }
```

`!` の先読みと同じパターン（`TkBang` / `TkBangBang` がある場合は参照）。

### B-4〜B-6: parser.fav — 型パース

**parse_type_expr の後置処理**:
```favnir
fn parse_type_expr_postfix(ty: TypeExpr) -> TypeExpr {
  if peek() == TkQuestion {
    consume(TkQuestion)
    TeOption(ty)
  } else if peek() == TkBang && not_effect_context() {
    consume(TkBang)
    TeResult(ty, TeSimple("String"))
  } else {
    ty
  }
}
```

`parse_type_expr` の末尾でこの関数を呼ぶ形にする。

**`??` 演算子**:
二項演算子 `OpQuestionQuestion` を追加し、`parse_binop` の優先順位表に挿入する。
優先順位は `||` と同等（低め、`&&` より低い）。

### B-7〜B-9: parser.fav — expr?（EQuestion）

`parse_postfix_expr` または `parse_unary_expr` の末尾でチェック:
```favnir
fn parse_postfix(expr: Expr) -> Expr {
  // ... 既存の . アクセス・関数呼び出し ...
  if peek() == TkQuestion {
    consume(TkQuestion)
    EQuestion(expr)
  } else {
    expr
  }
}
```

### B-10〜B-11: compiler.fav — EQuestion 脱糖 / ?? コード生成

**EQuestion の compile_expr**:
```favnir
EQuestion(inner) ->
  // 脱糖: match inner { Ok(v) -> v  Err(e) -> return Err(e) }
  let tmp = fresh_var()
  let ok_var = fresh_var()
  let err_var = fresh_var()
  compile_match(inner, [
    MatchArm(PConstr("Ok",  [PVar(ok_var)]),  EVar(ok_var)),
    MatchArm(PConstr("Err", [PVar(err_var)]), EReturn(EConstr("Err", [EVar(err_var)]))),
  ])
```

**OpQuestionQuestion の compile_expr**:
```favnir
BinOp(OpQuestionQuestion, lhs, rhs) ->
  // 脱糖: match lhs { Some(v) -> v  None -> rhs }
  let tmp = fresh_var()
  compile_match(lhs, [
    MatchArm(PConstr("Some", [PVar(tmp)]), EVar(tmp)),
    MatchArm(PConstr("None", []),          rhs),
  ])
```

---

## Phase C: checker.fav — 名目型ラッパー型チェック

### C-1: WrapperDef の AST 認識

`checker.fav` の `Item` 型定義に `IWrapper` を追加:
```favnir
type Item =
  // ... 既存 ...
  | IWrapper(WrapperDef)

type WrapperDef = {
  name:       String
  inner_ty:   String         // 内部型名（例: "Int", "String"）
  has_where:  Bool           // where あり/なし
  with_impls: List<String>   // ["Eq", "Show", ...]
}
```

### C-2: collect_wrapper_types

```favnir
fn collect_wrapper_types(items: List<Item>, env: Env) -> Env {
  List.fold_left(|e, item|
    match item {
      IWrapper(def) ->
        // コンストラクタを関数として登録
        // where なし: "Inner|Name"（Inner → Name）
        // where あり: "Inner|Result<Name,String>"
        let ret_ty = if def.has_where { "Result" } else { def.name }
        let scheme = def.inner_ty + "|" + ret_ty
        Env.set(e, def.name, scheme)
      _ -> e
    }
  , env, items)
}
```

`check(prog)` の呼び出し順:
```
collect_fn_schemes → collect_variant_constructors → collect_wrapper_types → check_items
```

### C-3: コンストラクタ呼び出しの型推論

`infer_call_user` / `infer_hm` の `ECall` ケースで `env` から名目型コンストラクタを検索。
**E0010 — WrapperTypeMismatch**:
```
"E0010: UserId の内部型は Int ですが、String が渡されました"
```

### C-4: パターンマッチ `Name(n)` の分解

`check_pattern` で `PWrapperDestr(name, inner_pat)` を処理:
- `name` が `env` に名目型ラッパーとして登録されているか確認
- `inner_pat` の変数に内部型を束縛して `env` に追加

### C-5: with 節の検証

既知インターフェースのリスト: `["Eq", "Show", "Serialize", "Deserialize"]`
それ以外が `with_impls` に含まれる場合 **E0011**:
```
"E0011: インターフェース Fooable は定義されていません"
```

### C-6: E0013 — QuestionOutsideResult

`check_fn_def` または `infer_hm` の `EQuestion` ケースで、
現在の関数の戻り型が `Result` でなければ E0013 を返す。
```
"E0013: ? は Result を返す関数内でのみ使用できます"
```

---

## Phase D: compiler.fav — 名目型コード生成 + with 自動合成

### D-1: WrapperDef のコード生成

**where なし**:
```favnir
// type UserId(Int) → コンストラクタは恒等関数（バイトコード上は値そのまま）
fn UserId(x: Int) -> UserId = x
```

**where あり**:
```favnir
// type Percent(Float) where |v| v >= 0.0 && v <= 100.0
fn Percent(v: Float) -> Result<Percent, String> =
  if (where_pred)(v) { Result.ok(v) }
  else { Result.err("Percent: validation failed") }
```

`compile_program` の `Item::Wrapper(def)` ケースで `compile_wrapper_def` を呼ぶ。

### D-2: with Eq 自動合成

```favnir
fn compile_with_eq(type_name: String) -> String =
  "fn eq(a: " + type_name + ", b: " + type_name + ") -> Bool = a == b\n"
```

### D-3: with Show 自動合成

名目型ラッパー:
```favnir
"fn show(t: " + type_name + ") -> String = String.from_int(t)"
// 内部型に応じた show（Int → String.from_int、Float → String.from_float、String → t）
```

レコード型（`Order with Show`）:
```favnir
// フィールド名: 値 の形式で結合
"fn show(t: Order) -> String = \"{ id: \" + String.from_int(t.id) + \" item: \" + t.item + \" }\""
```

### D-4: with Serialize / Deserialize 自動合成

```favnir
fn compile_with_serialize(type_name: String) -> String =
  "fn to_json(t: " + type_name + ") -> String = Json.encode_raw(t)\n"

fn compile_with_deserialize(type_name: String) -> String =
  "fn from_json(s: String) -> Result<" + type_name + ", String> = Json.decode_raw(s)\n"
```

### D-5: パターンマッチ `Name(n)` のコード生成

名目型ラッパーは内部型と同一の値 → `PWrapperDestr(name, PVar(n))` は `PVar(n)` と同等に生成。
`compile_pattern` で `PWrapperDestr` ケースを追加し inner_pat をそのまま処理。

---

## Phase E: 統合テスト

### T? / ?? / expr? 修正確認（5 件）

```rust
#[test]
fn option_type_question_mark_parsed() {
    // type alias: fn foo() -> Int? = None
    // fav run（Favnir pipeline）で動作することを確認
}

#[test]
fn result_type_bang_parsed() {
    // fn foo() -> Int! = Err("fail")
}

#[test]
fn null_coalesce_question_question() {
    // let x: Int? = None
    // let v = x ?? 42   → v == 42
}

#[test]
fn expr_question_propagates_error() {
    // fn parse(s: String) -> Result<Int, String> {
    //   bind n <- Int.parse(s)?
    //   Result.ok(n + 1)
    // }
}

#[test]
fn expr_question_e0013_outside_result() {
    // fn bad(s: String) -> String { Int.parse(s)?  s }
    // → E0013 が出ること
}
```

### 名目型ラッパー（9 件）

```rust
#[test]
fn wrapper_type_no_where() {
    // type UserId(Int)
    // let id = UserId(42)
    // match id { UserId(v) -> v }  → 42
}

#[test]
fn wrapper_type_with_where_ok() {
    // type Percent(Float) where |v| v >= 0.0 && v <= 100.0
    // bind pct <- Percent(50.0)  → Ok(Percent)
}

#[test]
fn wrapper_type_with_where_err() {
    // bind pct <- Percent(150.0)  → Err("Percent: validation failed")
}

#[test]
fn wrapper_type_mismatch_e0010() {
    // type UserId(Int)
    // let id = UserId("hello")  → E0010
}

#[test]
fn wrapper_pattern_match() {
    // match pct { Percent(v) -> v * 0.01 }  → 0.5
}

#[test]
fn wrapper_with_eq_synth() {
    // type UserId(Int) with Eq
    // eq(UserId(1), UserId(1))  → true
}

#[test]
fn wrapper_with_show_synth() {
    // type UserId(Int) with Show
    // show(UserId(42))  → "42"
}

#[test]
fn record_with_serialize() {
    // type Order with Serialize = { id: Int  item: String }
    // to_json(Order { id: 1  item: "book" })  → JSON 文字列
}

#[test]
fn unknown_interface_e0011() {
    // type Foo(Int) with Fooable  → E0011
}
```

### 全件確認

```rust
#[test]
fn v970_all_pass() {
    // cargo test v970 で 14 件全通過
}
```

---

## Phase F: self-check + Bootstrap 検証

```bash
cargo test checker_fav_wire_self_check   # self-check 通過
cargo test bootstrap                      # bytecode_A == bytecode_B 維持
cargo test                                # 全件通過（目標 1203 件以上）
```

---

## Phase G: ドキュメント・バージョン更新

- `fav/Cargo.toml` version → `"9.7.0"`
- `fav/self/cli.fav` バージョン文字列 → `"9.7.0"`
- `versions/v9.7.0/tasks.md` 完了チェック
- `memory/MEMORY.md` v9.7.0 完了を記録
- commit

---

## 実装上の注意点

1. **`where` / `with` キーワードの競合確認**
   `with` は現在 Favnir に存在しないキーワード。レクサーに `KwWith` / `KwWhere` を追加する必要があるか確認する（`where` は型制約に使われていないか確認）。

2. **名目型の内部表現はバイトコード上は素の値**
   `UserId(42)` はコンパイル後のバイトコードで `42` と同一。型チェッカーレベルのみで区別。
   これにより `with Eq` の `eq(a, b)` は `a == b` のままで正しく動く。

3. **`where` 述語の評価タイミング**
   コンストラクタ呼び出し時のみ評価（1回バリデーション保証）。
   パターンマッチによる分解時は述語再評価なし（「型が保証する」設計）。

4. **`with` 合成の優先順位**
   `with` で自動合成された関数と同名のユーザー定義関数が存在する場合、ユーザー定義を優先。
   これは将来の `impl Interface for Type` ブロック（v9.12.0 予定）への橋渡し。

5. **`expr?` の文脈判定**
   `T!` の `!` とエフェクト注釈 `-> String !Io` の `!` を区別する必要がある。
   型パース文脈での `!` はエフェクト注釈として処理し、値式文脈の `!` のみ `TeResult` に変換する。

6. **`T?` と `EQuestion(expr)` の区別**
   `T?` は型パース文脈（`parse_type_expr` 内）での `?`。
   `expr?` は式パース文脈（`parse_postfix` 内）での `?`。
   トークンは同じ `TkQuestion` だが文脈で使い分ける。

7. **`compile_with_show` でのレコード型対応**
   レコード型のフィールド情報を `compiler.fav` に保持するか、
   または `Json.encode_raw` に委譲して `Show` = JSON 文字列とする簡易実装も可。
   v9.7.0 では後者（`Json.encode_raw` 委譲）で実装し、v9.12.0 以降で洗練させる。
