# Favnir v0.4.0 実装計画

更新日: 2026-04-28

---

## Phase 1: 型変数と型単一化（Unification）

型推論の基盤。他の全フェーズが依存する。

### `Type::Var` の追加

`checker.rs` の `Type` 列挙体に `Var(String)` を追加する。

```rust
pub enum Type {
    // 既存 ...
    Var(String),          // 型変数: "T", "U", "$0"（fresh var）
    Cap(String, Vec<Type>), // cap インスタンス型: Ord<Int>
}
```

`Type::display()` / `Type::display_effect()` に対応するアームを追加する。

### Substitution（代入）の実装

`checker.rs` に `Subst` 構造体を追加する。

```rust
pub struct Subst {
    map: HashMap<String, Type>,
}

impl Subst {
    pub fn empty() -> Self { ... }
    pub fn singleton(var: String, ty: Type) -> Self { ... }
    pub fn apply(&self, ty: &Type) -> Type { ... }  // 型変数を置換
    pub fn compose(self, other: Subst) -> Subst { ... }  // 代入の合成
    pub fn extend(&mut self, var: String, ty: Type) { ... }
}
```

`apply` は再帰的に型を走査して置換する。`Type::Var(name)` に対して `map.get(name)` を見て再帰的に適用する（transitive closure）。

### Unification（単一化）の実装

`checker.rs` にフリー関数として実装する。

```rust
pub fn unify(t1: &Type, t2: &Type) -> Result<Subst, String> {
    match (t1, t2) {
        (Type::Var(a), Type::Var(b)) if a == b => Ok(Subst::empty()),
        (Type::Var(a), t) | (t, Type::Var(a)) => {
            if occurs(a, t) { return Err(format!("occurs check: {} in {:?}", a, t)); }
            Ok(Subst::singleton(a.clone(), t.clone()))
        }
        (Type::Int, Type::Int) => Ok(Subst::empty()),
        // ... 各コンクリート型のペア ...
        (Type::List(a), Type::List(b)) => unify(a, b),
        (Type::Arrow(a1, b1), Type::Arrow(a2, b2)) => {
            let s1 = unify(a1, a2)?;
            let s2 = unify(&s1.apply(b1), &s1.apply(b2))?;
            Ok(s2.compose(s1))
        }
        (Type::Named(n1, as1), Type::Named(n2, as2)) if n1 == n2 && as1.len() == as2.len() => {
            as1.iter().zip(as2).try_fold(Subst::empty(), |acc, (a, b)| {
                let s = unify(&acc.apply(a), &acc.apply(b))?;
                Ok(s.compose(acc))
            })
        }
        _ => Err(format!("cannot unify {:?} with {:?}", t1, t2))
    }
}

fn occurs(var: &str, ty: &Type) -> bool { ... }
```

### Fresh variable の生成

`Checker` に `fresh_counter: usize` フィールドを追加する。

```rust
fn fresh_var(&mut self) -> Type {
    let n = self.fresh_counter;
    self.fresh_counter += 1;
    Type::Var(format!("${}", n))
}
```

### Instantiation（具体化）

型スキーマ（型パラメータ付き型）を呼び出しサイトで具体化する。

```rust
fn instantiate(&mut self, type_params: &[String], ty: &Type) -> Type {
    let mut subst = Subst::empty();
    for tp in type_params {
        subst.extend(tp.clone(), self.fresh_var());
    }
    subst.apply(ty)
}
```

---

## Phase 2: Lexer / Parser の拡張

### 新トークン

| トークン | キーワード |
|---|---|
| `TokenKind::Cap` | `cap` |
| `TokenKind::Impl` | `impl` |

### AST の拡張

#### `TypeDef` に `type_params` を追加

```rust
pub struct TypeDef {
    pub visibility: Option<Visibility>,
    pub name: String,
    pub type_params: Vec<String>,   // 追加
    pub body: TypeBody,
    pub span: Span,
}
```

#### `FnDef` に `type_params` を追加

```rust
pub struct FnDef {
    // ...
    pub type_params: Vec<String>,   // 追加
    // ...
}
```

#### `TrfDef` に `type_params` を追加

```rust
pub struct TrfDef {
    // ...
    pub type_params: Vec<String>,   // 追加
    // ...
}
```

#### `CapDef` を新規追加

```rust
pub struct CapField {
    pub name: String,
    pub ty: TypeExpr,
    pub span: Span,
}

pub struct CapDef {
    pub visibility: Option<Visibility>,
    pub name: String,
    pub type_params: Vec<String>,
    pub fields: Vec<CapField>,
    pub span: Span,
}
```

#### `ImplDef` を新規追加

```rust
pub struct ImplDef {
    pub cap_name: String,
    pub type_args: Vec<TypeExpr>,
    pub methods: Vec<FnDef>,
    pub span: Span,
}
```

#### `Item` に追加

```rust
pub enum Item {
    // 既存 ...
    CapDef(CapDef),
    ImplDef(ImplDef),
}
```

### パーサの変更

#### 型パラメータの解析 (`parse_type_params`)

```rust
fn parse_type_params(&mut self) -> Result<Vec<String>, ParseError> {
    if self.peek() != &TokenKind::Lt {
        return Ok(vec![]);
    }
    self.advance(); // <
    let mut params = vec![self.expect_ident()?.0];
    while self.peek() == &TokenKind::Comma {
        self.advance();
        params.push(self.expect_ident()?.0);
    }
    self.expect(&TokenKind::Gt)?; // >
    Ok(params)
}
```

#### `parse_type_def` の更新

`parse_type_def(vis)` で `parse_type_params()` を呼ぶ。

#### `parse_fn_def` の更新

fn名の後に `parse_type_params()` を呼ぶ。

#### `parse_trf_def` の更新

trf名の後に `parse_type_params()` を呼ぶ。

#### `parse_cap_def`（新規）

```rust
fn parse_cap_def(&mut self, vis: Option<Visibility>) -> Result<CapDef, ParseError> {
    self.expect(&TokenKind::Cap)?;
    let name = self.expect_ident()?.0;
    let type_params = self.parse_type_params()?;
    self.expect(&TokenKind::Eq)?;
    self.expect(&TokenKind::LBrace)?;
    let mut fields = vec![];
    while self.peek() != &TokenKind::RBrace {
        let fname = self.expect_ident()?.0;
        self.expect(&TokenKind::Colon)?;
        let fty = self.parse_type_expr()?;
        fields.push(CapField { name: fname, ty: fty, span: Span::dummy() });
    }
    self.expect(&TokenKind::RBrace)?;
    Ok(CapDef { visibility: vis, name, type_params, fields, span: Span::dummy() })
}
```

#### `parse_impl_def`（新規）

```rust
fn parse_impl_def(&mut self) -> Result<ImplDef, ParseError> {
    self.expect(&TokenKind::Impl)?;
    let cap_name = self.expect_ident()?.0;
    self.expect(&TokenKind::Lt)?;
    let mut type_args = vec![self.parse_type_expr()?];
    while self.peek() == &TokenKind::Comma {
        self.advance();
        type_args.push(self.parse_type_expr()?);
    }
    self.expect(&TokenKind::Gt)?;
    self.expect(&TokenKind::LBrace)?;
    let mut methods = vec![];
    while self.peek() != &TokenKind::RBrace {
        // parse fn_def (各メソッドは fn キーワードで始まる)
        let fn_def = self.parse_fn_def(None)?;
        methods.push(fn_def);
    }
    self.expect(&TokenKind::RBrace)?;
    Ok(ImplDef { cap_name, type_args, methods, span: Span::dummy() })
}
```

#### `parse_item` の更新

`cap` トークンなら `parse_cap_def`、`impl` トークンなら `parse_impl_def` を呼ぶ。

---

## Phase 3: Checker への統合

### `Checker` 構造体の拡張

```rust
pub struct Checker {
    // 既存 ...
    type_params: HashSet<String>,                          // 現在の型パラメータスコープ
    subst: Subst,                                          // 型代入（単一化の結果）
    fresh_counter: usize,                                  // fresh var 生成カウンタ
    caps: HashMap<String, CapScope>,                       // 定義済み cap
    impls: HashMap<(String, String), ImplScope>,           // (cap名, 型名) → impl
}

pub struct CapScope {
    pub type_params: Vec<String>,
    pub fields: HashMap<String, TypeExpr>,  // field名 → 型式（型パラメータを含む）
}

pub struct ImplScope {
    pub methods: HashMap<String, Type>,     // method名 → 具体化済み型
}
```

### `register_item_signatures` の更新

- `Item::CapDef(cd)` → `caps` に登録
- `Item::ImplDef(id)` → 型を解決して `impls` に登録
- `Item::FnDef(fd)` → `type_params` を考慮したシグネチャ型を登録

### `check_item` の更新

- `Item::CapDef` → フィールドの型式を検証
- `Item::ImplDef` → 各メソッドを、対応する cap フィールドの型と照合して型チェック

### `check_fn_def` の更新

型パラメータのスコープを設定:

```rust
fn check_fn_def(&mut self, fd: &FnDef) {
    let old_params = std::mem::take(&mut self.type_params);
    for tp in &fd.type_params {
        self.type_params.insert(tp.clone());
    }
    // ... 既存処理 ...
    self.type_params = old_params;
}
```

### `resolve_type_expr` の更新

`TypeExpr::Named(name, [], _)` で `name` が `type_params` に含まれる場合は `Type::Var(name)` を返す。

```rust
TypeExpr::Named(name, args, _) => {
    if args.is_empty() && self.type_params.contains(name) {
        return Type::Var(name.clone());
    }
    // ... 既存処理 ...
}
```

### `check_apply` の更新

関数型が型変数を含む場合、単一化を行う:

1. 関数の型を取得
2. 型変数があれば `instantiate` して fresh vars に置換
3. 各引数を型チェックして unify
4. 代入を `subst` に蓄積
5. 返り値型に `subst.apply` して返す

### `FieldAccess` での cap インスタンス解決

`Expr::FieldAccess(Ident("Int"), "ord")` の場合:

```rust
// check_expr: FieldAccess
if let Some(scope) = self.impls.get(&(field.clone(), obj_type_name)) {
    // cap インスタンスの型を返す
    return Type::Cap(field.clone(), vec![obj_type]);
}
```

### built-in caps の登録

`register_builtins` で標準 cap インスタンスを `impls` に登録する（Rust 側で定義）:

```rust
// Int.eq, Int.ord, Int.show
self.register_builtin_impl("Eq", "Int", [("equals", fn(Int, Int) -> Bool)]);
self.register_builtin_impl("Ord", "Int", [("compare", ...), ("equals", ...)]);
// ...
```

---

## Phase 4: 評価器の変更

### cap インスタンスの評価

`ImplDef` を評価するとメソッドのクロージャが入ったレコードを生成し、グローバルの impl レジストリに登録する。

```rust
// eval.rs に追加
impl_registry: HashMap<(String, String), Value>,
// キー: ("Eq", "Int"), 値: Value::Record { "equals" → Value::Fn(...) }
```

### `FieldAccess` での cap アクセス

`Int.ord` → `impl_registry.get(("ord", "Int"))` を返す。

型名 (ident の左辺) が registered type の名前か組み込み型名の場合、cap アクセスとして処理する。

### cap メソッドの呼び出し

`cap_val.compare(a, b)` → `cap_val` は `Value::Record` → `cap_val.get("compare")` を呼ぶ `Value::Fn`。

通常の `FieldAccess` + `Apply` として既存のコードで動く（`cap_val` が `Value::Record` なので `record.field(args)` のパターン）。

### 組み込み cap インスタンスの登録

`Interpreter` 初期化時に組み込み cap を `impl_registry` に登録する（Rust のクロージャで実装）。

---

## Phase 5: Option / Result の統一

### 目標

`Type::Option(T)` → `Type::Named("Option", [T])` へ段階的に移行する。

### v0.4.0 での対応

1. 型チェッカーで `Type::Option(t)` と `Type::Named("Option", [t])` を単一化で同値扱いにする（unify 関数内で特殊ケース追加）
2. `TypeExpr::Optional(t, _)` のパース結果は `Type::Named("Option", [resolve(t)])` を返すよう変更
3. `Type::display()` で `Type::Named("Option", [t])` は `{}?` と表示（後方互換維持）
4. 評価器は `Type::Named("Option", ...)` のマッチを認識し、内部表現は `Value::Some` / `Value::None` を維持

---

## Phase 6: サンプルと動作確認

### `examples/generics.fav`

```fav
fn identity<T>(value: T) -> T { value }
fn fst<A, B>(a: A, _: B) -> A { a }

type Pair<T, U> = { first: T  second: U }

fn swap<T, U>(p: Pair<T, U>) -> Pair<U, T> {
    Pair { first: p.second, second: p.first }
}

public fn main() -> Unit !Io {
    IO.println(identity(42));
    IO.println(identity("hello"));
    bind p <- Pair { first: 1, second: "one" };
    bind s <- swap(p);
    IO.println(Debug.show(s))
}
```

### `examples/cap_sort.fav`

```fav
fn min_by<T>(a: T, b: T, ord: Ord<T>) -> T {
    if ord.compare(a, b) <= 0 { a } else { b }
}

public fn main() -> Unit !Io {
    IO.println(min_by(3, 5, Int.ord));
    IO.println(min_by("banana", "apple", String.ord))
}
```

### `examples/cap_user.fav`

```fav
type User = { name: String  score: Int }

impl Ord<User> {
    fn compare(a: User, b: User) -> Int {
        b.score - a.score
    }
    fn equals(a: User, b: User) -> Bool {
        a.score == b.score
    }
}

public fn main() -> Unit !Io {
    bind alice <- User { name: "Alice", score: 90 };
    bind bob   <- User { name: "Bob",   score: 75 };
    bind winner <- if User.ord.compare(alice, bob) <= 0 { alice } else { bob };
    IO.println(winner.name)
}
```

---

## 設計メモ

### 型変数の命名規則

ソース内の型パラメータ（`T`, `U` など）と fresh vars（`$0`, `$1` など）を区別する。fresh vars は `$` プレフィックスで名前空間を分離する。

### 単一化エラーの span

現状 `Span::dummy()` を使っている箇所が多い。型変数の単一化失敗時も dummy span で報告し、v0.5.0 で位置情報を改善する。

### `flw` とジェネリクス

`flw` は `<T>` 構文を持たない。`flw` は trf/fn 名のリスト (`steps: Vec<String>`) で定義され、
型パラメータを宣言する場所がない。

ただし、構成する trf が `Type::Var` を含む場合、checker は `flw` の型として `Arrow(Var, Var)` を推論する。
使用時に引数の型と `unify` することで具体型が確定する。

実装上は `register_item_signatures` で `flw` の各 step の型を `unify` しながら合成し、
最終的な入出力型（`Type::Var` を含んでも良い）を `env` に登録する。

### cap の `impl` と `fn` の違い

`impl` ブロック内の関数は通常の `fn` ではなく **メソッド定義**。型パラメータを持てず、必ず対応する cap フィールドの型に合致しなければならない。

### cap インスタンスの一意性

同一の `(cap名, 型名)` に対して複数の `impl` がある場合は E022 でエラー。

### `sort` の実装方針

`sort<T>(items: List<T>, ord: Ord<T>) -> List<T>` は評価器内で組み込みとして実装する（`ord.compare` を Rust から呼び出す形で対応）。ユーザーが `sort` を `.fav` で定義した場合はその定義を優先する。
