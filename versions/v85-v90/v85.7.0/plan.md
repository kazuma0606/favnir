# Plan: v85.7.0 — `fav new` テンプレート + `fav.toml [sap]` セクション追加

## Step 1: 前提確認

- `cargo test` を実行し、3,943 tests, 0 failures を確認する
- `fav/src/driver.rs` に `mod v85600_tests` が存在することを確認する（v85.6.0 完了済みの証拠）
- `fav/src/driver.rs` の `default_fav_toml()` に `# [snowflake]` が存在することを確認する

## Step 2: `default_fav_toml()` に `[sap]` コメントブロックを追加

`fav/src/driver.rs` の `default_fav_toml()` 関数を以下のように変更する。
`# [snowflake]` ブロックの末尾 `\n"` を `\n\n` + `[sap]` ブロック + `\n"` に変更する。

変更前（抜粋）:
```rust
         # schema    = \"PUBLIC\"\n"
    )
}
```

変更後（抜粋）:
```rust
         # schema    = \"PUBLIC\"\n\n\
         # [sap]\n\
         # base_url = \"${{SAP_BASE_URL}}\"   # SAP S/4HANA エンドポイント\n\
         # client   = \"100\"                  # SAP クライアント番号\n\
         # username = \"${{SAP_USER}}\"\n\
         # password = \"${{SAP_PASS}}\"\n\
         # auth     = \"basic\"                # \"basic\" | \"oauth2\"\n"
    )
}
```

## Step 3: `fav/src/driver.rs` に `mod v85700_tests` を追加

`mod v85600_tests { ... }` の直後に追加する。

```rust
#[cfg(test)]
mod v85700_tests {
    use crate::toml::parse_fav_toml_pub;

    #[test]
    fn fav_new_template_contains_sap_comment() {
        let template = super::default_fav_toml("test");
        assert!(
            template.contains("# [sap]"),
            "fav.toml template should contain [sap] comment section"
        );
    }

    #[test]
    fn sap_toml_section_parses_correctly() {
        let toml_str = "[sap]\nbase_url = \"https://example.com\"\n";
        let parsed = parse_fav_toml_pub(toml_str);
        let cfg = parsed.sap.expect("sap config should be parsed");
        assert_eq!(cfg.base_url.as_deref(), Some("https://example.com"));
    }
}
```

## Step 4: `cargo test` で全 pass 確認

```
cargo test 2>&1 | grep "test result"
# 期待: 3945 tests, 0 failures
```

## Step 5: CHANGELOG 更新

`CHANGELOG.md` の先頭に v85.7.0 エントリを追加する。

## Step 6: CI 事前確認

以下はすべて `fav/` ディレクトリで実行する。

```
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
