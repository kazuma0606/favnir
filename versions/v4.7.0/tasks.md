# Favnir v4.7.0 タスクリスト — Env Rune（環境変数管理）

作成日: 2026-05-17
完了日: 2026-05-17

---

## Phase 0: バージョン更新 ✅

- [x] `fav/Cargo.toml` の version を `"4.7.0"` に変更
- [x] `fav/src/main.rs` のヘルプ文字列・バージョン表示を `4.7.0` に更新

---

## Phase 1: VM プリミティブ追加（`fav/src/backend/vm.rs`）✅

- [x] `EnvConfig` 構造体 + `Default` + `set_env_config`
- [x] `ENV_CONFIG` thread_local
- [x] `env_resolve_key(key)` ヘルパー（prefix 適用）
- [x] `parse_dotenv_content(content)` — `pub(crate)` ヘルパー（driver.rs で共有）
- [x] `Env.get_raw` — `Variant("some"/"none", ...)`
- [x] `Env.require_raw` — `ok_vm / err_vm("ENV_MISSING: key")`
- [x] `Env.get_int_raw` — `ok_vm(Int) / err_vm("ENV_PARSE_INT: ...")`
- [x] `Env.get_bool_raw` — `true/false/1/0/yes/no/on/off` 対応
- [x] `Env.load_dotenv_raw` — ファイルパース + `unsafe { set_var }` （上書きなし）
- [x] `Env.all_raw` — `std::env::vars()` → `VMValue::Record`

---

## Phase 2: `fav.toml` 拡張（`fav/src/toml.rs`）✅

- [x] `EnvConfig` 構造体（`dotenv: Option<String>`, `prefix: String` + `Default`）
- [x] `FavToml` に `pub env: Option<EnvConfig>` 追加
- [x] `[env]` セクションパース（内部変数名 `env_cfg`、型注釈 `let mut current: EnvConfig =`）
- [x] FavToml literal に `env: None` 追加（checker.rs ×2、resolver.rs ×2、driver.rs ×1）

---

## Phase 3: checker.rs 変更 ✅

- [x] `"Env"` を `BUILTIN_EFFECTS` に追加（E0252 回避）
- [x] `"DuckDb"` も `BUILTIN_EFFECTS` に追加（漏れていた）
- [x] `require_env_effect` 関数（E0312）
- [x] `("Env", "get_raw")` → `require_env_effect` + `Option<String>`
- [x] `("Env", "require_raw")` → `require_env_effect` + `Result<String, String>`
- [x] `("Env", "get_int_raw")` → `require_env_effect` + `Result<Int, String>`
- [x] `("Env", "get_bool_raw")` → `require_env_effect` + `Result<Bool, String>`
- [x] `("Env", "load_dotenv_raw")` → `require_env_effect` + `Result<Unit, String>`
- [x] `("Env", "all_raw")` → `require_env_effect` + `Map<String, String>`
- [x] `check_test_def` の `current_effects` に `Effect::Unknown("Env")` 追加

---

## Phase 4: compiler.rs 変更 ✅

- [x] 既に `"Env"` が両方の namespace リストに存在 → 変更なし

---

## Phase 5: driver.rs 変更 ✅

- [x] import に `EnvConfig`, `parse_dotenv_content`, `set_env_config` 追加
- [x] `cmd_run` で dotenv 自動ロード（`unsafe { set_var }` + 上書きなし）
- [x] `cmd_run` で `set_env_config` 呼び出し

---

## Phase 6: rune ファイル作成（`runes/env/`）✅

- [x] `access.fav` — `get`, `get_opt`, `require`
- [x] `typed.fav` — `get_int`, `require_int`, `get_bool`, `require_bool`
- [x] `dotenv.fav` — `load_dotenv`, `load_dotenv_or_ignore`
- [x] `env.fav`（barrel）— `use access.*`, `use typed.*`, `use dotenv.*`
- [x] `env.test.fav` — 14 件のテスト（`let` 不使用・1ブロック1式）
- [x] `test_fixtures/test.env` — dotenv テスト用フィクスチャ

---

## Phase 7: テスト追加 ✅

### vm_stdlib_tests.rs（8 件）
- [x] `env_get_raw_returns_some`
- [x] `env_get_raw_returns_none`
- [x] `env_require_raw_ok`
- [x] `env_require_raw_err`
- [x] `env_get_int_raw_ok`
- [x] `env_get_int_raw_parse_err`
- [x] `env_get_bool_raw_true`
- [x] `env_load_dotenv_raw_ok`

### driver.rs 統合テスト（5 件）
- [x] `env_get_in_favnir_source`
- [x] `env_require_missing_in_favnir_source`
- [x] `env_get_int_in_favnir_source`
- [x] `env_get_bool_in_favnir_source`
- [x] `env_rune_test_file_passes`

---

## Phase 8: examples 追加 ✅

- [x] `examples/env_demo/fav.toml`
- [x] `examples/env_demo/.env`
- [x] `examples/env_demo/src/main.fav`

---

## 完了条件 ✅

- [x] `cargo build` が通る
- [x] 既存 861 件が全て pass
- [x] 新規テスト 27 件が pass（Rust 8 件 + Favnir 14 件 + 統合 5 件）
- [x] 874 件全て pass（2026-05-17 確認）

---

## 実装メモ（次バージョンへの引き継ぎ）

- **`"Env"` は `BUILTIN_EFFECTS` に追加必要** — `Effect::Unknown("Env")` を使う際は `BUILTIN_EFFECTS` 配列への追加を忘れずに（`"DuckDb"` も同様）
- **`env` 変数名衝突**: toml.rs で `env_cfg` を使用（`std::env` との衝突回避）
- **`std::env::set_var` は unsafe** (Rust 2024) — `unsafe { set_var(...) }` でラップ必要
- **テストブロックの 1 式制限**: `load_dotenv_or_ignore()` + `assert(true)` を 1 ブロックに書けない。`load_dotenv_or_ignore()` のみ or 別のテストに分ける
- **`parse_dotenv_content` は `pub(crate)`** — driver.rs と vm.rs で共有
- **`Env.get` / `Env.get_or`（旧 v3.3.0 primitives）は残す** — 後方互換のため削除しない
