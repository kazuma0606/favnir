# Tasks: v99.8.0 — 総合ドキュメント

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/v95-v100/v99.8.0/` ディレクトリが存在することを確認する（存在しなければ `mkdir versions/v95-v100/v99.8.0/` で作成する）
- [x] `versions/v95-v100/v99.7.0/tasks.md` の Status が `COMPLETE` であることを確認する
- [x] `versions/current.md` の最新安定版が `v99.7.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v99700_tests` が存在することを確認する（v99.7.0 完了済みの証拠）
- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,271 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `99.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 99.0.0 のまま）
- [x] `fav/tmp/hello.fav` が存在することを確認する
- [x] `fav/tmp/hello.fav` の内容に `fn add` と `fn main` が含まれることを確認する

## T1: site/content/docs/guides/ ディレクトリ確認

- [x] `site/content/docs/guides/` ディレクトリが存在することを確認する（存在しなければ作成する）

## T2: sap-platform.mdx を新規作成

- [x] `site/content/docs/guides/sap-platform.mdx` を新規作成する
- [x] ファイルに `SAP Platform` というキーワードが含まれることを確認する
- [x] frontmatter（title・description）が含まれることを確認する
- [x] コードサンプルが `bind` 構文を使用していることを確認する（`let` は使用禁止）

## T3: sap-migration.mdx を新規作成

- [x] `site/content/docs/guides/sap-migration.mdx` を新規作成する
- [x] ファイルに `migration` または `移行` というキーワードが含まれることを確認する
- [x] frontmatter（title・description）が含まれることを確認する

## T4: sap-enterprise-checklist.mdx を新規作成

- [x] `site/content/docs/guides/sap-enterprise-checklist.mdx` を新規作成する
- [x] ファイルに `checklist` または `チェック` というキーワードが含まれることを確認する
- [x] frontmatter（title・description）が含まれることを確認する

## T5: driver.rs に mod v99800_tests を追加

- [x] `mod v99700_tests` の直後に `mod v99800_tests`（2 テスト）を追加する:
  - `sap_platform_mdx_exists`: `../site/content/docs/guides/sap-platform.mdx` の存在を確認
  - `sap_platform_all_docs_have_keywords`: 3 ファイルのキーワード存在確認（3 アサート）
- [x] `mod v99800_tests` ブロック先頭に `// use super::* は不要（std::fs のみ使用）` という Rust コメントを 1 行追記する
- [x] テストが `std::fs::read_to_string` を使用していることを確認する

## T6: cargo test で全 pass 確認

- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、4,273 tests, 0 failures であることを確認する

## T7: CHANGELOG.md に v99.8.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v99.8.0]` エントリを追加する

## T8: versions/current.md 更新

- [x] `最終更新:` ヘッダーを `v99.8.0` に更新する
- [x] 最新安定版を `v99.8.0` に更新する（テスト数 4,273）

<!-- MILESTONE.md 更新は宣言版（v100.0.0）で対応予定（patch version は対象外） -->

## T-last: CI 事前確認（T6 の `cargo test` 全 pass 確認後・T7/T8 完了後に実施すること。`cargo test` 再実行は不要。Clippy / fmt のみ確認する）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
