# Tasks: v93.6.0 — `fav infer --from sap --metadata-file <path>` CLI

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,130 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v93500_tests` が存在することを確認する（v93.5.0 完了済みの証拠）
- [x] `fav/src/infer.rs` が存在し `infer_from_sap_metadata_url` が含まれることを確認する
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: `fav/src/infer.rs` に `infer_from_sap_metadata_file` を追加する

- [x] `infer_from_sap_metadata_url` の直後に `infer_from_sap_metadata_file(path: &str) -> String` 関数を追加する（v93.6.0 スタブ）
- [x] `path` をそのまま `-- Source:` 行に埋め込む出力形式を実装する

## T2: `fav/self/cli.fav` を更新する（4 か所）

- [x] **6a**: `CliCmd` 型に `| CmdInferSapMetadataFile(String, String)` バリアントを追加する（`CmdInferSapMetadata` の直後）
- [x] **6b**: `parse_infer_cmd` に `find_flag_value(rest, "--metadata-file", "")` および `find_flag_value(rest, "--output", "")` 取得を追加し、`from == "sap"` かつ `metadata_file != ""` のブランチを追加する（`--output` は `_out` に渡すが v93.6.0 スタブでは無視）
- [x] **6c**: `run_infer_sap_metadata_file(ctx: AppCtx, path: String, _out: String) -> Unit` スタブ関数を追加する（`run_infer_sap_metadata` の直後）
- [x] **6d**: `main` の `match cmd` に `CmdInferSapMetadataFile(parts) => run_infer_sap_metadata_file(ctx, parts._0, parts._1)` アームを追加する（`CmdInferSapMetadata` アームの直後）

## T3: `cargo build` でコンパイル確認

- [x] `cargo build` を実行し、コンパイルエラーがないことを確認する

## T3a: `fav fmt --check self/cli.fav` で整合性確認

- [x] `./target/debug/fav fmt --check self/cli.fav` を実行し、フォーマット違反がないことを確認する（exit 0）

## T4: `driver.rs` に `mod v93600_tests` を追加する

- [x] `mod v93500_tests { ... }` の直後に `#[cfg(test)] mod v93600_tests { ... }` を追加する
- [x] `infer_sap_metadata_file_function_defined` テストを実装する（`src/infer.rs` に `infer_from_sap_metadata_file` が含まれる）
- [x] `cli_fav_has_metadata_file_flag` テストを実装する（`self/cli.fav` に `metadata-file` が含まれる）

## T5: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,132 tests, 0 failures であることを確認する

## T5a: CHANGELOG.md を更新する

- [x] `CHANGELOG.md` に v93.6.0 のエントリを追加する

## T6b: ロードマップ本文を修正する

- [x] `versions/roadmap/roadmap-v93.1-v94.0.md` の v93.6.0 本文（行 241）の `4117 + 2 = 4119` を `4130 + 2 = 4132` に更新する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/cli.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## T7: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする
