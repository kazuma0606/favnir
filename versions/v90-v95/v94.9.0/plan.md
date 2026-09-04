# Plan: v94.9.0 — 安定化・コードフリーズ

## 依存関係

テスト追加のみのバージョン。変更ファイルは `driver.rs` と `CHANGELOG.md` の 2 件のみ。

---

## Step 1: `cargo test` ベースライン確認

```bash
cargo test 2>&1 | grep "test result"
```

4,158 tests, 0 failures であることを確認する。

---

## Step 2: `driver.rs` に `mod v94900_tests` を追加する

`mod v94800_tests { ... }` の直後に追加する。

```rust
#[cfg(test)]
mod v94900_tests {
    // use super::* は不要（std::fs / std::path のみ使用）
    #[test]
    fn sap_advanced_smoke_all_features() {
        assert!(
            std::path::Path::new("../runes/sap-odata/batch.fav").exists(),
            "runes/sap-odata/batch.fav should exist ($batch)"
        );
        assert!(
            std::path::Path::new("../runes/sap-odata/query_builder.fav").exists(),
            "runes/sap-odata/query_builder.fav should exist (QueryBuilder<T>)"
        );
        assert!(
            std::path::Path::new("../fav/src/sap_metadata.rs").exists()
                || std::path::Path::new("src/sap_metadata.rs").exists(),
            "sap_metadata.rs should exist (Metadata Infer)"
        );
        assert!(
            std::path::Path::new("../infra/lambda/sap-sync/main.tf").exists(),
            "infra/lambda/sap-sync/main.tf should exist (SnapStart Lambda)"
        );
    }

    #[test]
    fn sap_advanced_era_doc_complete() {
        assert!(
            std::path::Path::new("../site/content/docs/guides/sap-integration.mdx").exists(),
            "site/content/docs/guides/sap-integration.mdx should exist"
        );
    }
}
```

> **注意**: `sap_metadata.rs` のパスは driver.rs が `fav/` 直下から実行されるため
> `"src/sap_metadata.rs"` が正しい。`"../fav/src/sap_metadata.rs"` は不正なパス。
> `Path::new("src/sap_metadata.rs").exists()` を使う。

---

## Step 3: `CHANGELOG.md` に v94.9.0 エントリを追記する

先頭に追加する:

```markdown
## [v94.9.0] — 2026-08-30 — 安定化・コードフリーズ

### Added
- `fav/src/driver.rs` — `mod v94900_tests`（テスト 2 件）を追加
  - `sap_advanced_smoke_all_features`: SAP Advanced Era 全成果物の存在確認
    （batch.fav / query_builder.fav / sap_metadata.rs / infra/lambda/sap-sync/main.tf）
  - `sap_advanced_era_doc_complete`: `site/content/docs/guides/sap-integration.mdx` が存在する
- 合計テスト数: **4,160**（+2）
```

---

## Step 4: `cargo test` で 4,160 tests 確認

```bash
cargo test 2>&1 | grep "test result"
```

---

## Step 5: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
