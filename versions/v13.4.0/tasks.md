# v13.4.0 Tasks — CommonCtx / LoadCtx / WriteCtx / MigrateCtx + E0021

Date: 2026-06-10
Branch: feat/v13-capability-context

---

## Phase A — `InterfaceDef` に `is_context` フラグ追加

- [ ] A-1: `fav/src/middle/checker.rs` — `InterfaceDef` 構造体に `is_context: bool` フィールドを追加
  ```rust
  pub struct InterfaceDef {
      pub super_interface: Option<String>,
      pub methods: HashMap<String, Type>,
      pub is_context: bool,   // v13.4.0: true = context interface, false = capability
  }
  ```
- [ ] A-2: `register_context_interface` メソッドを追加（`is_context: true` で登録）
  ```rust
  fn register_context_interface(&mut self, name: String, super_interface: Option<String>, fields: HashMap<String, Type>) {
      self.interfaces.insert(name, InterfaceDef { super_interface, methods: fields, is_context: true });
  }
  ```
- [ ] A-3: 既存の `register_interface` を更新（`is_context: false` を追加）
- [ ] A-4: `grep "InterfaceDef {" src/middle/checker.rs` で全件確認し、`is_context` フィールドを補完
- [ ] A-5: `cargo build` でコンパイルエラーなし確認

---

## Phase B — Context interface の登録

- [ ] B-1: `fav/src/middle/checker.rs` の `register_builtin_capabilities` に以下を追加（`StorageWrite` 登録の直後）
  - `CommonCtx`: `io: Io`, `env: Env`（`super_interface: None`）
  - `LoadCtx`: `db: DbRead`（`super_interface: Some("CommonCtx")`）
  - `WriteCtx`: `db: DbWrite`, `storage: StorageWrite`（`super_interface: Some("CommonCtx")`）
  - `MigrateCtx`: `db_read: DbRead`, `db_write: DbWrite`（`super_interface: Some("CommonCtx")`）
- [ ] B-2: フィールド型は `Type::Interface("DbRead".into(), vec![])` 形式で格納
- [ ] B-3: `cargo build` でコンパイルエラーなし確認

---

## Phase C — `resolve_field_access` の E0020 / E0021 分岐

- [ ] C-1: `fav/src/middle/checker.rs` の `Named` 型ブランチ（line ~4591）を更新
  - `is_context` フラグを確認し、`true` なら E0021、`false` なら E0020 を emit
  ```rust
  let is_ctx = self.interface_registry.interfaces.get(iface_name).map(|d| d.is_context).unwrap_or(false);
  if is_ctx {
      self.type_error("E0021", format!("capability `{}` not in context `{}`", field, iface_name), span);
  } else {
      self.type_error("E0020", format!("interface `{}` has no method `{}`", iface_name, field), span);
  }
  ```
- [ ] C-2: `Interface(name, [])` ブランチ（line ~4613）にも同様の分岐を追加
- [ ] C-3: `cargo build` でコンパイルエラーなし確認

---

## Phase D — E0021 ヘルプテキスト + error_catalog.rs

- [ ] D-1: `fav/src/driver.rs` の `get_help_text` に E0021 エントリを追加
  ```rust
  "E0021" => &[
      "switch to a context that includes this capability",
      "LoadCtx provides: db(DbRead), io, env",
      "WriteCtx provides: db(DbWrite), storage, io, env",
      "MigrateCtx provides: db_read, db_write, io, env",
  ],
  ```
- [ ] D-2: `fav/src/error_catalog.rs` に E0021 エントリを追加（既存パターンに従う）

---

## Phase E — テスト追加

- [ ] E-1: `v133000_tests::version_is_13_3_0` をコメントアウト
- [ ] E-2: `fav/src/driver.rs` 末尾に `v134000_tests` モジュールを追加
  - [ ] `version_is_13_4_0` — `CARGO_PKG_VERSION == "13.4.0"`
  - [ ] `context_interfaces_registered` — CommonCtx / LoadCtx / WriteCtx / MigrateCtx が `InterfaceRegistry` に登録済み
  - [ ] `load_ctx_has_db_field` — `lookup_declared_method("LoadCtx", "db")` が `Interface("DbRead", [])` を返す
  - [ ] `load_ctx_inherits_io` — `fn f(ctx: LoadCtx) { ctx.io.println("x") }` → no error（CommonCtx 継承）
  - [ ] `load_ctx_allows_db_read` — `fn f(ctx: LoadCtx) { ctx.db.query("SELECT 1", List.empty()) }` → no error
  - [ ] `load_ctx_rejects_db_write` — `ctx.db.execute(...)` on LoadCtx → E0020（DbRead に execute なし）
  - [ ] `load_ctx_rejects_storage` — `ctx.storage.put(...)` on LoadCtx → E0021（LoadCtx に storage なし）
  - [ ] `write_ctx_allows_db_write` — `fn f(ctx: WriteCtx) { ctx.db.execute(...) }` → no error
  - [ ] `write_ctx_allows_storage` — `fn f(ctx: WriteCtx) { ctx.storage.put(...) }` → no error
  - [ ] `migrate_ctx_has_both_db` — `ctx.db_read.query(...)` と `ctx.db_write.execute(...)` が共に通る
  - [ ] `e0021_not_e0020_for_ctx_field` — context interface のフィールド未発見は E0021、capability のメソッド未発見は E0020
- [ ] E-3: `cargo test -- --test-threads=1` 全件パス確認

---

## Phase F — バージョンバンプ + コミット

- [ ] F-1: `fav/Cargo.toml` → `version = "13.4.0"`
- [ ] F-2: `cargo test -- --test-threads=1` 全件パス確認
- [ ] F-3: self-check
  ```bash
  ./target/debug/fav check self/compiler.fav
  ./target/debug/fav check self/checker.fav
  ```
- [ ] F-4: `git add` + `git commit -m "feat: v13.4.0 — CommonCtx/LoadCtx/WriteCtx/MigrateCtx + E0021"`
- [ ] F-5: `git push origin feat/v13-capability-context`

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `InterfaceDef` に `is_context: bool` フィールドが追加される | |
| CommonCtx / LoadCtx / WriteCtx / MigrateCtx が `InterfaceRegistry` に登録される | |
| `LoadCtx` が `CommonCtx` から `io` / `env` を継承する | |
| `ctx.storage.put(...)` on LoadCtx → E0021 | |
| `ctx.db.execute(...)` on LoadCtx → E0020（DbRead に execute なし） | |
| E0021 のヘルプテキストが `get_help_text` に追加される | |
| `error_catalog.rs` に E0021 が追加される | |
| `cargo test -- --test-threads=1` 全件パス | |
| `CARGO_PKG_VERSION == "13.4.0"` | |
| `git push origin feat/v13-capability-context` | |
