# Favnir v10.8.0 Tasks

Date: 2026-06-04
Theme: fav infer --from snowflake（スキーマ自動生成）

---

## Phase A: vm.rs — ヘルパーを pub(crate) に昇格

- [x] A-1: `snowflake_read_env` → `pub(crate) fn`
- [x] A-2: `snowflake_generate_jwt` → `pub(crate) fn`
- [x] A-3: `snowflake_api_post` → `pub(crate) fn`

---

## Phase B: driver.rs — 型マッピング + cmd_infer_snowflake

- [x] B-1: `snowflake_col_type_to_favnir(col_type: &str, nullable: bool) -> InferredType` 追加
- [x] B-2: `snowflake_infer_table(table: &str) -> Result<String, String>` 追加
- [x] B-3: `cmd_infer_snowflake(table: &str, out_path: Option<&str>)` 追加

---

## Phase C: main.rs — `--from snowflake` フラグ追加

- [x] C-1: `fav infer` の引数パースに `--from` / `--table` フラグ追加
- [x] C-2: `from_source == "snowflake"` の場合に `cmd_infer_snowflake` を呼ぶ dispatch 追加

---

## Phase D: vm.rs — `Snowflake.infer_table_raw` primitive 追加

- [x] D-1: `call_builtin` に `"Snowflake.infer_table_raw"` ブランチ追加

---

## Phase E: compiler.rs / checker.rs / checker.fav 更新

- [x] E-1: `checker.rs` に `("Snowflake", "infer_table_raw")` 型シグネチャ追加
- [x] E-2: `checker.fav` の `snowflake_fn` に `"infer_table_raw"` ブランチ追加

---

## Phase F: cli.fav 更新

- [x] F-1: `CliCmd` に `CmdInferSnowflake(String, String)` 追加
- [x] F-2: `parse_infer_cmd` 関数追加（`--from snowflake --table <name> [--out <path>]`）
- [x] F-3: `run_infer_snowflake(table, out)` 関数追加（`Snowflake.infer_table_raw` 呼び出し）
- [x] F-4: `parse_named_cmd` に `"infer" => parse_infer_cmd(args)` 追加
- [x] F-5: `main` の match に `CmdInferSnowflake(parts) => run_infer_snowflake(...)` 追加
- [x] F-6: `run_help` に `infer` コマンド説明追加

---

## Phase G: テスト追加

- [x] G-1: `driver.rs` `v10800_tests` モジュール追加（6 件）
  - [x] G-1a: `snowflake_number_maps_to_int`
  - [x] G-1b: `snowflake_float_maps_to_float`
  - [x] G-1c: `snowflake_varchar_maps_to_string`
  - [x] G-1d: `snowflake_boolean_maps_to_bool`
  - [x] G-1e: `snowflake_nullable_wraps_option`
  - [x] G-1f: `snowflake_timestamp_maps_to_string`
- [x] G-2: `cargo test v10800` — 6 件通過

---

## Phase H: バージョン更新

- [x] H-1: `fav/Cargo.toml` version → `"10.8.0"`
- [x] H-2: `fav/self/cli.fav` の `run_version` → `"10.8.0"`

---

## Phase I: self-check + cargo test

- [x] I-1: `fav check --legacy-check self/compiler.fav` — エラーなし
- [x] I-2: `cargo test bootstrap` — 通過
- [x] I-3: `cargo test` — 全件通過（1282 件）

---

## Phase J: 完了処理

- [x] J-1: 本ファイル完了チェック
- [x] J-2: `memory/MEMORY.md` に v10.8.0 完了を記録
- [x] J-3: commit

---

## 完了条件

| 条件 | 状態 |
|---|---|
| `fav infer --from snowflake --table <name>` が型定義を出力する | ✓ |
| Snowflake 型 → Favnir 型のマッピングテスト 6 件通過 | ✓ |
| cli.fav で `fav infer --from snowflake` が使える | ✓ |
| `cargo test bootstrap` 通過 | ✓ |
| `cargo test` 全件通過 | ✓ |
