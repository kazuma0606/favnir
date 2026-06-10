# v13.6.0 Tasks — ctx.field.method() 構文実装 + E2E デモ書き換え

Date: 2026-06-10
Branch: feat/v13-capability-context

---

## 実装内容

### 構文サポート（ast_lower_checker.rs）
- [x] `lower_apply` に `FieldAccess(FieldAccess(_, cap_name), method)` → `ECall("AppCtx.{cap_name}", method, args)` ケースを追加
  - `ctx.db.execute(...)` → checker が `builtin_ret_ty("AppCtx.db", "execute")` → `"Unknown"` → E0007 なし
  - `ctx.io.println(...)` / `ctx.storage.put(...)` も同様

### checker.rs 修正
- [x] `BUILTIN_EFFECTS` に `"Gen"` を追加（`!Gen` 宣言 → E0252 なし）

### E2E デモ書き換え（型チェックのみ）
- [x] `infra/e2e-demo/fav2py/src/pipeline.fav`
  - `AppCtx.db_execute(ctx, ...)` → `ctx.db.execute(...)`
  - `AppCtx.db_query(ctx, ...)` → `ctx.db.query(...)`
  - `AppCtx.storage_put(ctx, ...)` → `ctx.storage.put(...)`
- [x] `infra/e2e-demo/airgap/src/analyze.fav`
  - `AppCtx.io_println(ctx, ...)` → `ctx.io.println(...)`
  - `AppCtx.storage_put(ctx, ...)` → `ctx.storage.put(...)`
  - `main() -> Result<Unit, String> !IO` + `chain ctx <-` に変更（E0224 修正）

### テスト（driver.rs v136000_tests）
- [x] `version_is_13_6_0`
- [x] `app_ctx_db_execute_no_e0007`
- [x] `app_ctx_storage_put_no_e0007`
- [x] `e2e_fav2py_ctx_based_compiles`
- [x] `e2e_airgap_ctx_based_compiles`
- [x] `w009_count_fav2py_zero`
- [x] `w009_count_airgap_zero`

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `ctx.field.method()` 呼び出しで E0007 が出ない | ✅ |
| `fav2py/pipeline.fav` が型チェックをパス | ✅ |
| `airgap/analyze.fav` が型チェックをパス | ✅ |
| 両デモに W009 警告がゼロ | ✅ |
| `cargo test v136000` 全 7 件パス | ✅ |
| `CARGO_PKG_VERSION == "13.6.0"` | ✅ |
