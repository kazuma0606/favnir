# v13.4.0 Implementation Plan

Date: 2026-06-10

---

## Phase A — `InterfaceDef` に `is_context` フラグ追加

**ファイル**: `fav/src/middle/checker.rs`

### A-1: `InterfaceDef` 構造体を更新

```rust
pub struct InterfaceDef {
    pub super_interface: Option<String>,
    pub methods: HashMap<String, Type>,
    pub is_context: bool,   // v13.4.0: true = context interface, false = capability
}
```

### A-2: `register_interface` のシグネチャ更新

既存の `register_interface` に `is_context: bool` 引数を追加するか、
専用メソッド `register_context_interface` を追加する。

**推奨: 専用メソッドを追加（既存コードへの影響を最小化）**

```rust
fn register_context_interface(
    &mut self,
    name: String,
    super_interface: Option<String>,
    fields: HashMap<String, Type>,
) {
    self.interfaces.insert(name, InterfaceDef {
        super_interface,
        methods: fields,
        is_context: true,
    });
}
```

既存の `register_interface` は `is_context: false` を使うよう更新:

```rust
pub fn register_interface(
    &mut self,
    name: String,
    super_interface: Option<String>,
    methods: HashMap<String, Type>,
) {
    self.interfaces.insert(name, InterfaceDef {
        super_interface,
        methods,
        is_context: false,
    });
}
```

### A-3: コンパイル確認

`InterfaceDef` の構造体初期化箇所（`register_interface` 以外）も `is_context` フィールドを追加。
`grep -n "InterfaceDef {" src/middle/checker.rs` で全件確認。

---

## Phase B — Context interface の登録

**ファイル**: `fav/src/middle/checker.rs` の `register_builtin_capabilities`

`StorageWrite` 登録の直後に追加:

```rust
// v13.4.0: Context composite interfaces
// CommonCtx — 全コンテキストの基底（io + env）
let mut common_ctx = HashMap::new();
common_ctx.insert("io".into(),  Type::Interface("Io".into(),  vec![]));
common_ctx.insert("env".into(), Type::Interface("Env".into(), vec![]));
self.register_context_interface("CommonCtx".into(), None, common_ctx);

// LoadCtx: CommonCtx + db: DbRead（読み取り専用ステージ）
let mut load_ctx = HashMap::new();
load_ctx.insert("db".into(), Type::Interface("DbRead".into(), vec![]));
self.register_context_interface("LoadCtx".into(), Some("CommonCtx".into()), load_ctx);

// WriteCtx: CommonCtx + db: DbWrite + storage: StorageWrite（書き込みステージ）
let mut write_ctx = HashMap::new();
write_ctx.insert("db".into(),      Type::Interface("DbWrite".into(),     vec![]));
write_ctx.insert("storage".into(), Type::Interface("StorageWrite".into(), vec![]));
self.register_context_interface("WriteCtx".into(), Some("CommonCtx".into()), write_ctx);

// MigrateCtx: CommonCtx + db_read: DbRead + db_write: DbWrite（マイグレーションステージ）
let mut migrate_ctx = HashMap::new();
migrate_ctx.insert("db_read".into(),  Type::Interface("DbRead".into(),  vec![]));
migrate_ctx.insert("db_write".into(), Type::Interface("DbWrite".into(), vec![]));
self.register_context_interface("MigrateCtx".into(), Some("CommonCtx".into()), migrate_ctx);
```

---

## Phase C — `resolve_field_access` の E0020 / E0021 分岐

**ファイル**: `fav/src/middle/checker.rs`

現在 E0020 を emit している 2 か所（`Named` 型ブランチと `Interface(name, [])` ブランチ）を更新:

```rust
// 分岐ヘルパー（インライン）
let is_ctx = self.interface_registry
    .interfaces.get(iface_name)
    .map(|d| d.is_context)
    .unwrap_or(false);

if is_ctx {
    self.type_error(
        "E0021",
        format!("capability `{}` not in context `{}`", field, iface_name),
        span,
    );
} else {
    self.type_error(
        "E0020",
        format!("interface `{}` has no method `{}`", iface_name, field),
        span,
    );
}
return Type::Error;
```

**適用箇所**:
1. `Named` 型ブランチ（line ~4591）
2. `Interface(name, [])` ブランチ（line ~4613）

---

## Phase D — E0021 ヘルプテキスト + error_catalog.rs

**ファイル**: `fav/src/driver.rs` の `get_help_text`

```rust
"E0021" => &[
    "switch to a context that includes this capability",
    "LoadCtx provides: db(DbRead), io, env",
    "WriteCtx provides: db(DbWrite), storage, io, env",
    "MigrateCtx provides: db_read, db_write, io, env",
],
```

`fav/src/error_catalog.rs` に E0021 エントリを追加（既存パターンに従う）。

---

## Phase E — テストと動作確認

**ファイル**: `fav/src/driver.rs` の末尾に `v134000_tests` モジュールを追加

テストケース一覧（spec の表を参照）。

主要パターン:
```rust
fn check_src(src: &str) -> (Vec<TypeError>, Vec<FavWarning>) {
    let prog = Parser::parse_str(src, "test.fav").expect("parse error");
    Checker::check_program(&prog)
}

// LoadCtx inherits io from CommonCtx
let src = r#"
interface WithLoad { db: DbRead  io: Io }
public fn f(ctx: WithLoad) -> Unit {
    ctx.io.println("loading...")
}
"#;
// (Using inline interface here since LoadCtx is built-in and registered)
```

**注意**: built-in context interface のテストは `InterfaceRegistry` から直接確認するか、
または `check_src` 内でユーザー定義 interface で同等の構造を再現して検証する。

---

## Phase F — バージョンバンプ + コミット

1. `fav/Cargo.toml` → `version = "13.4.0"`
2. `v133000_tests::version_is_13_3_0` をコメントアウト
3. `cargo test -- --test-threads=1` 全件パス確認
4. `git add` + `git commit -m "feat: v13.4.0 — CommonCtx/LoadCtx/WriteCtx/MigrateCtx + E0021"`
5. `git push origin feat/v13-capability-context`

---

## 技術的注意点

### `Interface` 型のフィールド格納

Context interface のフィールドは `Type::Interface("DbRead", vec![])` として `methods` HashMap に格納する。
`lookup_declared_method` は既に inheritance chain を辿るため、`LoadCtx` で `db` を lookup すると
直接返り、`io` / `env` は CommonCtx 継承経由で返る。

### `InterfaceDef` 初期化箇所の全件確認

`register_interface` 以外で `InterfaceDef { super_interface, methods }` を直接構築している箇所が
ある場合（`checker.rs` 後半の user-defined interface 処理）、`is_context: false` を追加する必要がある。

具体的には `grep "InterfaceDef {" src/middle/checker.rs` で確認し、
user-defined `interface` 宣言の処理箇所に `is_context: false` を追加する。

### `register_interface` の呼び出し箇所（`checker.rs` 後半）

`checker.rs` の後半（Checker::check_interface_def 等）でも `register_interface` を呼んでいる可能性がある。
これらはシグネチャ変更不要（既存の `register_interface` は `is_context: false` を使う）。
