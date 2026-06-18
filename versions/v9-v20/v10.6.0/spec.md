# Favnir v10.6.0 Spec

Date: 2026-06-04
Theme: Snowflake Rune 実装（runes/snowflake/）

---

## 概要

`import rune "snowflake"` で使える Snowflake Rune を実装する。
LLM Rune（v9.6.0）と同じ 4 ファイル構成（rune.toml / snowflake.fav / client.fav / snowflake.test.fav）。

VM Primitive（v10.2.0 で実装済み）:
- `Snowflake.execute_raw(sql: String) -> Result<String, String>` — DML 実行
- `Snowflake.query_raw(sql: String) -> Result<String, String>` — クエリ（JSON 配列文字列を返す）

どちらも以下の環境変数から接続情報を読む：
- `SNOWFLAKE_ACCOUNT` / `SNOWFLAKE_USER` / `SNOWFLAKE_PRIVATE_KEY` / `SNOWFLAKE_PUBLIC_KEY_FP`
- オプション: `SNOWFLAKE_WAREHOUSE` / `SNOWFLAKE_ROLE` / `SNOWFLAKE_DATABASE` / `SNOWFLAKE_SCHEMA`

---

## ファイル構成

```
runes/snowflake/
  rune.toml           — rune メタデータ
  snowflake.fav       — public API エントリ（use client.{...} で再エクスポート）
  client.fav          — 実装（execute / query<T>）
  snowflake.test.fav  — テスト（資格情報なし → Err）
```

---

## API 設計

### `execute(sql: String) -> Result<String, String> !Snowflake`

DML（INSERT / UPDATE / DELETE / CREATE 等）を実行する。
成功時は `"ok"` を返す（行数は含まない）。

```favnir
public fn execute(sql: String) -> Result<String, String> !Snowflake {
    Snowflake.execute_raw(sql)
}
```

### `query<T>(sql: String) -> Result<List<T>, String> !Snowflake`

SELECT クエリを実行し、行を型 T の List に変換して返す。
内部では `Snowflake.query_raw` → `Json.parse_raw` → `Schema.adapt` のパイプラインを使う。

```favnir
public fn query<T>(sql: String) -> Result<List<T>, String> !Snowflake {
    match Snowflake.query_raw(sql) {
        Err(e) => Result.err(e)
        Ok(raw) =>
            match Json.parse_raw(raw) {
                Err(e) => Result.err(String.concat("snowflake.query: ", e))
                Ok(parsed) =>
                    match Schema.adapt(parsed, type_name_of<T>()) {
                        Err(_) => Result.err("snowflake.query: schema error")
                        Ok(rows) => Result.ok(rows)
                    }
            }
    }
}
```

---

## テスト設計

資格情報（SNOWFLAKE_ACCOUNT 等）が設定されていない環境でテストする。
`execute` / `query` はいずれも `Result.is_err` を返すこと。

### Rust テスト（driver.rs `v10600_tests`）

```rust
mod v10600_tests {
    use super::tests::run_fav_test_file_with_runes;

    #[test]
    fn snowflake_rune_test_file_passes() {
        unsafe {
            std::env::remove_var("SNOWFLAKE_ACCOUNT");
            std::env::remove_var("SNOWFLAKE_PRIVATE_KEY");
        }
        let results = run_fav_test_file_with_runes("runes/snowflake/snowflake.test.fav");
        let failures: Vec<_> = results.iter().filter(|(_, ok, _)| !ok).collect();
        assert!(failures.is_empty(), "snowflake.test.fav failures: {:?}", failures);
    }
}
```

---

## バージョン更新

- `fav/Cargo.toml`: `version = "10.6.0"`
- `fav/self/cli.fav`: `run_version` → `"10.6.0"`
