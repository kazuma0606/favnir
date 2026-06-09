# v13.4.0 Spec — CommonCtx / LoadCtx / WriteCtx / MigrateCtx 実装

Date: 2026-06-10
Branch: feat/v13-capability-context

---

## テーマ

ステージ別コンテキスト interface を型システムに組み込む。

v13.2.0〜v13.3.0 で個別 capability interface（DbRead / DbWrite / StorageRead /
StorageWrite / HttpClient / Io / Env）が揃った。
本バージョンでは、これらを組み合わせた「用途別コンテキスト interface」を実装し、
ステージが必要な capability だけを宣言・アクセスできる設計を実現する。

---

## 1. コンテキスト interface の型定義

```
interface CommonCtx {
    io:  Io
    env: Env
}

interface LoadCtx: CommonCtx {
    db: DbRead
}

interface WriteCtx: CommonCtx {
    db:      DbWrite
    storage: StorageWrite
}

interface MigrateCtx: CommonCtx {
    db_read:  DbRead
    db_write: DbWrite
}
```

**継承**:
- `LoadCtx: CommonCtx` → LoadCtx は `io` と `env` フィールドを CommonCtx から継承する
- `ctx.io.println(...)` は `fn load(ctx: LoadCtx)` 内で有効（継承経由）
- `ctx.db.query(...)` は `fn load(ctx: LoadCtx)` 内で有効（LoadCtx 固有フィールド）

---

## 2. E0020 / E0021 エラーコードの使い分け

### E0020 — capability interface のメソッドが存在しない

```
fn run(ctx: LoadCtx) -> Result<Int, String> {
    ctx.db.execute("INSERT ...", List.empty())
    //    ^^^^^^^^
    // E0020: interface `DbRead` has no method `execute`
    //   = help: use a DbWrite context to call execute
}
```

`ctx.db` の型は `DbRead`。`DbRead` に `execute` メソッドはないため E0020。

### E0021 — コンテキスト interface にフィールドが存在しない

```
fn run(ctx: LoadCtx) -> Result<Unit, String> {
    ctx.storage.put("bucket", "key", "body")
    //  ^^^^^^^
    // E0021: capability `storage` not in context `LoadCtx`
    //   = help: use a WriteCtx to access storage capability
    //   = note: LoadCtx provides: db, io, env
}
```

`LoadCtx` に `storage` フィールドはないため E0021。

**区別の根拠**:
- E0020: capability interface の `methods` テーブルを参照したが見つからない
- E0021: context interface の `fields` テーブルを参照したが見つからない
- `InterfaceDef` に `is_context: bool` フラグを追加して区別する

---

## 3. `InterfaceDef` への `is_context` フラグ追加

```rust
pub struct InterfaceDef {
    pub super_interface: Option<String>,
    pub methods: HashMap<String, Type>,
    pub is_context: bool,   // NEW: true → context interface, false → capability interface
}
```

- **context interface** (`is_context = true`): フィールドが capability interface 型を持つ
  → フィールド未発見時に E0021 を emit
- **capability interface** (`is_context = false`): フィールドがメソッド（Fn 型）を持つ
  → メソッド未発見時に E0020 を emit

`register_interface` のシグネチャに `is_context: bool` を追加するか、
`register_context_interface` という専用メソッドを追加する。

---

## 4. Context interface の登録（`register_builtin_capabilities` に追加）

フィールドは `Type::Interface(name, vec![])` として `methods` HashMap に格納する。
（既存 `lookup_declared_method` が継承チェーンを辿るため、そのまま利用できる）

```rust
// CommonCtx
let mut common_ctx = HashMap::new();
common_ctx.insert("io".into(),  Type::Interface("Io".into(),  vec![]));
common_ctx.insert("env".into(), Type::Interface("Env".into(), vec![]));
self.register_context_interface("CommonCtx".into(), None, common_ctx);

// LoadCtx: CommonCtx + db: DbRead
let mut load_ctx = HashMap::new();
load_ctx.insert("db".into(), Type::Interface("DbRead".into(), vec![]));
self.register_context_interface("LoadCtx".into(), Some("CommonCtx".into()), load_ctx);

// WriteCtx: CommonCtx + db: DbWrite + storage: StorageWrite
let mut write_ctx = HashMap::new();
write_ctx.insert("db".into(),      Type::Interface("DbWrite".into(),    vec![]));
write_ctx.insert("storage".into(), Type::Interface("StorageWrite".into(), vec![]));
self.register_context_interface("WriteCtx".into(), Some("CommonCtx".into()), write_ctx);

// MigrateCtx: CommonCtx + db_read: DbRead + db_write: DbWrite
let mut migrate_ctx = HashMap::new();
migrate_ctx.insert("db_read".into(),  Type::Interface("DbRead".into(),  vec![]));
migrate_ctx.insert("db_write".into(), Type::Interface("DbWrite".into(), vec![]));
self.register_context_interface("MigrateCtx".into(), Some("CommonCtx".into()), migrate_ctx);
```

---

## 5. `resolve_field_access` の E0020 / E0021 分岐

`checker.rs` の E0020 emit 箇所（2 か所）を `is_context` フラグで分岐する:

```rust
if let Some(def) = self.interface_registry.interfaces.get(interface_name) {
    if def.is_context {
        // context interface — フィールドが見つからない → E0021
        self.type_error(
            "E0021",
            format!("capability `{}` not in context `{}`", field, interface_name),
            span,
        );
    } else {
        // capability interface — メソッドが見つからない → E0020
        self.type_error(
            "E0020",
            format!("interface `{}` has no method `{}`", interface_name, field),
            span,
        );
    }
}
```

---

## 6. `error_catalog.rs` と `get_help_text` への E0021 追加

```
E0021: capability `storage` not in context `LoadCtx`
  --> pipeline.fav:12:5
   |
12 |     ctx.storage.put("bucket", "key", "body")
   |     ^^^^^^^^^^^
   |
   = help: use a WriteCtx to access storage capability
   = help: available in LoadCtx: db, io, env
   = note: switch context type to WriteCtx or MigrateCtx for write operations
```

---

## 7. テスト

| テスト名 | 内容 |
|---|---|
| `version_is_13_4_0` | Cargo.toml バージョン確認 |
| `context_interfaces_registered` | CommonCtx / LoadCtx / WriteCtx / MigrateCtx が登録済み |
| `load_ctx_has_db_field` | `InterfaceRegistry::lookup_declared_method("LoadCtx", "db")` が `Interface("DbRead", [])` を返す |
| `load_ctx_inherits_io` | `fn f(ctx: LoadCtx) { ctx.io.println("x") }` → no error (CommonCtx 継承) |
| `load_ctx_allows_db_read` | `fn f(ctx: LoadCtx) { ctx.db.query("SELECT 1", List.empty()) }` → no error |
| `load_ctx_rejects_db_write` | `ctx.db.execute(...)` on LoadCtx → E0020 (DbRead に execute なし) |
| `load_ctx_rejects_storage` | `ctx.storage.put(...)` on LoadCtx → E0021 (LoadCtx に storage なし) |
| `write_ctx_allows_db_write` | `fn f(ctx: WriteCtx) { ctx.db.execute(...) }` → no error |
| `write_ctx_allows_storage` | `fn f(ctx: WriteCtx) { ctx.storage.put(...) }` → no error |
| `migrate_ctx_has_both_db` | `ctx.db_read.query(...)` と `ctx.db_write.execute(...)` が共に通る |
| `e0021_not_e0020_for_ctx_field` | context interface のフィールド未発見は E0021、capability のメソッド未発見は E0020 |
