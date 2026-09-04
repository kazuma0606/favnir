# Tasks: v93.5.0 — `fav infer --from sap --metadata <url>` CLI

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,128 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v93400_tests` が存在することを確認する（v93.4.0 完了済みの証拠）
- [x] `fav/src/sap_metadata.rs` が存在し `enum_type_to_favnir` が含まれることを確認する
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: `fav/src/infer.rs` を新規作成する

- [x] `infer_from_sap_metadata_url(url: &str) -> String` 関数を実装する（v93.5.0 スタブ）
- [x] URL からサービス名を抽出するロジックを実装する（`rsplit('/')` + `trim_end_matches("/$metadata")`）
- [x] ヘッダーコメント形式の出力文字列を生成するロジックを実装する

## T2: `fav/src/main.rs` に `mod infer;` を追加する

- [x] `mod sap_metadata;` の直後に `mod infer;` を追加する

## T3: `fav/self/cli.fav` を更新する（4 か所）

- [x] **3a**: `CliCmd` 型に `| CmdInferSapMetadata(String, String)` バリアントを追加する（`| CmdInferSnowflake` の直後）
- [x] **3b**: `parse_infer_cmd` に `find_flag_value(rest, "--metadata", "")` 取得を追加し、`from == "sap"` ブランチを追加する
- [x] **3c**: `run_infer_sap_metadata(ctx: AppCtx, url: String, _out: String) -> Unit` スタブ関数を追加する（`run_infer_snowflake` の直後）
- [x] **3d**: `main` の `match cmd` に `CmdInferSapMetadata(parts) => run_infer_sap_metadata(ctx, parts._0, parts._1)` アームを追加する（`CmdInferSnowflake` アームの直後）

## T4: `cargo build` でコンパイル確認

- [x] `cargo build` を実行し、コンパイルエラーがないことを確認する

## T4a: `fav fmt --check self/cli.fav` で整合性確認

- [x] `./target/debug/fav fmt --check self/cli.fav` を実行し、フォーマット違反がないことを確認する（`fav fmt` で自動整形済み）

## T5: `driver.rs` に `mod v93500_tests` を追加する

- [x] ファイル末尾の `mod v93400_tests { ... }` の直後に `#[cfg(test)] mod v93500_tests { ... }` を追加する
- [x] `infer_sap_metadata_url_function_defined` テストを実装する（`src/infer.rs` に `infer_from_sap_metadata_url` が含まれる）
- [x] `cli_fav_has_from_sap_flag` テストを実装する（`self/cli.fav` に `from sap` が含まれる）

## T6: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,130 tests, 0 failures であることを確認する

## T6a: CHANGELOG.md を更新する

- [x] `CHANGELOG.md` に v93.5.0 のエントリを追加する

## T6b: ロードマップ本文を修正する

- [x] `versions/roadmap/roadmap-v93.1-v94.0.md` の v93.5.0 本文（行 216）の `4115 + 2 = 4117` を `4128 + 2 = 4130` に更新する
- [x] 同ファイル行 195 の `-- Generated at: 2026-08-25` を v93.5.0 スタブ出力に合わせて修正する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## T7: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする
