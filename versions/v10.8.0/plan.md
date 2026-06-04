# Favnir v10.8.0 Plan

Date: 2026-06-04
Theme: fav infer --from snowflake（スキーマ自動生成）

---

## Phase A: vm.rs — ヘルパーを pub(crate) に昇格

`snowflake_read_env` / `snowflake_generate_jwt` / `snowflake_api_post` を
`pub(crate) fn` に変更する（driver.rs から呼べるようにする）。

```rust
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn snowflake_read_env(key: &str) -> Result<String, String> { ... }

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn snowflake_generate_jwt(...) -> Result<String, String> { ... }

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn snowflake_api_post(...) -> Result<serde_json::Value, String> { ... }
```

---

## Phase B: driver.rs — 型マッピング + cmd_infer_snowflake

### B-1: `snowflake_col_type_to_favnir` 追加

```rust
fn snowflake_col_type_to_favnir(col_type: &str, nullable: bool) -> InferredType {
    let base = match col_type.to_uppercase().as_str() {
        "NUMBER" | "DECIMAL" | "NUMERIC"
        | "INT" | "INTEGER" | "BIGINT" | "SMALLINT" | "TINYINT" | "BYTEINT" => InferredType::Int,
        "FLOAT" | "FLOAT4" | "FLOAT8" | "DOUBLE" | "REAL" => InferredType::Float,
        "BOOLEAN" => InferredType::Bool,
        _ => InferredType::FavString,
    };
    if nullable {
        InferredType::Option(Box::new(base))
    } else {
        base
    }
}
```

### B-2: `snowflake_infer_table` 追加

```rust
#[cfg(not(target_arch = "wasm32"))]
fn snowflake_infer_table(table: &str) -> Result<String, String> {
    use crate::backend::vm::{snowflake_read_env, snowflake_generate_jwt, snowflake_api_post};
    let account   = snowflake_read_env("SNOWFLAKE_ACCOUNT")?;
    let user      = snowflake_read_env("SNOWFLAKE_USER")?;
    let privkey   = snowflake_read_env("SNOWFLAKE_PRIVATE_KEY")?;
    let pubkey_fp = snowflake_read_env("SNOWFLAKE_PUBLIC_KEY_FP")?;
    let jwt = snowflake_generate_jwt(&account, &user, &privkey, &pubkey_fp)?;
    let sql = format!(
        "SELECT COLUMN_NAME, DATA_TYPE, IS_NULLABLE \
         FROM INFORMATION_SCHEMA.COLUMNS \
         WHERE TABLE_NAME = '{}' \
         ORDER BY ORDINAL_POSITION",
        table.to_uppercase()
    );
    let body = serde_json::json!({ "statement": sql, "timeout": 60 });
    let resp = snowflake_api_post(&account, &jwt, &body)?;
    let cols: Vec<String> = resp["resultSetMetaData"]["rowType"]
        .as_array().unwrap_or(&vec![])
        .iter().map(|c| c["name"].as_str().unwrap_or("").to_string()).collect();
    let rows_data: Vec<Vec<String>> = resp["data"]
        .as_array().unwrap_or(&vec![])
        .iter()
        .map(|row| {
            row.as_array().unwrap_or(&vec![])
                .iter().map(|v| v.as_str().unwrap_or("").to_string()).collect()
        })
        .collect();
    let col_idx = |name: &str| cols.iter().position(|c| c == name).unwrap_or(usize::MAX);
    let name_idx     = col_idx("COLUMN_NAME");
    let type_idx     = col_idx("DATA_TYPE");
    let nullable_idx = col_idx("IS_NULLABLE");
    let type_name = table_name_to_type_name(table);
    let source = format!("--from snowflake --table {}", table.to_uppercase());
    let fields: Vec<InferredField> = rows_data.iter().map(|row| {
        let col_name = row.get(name_idx).cloned().unwrap_or_default().to_lowercase();
        let col_type = row.get(type_idx).cloned().unwrap_or_default();
        let nullable_str = row.get(nullable_idx).cloned().unwrap_or_default();
        let nullable = nullable_str.to_uppercase() == "YES" || nullable_str.to_uppercase() == "Y";
        InferredField { name: col_name, ty: snowflake_col_type_to_favnir(&col_type, nullable) }
    }).collect();
    let def = InferredTypeDef { name: type_name, fields, source };
    Ok(format_type_def(&def))
}
```

### B-3: `cmd_infer_snowflake` 追加

```rust
pub fn cmd_infer_snowflake(table: &str, out_path: Option<&str>) {
    let output = snowflake_infer_table(table).unwrap_or_else(|e| {
        eprintln!("error: {}", e);
        process::exit(1);
    });
    write_infer_output(&output, out_path);
}
```

---

## Phase C: main.rs — `--from snowflake` フラグ追加

`fav infer` の引数パース部分に `--from` / `--table` フラグを追加する。

```rust
"--from" => {
    from_source = Some(args.get(i + 1).cloned().unwrap_or_default());
    i += 2;
}
"--table" => {
    table_name = Some(args.get(i + 1).cloned().unwrap_or_default());
    i += 2;
}
```

dispatch:
```rust
if from_source.as_deref() == Some("snowflake") {
    let table = table_name.as_deref().unwrap_or_else(|| {
        eprintln!("error: --from snowflake requires --table <name>");
        process::exit(1);
    });
    cmd_infer_snowflake(table, out_path.as_deref());
    return;
}
```

---

## Phase D: vm.rs — `Snowflake.infer_table_raw` primitive 追加

cli.fav から呼べる VM primitive:

```rust
"Snowflake.infer_table_raw" => {
    let table = vm_string(args.into_iter().next()
        .ok_or_else(|| "Snowflake.infer_table_raw requires a table argument")?, ...)?;
    match snowflake_infer_table_impl(&table) {
        Ok(s)  => Ok(ok_vm(VMValue::Str(s))),
        Err(e) => Ok(err_vm(VMValue::Str(e))),
    }
}
```

`snowflake_infer_table_impl` は Phase B の `snowflake_infer_table` ロジックを
`pub(crate)` で共有する（driver.rs からも呼ぶ）。

---

## Phase E: compiler.rs / checker.rs / checker.fav 更新

### compiler.rs
`"Snowflake"` は既に追加済み（追加不要）。

### checker.rs
`("Snowflake", "infer_table_raw")` の型シグネチャを追加:
```rust
("Snowflake", "infer_table_raw") => // String -> Result<String, String>
```

### checker.fav
`snowflake_fn` に `"infer_table_raw"` branch 追加:
```favnir
fn snowflake_fn(fname: String) -> String {
    if fname == "execute_raw"    { "Result" }
    else { if fname == "query_raw"      { "Result" }
    else { if fname == "infer_table_raw" { "Result" }
    else { "Result" } } }
}
```

---

## Phase F: cli.fav 更新

### F-1: `CliCmd` に `CmdInferSnowflake(String, String)` 追加

### F-2: `parse_infer_cmd` 追加

```favnir
fn parse_infer_cmd(args: List<String>) -> CliCmd {
    bind rest  <- List.drop(args, 1)
    bind from  <- find_flag_value(rest, "--from", "")
    bind table <- find_flag_value(rest, "--table", "")
    bind out   <- find_flag_value(rest, "--out", "")
    if from == "snowflake" {
        if table == "" {
            CmdUnknown("infer --from snowflake requires --table <name>")
        } else {
            CmdInferSnowflake(table, out)
        }
    } else {
        CmdUnknown("infer: use --from snowflake --table <name>")
    }
}
```

### F-3: `run_infer_snowflake` 追加

```favnir
fn run_infer_snowflake(table: String, out: String) -> Unit !Snowflake !IO {
    match Snowflake.infer_table_raw(table) {
        Err(e) => {
            bind _ <- IO.write_stderr_raw(String.concat("error: ", e))
            IO.exit_raw(1)
        }
        Ok(typedef) =>
            if out == "" {
                IO.println(typedef)
            } else {
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

### F-4: `parse_named_cmd` に `"infer"` 追加

```favnir
else { if cmd == "infer" { parse_infer_cmd(args) }
```

### F-5: `main` の match に `CmdInferSnowflake` 追加

```favnir
CmdInferSnowflake(parts) => run_infer_snowflake(parts._0, parts._1)
```

### F-6: `run_help` に `infer` 行を追加

```favnir
IO.println("  infer --from snowflake --table <name> [--out <file>]  Infer type from Snowflake table")
```

---

## Phase G: バージョン更新 + テスト

```bash
cargo test v10800        # 6 件通過
cargo test bootstrap     # 通過
cargo test               # 全件確認（目標: 1282 件 = 1276 + 6）
```
