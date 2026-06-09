# v13.5.0 Tasks — AppCtx 具象型 + `Ctx.build` / `Ctx.mock` Rune 実装

Date: 2026-06-10
Branch: feat/v13-capability-context

---

## Phase A — `AppCtx` を型チェッカーに登録

- [x] A-1: `fav/src/middle/checker.rs` — `register_builtin_capabilities` の末尾に AppCtx の impl エントリを追加
  - `(AppCtx, CommonCtx)` / `(AppCtx, LoadCtx)` / `(AppCtx, WriteCtx)` / `(AppCtx, MigrateCtx)`
- [x] A-2: 同箇所に MockDb / MockStorage の impl エントリを追加
  - `(MockDb, DbRead)` / `(MockDb, DbWrite)` / `(MockStorage, StorageWrite)`
- [x] A-3: `fav/src/middle/checker.rs` — 引数型チェック時に Named 型 → Interface 型の impl 互換チェックを追加
  - `Type::Fn` unify ループ内で `interface_registry.is_implemented(iface_name, ty_name)` を呼び出す
- [x] A-4: `cargo build` でコンパイルエラーなし確認

---

## Phase B — VM Primitives（`Ctx.build_raw` / `Ctx.mock_raw`）

- [x] B-1: `fav/src/backend/vm.rs` — `"Ctx.build_raw"` primitive を追加
  - 引数: `db_url: String, aws_region: String, s3_bucket: String`
  - 戻り値: `Result<String, String>`（db_url 空なら `Err("missing env: DATABASE_URL")`）
- [x] B-2: `fav/src/backend/vm.rs` — `"Ctx.mock_raw"` primitive を追加
  - 引数: `seed_rows: List<String>`
  - 戻り値: `String`（JSON 表現）
- [x] B-3: Ctx namespace を `is_known_builtin_namespace` に追加
- [x] B-4: `cargo build` でコンパイルエラーなし確認

---

## Phase C — Rune ファイル作成

- [x] C-1: `fav/runes/ctx/ctx.fav` を作成
  - `public fn build(env: Env) -> Result<String, String>` — `Ctx.build_raw` を呼ぶラッパー
  - `public fn mock(seed_rows: List<String>) -> String` — `Ctx.mock_raw` を呼ぶラッパー
- [x] C-2: `fav/runes/ctx/mock_db.fav` を作成
  - `type MockDb(List<String>)`
  - `fn MockDb.empty()` / `fn MockDb.seed(rows: List<String>)`
  - `impl DbRead for MockDb`: `query` / `query1`
  - `impl DbWrite for MockDb`: `execute` → `Result.ok(0)`
- [x] C-3: `fav/runes/ctx/mock_storage.fav` を作成
  - `type MockStorage(List<String>)`
  - `fn MockStorage.empty()`
  - `impl StorageWrite for MockStorage`: `put` / `delete` → `Result.ok(Unit)`
- [x] C-4: 各 Rune ファイルを作成済み

---

## Phase D — `fav.toml` `[context]` セクション

- [x] D-1: `fav/src/toml.rs` — `ContextConfig` 構造体を追加
  ```rust
  pub struct ContextConfig {
      pub db_url:  Option<String>,
      pub storage: Option<String>,
      pub http:    Option<String>,
  }
  ```
- [x] D-2: `FavToml` 構造体に `pub context: Option<ContextConfig>` フィールドを追加
- [x] D-3: `[context]` セクション解析を追加（`toml.rs` の match arm）
- [x] D-4: 全 `FavToml {}` initializer に `context: None` を追加（driver.rs / checker.rs / resolver.rs）

---

## Phase E — テスト追加

- [x] E-1: `v134000_tests::version_is_13_4_0` をコメントアウト
- [x] E-2: `fav/src/driver.rs` 末尾に `v135000_tests` モジュールを追加（9 tests）
  - [x] `version_is_13_5_0`
  - [x] `app_ctx_satisfies_load_ctx`
  - [x] `app_ctx_satisfies_write_ctx`
  - [x] `app_ctx_satisfies_migrate_ctx`
  - [x] `app_ctx_satisfies_common_ctx`
  - [x] `mock_db_satisfies_db_read`
  - [x] `mock_db_satisfies_db_write`
  - [x] `mock_storage_satisfies_storage_write`
  - [x] `ctx_rune_build_accepted`
- [x] E-3: `cargo test` 全件パス確認（1462 passed）

---

## Phase F — バージョンバンプ + コミット

- [x] F-1: `fav/Cargo.toml` → `version = "13.5.0"`
- [x] F-2: `cargo test` 全件パス確認（1462 passed）
- [x] F-3: self-check
  ```bash
  ./target/debug/fav check self/compiler.fav
  ./target/debug/fav check self/checker.fav
  ```
- [ ] F-4: `git add` + `git commit -m "feat: v13.5.0 — AppCtx + Ctx.build/Ctx.mock + MockDb/MockStorage"`
- [ ] F-5: `git push origin feat/v13-capability-context`

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| InterfaceRegistry.impls に `(AppCtx, LoadCtx)` 等 4 エントリが存在 | ✅ |
| `(MockDb, DbRead)` / `(MockDb, DbWrite)` / `(MockStorage, StorageWrite)` が登録済み | ✅ |
| `fn load(ctx: LoadCtx)` に AppCtx を渡してもエラーなし | ✅ |
| `Ctx.build_raw` VM primitive が存在 | ✅ |
| `runes/ctx/mock_db.fav` が DbRead + DbWrite を実装 | ✅ |
| `runes/ctx/mock_storage.fav` が StorageWrite を実装 | ✅ |
| `fav.toml` の `[context]` が ContextConfig として解析される | ✅ |
| `cargo test` 全件パス（1462 passed） | ✅ |
| `CARGO_PKG_VERSION == "13.5.0"` | ✅ |
| `git push origin feat/v13-capability-context` | |
