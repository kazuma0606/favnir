# Favnir v0.4.0 仕様書

更新日: 2026-04-28

## 概要

v0.4.0 はジェネリクスと capability (cap) を追加するバージョン。

- **ジェネリクス**: `type`, `fn`, `trf`, `flw` に型パラメータ `<T, U>` を付けられるようにする。型変数の単一化（単相 HM 推論）を実装する。
- **cap システム**: 型クラスに相当する能力を `cap` として定義し、呼び出し元が明示的に渡す設計にする。暗黙的な解決は行わない。

---

## スコープ

### v0.4.0 で追加するもの

- `Type::Var(String)` — 型変数の内部表現
- 型変数の単一化（unification）と代入（substitution）
- `type Pair<T, U> = { ... }` — 型パラメータ付き型定義
- `fn identity<T>(v: T) -> T` — 型パラメータ付き関数定義
- `trf MapOption<T, U>: Option<T> -> Option<U>` — 型パラメータ付き変換定義
- `flw` — 型パラメータ構文なし。generic な trf の合成で自動的に generic な型になる
- `cap Eq<T> = { ... }` — capability の定義構文
- `impl Eq<Int> { ... }` — capability の実装構文
- `Type.cap_name` — cap インスタンスへのアクセス (`Int.eq`, `User.ord`)
- 関数パラメータとしての cap 型 (`ord: Ord<T>`)
- 標準 cap: `Eq<T>`, `Ord<T>`, `Show<T>`
- 主要型への組み込み cap 実装 (`Int`, `Float`, `String`, `Bool`)
- `Option<T>` / `Result<T, E>` を generic ADT として再定義（内部的に `Type::Named` に統一、`T?` / `T!` は sugar として保持）

### v0.4.0 でも含まないもの

- 高カインド型 (`Functor<F<_>>`)
- 付随型 (associated types)
- 複数 cap 境界 (`T: Ord + Show`) — cap は 1 つずつ引数として渡す
- `where` 節
- `cap extends` (cap の継承階層) — Ord が Eq の比較関数も定義する形で対応
- 暗黙的な cap 解決 (常に明示的)
- generic cap 実装 (`impl<T: Ord> Ord<List<T>>`) — v0.5.0 以降
- let 多相（`bind` 束縛での多相化）— 関数呼び出し単位の単相化のみ
- bytecode への影響（ツリーウォーキングインタープリタのまま）

---

## 型変数

### 表記

型パラメータ名は 1 文字の大文字 (`T`, `U`, `A`, `B`) または複数文字の大文字始まり (`Elem`, `Key`, `Val`) で書く。

```fav
fn identity<T>(value: T) -> T { value }
fn fst<A, B>(pair: Pair<A, B>) -> A { pair.first }
```

### 内部表現

```rust
pub enum Type {
    // 既存 ...
    Var(String),   // 型変数: T, U, Elem など
}
```

`Type::Var` は monomorphization 時に置換される。型変数は `Vec<String>` で定義順に管理する。

### 呼び出しサイトでの単相化

関数呼び出しのたびに型変数を新鮮な変数（`$0`, `$1`, ...）に置換してから引数型と単一化する。

```fav
// identity : ∀T. T -> T
identity(42)    // T := Int  → 型: Int
identity("hi")  // T := String → 型: String
```

---

## ジェネリック型定義

### 構文

```
type_def = vis? "type" IDENT type_params? "=" type_body
type_params = "<" IDENT ("," IDENT)* ">"
```

### 例

```fav
// レコード型
type Pair<T, U> = {
    first:  T
    second: U
}

// 代数的データ型
type Tree<T> = | Leaf | Node { value: T  left: Tree<T>  right: Tree<T> }

// Option を generic ADT として定義（既存の特殊ケースを廃止）
type Option<T> = | none | some(T)

// Result を generic ADT として定義
type Result<T, E> = | ok(T) | err(E)
```

### 型定義の使用

型パラメータが具体型に展開されて使われる:

```fav
bind p <- Pair { first: 1, second: "hello" }
// p : Pair<Int, String>

bind t <- Node { value: 42, left: Leaf, right: Leaf }
// t : Tree<Int>
```

---

## ジェネリック関数定義

### 構文

```
fn_def = vis? "fn" IDENT type_params? "(" params? ")" "->" type_expr effects? block
```

### 例

```fav
fn identity<T>(value: T) -> T {
    value
}

fn const_<T, U>(value: T, _: U) -> T {
    value
}

fn map_option<T, U>(opt: Option<T>, f: T -> U) -> Option<U> {
    match opt {
        none    => none
        some(v) => some(f(v))
    }
}
```

### `fn` パラメータとしての関数型

```fav
fn apply<T, U>(f: T -> U, value: T) -> U {
    f(value)
}
```

---

## ジェネリック変換定義

### 構文

```
trf_def = vis? "trf" IDENT type_params? ":" type_expr "->" type_expr effects? "=" "|" params "|" block
```

### 例

```fav
// Option<T> の要素を変換する trf
trf MapOption<T, U>: Option<T> -> Option<U> = |opt| {
    match opt {
        none    => none
        some(v) => some(v)   // 実装は簡略化
    }
}

// 2 型変数を持つ trf — List<T> を List<Pair<T, U>> に拡張
trf ZipWith<T, U>: List<T> -> List<Pair<T, U>> = |xs| { xs }
```

`trf` の型パラメータは入力型・出力型の中で使われる。

---

## `flw` とジェネリクス

### flw 自体は型パラメータを持たない

`flw` は trf を `|>` でつないだ**名前付き合成**であり、自分の `<T>` 構文は持たない。
構成する trf が generic であれば、合成時の型単一化を通じて flw も generic な型を持つ。

```
flw_def = "flw" IDENT "=" IDENT ("|>" IDENT)*
```

型パラメータは書けない（書く必要がない）。

### generic trf の合成例

```fav
trf WrapSome<T>:   T         -> Option<T>  = |x|  { some(x) }
trf UnwrapOr0<T>:  Option<T> -> T          = |opt| { match opt { none => opt  some(v) => v } }

// WrapSome の出力型 Option<T> と UnwrapOr0 の入力型 Option<T> が一致 → 合成可
flw RoundTrip = WrapSome |> UnwrapOr0
// 推論: RoundTrip の型 = T -> T  (T は使用時に単一化)
```

型が合わない場合は通常通り E003 を報告する。

---

## `cap` (capability) システム

### 概念

`cap` は「型が持つ能力」を定義する構造体。TypeScript のインターフェイスや Haskell の型クラスに相当するが、常に明示的に渡す（暗黙解決なし）。

### cap の型引数は必須

`cap` の型パラメータは **1 つ以上必須**とする。型引数なしの interface は `type` で表現する。

```
cap_def = vis? "cap" IDENT type_params "=" "{" cap_field+ "}"
//                         ^^^^^^^^^^^
//                         必須: <T> や <T, U>
type_params = "<" IDENT ("," IDENT)* ">"
```

**理由**: 型引数なしの `cap Printable = { print: String -> Unit }` は実質 `type Printable = { print: String -> Unit }` と等価であり、区別する意味がない。`cap` の意義は `T` を持つことで「任意の型 T に対してこの能力を提供できる」という多相性にある。

### 標準 cap の定義

```fav
cap Eq<T> = {
    equals: T -> T -> Bool
}

cap Ord<T> = {
    compare: T -> T -> Int   // 負: 小, 0: 等, 正: 大
    equals:  T -> T -> Bool
}

cap Show<T> = {
    show: T -> String
}
```

### cap の実装

```fav
impl Eq<Int> {
    fn equals(a: Int, b: Int) -> Bool {
        a == b
    }
}

impl Ord<Int> {
    fn compare(a: Int, b: Int) -> Int {
        if a < b { -1 } else if a > b { 1 } else { 0 }
    }
    fn equals(a: Int, b: Int) -> Bool { a == b }
}

impl Show<Int> {
    fn show(value: Int) -> String {
        Debug.show(value)
    }
}
```

### cap インスタンスへのアクセス

型名の名前空間経由でアクセスする:

```fav
Int.eq    // : Eq<Int>
Int.ord   // : Ord<Int>
Int.show  // : Show<Int>
String.eq
String.ord
String.show
Bool.eq
Bool.show
Float.ord
```

ユーザー定義型の cap インスタンスも同様にアクセスできる:

```fav
type Point = { x: Float  y: Float }

impl Eq<Point> {
    fn equals(a: Point, b: Point) -> Bool {
        a.x == b.x
    }
}

// アクセス
Point.eq   // : Eq<Point>
```

複合型への impl も書ける:

```fav
type Pair<T, U> = { first: T  second: U }

impl Show<Pair<Int, String>> {
    fn show(p: Pair<Int, String>) -> String {
        Debug.show(p.first)
    }
}

// アクセス
// (型名を具体化して登録する — v0.4.0 は単相 impl のみ)
// Pair_Int_String.show  ← 内部キー: ("show", "Pair<Int,String>")
```

> **Note**: v0.4.0 の `impl` は**単相**（具体型のみ）。`impl Show<Pair<T, U>>` のような汎用 impl は v0.5.0 以降で対応する。

### cap を使う関数

cap インスタンスを通常のパラメータとして受け取る:

```fav
fn sort<T>(items: List<T>, ord: Ord<T>) -> List<T> {
    // ... ord.compare を使ったソート実装 ...
    items
}

fn unique<T>(items: List<T>, eq: Eq<T>) -> List<T> {
    // ... eq.equals を使った重複排除 ...
    items
}

fn show_all<T>(items: List<T>, s: Show<T>) -> Unit !Io {
    IO.println(List.map(items, |x| { s.show(x) }))
}
```

呼び出し:

```fav
bind sorted  <- sort([3, 1, 2], Int.ord)
bind uniq    <- unique(["a", "b", "a"], String.eq)
bind _       <- show_all([1, 2, 3], Int.show)
```

### cap の型

`cap` インスタンスの型は `CapName<T>` として表現される。内部的には `Type::Cap(name, args)` で保持する。

---

## `Option<T>` / `Result<T, E>` の統一

v0.3.0 まで `Type::Option(Box<Type>)` / `Type::Result(Box<Type>, Box<Type>)` として特殊ケース。
v0.4.0 では `Type::Named("Option", [T])` / `Type::Named("Result", [T, E])` に統一する。

### 移行方針

1. `Type::Option` / `Type::Result` を残しつつ、checker の型解決で `Type::Named("Option", ...)` / `Type::Named("Result", ...)` も同一視する（別名として扱う）
2. 評価器は引き続き Option/Result の内部表現 (`Value::Some(Box<Value>)` など) を使う
3. `T?` は `Option<T>` の sugar として parser 段階で維持する
4. v0.5.0 で完全に `Type::Named` に移行し、`Type::Option` / `Type::Result` を削除する

v0.4.0 では **後方互換を維持しながら段階的に統一を進める**。

---

## 用語整理: `cap` と `effect` の住み分け

v0.4.0 では「capability」という語が 2 つの異なる概念に関わるため、明確に区別する。

| 概念 | 構文 | 種類 | 目的 |
|---|---|---|---|
| **type cap** | `cap Ord<T> = { ... }` / `impl Ord<Int>` | コンパイル時 | 型に対する操作の多相的インターフェイス |
| **effect** | `!Io`, `!Db`, `!Emit<T>` | コンパイル時 + ランタイム | 関数が起こしうる副作用の追跡 |

### type cap (`cap`)

- Haskell の型クラス、Rust の trait に相当
- 「型 `T` がこの操作セットを持つ」という**多相的インターフェイス**
- 常に**明示的に引数として渡す**（暗黙解決なし）
- 純粋にコンパイル時の概念（実行時は `Value::Record` として渡される）

```fav
cap Ord<T> = { compare: T -> T -> Int }
fn sort<T>(xs: List<T>, ord: Ord<T>) -> List<T> { ... }
```

### effect (`!Io`, `!Db`, ...)

- 関数が実行時に起こしうる副作用を型に記録する
- 「この関数を呼ぶには `!Db` 文脈が必要」という**副作用の境界管理**
- checker が呼び出し文脈を検証し、実行時に対応するランタイムが存在することを前提にする

```fav
fn create(name: String) -> Int !Db { Db.execute("INSERT ...") }
```

### 将来の runtime capability（v0.9.0+）

ロードマップ v0.9.0 の「capability runtime」は WASM 上でのエフェクト dispatch 機構を指す。
これは `type cap` とも `effect` とも異なる**ランタイムの実装詳細**（host 関数の注入）であり、
v0.4.0 の仕様とは無関係。名前の衝突を避けるため、将来の実装では `runtime_cap` / `host_cap` などの
名称を検討する。

### まとめ

v0.4.0 において `cap` キーワードは**型レベルの多相インターフェイス**のみを指す。
runtime の「capability」とは切り離して理解する。

---

## 型単一化アルゴリズム

### 実装方針

制約ベースの単一化（Robinson's algorithm）の最小版を実装する。

```rust
pub struct Subst {
    map: HashMap<String, Type>,
}

impl Subst {
    pub fn apply(&self, ty: &Type) -> Type { ... }
    pub fn compose(self, other: Subst) -> Subst { ... }
}

pub fn unify(t1: &Type, t2: &Type) -> Result<Subst, UnifyError> { ... }
```

### 単一化のルール

| t1 | t2 | 結果 |
|---|---|---|
| `Var("T")` | any `t` | `{T → t}` (occurs check 後) |
| any `t` | `Var("T")` | `{T → t}` |
| `Int` | `Int` | `{}` |
| `List<T>` | `List<U>` | `unify(T, U)` |
| `Arrow(A, B)` | `Arrow(C, D)` | `unify(A,C) ∘ unify(B,D)` |
| `Named(n, as)` | `Named(n, bs)` | 各引数を順に unify |
| 不一致 | — | `UnifyError` |

### Occurs Check

`unify(T, List<T>)` のような無限型を防ぐ:

```
occurs("T", List<T>) → true → UnifyError
```

### 呼び出し時の instantiation

```rust
fn instantiate(&self, type_params: &[String], ty: &Type) -> (Type, Subst) {
    // 各型パラメータを新鮮な変数 $0, $1, ... に置換した型と代入を返す
}
```

---

## AST の変更

### `TypeDef` への型パラメータ追加

```rust
pub struct TypeDef {
    pub visibility: Option<Visibility>,
    pub name: String,
    pub type_params: Vec<String>,   // 新規: ["T", "U"] for type Pair<T, U>
    pub body: TypeBody,
    pub span: Span,
}
```

### `FnDef` への型パラメータ追加

```rust
pub struct FnDef {
    pub visibility: Option<Visibility>,
    pub name: String,
    pub type_params: Vec<String>,   // 新規: ["T", "U"] for fn f<T, U>(...)
    pub params: Vec<Param>,
    pub return_ty: TypeExpr,
    pub effects: Vec<Effect>,
    pub body: Block,
    pub span: Span,
}
```

### `TrfDef` への型パラメータ追加

```rust
pub struct TrfDef {
    pub visibility: Option<Visibility>,
    pub name: String,
    pub type_params: Vec<String>,   // 新規
    pub input_ty: TypeExpr,
    pub output_ty: TypeExpr,
    pub effects: Vec<Effect>,
    pub params: Vec<Param>,
    pub body: Block,
    pub span: Span,
}
```

### `CapDef` (新規)

```rust
pub struct CapField {
    pub name: String,
    pub ty: TypeExpr,   // 関数型: T -> T -> Bool など
    pub span: Span,
}

pub struct CapDef {
    pub visibility: Option<Visibility>,
    pub name: String,
    pub type_params: Vec<String>,   // ["T"] for cap Eq<T>
    pub fields: Vec<CapField>,
    pub span: Span,
}
```

### `ImplDef` (新規)

```rust
pub struct ImplDef {
    pub cap_name: String,          // "Eq"
    pub type_args: Vec<TypeExpr>,  // [Int] for impl Eq<Int>
    pub methods: Vec<FnDef>,
    pub span: Span,
}
```

### `Item` への追加

```rust
pub enum Item {
    // 既存 ...
    CapDef(CapDef),
    ImplDef(ImplDef),
}
```

---

## Checker の変更

### `Type` への追加

```rust
pub enum Type {
    // 既存 ...
    Var(String),                     // 型変数 (型パラメータ, fresh var)
    Cap(String, Vec<Type>),          // cap インスタンスの型: Ord<Int>
}
```

### `Checker` フィールドの追加

```rust
pub struct Checker {
    // 既存 ...
    type_params: HashSet<String>,              // 現在のスコープの型パラメータ
    subst: Subst,                              // 型代入
    fresh_counter: usize,                      // fresh var 生成カウンタ
    caps: HashMap<String, CapScope>,           // 定義済み cap
    impls: HashMap<(String, String), ImplScope>, // (CapName, TypeName) → impl
}
```

### `CapScope` / `ImplScope`

```rust
pub struct CapScope {
    pub type_params: Vec<String>,
    pub fields: HashMap<String, Type>,   // field名 → 型
}

pub struct ImplScope {
    pub methods: HashMap<String, Type>,  // method名 → 型
}
```

### check_fn_def の変更

型パラメータを `type_params` に登録してからパラメータ・返り値型を解決する。

```rust
fn check_fn_def(&mut self, fd: &FnDef) {
    self.type_params.clear();
    for tp in &fd.type_params {
        self.type_params.insert(tp.clone());
    }
    // ... 既存処理 ...
}
```

### 関数呼び出し時の型推論

```rust
fn check_generic_call(&mut self, fn_ty: &Type, args: &[Expr], span: &Span) -> Type {
    // 1. fn_ty の型変数を fresh vars で instantiate
    // 2. 引数ごとに unify
    // 3. 返り値型に代入を適用して返す
}
```

---

## 評価器の変更

### cap インスタンスのランタイム表現

cap インスタンスは `Value::Record` として表現する。`impl` の各メソッドがフィールド値（クロージャ）になる。

```
Int.ord = Value::Record {
    "compare" → Value::Fn(|a, b| ...),
    "equals"  → Value::Fn(|a, b| ...),
}
```

### `Type.cap_name` のアクセス

`FieldAccess(Ident("Int"), "ord")` → 組み込み/登録済み impl を返す。

### ImplDef の評価

`impl Eq<Int> { fn equals(...) }` を評価すると `Value::Record` を生成し、グローバルの impl レジストリに登録する。`Int.eq` でアクセス可能にする。

---

## 新しい構文のまとめ

### `cap` 定義

```
cap_def = vis? "cap" IDENT type_params "=" "{" cap_field+ "}"
cap_field = IDENT ":" type_expr
```

### `impl` 定義

```
impl_def = "impl" IDENT "<" type_expr ("," type_expr)* ">" "{" fn_def+ "}"
```

### 型パラメータの構文

```
type_params = "<" IDENT ("," IDENT)* ">"
```

型パラメータは定義側のみ。使用側は通常の型引数として `<ConcreteType>` で書く。

---

## エラーコード

| コード | 内容 |
|---|---|
| E017 | 型変数が未解決（未束縛の型変数が残った） |
| E018 | 型単一化の失敗（型が合わない） |
| E019 | occurs check の失敗（無限型） |
| E020 | cap が定義されていない |
| E021 | cap の実装 (impl) が存在しない |
| E022 | impl のメソッドが cap の定義と合わない |
| E023 | 型パラメータの個数が合わない |

---

## 標準 cap インスタンス（組み込み）

v0.4.0 では以下の cap インスタンスをインタープリタに組み込む（`.fav` で書かず Rust で実装）。

| 型 | Eq | Ord | Show |
|---|---|---|---|
| `Int` | `Int.eq` | `Int.ord` | `Int.show` |
| `Float` | `Float.eq` | `Float.ord` | `Float.show` |
| `String` | `String.eq` | `String.ord` | `String.show` |
| `Bool` | `Bool.eq` | — | `Bool.show` |

---

## 例

### identity と const

```fav
fn identity<T>(value: T) -> T { value }
fn const_<T, U>(value: T, _: U) -> T { value }

public fn main() -> Unit !Io {
    bind n <- identity(42);
    bind s <- identity("hello");
    IO.println(n);
    IO.println(s);
    IO.println(const_(99, "ignored"))
}
```

### Pair 型

```fav
type Pair<T, U> = {
    first:  T
    second: U
}

fn swap<T, U>(p: Pair<T, U>) -> Pair<U, T> {
    Pair { first: p.second, second: p.first }
}

public fn main() -> Unit !Io {
    bind p    <- Pair { first: 1, second: "one" };
    bind swapped <- swap(p);
    IO.println(swapped.first)   // "one"
}
```

### sort と cap

```fav
fn sort<T>(items: List<T>, ord: Ord<T>) -> List<T> {
    // 実装（評価器レベルで対応）
    items
}

public fn main() -> Unit !Io {
    bind sorted <- sort([3, 1, 4, 1, 5], Int.ord);
    IO.println(Debug.show(sorted))
}
```

### ユーザー定義型への cap 実装

```fav
type Point = { x: Float  y: Float }

impl Eq<Point> {
    fn equals(a: Point, b: Point) -> Bool {
        a.x == b.x
    }
}

public fn main() -> Unit !Io {
    bind p1 <- Point { x: 1.0, y: 2.0 };
    bind p2 <- Point { x: 1.0, y: 3.0 };
    IO.println(Point.eq.equals(p1, p2))   // false
}
```

---

## 完了条件

- `fn identity<T>(value: T) -> T` が型チェックを通って実行できる
- `type Pair<T, U> = { ... }` が定義できて `Pair<Int, String>` として使える
- `fn map<T, U>(items: List<T>, f: T -> U) -> List<U>` が書けて動く
- `cap Ord<T>` が定義できる
- `impl Ord<Int>` が書けて `Int.ord` でアクセスできる
- `sort([3,1,2], Int.ord)` が型チェックを通って正しく動く
- `Option<T>` / `Result<T, E>` が型パラメータ付き型として扱える（`T?` sugar 維持）
- E017〜E023 が適切に報告される
- 147 (+ 新規) テストが全パス
