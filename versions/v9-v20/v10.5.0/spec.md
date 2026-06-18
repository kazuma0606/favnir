# Favnir v10.5.0 Spec

Date: 2026-06-04
Theme: Snowflake × Favnir pipeline — E2E コンパイル確認

---

## 概要

ロードマップでは「compiler.fav の builtin NS リストに `"Snowflake"` を追加」と
記述されていたが、実装調査の結果：

- **`compiler.fav`** には NS ホワイトリストが存在しない。
  NS 修飾呼び出し（`Snowflake.execute_raw`）は `CVName("Snowflake.execute_raw")` として
  そのまま emit され、VM 実行時に `call_builtin` で解決される。
- **`compiler.rs`（Rust）** の NS リスト 2 箇所には v10.2.0 で `"Snowflake"` 追加済み。
- **`vm.rs`** の `call_builtin` には v10.2.0 で `"Snowflake.execute_raw"` / `"Snowflake.query_raw"` 追加済み。

したがって v10.5.0 のコード変更はなし。
代わりに **Favnir pipeline（compiler.fav 経由）を通じた E2E コンパイルテスト** を追加し、
Snowflake 呼び出しを含む Favnir ソースがコンパイルエラーなく通ることを証明する。

---

## 前提（v10.4.0 完了時点）

- Rust checker（`checker.rs`）: `Effect::Snowflake` / `require_snowflake_effect` / E0314 追加済み
- checker.fav: `snowflake_fn` / `builtin_ret_ty` / `ns_to_effect` に Snowflake 追加済み
- `vm.rs`: `Snowflake.execute_raw` / `Snowflake.query_raw` primitive 追加済み
- `compiler.rs`: NS リスト 2 箇所に `"Snowflake"` 追加済み
- `cargo test` 1269 件通過

---

## テスト設計

### テスト 1: `snowflake_compiles_with_favnir_pipeline`

`compile_src_str_to_bytes`（Favnir pipeline）を使い、
`Snowflake.execute_raw` を呼ぶ関数を含むソースがコンパイルできることを確認する。

```rust
let src = r#"
fn run(sql: String) -> Result<String, String> !Snowflake {
  Snowflake.execute_raw(sql)
}
"#;
let result = crate::compiler_fav_runner::compile_src_str_to_bytes(src);
assert!(result.is_ok(), "Snowflake compile via Favnir pipeline failed: {:?}", result);
```

### テスト 2: `snowflake_query_compiles_with_favnir_pipeline`

`Snowflake.query_raw` についても同様に確認。

```rust
let src = r#"
fn query(sql: String) -> Result<String, String> !Snowflake {
  Snowflake.query_raw(sql)
}
"#;
let result = crate::compiler_fav_runner::compile_src_str_to_bytes(src);
assert!(result.is_ok(), "Snowflake query compile via Favnir pipeline failed: {:?}", result);
```

---

## バージョン更新

- `fav/Cargo.toml`: `version = "10.5.0"`
- `fav/self/cli.fav`: `run_version` → `"10.5.0"`
