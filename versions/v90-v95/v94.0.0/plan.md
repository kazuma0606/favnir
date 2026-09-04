# Plan: v94.0.0 — SAP Metadata Infer 1.0 宣言 ★クリーンアップ

## Implementation Steps

### Step 1: `cargo clean` を実施する

```bash
cargo clean
```

- `fav/tmp/hello.fav` の存在を確認し、消えていた場合は復元する:
  ```
  fn add(a: Int, b: Int) -> Int { a + b }
  fn main() -> Bool { add(1, 2) == 3 }
  ```

### Step 2: `fav/Cargo.toml` バージョンを更新する

`version = "93.0.0"` → `version = "94.0.0"` に変更する。

### Step 3: `CHANGELOG.md` に v94.0.0 エントリを追加する

**※ v94000_tests 追加より先に実施すること**（`changelog_has_v94_0_0` テストが先に通る必要があるため）。

### Step 4: `MILESTONE.md` に v94.0.0 エントリを追加する

先頭に v94.0.0 — SAP Metadata Infer 1.0 の宣言ブロックを追加する。

### Step 5: `README.md` に v94.0 宣言セクションを追加する

既存の v93.0 セクションの前に v94.0 セクションを追加する。

### Step 6: `versions/current.md` を v94.0.0 に更新する

「最新安定版」欄を v94.0.0 — SAP Metadata Infer 1.0 宣言、4,142 tests に更新する。

### Step 7: `driver.rs` の全 `cargo_toml_version_is_X_0_0` テストを一括更新する

```bash
sed -i 's/93\.0\.0/94.0.0/g' src/driver.rs
```

これにより全ての `cargo_toml_version_is_X_0_0` テストの assert 内文字列が `"93.0.0"` → `"94.0.0"` に更新される。
Cargo.toml の `version = "94.0.0"` と一致するようになる。

### Step 8: `driver.rs` に `mod v94000_tests` を追加する

`mod v93900_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v94000_tests {
    #[test]
    fn cargo_toml_version_is_94_0_0() {
        let content = std::fs::read_to_string("Cargo.toml")
            .expect("Cargo.toml should exist");
        assert!(content.contains("version = \"94.0.0\""),
            "Cargo.toml should have version 94.0.0");
    }

    #[test]
    fn changelog_has_v94_0_0() {
        let content = std::fs::read_to_string("../CHANGELOG.md")
            .expect("CHANGELOG.md should exist");
        assert!(content.contains("v94.0.0"), "CHANGELOG.md should mention v94.0.0");
    }

    #[test]
    fn milestone_has_sap_metadata_infer() {
        let content = std::fs::read_to_string("../MILESTONE.md")
            .expect("MILESTONE.md should exist");
        assert!(
            content.contains("SAP Metadata Infer"),
            "MILESTONE.md should mention SAP Metadata Infer"
        );
    }

    #[test]
    fn readme_mentions_metadata_infer() {
        let content = std::fs::read_to_string("../README.md")
            .expect("README.md should exist");
        assert!(
            content.contains("Metadata Infer"),
            "README.md should mention Metadata Infer"
        );
    }
}
```

### Step 9: `cargo build` でコンパイル確認

```bash
cargo build
```

### Step 10: `cargo test` で全 pass 確認

```bash
cargo test 2>&1 | grep "test result"
```
→ `4142 tests, 0 failures`

### Step 11: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
