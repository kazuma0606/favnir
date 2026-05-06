# Favnir v1.1.0 実装計画

作成日: 2026-05-06

> スコープを守ることが最優先。各フェーズの Done definition を超えない。
>
> **前提**: v1.0.0 完了（321 テスト通過）
>
> **設計ドキュメント**: `dev/post-v1/roadmap/fav-abstraction-system.md`

---

## 実装順序

```
Phase 0 (version bump)
  → Phase 1 (AST + Lexer + Parser)          ← 全フェーズの前提
  → Phase 2 (Checker: InterfaceRegistry)    ← Phase 3/4/5 の前提
  → Phase 3 (Auto-synthesis + with)
  → Phase 4 (標準 interface 移行)            ← Phase 3 と並行可
  → Phase 5 (Gen + Field)                   ← Phase 4 完了後
  → Phase 6 (cap 非推奨警告)                 ← Phase 2 完了後、独立
  → Phase 7 (テスト・ドキュメント)
```

---

## Phase 0: バージョン更新

### Cargo.toml

```toml
version = "1.1.0"
```

### main.rs

```rust
const HELP: &str = "fav - Favnir language toolchain v1.1.0\n...";
```

---

## Phase 1: AST + Lexer + Parser

### 1-1: ast.rs の追加

`InterfaceDecl`・`ImplDecl` ノードを追加。`Program` に含める。

```rust
// ast.rs

#[derive(Debug, Clone)]
pub struct InterfaceMethod {
    pub name: String,
    pub ty:   TypeExpr,     // Self -> String など
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct InterfaceDecl {
    pub name:            String,
    pub super_interface: Option<String>,  // Ord : Eq の "Eq"
    pub methods:         Vec<InterfaceMethod>,
    pub span:            Span,
}

#[derive(Debug, Clone)]
pub struct ImplDecl {
    pub interface_names: Vec<String>,         // impl Show, Eq for T → ["Show", "Eq"]
    pub type_name:       String,
    pub type_params:     Vec<String>,         // impl Show for List<T> → ["T"]
    pub methods:         Vec<(String, Expr)>, // 手書き実装のメソッド本体
    pub is_auto:         bool,                // true = 本体なし（自動合成）
    pub span:            Span,
}
```

`TypeDecl` に `with_interfaces: Vec<String>` フィールドを追加:

```rust
// 既存の TypeDecl に追加
pub struct TypeDecl {
    pub name:             String,
    pub type_params:      Vec<String>,
    pub fields:           Vec<FieldDecl>,
    pub with_interfaces:  Vec<String>,  // 追加: type T with Show, Eq { ... }
    pub span:             Span,
}
```

`Program` に `interface_decls` と `impl_decls` を追加:

```rust
pub struct Program {
    pub fn_defs:         Vec<FnDef>,
    pub trf_defs:        Vec<TrfDef>,
    pub flw_defs:        Vec<FlwDef>,
    pub type_defs:       Vec<TypeDecl>,
    pub use_decls:       Vec<UseDecl>,
    pub interface_decls: Vec<InterfaceDecl>,   // 追加
    pub impl_decls:      Vec<ImplDecl>,        // 追加
}
```

### 1-2: lexer.rs の変更

新規キーワードを追加:

```rust
// lexer.rs の keyword match に追加
"interface" => Token::Interface,
"with"      => Token::With,
// "impl" と "for" は v0.4.0 の cap システムで既に追加済みか確認
// 未追加なら追加する
"impl"      => Token::Impl,
"for"       => Token::For,
```

対応する `Token` バリアントを `lexer.rs` に追加:

```rust
Interface,
With,
// Impl, For が未定義なら追加
```

### 1-3: parser.rs の変更

#### `parse_interface_decl`

```rust
// "interface" Name (":" SuperName)? "{" method* "}"
fn parse_interface_decl(&mut self) -> Result<InterfaceDecl, ParseError> {
    let span_start = self.current_span();
    self.expect(Token::Interface)?;
    let name = self.expect_ident()?;

    let super_interface = if self.peek_is(Token::Colon) {
        self.advance();
        Some(self.expect_ident()?)
    } else {
        None
    };

    self.expect(Token::LBrace)?;
    let mut methods = vec![];
    while !self.peek_is(Token::RBrace) {
        let method_name = self.expect_ident()?;
        self.expect(Token::Colon)?;
        let ty = self.parse_type_expr()?;
        methods.push(InterfaceMethod { name: method_name, ty, span: self.current_span() });
    }
    self.expect(Token::RBrace)?;

    Ok(InterfaceDecl { name, super_interface, methods, span: span_start })
}
```

#### `parse_impl_decl`

```rust
// "impl" InterfaceName ("," InterfaceName)* "for" TypeName ("{" method* "}")?
fn parse_impl_decl(&mut self) -> Result<ImplDecl, ParseError> {
    let span_start = self.current_span();
    self.expect(Token::Impl)?;

    let mut interface_names = vec![self.expect_ident()?];
    while self.peek_is(Token::Comma) {
        self.advance();
        interface_names.push(self.expect_ident()?);
    }

    self.expect(Token::For)?;
    let type_name = self.expect_ident()?;

    // 本体あり vs 本体なし（自動合成）
    let (methods, is_auto) = if self.peek_is(Token::LBrace) {
        self.advance();
        let mut methods = vec![];
        while !self.peek_is(Token::RBrace) {
            let method_name = self.expect_ident()?;
            self.expect(Token::Eq)?;
            let body = self.parse_expr()?;
            methods.push((method_name, body));
        }
        self.expect(Token::RBrace)?;
        (methods, false)
    } else {
        (vec![], true)  // 本体なし = 自動合成
    };

    Ok(ImplDecl { interface_names, type_name, type_params: vec![], methods, is_auto, span: span_start })
}
```

#### `parse_type_decl` の拡張（`with` 糖衣構文）

既存の型宣言パーサーに `with` 節を追加:

```rust
// "type" Name ("with" Interface ("," Interface)*)? "{" ... "}"
let with_interfaces = if self.peek_is(Token::With) {
    self.advance();
    let mut ifaces = vec![self.expect_ident()?];
    while self.peek_is(Token::Comma) {
        self.advance();
        ifaces.push(self.expect_ident()?);
    }
    ifaces
} else {
    vec![]
};
```

#### トップレベルパーサーへの組み込み

`parse_program` の `match` に `Token::Interface` と `Token::Impl` を追加:

```rust
Token::Interface => {
    program.interface_decls.push(self.parse_interface_decl()?);
}
Token::Impl => {
    program.impl_decls.push(self.parse_impl_decl()?);
}
```

---

## Phase 2: 型検査統合（InterfaceRegistry）

### 2-1: InterfaceRegistry の定義

`middle/checker.rs` に追加:

```rust
#[derive(Debug, Clone)]
struct InterfaceDef {
    super_interface: Option<String>,
    methods:         Vec<(String, Type)>,  // (name, type_sig with Self resolved)
}

#[derive(Debug, Clone)]
struct ImplEntry {
    methods: HashMap<String, Type>,  // method name → 実際の型
    is_auto: bool,
}

struct InterfaceRegistry {
    interfaces: HashMap<String, InterfaceDef>,
    impls:      HashMap<(String, String), ImplEntry>,  // (interface_name, type_name)
}

impl InterfaceRegistry {
    fn new() -> Self { ... }
    fn register_interface(&mut self, decl: &InterfaceDecl) { ... }
    fn register_impl(&mut self, decl: &ImplDecl) { ... }
    fn is_implemented(&self, interface: &str, type_name: &str) -> bool { ... }
    fn lookup_method(&self, interface: &str, type_name: &str, method: &str) -> Option<&Type> { ... }
}
```

`Checker` フィールドに追加:

```rust
pub struct Checker {
    // ... 既存フィールド ...
    interface_registry: InterfaceRegistry,
}
```

### 2-2: `Type` の拡張

`checker.rs` の `Type` enum に追加:

```rust
pub enum Type {
    // ... 既存 ...
    Interface(String, Vec<Type>),   // "Ord", [User]  — interface value の型
}
```

### 2-3: interface 宣言の型検査

`check_program` で最初に全 `interface_decls` を登録:

```rust
for decl in &program.interface_decls {
    self.interface_registry.register_interface(decl);
}
```

### 2-4: `impl` 手書き実装の型検査

`impl` 宣言を処理するとき:

1. interface_names の各 interface が登録済みか確認（未定義なら E041）
2. 各メソッドの本体型が interface のシグネチャと一致するか確認（不一致なら E042）
3. スーパーインターフェースが満たされているか確認（例: `impl Ord for T` があれば `impl Eq for T` も必要。なければ E043）

```rust
fn check_impl_decl(&mut self, decl: &ImplDecl) -> Vec<TypeError> {
    let mut errors = vec![];
    for iface_name in &decl.interface_names {
        match self.interface_registry.interfaces.get(iface_name) {
            None => errors.push(TypeError::new(E041, ...)),
            Some(iface_def) => {
                // スーパーインターフェース充足チェック
                if let Some(super_name) = &iface_def.super_interface {
                    if !self.interface_registry.is_implemented(super_name, &decl.type_name) {
                        errors.push(TypeError::new(E043, ...));
                    }
                }
                // 手書き実装の場合はメソッド型検査
                if !decl.is_auto {
                    for (method_name, body) in &decl.methods {
                        let expected = iface_def.method_type(method_name, &decl.type_name);
                        let actual   = self.check_expr(body);
                        if !self.unify(&expected, &actual).is_ok() {
                            errors.push(TypeError::new(E042, ...));
                        }
                    }
                }
            }
        }
    }
    errors
}
```

### 2-5: 明示的な値渡しの型検査

`fn sort<T>(items: List<T>, ord: Ord<T>) -> List<T>` のパラメータ型として
`Type::Interface("Ord", [Type::Var("T")])` を扱う。

呼び出し時 `sort(users, User.ord)`:
- `User.ord` の型は `Type::Interface("Ord", [Type::Con("User")])`
- checker が `Ord<User>` の `impl` が存在するか確認（なければ E043）

**エラーコード一覧（新規）**

| コード | 内容 |
|---|---|
| E041 | 未定義 interface を `impl` しようとした |
| E042 | `impl` メソッドの型が interface シグネチャと不一致 |
| E043 | 値渡し時または `impl` 時に要求 interface が未実装 |
| E044 | 自動合成時にフィールドが interface を未実装 |
| W010 | `cap` キーワードの使用（deprecated） |

---

## Phase 3: 自動合成 + `with` 糖衣構文

### 3-1: 自動合成ロジック

`impl Show, Eq for UserRow`（本体なし）が来たとき:

```rust
fn synthesize_impl(&mut self, decl: &ImplDecl) -> Vec<TypeError> {
    let mut errors = vec![];
    let type_def = self.lookup_type_def(&decl.type_name);

    for iface_name in &decl.interface_names {
        // 全フィールドが interface を実装しているか確認
        for field in &type_def.fields {
            if !self.interface_registry.is_implemented(iface_name, &field.ty.name()) {
                errors.push(TypeError::new(E044,
                    format!("field `{}` of type `{}` does not implement `{}`",
                            field.name, field.ty.name(), iface_name)));
            }
        }
        if errors.is_empty() {
            // 合成実装を ImplEntry として登録
            let entry = self.build_synthesized_impl(iface_name, &type_def);
            self.interface_registry.impls.insert(
                (iface_name.clone(), decl.type_name.clone()), entry
            );
        }
    }
    errors
}
```

合成ルール:
- `show` : `"{field1: {field1.show()}, ...}"` 形式の文字列を生成
- `eq`   : 全フィールドの eq を AND で結合
- `gen`  : 全フィールドに `gen(derived_seed)` を呼ぶ

### 3-2: `with` 糖衣構文の処理

`parse_type_decl` で取得した `with_interfaces` を、`check_program` 内で
`ImplDecl { is_auto: true, ... }` として展開してから `check_impl_decl` を呼ぶ:

```rust
for type_def in &program.type_defs {
    if !type_def.with_interfaces.is_empty() {
        let synthetic_impl = ImplDecl {
            interface_names: type_def.with_interfaces.clone(),
            type_name:       type_def.name.clone(),
            is_auto:         true,
            methods:         vec![],
            ...
        };
        self.check_impl_decl(&synthetic_impl);
    }
}
```

---

## Phase 4: 標準 interface 移行（Eq / Ord / Show）

### 設計方針

既存の `IMPL_REGISTRY`（v0.4.0 の cap 用 thread-local）は **残す**。
v1.1.0 では `InterfaceRegistry` に標準 interface の定義と組み込み型の実装を内部登録し、
`cap` ベースの呼び出しは `IMPL_REGISTRY` 経由のまま動作させる。

```
IMPL_REGISTRY (旧 cap)     InterfaceRegistry (新)
      │                           │
      │  ← ブリッジ: 同じ型か確認   │
      └──────────────────────────▶│ (v2.0.0 で IMPL_REGISTRY 削除)
```

### 標準 interface の内部定義

`Checker::new()` 内で以下を登録:

```rust
fn register_builtin_interfaces(registry: &mut InterfaceRegistry) {
    // Show
    registry.register_builtin("Show", None, vec![("show", "Self -> String")]);
    for ty in ["Int", "Float", "Bool", "String"] {
        registry.register_builtin_impl("Show", ty);
    }

    // Eq
    registry.register_builtin("Eq", None, vec![("eq", "Self -> Self -> Bool")]);
    for ty in ["Int", "Float", "Bool", "String"] {
        registry.register_builtin_impl("Eq", ty);
    }

    // Ord : Eq
    registry.register_builtin("Ord", Some("Eq"), vec![("compare", "Self -> Self -> Int")]);
    for ty in ["Int", "Float", "String"] {
        registry.register_builtin_impl("Ord", ty);
    }
}
```

組み込み型の `List<T>`, `Option<T>`, `Result<T,E>` については、
`T` が `Show`/`Eq` を持つときのみ自動登録する（`synthesize_impl` の再帰呼び出し）。

---

## Phase 5: `Gen` + `Field` interface 定義

### Gen interface

```rust
// Checker::new() の組み込み登録に追加
registry.register_builtin("Gen", None, vec![("gen", "Int? -> Self")]);

// 組み込み型の Gen
for ty in ["Int", "Float", "Bool", "String"] {
    registry.register_builtin_impl("Gen", ty);
}

// List<T>, Option<T> は T が Gen を持つとき自動登録
```

`impl Gen for UserRow`（本体なし）の合成ロジック:
- 全フィールドが `Gen` を実装 → フィールドごとに `gen(seed ^ field_index)` を呼ぶ合成実装を登録
- フィールドに `Gen` 未実装があれば E044

### Field 系列

```rust
registry.register_builtin("Semigroup", None,     vec![("combine", "Self -> Self -> Self")]);
registry.register_builtin("Monoid",    Some("Semigroup"), vec![("empty", "Self")]);
registry.register_builtin("Group",     Some("Monoid"),    vec![("inverse", "Self -> Self")]);
registry.register_builtin("Ring",      Some("Monoid"),    vec![("multiply", "Self -> Self -> Self")]);
registry.register_builtin("Field",     Some("Ring"),      vec![("divide",   "Self -> Self -> Self!")]);

// Float は Field まで
for iface in ["Semigroup", "Monoid", "Group", "Ring", "Field"] {
    registry.register_builtin_impl(iface, "Float");
}
// Int は Ring まで（Field は除く: 整数除算は切り捨てのため）
for iface in ["Semigroup", "Monoid", "Group", "Ring"] {
    registry.register_builtin_impl(iface, "Int");
}
```

演算子オーバーロードは v1.1.0 では **型検査の登録のみ**。
実際の `+`/`*`/`/` の挙動は今まで通り（組み込み型にハードコード）。
将来的には `Semigroup::combine` に委譲する。

---

## Phase 6: `cap` 非推奨警告

### 実装方針

パーサーで `Token::Cap` を検出したとき、`TypeError` ではなく `TypeWarning` を生成する。
コンパイルは通すが、`fav check` 時に警告一覧に追加する。

```rust
// checker.rs に追加
pub struct TypeWarning {
    pub code:    String,  // "W010"
    pub message: String,
    pub span:    Span,
}
```

`Checker` に `pub warnings: Vec<TypeWarning>` フィールドを追加。

`check_cap_decl` の先頭に挿入:

```rust
fn check_cap_decl(&mut self, decl: &CapDecl) {
    self.warnings.push(TypeWarning {
        code:    "W010".into(),
        message: format!("`cap` is deprecated. Use `interface` instead."),
        span:    decl.span.clone(),
    });
    // 既存の cap 型検査を続行
    ...
}
```

### fav check の出力変更

`driver.rs` の `cmd_check` で `checker.warnings` を表示:

```rust
for warning in &checker.warnings {
    eprintln!("warning[{}]: {} ({}:{})", warning.code, warning.message,
              path, warning.span.line);
}
```

`--no-warn` フラグで W010 を抑制:

```rust
if !args.no_warn {
    print_warnings(&checker.warnings);
}
```

---

## Phase 7: テスト・ドキュメント

### テスト追加場所

- `middle/checker.rs` のインラインテスト（`#[cfg(test)]` 内）
- または `src/integration/interface_tests.rs`（既存の integration テストの隣）

### example ファイル

```
examples/
  interface_basic.fav   -- interface / impl の基本使用例
  interface_auto.fav    -- with 糖衣構文と自動合成の例
  algebraic.fav         -- Field / Ring を使った加重平均の例
```

### langspec.md 更新

`versions/v1.0.0/langspec.md` の「5. モジュールシステム」節の後に「6. interface システム」を追加。
旧 `cap` の説明は「6.x. 後方互換: cap キーワード（非推奨）」として残す。

### Cargo.toml

v1.1.0 では **Cargo.toml への依存追加なし**（標準ライブラリのみで実装可能）。

---

## 先送り一覧

| 制約 | バージョン |
|---|---|
| `abstract type` / `abstract stage` / `abstract seq` | v1.3.0 |
| `invariant` | v1.2.0 |
| `Stat.one<T>` の実際の動作 | v1.5.0（Gen interface の利用側） |
| 演算子オーバーロードの実際の委譲 | v2.0.0 以降 |
| `interface` を rune 境界を越えて使う | v1.3.0 以降 |
| IMPL_REGISTRY の削除 | v2.0.0 |
| `fav migrate` コマンド | v2.0.0 |
