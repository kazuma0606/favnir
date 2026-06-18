# Favnir v10.7.0 Tasks

Date: 2026-06-04
Theme: fav.toml Snowflake 設定対応

---

## Phase A: toml.rs 更新

- [x] A-1: `SnowflakeTomlConfig` 構造体追加（`AwsTomlConfig` の直後）
- [x] A-2: `FavToml` に `pub snowflake: Option<SnowflakeTomlConfig>` フィールド追加
- [x] A-3: `parse_fav_toml` に `[snowflake]` セクション解析追加
  - [x] A-3a: 変数宣言 `let mut snowflake_cfg`
  - [x] A-3b: `if trimmed == "[snowflake]"` セクション検出
  - [x] A-3c: `"snowflake" =>` マッチアーム（account / user / warehouse / role / database / schema）
  - [x] A-3d: `FavToml { ... }` に `snowflake: snowflake_cfg` 追加
- [x] A-4: `expand_env_vars(s: &str) -> String` 公開関数追加（`${VAR}` 展開）

---

## Phase B: driver.rs 更新

- [x] B-1: `inject_snowflake_config(cfg: &SnowflakeTomlConfig)` 関数追加
  - env var 未設定時のみ `set_var`（上書きなし）
  - `expand_env_vars` で値を展開
- [x] B-2: `run_with_favnir_pipeline_project` 内で `inject_snowflake_config` を呼ぶ
- [x] B-3: `load_run_config`（legacy path 含む全 run 経路）で `inject_snowflake_config` を呼ぶ
- [x] B-4: `default_fav_toml` にコメントアウト `[snowflake]` 例を追加

---

## Phase C: テスト追加

- [x] C-1: `toml.rs` tests — `toml_snowflake_section_parsed`
- [x] C-2: `toml.rs` tests — `toml_snowflake_env_var_expanded`
- [x] C-3: `driver.rs` `v10700_tests` モジュール追加（2 件）
  - [x] C-3a: `toml_snowflake_inject_sets_env_vars`
  - [x] C-3b: `toml_snowflake_inject_does_not_overwrite_existing_env`
- [x] C-4: `cargo test toml_snowflake` + `cargo test v10700` — 4 件通過

---

## Phase D: バージョン更新

- [x] D-1: `fav/Cargo.toml` version → `"10.7.0"`
- [x] D-2: `fav/self/cli.fav` の `run_version` → `"10.7.0"`

---

## Phase E: self-check + cargo test

- [x] E-1: `fav check --legacy-check self/compiler.fav` — エラーなし
- [x] E-2: `cargo test bootstrap` — 通過
- [x] E-3: `cargo test` — 全件通過（1276 件）

---

## Phase F: 完了処理

- [x] F-1: 本ファイル完了チェック
- [x] F-2: `memory/MEMORY.md` に v10.7.0 完了を記録
- [x] F-3: commit

---

## 完了条件

| 条件 | 状態 |
|---|---|
| `[snowflake]` セクションが `fav.toml` から解析できる | ✓ |
| `${ENV_VAR}` 形式の環境変数参照が展開される | ✓ |
| toml 設定が env var として注入される（上書きなし） | ✓ |
| `fav new` テンプレートに `[snowflake]` 例が含まれる | ✓ |
| `cargo test bootstrap` 通過 | ✓ |
| `cargo test` 全件通過 | ✓ |
