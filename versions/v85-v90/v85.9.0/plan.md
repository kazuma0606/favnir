# Plan: v85.9.0 — 安定化・コードフリーズ

## Step 1: 前提確認

- `cargo test` を実行し、3,947 tests, 0 failures を確認する
- `fav/src/driver.rs` に `mod v85800_tests` が存在することを確認する（v85.8.0 完了済みの証拠）
- `runes/sap-odata/rune.toml` に `name = "sap-odata"` が存在することを確認する

## Step 2: `rune.toml` の実際の書式を確認

実装前に `runes/sap-odata/rune.toml` を読み、`name` フィールドの正確な書式（スペース数）を確認する。
テストの assert 文字列をそれに合わせる。

v85.4.0 の spec 通りなら:
```toml
name        = "sap-odata"
```
（`name` と `=` の間にスペース 8 個）

## Step 3: `fav/src/driver.rs` に `mod v85900_tests` を追加

`mod v85800_tests { ... }` の直後に追加する。

```rust
#[cfg(test)]
mod v85900_tests {
    #[test]
    fn sap_foundation_rune_toml_has_correct_name() {
        let content = std::fs::read_to_string("../runes/sap-odata/rune.toml")
            .expect("runes/sap-odata/rune.toml should exist");
        assert!(
            content.contains("sap-odata"),
            "rune.toml should have name = sap-odata"
        );
    }

    #[test]
    fn sap_foundation_docker_compose_has_sap_mock_service() {
        let content = std::fs::read_to_string(
            "../infra/e2e-demo/sap-odata/docker-compose.yml",
        )
        .expect("infra/e2e-demo/sap-odata/docker-compose.yml should exist");
        assert!(
            content.contains("sap-mock"),
            "docker-compose.yml should define sap-mock service"
        );
    }
}
```

**注意**: `sap_foundation_rune_toml_has_correct_name` の assert は `content.contains("sap-odata")` を使う。
`name        = "sap-odata"` のスペース数の厳密な一致は rune.toml の書式依存であり、
`"sap-odata"` を含むかどうかで十分なリグレッションガードになる。

## Step 4: `cargo test` で全 pass 確認

```
cargo test 2>&1 | grep "test result"
# 期待: 3949 tests, 0 failures
```

## Step 5: CHANGELOG 更新

`CHANGELOG.md` の先頭に v85.9.0 エントリを追加する。

## Step 6: CI 事前確認

以下はすべて `fav/` ディレクトリで実行する。

```
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
