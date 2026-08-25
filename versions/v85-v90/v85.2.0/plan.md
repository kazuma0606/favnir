# Plan: v85.2.0 — `SapConfig` Favnir 型 + `sap_config_from_env()`

## Step 1: 前提確認

- `cargo test` を実行し、3,933 tests, 0 failures を確認する
- `fav/src/driver.rs` に `mod v85100_tests` が存在することを確認する（v85.1.0 完了済みの証拠）
- `runes/` ディレクトリが存在することを確認する（他の Rune の隣に配置する）

## Step 2: `runes/sap-odata/` ディレクトリを作成

```bash
mkdir -p runes/sap-odata
```

（v85.4.0 で `rune.toml` / `sap_odata.fav` を追加するため、まず骨格ディレクトリのみ作成）

## Step 3: `runes/sap-odata/types.fav` を作成

以下の内容でファイルを作成する。

```favnir
-- SAP OData v4 接続設定型（v85.2.0）

type SapConfig = {
    base_url: String,
    client:   String,
    username: String,
    password: String,
    auth:     String
}

-- SAP 接続設定を環境変数から読み取る。
-- fav.toml [sap] の設定は inject_sap_config() によって
-- SAP_BASE_URL / SAP_USER / SAP_PASS / SAP_CLIENT / SAP_AUTH として注入済み。
-- Env.require は Result<String, String> を返す → bind を使う。
-- Env.get_or はデフォルト値付きで String を直接返す → bind 不要。
public fn sap_config_from_env() -> Result<SapConfig, String> {
    bind base_url <- Env.require("SAP_BASE_URL")
    bind username <- Env.require("SAP_USER")
    bind password <- Env.require("SAP_PASS")
    Result.ok(SapConfig {
        base_url,
        username,
        password,
        client: Env.get_or("SAP_CLIENT", "100"),
        auth:   Env.get_or("SAP_AUTH", "basic")
    })
}
```

## Step 4: `fav/src/driver.rs` に `mod v85200_tests` を追加

`mod v85100_tests { ... }` の直後に追加する。

```rust
#[cfg(test)]
mod v85200_tests {
    #[test]
    fn sap_config_from_env_returns_ok_when_vars_set() {
        let content = std::fs::read_to_string("../../runes/sap-odata/types.fav")
            .expect("runes/sap-odata/types.fav should exist");
        assert!(
            content.contains("sap_config_from_env"),
            "types.fav should define sap_config_from_env"
        );
        assert!(
            content.contains("SapConfig"),
            "types.fav should define SapConfig type"
        );
    }

    #[test]
    fn sap_config_from_env_returns_err_when_base_url_missing() {
        let content = std::fs::read_to_string("../../runes/sap-odata/types.fav")
            .expect("runes/sap-odata/types.fav should exist");
        assert!(
            content.contains("Env.require(\"SAP_BASE_URL\")"),
            "types.fav should require SAP_BASE_URL"
        );
    }
}
```

## Step 5: `cargo test` で全 pass 確認

```
cargo test 2>&1 | grep "test result"
# 期待: 3935 tests, 0 failures
```

## Step 6: CHANGELOG 更新

`CHANGELOG.md` の先頭に v85.2.0 エントリを追加する。

## Step 7: CI 事前確認

以下はすべて `fav/` ディレクトリで実行する。

```
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
