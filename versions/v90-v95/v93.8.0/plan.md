# Plan: v93.8.0 — サイトドキュメント更新

## Implementation Steps

### Step 1: `site/content/docs/cli/infer.mdx` を新規作成する

`site/content/docs/cli/run.mdx` の構造を参考に MDX ファイルを作成する。
必須要素:
- `sap-metadata` を含むセクション ID またはリンク（テスト `docs_infer_mentions_sap_metadata` の対象）
- `--from sap --metadata <url>` コマンド例
- `--from sap --metadata-file <path>` コマンド例

### Step 2: `site/content/docs/runes/sap-odata.mdx` を更新する

既存ファイルを読んで末尾に追加:
- EDM 型 → Favnir 型マッピング表（見出しに `metadata` を含める — これがテスト `docs_sap_odata_mentions_metadata_infer` の対象）
- `NavigationProperty` → `ExpandClause` 対応表
- 注意: 既存ファイルに `metadata` は含まれていないため、マッピング表の追加が必須

### Step 3: `driver.rs` に `mod v93800_tests` を追加する

`mod v93700_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v93800_tests {
    #[test]
    fn docs_infer_mentions_sap_metadata() {
        let src = std::fs::read_to_string(
            "../site/content/docs/cli/infer.mdx",
        )
        .expect("site/content/docs/cli/infer.mdx should be readable");
        assert!(
            src.contains("sap-metadata"),
            "infer.mdx should mention sap-metadata"
        );
    }

    #[test]
    fn docs_sap_odata_mentions_metadata_infer() {
        let src = std::fs::read_to_string(
            "../site/content/docs/runes/sap-odata.mdx",
        )
        .expect("site/content/docs/runes/sap-odata.mdx should be readable");
        assert!(
            src.contains("metadata"),
            "sap-odata.mdx should mention metadata"
        );
    }
}
```

注意: `driver.rs` のテストは `fav/` ディレクトリを作業ディレクトリとして実行されるため、
`site/` へのパスは `../site/` となる。

### Step 4: `cargo build` でコンパイル確認

```bash
cargo build
```

### Step 5: `cargo test` で全 pass 確認

```bash
cargo test 2>&1 | grep "test result"
```
→ `4136 tests, 0 failures`

### Step 6: CHANGELOG.md を更新する

### Step 7: ロードマップ本文を確認する（T6b）

`roadmap-v93.1-v94.0.md` の v93.8.0 本文は v93.7.0 T6b で既に `4134 + 2 = 4136` に修正済み。
念のため確認し、誤りがあれば修正する。

### Step 8: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
