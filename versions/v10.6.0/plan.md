# Favnir v10.6.0 Plan

Date: 2026-06-04
Theme: Snowflake Rune 実装（runes/snowflake/）

---

## Phase A: Rune ファイル作成

### A-1: `runes/snowflake/rune.toml`

```toml
[rune]
name = "snowflake"
version = "10.6.0"
entry = "snowflake.fav"
description = "Snowflake rune — SQL API v2 経由のクエリ・DML 実行（!Snowflake エフェクト）"
```

### A-2: `runes/snowflake/snowflake.fav`

```favnir
// runes/snowflake/snowflake.fav — Favnir Snowflake rune public API (v10.6.0)
// Snowflake SQL API v2 経由でクエリ・DML を実行する。
//
// 使用例:
//   import rune "snowflake"
//
//   type Order = { order_id: Int  customer: String  amount: Float }
//
//   fn get_orders(sql: String) -> Result<List<Order>, String> !Snowflake {
//       snowflake.query<Order>(sql)
//   }
//
// 環境変数:
//   SNOWFLAKE_ACCOUNT / SNOWFLAKE_USER / SNOWFLAKE_PRIVATE_KEY / SNOWFLAKE_PUBLIC_KEY_FP
//   オプション: SNOWFLAKE_WAREHOUSE / SNOWFLAKE_ROLE / SNOWFLAKE_DATABASE / SNOWFLAKE_SCHEMA

use client.{ execute, query }
```

### A-3: `runes/snowflake/client.fav`

```favnir
// runes/snowflake/client.fav — Snowflake クライアント (v10.6.0)
// Snowflake SQL API v2 への接続は VM primitive が担う。
//
// 環境変数:
//   SNOWFLAKE_ACCOUNT       — Snowflake アカウント識別子（必須）
//   SNOWFLAKE_USER          — ユーザー名（必須）
//   SNOWFLAKE_PRIVATE_KEY   — RSA 秘密鍵 PEM（必須）
//   SNOWFLAKE_PUBLIC_KEY_FP — 公開鍵フィンガープリント（必須）
//   SNOWFLAKE_WAREHOUSE     — ウェアハウス名（省略可）
//   SNOWFLAKE_ROLE          — ロール名（省略可）
//   SNOWFLAKE_DATABASE      — データベース名（省略可）
//   SNOWFLAKE_SCHEMA        — スキーマ名（省略可）

// execute — DML（INSERT / UPDATE / DELETE / CREATE 等）を実行する
public fn execute(sql: String) -> Result<String, String> !Snowflake {
    Snowflake.execute_raw(sql)
}

// query<T> — SELECT クエリを実行し、行を型 T の List に変換して返す
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

### A-4: `runes/snowflake/snowflake.test.fav`

```favnir
// runes/snowflake/snowflake.test.fav — Snowflake rune テスト (v10.6.0)
// 資格情報が設定されていない環境での動作確認
import "snowflake"

test "snowflake_execute_no_creds_is_err" {
    bind result <- snowflake.execute("SELECT 1")
    assert(Result.is_err(result))
}

test "snowflake_query_no_creds_is_err" {
    bind result <- snowflake.query<String>("SELECT 1")
    assert(Result.is_err(result))
}
```

---

## Phase B: Rust テスト追加（driver.rs 末尾）

```rust
// ── v10600_tests (v10.6.0) — Snowflake Rune ──────────────────────────────────
#[cfg(test)]
mod v10600_tests {
    use super::tests::run_fav_test_file_with_runes;

    #[test]
    fn snowflake_rune_test_file_passes() {
        // 資格情報を確実に unset してテスト（no_creds_is_err アサーションが通る）
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

## Phase C: バージョン更新

### C-1: `fav/Cargo.toml` version → `"10.6.0"`

### C-2: `fav/self/cli.fav` の `run_version` → `"10.6.0"`

---

## Phase D: self-check + cargo test

```bash
# D-1: compiler.fav self-check
fav check --legacy-check self/compiler.fav

# D-2: 新規テスト確認
cargo test v10600

# D-3: bootstrap 維持確認
cargo test bootstrap

# D-4: 全件確認（目標: 1272 件 = 1271 + 1 新規）
cargo test
```
