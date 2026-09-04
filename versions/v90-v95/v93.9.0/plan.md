# Plan: v93.9.0 — 安定化・コードフリーズ

## Implementation Steps

### Step 1: ベースライン確認

```bash
cargo test 2>&1 | grep "test result"
```
→ `4136 tests, 0 failures` を確認する。

### Step 2: `driver.rs` に `mod v93900_tests` を追加する

`mod v93800_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v93900_tests {
    #[test]
    fn sap_metadata_smoke_url_and_file_cli() {
        let src = std::fs::read_to_string("self/cli.fav")
            .expect("self/cli.fav should be readable");
        assert!(
            src.contains("from sap"),
            "cli.fav should handle --from sap"
        );
        assert!(
            src.contains("metadata-file"),
            "cli.fav should handle --metadata-file"
        );
    }

    #[test]
    fn sap_metadata_parser_handles_entity_and_enum() {
        let src = std::fs::read_to_string("src/sap_metadata.rs")
            .expect("src/sap_metadata.rs should be readable");
        assert!(
            src.contains("entity_type_to_favnir"),
            "sap_metadata.rs should define entity_type_to_favnir"
        );
        assert!(
            src.contains("enum_type_to_favnir"),
            "sap_metadata.rs should define enum_type_to_favnir"
        );
    }
}
```

### Step 3: `cargo build` でコンパイル確認

```bash
cargo build
```

### Step 4: `cargo test` で全 pass 確認

```bash
cargo test 2>&1 | grep "test result"
```
→ `4138 tests, 0 failures`

### Step 5: CHANGELOG.md を更新する

### Step 6: ロードマップ本文を確認する（T6b）

`roadmap-v93.1-v94.0.md` の v93.9.0 本文が `4136 + 2 = 4138` になっていることを確認する
（v93.7.0 T6b で修正済み）。

### Step 7: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
