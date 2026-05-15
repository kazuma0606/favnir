# Favnir v0.3.0 実装計画

更新日: 2026-04-28

---

## Phase 1: Lexer / Parser

### 新トークン・キーワード

| 追加 | 内容 |
|---|---|
| `TokenKind::Namespace` | `namespace` キーワード |
| `TokenKind::Use` | `use` キーワード |

`rune` キーワードは v0.3.0 では構文に現れないため、追加不要。

### AST 拡張

```rust
// Item への追加
pub enum Item {
    NamespaceDecl(String, Span),        // namespace data.users
    UseDecl(Vec<String>, Span),         // use data.users.create → ["data","users","create"]
    TypeDef(TypeDef),
    FnDef(FnDef),
    TrfDef(TrfDef),
    FlwDef(FlwDef),
}
```

`Program` 構造体の変更:

```rust
pub struct Program {
    pub namespace: Option<String>,       // ファイル先頭の namespace 宣言
    pub uses: Vec<Vec<String>>,          // use 宣言一覧
    pub items: Vec<Item>,                // 定義一覧 (TypeDef / FnDef / ...)
}
```

`namespace` と `use` は `Program` のトップレベルフィールドに分離する。
`Item::NamespaceDecl` / `Item::UseDecl` は内部表現用に保持してもよいが、
checker / eval は `program.namespace` / `program.uses` を参照する。

### パーサ変更

`parse_program()` の先頭で `namespace` / `use` を収集する:

```rust
fn parse_program(&mut self) -> Result<Program, ParseError> {
    // 1. namespace (optional, must be first)
    let namespace = if self.peek() == &TokenKind::Namespace {
        Some(self.parse_namespace_decl()?)
    } else {
        None
    };

    // 2. use 宣言 (0 個以上)
    let mut uses = Vec::new();
    while self.peek() == &TokenKind::Use {
        uses.push(self.parse_use_decl()?);
    }

    // 3. 定義
    let mut items = Vec::new();
    while self.peek() != &TokenKind::Eof {
        items.push(self.parse_item()?);
    }

    Ok(Program { namespace, uses, items })
}
```

`parse_namespace_decl()`:

```rust
fn parse_namespace_decl(&mut self) -> Result<String, ParseError> {
    self.expect(&TokenKind::Namespace)?;
    let path = self.parse_module_path()?;   // IDENT ("." IDENT)*
    Ok(path.join("."))
}
```

`parse_use_decl()`:

```rust
fn parse_use_decl(&mut self) -> Result<Vec<String>, ParseError> {
    self.expect(&TokenKind::Use)?;
    self.parse_module_path()    // Vec<String>
}
```

`parse_module_path()`:

```rust
fn parse_module_path(&mut self) -> Result<Vec<String>, ParseError> {
    let mut parts = Vec::new();
    let (name, _) = self.expect_ident()?;
    parts.push(name);
    while self.peek() == &TokenKind::Dot {
        self.advance();
        let (seg, _) = self.expect_ident()?;
        parts.push(seg);
    }
    Ok(parts)
}
```

---

## Phase 2: fav.toml パーサ

`fav.toml` を読み込む最小パーサを `src/toml.rs` として実装する。
外部クレートは使わず、必要な箇所だけ手書きで解析する。

```rust
// src/toml.rs
pub struct FavToml {
    pub name: String,
    pub version: String,
    pub src: String,   // デフォルト "."
}

impl FavToml {
    pub fn load(project_root: &Path) -> Option<Self> { ... }
    pub fn find_root(start: &Path) -> Option<PathBuf> { ... }
}
```

`find_root` は指定ディレクトリから上位に向かって `fav.toml` を探す。

### fav.toml のパース方針

`toml` クレートは追加せず、最小限のラインパーサで対応する:

```rust
// [rune] セクションの key = "value" を読む
fn parse_fav_toml(content: &str) -> FavToml { ... }
```

対応するのは `name`, `version`, `src` のみ。
`#` コメント行はスキップ。

---

## Phase 3: Module Resolver

`src/resolver.rs` を新規作成する。

```rust
pub struct Resolver {
    /// fav.toml の情報 (プロジェクトモードのみ)
    pub toml: Option<FavToml>,
    /// プロジェクトルート
    pub root: Option<PathBuf>,
    /// ロード済みモジュールのキャッシュ
    /// key: モジュールパス ("data.users")
    /// value: そのモジュールの公開シンボル (name → (Type, Visibility))
    pub modules: HashMap<String, ModuleScope>,
    /// 循環検出用: 現在ロード中のモジュールパス
    loading: HashSet<String>,
}
```

```rust
pub struct ModuleScope {
    pub symbols: HashMap<String, (Type, Visibility)>,
}
```

### `Resolver::resolve_use`

```rust
pub fn resolve_use(
    &mut self,
    use_path: &[String],
    span: &Span,
    errors: &mut Vec<TypeError>,
) -> Option<(String, Type)> {
    // use_path = ["data", "users", "create"]
    // sym_name = "create"
    // mod_path = "data.users"
    // → src/data/users.fav をロードして "create" を返す
}
```

### ファイルパス変換

```
mod_path: "data.users"
src_dir:  "src"
→ file:   "src/data/users.fav"
```

```rust
fn mod_path_to_file(root: &Path, src: &str, mod_path: &str) -> PathBuf {
    let rel: PathBuf = mod_path.split('.').collect::<PathBuf>();
    root.join(src).join(rel).with_extension("fav")
}
```

### モジュールのロード・型チェック

```rust
fn load_module(&mut self, mod_path: &str, errors: &mut Vec<TypeError>) -> Option<&ModuleScope> {
    if self.modules.contains_key(mod_path) {
        return self.modules.get(mod_path);
    }
    if self.loading.contains(mod_path) {
        // 循環 import → E012
        return None;
    }
    self.loading.insert(mod_path.to_string());

    let file = mod_path_to_file(...);
    let source = std::fs::read_to_string(&file).ok()?;
    let tokens = Lexer::new(&source).tokenize()?;
    let program = Parser::new(tokens).parse()?;

    // 再帰: このモジュールの use を解決してから checker を走らせる
    let mut checker = Checker::new_with_resolver(self);
    checker.check(&program);

    // public シンボルだけを ModuleScope に収録
    let scope = extract_public_scope(&program, &checker);
    self.modules.insert(mod_path.to_string(), scope);
    self.loading.remove(mod_path);
    self.modules.get(mod_path)
}
```

---

## Phase 4: Checker への統合

### `Checker` の変更

```rust
pub struct Checker {
    env: TyEnv,
    pub errors: Vec<TypeError>,
    type_defs: HashMap<String, TypeBody>,
    current_effects: Vec<Effect>,
    // 新規
    current_file: Option<PathBuf>,            // 現在チェック中のファイル
    current_rune_root: Option<PathBuf>,       // rune root (fav.toml のある dir)
    resolver: Option<Arc<Mutex<Resolver>>>,   // モジュール解決器
    // import されたシンボルの visibility 情報
    imported: HashMap<String, (Type, Visibility, PathBuf)>,
                                              // name → (type, vis, source_file)
}
```

### `check_program` の変更

```rust
pub fn check_program(&mut self, program: &Program) {
    // 1. use 宣言を解決して env に追加
    for use_path in &program.uses {
        self.check_use_decl(use_path);
    }
    // 2. 定義を型チェック (既存処理)
    for item in &program.items {
        self.check_item(item);
    }
}
```

### `check_use_decl`

```rust
fn check_use_decl(&mut self, use_path: &[String]) {
    let sym_name = use_path.last().unwrap();
    let mod_path = use_path[..use_path.len()-1].join(".");

    // resolver にモジュールをロードさせる
    if let Some(resolver) = &self.resolver {
        let mut r = resolver.lock().unwrap();
        if let Some(scope) = r.load_module(&mod_path, &mut self.errors) {
            match scope.symbols.get(sym_name) {
                None => {
                    self.type_error("E013", &format!("`{}` は `{}` に存在しない", sym_name, mod_path), &Span::dummy());
                }
                Some((ty, vis)) if *vis != Visibility::Public => {
                    self.type_error("E014", &format!("`{}` は public でない", sym_name), &Span::dummy());
                }
                Some((ty, _)) => {
                    self.env.insert(sym_name.clone(), ty.clone());
                    self.imported.insert(sym_name.clone(), (ty.clone(), Visibility::Public, ...));
                }
            }
        }
    }
}
```

### visibility enforcement

既存の `check_fn_call` / `check_apply` に参照先の visibility チェックを追加する:

```rust
fn check_symbol_visibility(&mut self, name: &str, span: &Span) {
    if let Some((_, vis, source_file)) = self.imported.get(name) {
        // private: 別ファイルから参照 → E015
        if *vis == Visibility::Private && source_file != self.current_file.as_ref().unwrap() {
            self.type_error("E015", &format!("`{}` は private — 別ファイルから参照不可", name), span);
        }
        // internal: 別 rune から参照 → E016 (v0.3.0 では single-rune のため未発動)
    }
}
```

---

## Phase 5: CLI 変更

### `main.rs` の変更

#### `fav run` のエントリポイント探索

```rust
fn cmd_run(file: Option<&str>, db_url: &str) {
    let (path, root) = if let Some(f) = file {
        (PathBuf::from(f), None)
    } else {
        // fav.toml を探してエントリポイントを自動検出
        let root = FavToml::find_root(&std::env::current_dir().unwrap())?;
        let toml = FavToml::load(&root)?;
        let src = root.join(&toml.src);
        let entry = src.join("main.fav");
        (entry, Some(root))
    };
    // ...
}
```

#### `fav check` のプロジェクトモード

```rust
fn cmd_check(file: Option<&str>) {
    if let Some(f) = file {
        // 既存: 単一ファイルチェック
        check_single_file(f);
    } else {
        // 新規: プロジェクト全体チェック
        let root = FavToml::find_root(&std::env::current_dir().unwrap())?;
        let toml = FavToml::load(&root)?;
        let src = root.join(&toml.src);
        check_all_files_in_dir(&src, &root);
    }
}
```

### 引数パース変更

```
fav run [--db <url>] [<file>]    // <file> を省略可能に
fav check [<file>]               // <file> を省略可能に
fav explain [<file>]             // <file> を省略可能に
```

---

## Phase 6: サンプルの作成

### `examples/multi_file/`

複数ファイル構成のサンプルプロジェクトを作成する。

```
examples/multi_file/
  fav.toml
  src/
    main.fav
    data/
      users.fav
```

動作確認:

```
cd examples/multi_file
fav check             # rune 全体をチェック
fav run --db :memory: # main.fav を実行
fav explain           # 全定義の type / effect / vis を表示
```

---

## 設計メモ

### `Checker` と `Resolver` の関係

`Checker` が `Resolver` を所有するのではなく、`Resolver` を引数で受け取る設計にする。
`Resolver` はファイルをロードするたびに新しい `Checker` を生成して使う。
ループしないよう `loading: HashSet` で循環検出する。

### `Checker` の再利用

現在の `Checker::new()` は単一ファイル用。
`Checker::new_with_resolver(resolver)` を追加して、モジュール解決つきで使う。

### toml クレートを使わない理由

`fav.toml` の v0.3.0 の仕様は 3 キーのみ。外部クレートを増やさず、
シンプルなラインパーサで十分。v1.0.0 で workspace など複雑な設定が必要になったとき、
正式な toml クレートを導入する。

### `Span::dummy()`

use 宣言のエラーを報告するとき、import 元ファイルのスパン情報がない場合がある。
v0.3.0 では `Span::dummy()` (line: 0, col: 0) を使い、
v0.4.0 以降でより正確な位置情報に改善する。

### 単一ファイルモードの後方互換

`fav.toml` がない場合は v0.2.0 と同じ動作をする。
`use` があるのに `fav.toml` がない場合は E013 (モジュールが見つからない) を出す。
