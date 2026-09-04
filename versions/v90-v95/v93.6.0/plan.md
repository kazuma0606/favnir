# Plan: v93.6.0 — `fav infer --from sap --metadata-file <path>` CLI

## Implementation Steps

### Step 1: `fav/src/infer.rs` に `infer_from_sap_metadata_file` を追加

`infer_from_sap_metadata_url` の直後に関数を追加する。
引数 `path: &str` をそのまま出力に埋め込む（スタブ）。
URL 版と同様のヘッダーコメント形式を使用する。

### Step 2: `fav/self/cli.fav` を更新する（4 か所）

依存関係の順に実施する:

2a. `CliCmd` 型に `| CmdInferSapMetadataFile(String, String)` バリアントを追加する。
    `CmdInferSapMetadata(String, String)` の直後に挿入する。

2b. `parse_infer_cmd` に `--metadata-file` フラグ取得と `from == "sap"` かつ
    `metadata_file_flag != ""` のブランチを追加する。
    `from == "sap"` かつ `metadata_flag != ""` のブランチ（CmdInferSapMetadata）より先に評価する。

2c. `run_infer_sap_metadata_file(ctx: AppCtx, path: String, _out: String) -> Unit` スタブ関数を追加する。
    `run_infer_sap_metadata` の直後に配置する。

2d. `main` の `match cmd` に `CmdInferSapMetadataFile(parts) =>` アームを追加する。
    `CmdInferSapMetadata` アームの直後に挿入する。

### Step 3: `cargo build` でコンパイル確認

```bash
cargo build
```

### Step 3a: `fav fmt` で整形確認

```bash
./target/debug/fav fmt --check self/cli.fav
```
違反があれば `fav fmt self/cli.fav` で自動整形し、再確認する。

### Step 4: `driver.rs` に `mod v93600_tests` を追加

`mod v93500_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v93600_tests {
    #[test]
    fn infer_sap_metadata_file_function_defined() {
        let src = std::fs::read_to_string("src/infer.rs").unwrap();
        assert!(src.contains("infer_from_sap_metadata_file"));
    }

    #[test]
    fn cli_fav_has_metadata_file_flag() {
        let src = std::fs::read_to_string("self/cli.fav").unwrap();
        assert!(src.contains("metadata-file"));
    }
}
```

### Step 5: `cargo test` で全 pass 確認

```bash
cargo test 2>&1 | grep "test result"
```
→ `4132 tests, 0 failures`

### Step 6: CHANGELOG.md を更新する

### Step 7: ロードマップ本文のテスト数を修正する（T6b）

`roadmap-v93.1-v94.0.md` の v93.6.0 本文（行 241）の `4117 + 2 = 4119` を `4130 + 2 = 4132` に修正する。

### Step 8: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
