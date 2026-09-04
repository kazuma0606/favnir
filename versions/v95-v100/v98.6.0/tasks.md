# Tasks: v98.6.0 — `fav report --sap`（ローカル HTML レポート生成コマンド）

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v98.5.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v98500_tests` が存在することを確認する（v98.5.0 完了済みの証拠）
- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,245 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `98.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 98.0.0 のまま）

## T1: driver.rs に cmd_report_sap を実装

- [x] `fav/src/driver.rs` に `pub fn cmd_report_sap(entity: &str, from: &str, to: &str, output: &str) -> i32` を追加する
- [x] `println!("Fetching {} from SAP... 1,234 records", entity)` を出力する
- [x] `println!("Generating report...")` を出力する
- [x] HTML 文字列を `format!` で生成し `std::fs::write(output, html)` で書き出す
  - HTML に `"SAP Report: {entity}"` が含まれていること
- [x] 書き込み成功時は `println!("Saved: {}", output)` を出力し `0` を返す
- [x] 書き込み失敗時は `eprintln!` してから `1` を返す

## T2: main.rs に Some("report") ケースを追加

- [x] `Some("sap-mock") => { ... }` ブロックの後、`Some("ai") => { ... }` の直前に `// ── v98.6.0: fav report` ブロックを挿入する
- [x] `--sap` フラグが未指定の場合はエラーメッセージを出力して `process::exit(1)` する
- [x] `--entity` / `--from` / `--to` / `--output` フラグを `args.iter().position(...)` パターンで解析する
- [x] デフォルト値: `entity="SalesOrder"`, `from=""`, `to=""`, `output="report.html"`
- [x] `driver::cmd_report_sap(entity, from, to, output)` を呼び出し、戻り値を `process::exit(code)` に渡す

## T3: driver.rs に mod v98600_tests を追加

- [x] `mod v98500_tests` の直後に `mod v98600_tests`（2 テスト）を追加する:
  - `cmd_report_sap_exists`: 関数ポインタのキャストでコンパイル存在確認
  - `cmd_report_sap_generates_html`: 一時ファイルへの書き出し + HTML 内容検証
- [x] `mod v98600_tests` ブロック先頭に `// use super::* は不要（driver 関数を直接呼ぶ）` という Rust コメントを 1 行追記する
- [x] テスト後に一時ファイルを `std::fs::remove_file` で削除する

## T4: cargo test で全 pass 確認

- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、4,247 tests, 0 failures であることを確認する

## T5: CHANGELOG.md に v98.6.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v98.6.0]` エントリを追加する

## T6: versions/current.md 更新

- [x] `最終更新:` ヘッダーを `v98.6.0` に更新する
- [x] 最新安定版を `v98.6.0` に更新する（テスト数 4,247）

<!-- MILESTONE.md 更新は宣言版（v99.0.0）で対応予定（patch version は対象外） -->
<!-- site MDX ドキュメントは v98.8.0 で対応予定（本バージョンはスコープ外） -->

## T-last: CI 事前確認（T4 の `cargo test` 全 pass 確認後・T5/T6 完了後に実施すること。`cargo test` 再実行は不要。Clippy / fmt のみ確認する）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
