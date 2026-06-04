# Favnir v10.8.0 Spec

Date: 2026-06-04
Theme: fav infer --from snowflake（スキーマ自動生成）

---

## 概要

`fav infer --from snowflake --table <name>` で Snowflake テーブル定義から
Favnir 型を自動生成する。
既存の `fav infer --db <sqlite>` と同じ出力形式（`format_type_def`）を再利用する。

---

## アーキテクチャ方針

`fav infer` は現在 Rust main.rs 路線のみ（cli.fav には未実装）。
v10.8.0 では以下を追加する:

1. **`vm.rs`**: Snowflake ヘルパー関数を `pub(crate)` に昇格
2. **`driver.rs`**: 型マッピング関数 + `cmd_infer_snowflake` 追加
3. **`main.rs`**: `--from snowflake` フラグを `fav infer` に追加
4. **`vm.rs`**: `Snowflake.infer_table_raw(table) -> Result<String, String>` primitive 追加
5. **`compiler.rs` / `checker.rs` / `checker.fav`**: NS/型シグネチャ更新
6. **`cli.fav`**: `CmdInferSnowflake` + `parse_infer_cmd` + `run_infer_snowflake`

---

## Snowflake → Favnir 型マッピング

`snowflake_col_type_to_favnir(col_type: &str, nullable: bool) -> InferredType`

| Snowflake 型 | Favnir 型 |
|---|---|
| NUMBER / DECIMAL / NUMERIC / INT / INTEGER / BIGINT / SMALLINT / TINYINT / BYTEINT | Int |
| FLOAT / FLOAT4 / FLOAT8 / DOUBLE / REAL | Float |
| BOOLEAN | Bool |
| VARCHAR / STRING / TEXT / CHAR / CHARACTER / NCHAR / NVARCHAR / NVARCHAR2 | String |
| DATE / TIME / TIMESTAMP / TIMESTAMP_LTZ / TIMESTAMP_NTZ / TIMESTAMP_TZ | String |
| VARIANT / OBJECT / ARRAY / その他 | String |
| `IS_NULLABLE = 'YES'` | `Option<上記>` |

---

## `cmd_infer_snowflake` の動作

1. 環境変数（SNOWFLAKE_ACCOUNT 等）から接続情報を読む
2. `INFORMATION_SCHEMA.COLUMNS` クエリを `snowflake_api_post`（pub(crate)化）で実行
3. レスポンスの各行から (COLUMN_NAME, DATA_TYPE, IS_NULLABLE) を取得
4. `snowflake_col_type_to_favnir` でマッピング
5. `format_type_def`（既存）でフォーマットして標準出力 / ファイル出力

### クエリ

```sql
SELECT COLUMN_NAME, DATA_TYPE, IS_NULLABLE
FROM INFORMATION_SCHEMA.COLUMNS
WHERE TABLE_NAME = 'ORDERS'
ORDER BY ORDINAL_POSITION
```

---

## `Snowflake.infer_table_raw` VM primitive（cli.fav 向け）

```
"Snowflake.infer_table_raw" => args: [table: String] -> Result<String, String>
```

- INFORMATION_SCHEMA.COLUMNS を呼び出し、Favnir 型定義文字列を返す
- cli.fav から `run_infer_snowflake` で呼び出す

---

## cli.fav 追加内容

```favnir
| CmdInferSnowflake(String, String)   // (table, out_path)
```

```favnir
fn parse_infer_cmd(args: List<String>) -> CliCmd {
    bind rest  <- List.drop(args, 1)
    bind from  <- find_flag_value(rest, "--from", "")
    bind table <- find_flag_value(rest, "--table", "")
    bind out   <- find_flag_value(rest, "--out", "")
    if from == "snowflake" {
        if table == "" { CmdUnknown("infer --from snowflake requires --table <name>") }
        else { CmdInferSnowflake(table, out) }
    } else {
        CmdUnknown("infer: use --from snowflake --table <name>")
    }
}
```

```favnir
fn run_infer_snowflake(table: String, out: String) -> Unit !Snowflake !IO {
    match Snowflake.infer_table_raw(table) {
        Err(e) => {
            bind _ <- IO.write_stderr_raw(String.concat("error: ", e))
            IO.exit_raw(1)
        }
        Ok(typedef) =>
            if out == "" { IO.println(typedef) }
            else {
                match IO.write_file_raw(out, typedef) {
                    Err(e) => {
                        bind _ <- IO.write_stderr_raw(String.concat("error: cannot write: ", e))
                        IO.exit_raw(1)
                    }
                    Ok(_) => IO.println(String.concat("written: ", out))
                }
            }
    }
}
```

---

## テスト設計

### 型マッピング単体テスト（driver.rs tests — 実接続不要）

```rust
mod v10800_tests {
    use super::snowflake_col_type_to_favnir;
    use super::{InferredType, format_inferred_type};

    #[test] fn snowflake_number_maps_to_int() { ... }
    #[test] fn snowflake_float_maps_to_float() { ... }
    #[test] fn snowflake_varchar_maps_to_string() { ... }
    #[test] fn snowflake_boolean_maps_to_bool() { ... }
    #[test] fn snowflake_nullable_wraps_option() { ... }
    #[test] fn snowflake_timestamp_maps_to_string() { ... }
}
```

新規テスト数: **6 件**

---

## バージョン更新

- `fav/Cargo.toml`: `version = "10.8.0"`
- `fav/self/cli.fav`: `run_version` → `"10.8.0"`
