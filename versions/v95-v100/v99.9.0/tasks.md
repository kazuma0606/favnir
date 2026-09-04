# Tasks: v99.9.0 — コードフリーズ・最終確認

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/v95-v100/v99.9.0/` ディレクトリが存在することを確認する（存在しなければ `mkdir versions/v95-v100/v99.9.0/` で作成する）
- [x] `versions/v95-v100/v99.8.0/tasks.md` の Status が `COMPLETE` であることを確認する
- [x] `versions/current.md` の最新安定版が `v99.8.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v99800_tests` が存在することを確認する（v99.8.0 完了済みの証拠）
- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,273 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `99.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 99.0.0 のまま）
- [x] `fav/tmp/hello.fav` が存在することを確認する
- [x] `fav/tmp/hello.fav` の内容に `fn add` と `fn main` が含まれることを確認する

## T1: 全 SAP ガイドドキュメントのリンク切れ確認（手動）

- [x] `site/content/docs/guides/sap-platform.mdx` が存在し、`sap-migration` と `sap-enterprise-checklist` への参照が含まれることを確認する
- [x] `site/content/docs/guides/sap-migration.mdx` が存在し、`sap-platform` と `sap-enterprise-checklist` への参照が含まれることを確認する
- [x] `site/content/docs/guides/sap-enterprise-checklist.mdx` が存在し、`sap-platform` と `sap-migration` への参照が含まれることを確認する

## T2: versions/current.md の「次に切る版」欄を v100.0.0 に更新

- [x] `versions/current.md` の「次に切る版」セクションに `v100.0.0` と「Favnir SAP Platform 1.0 宣言」の記述を追加する
- [x] `versions/current.md` に `v100` キーワードが含まれることを確認する（T5 テストの前提）

## T3: driver.rs に mod v99900_tests を追加

- [x] `mod v99800_tests` の直後に `mod v99900_tests`（2 テスト）を追加する:
  - `sap_guide_docs_all_exist`: 3 ガイド MDX ファイルの存在確認（3 `expect()`）
  - `current_md_next_version_is_v100`: `versions/current.md` に `"v100"` が含まれることを確認（1 アサート）
- [x] `mod v99900_tests` ブロック先頭に `// use super::* は不要（std::fs のみ使用）` という Rust コメントを 1 行追記する
- [x] テストが `std::fs::read_to_string` を使用していることを確認する

## T4: cargo test で全 pass 確認

- [x] `cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、4,275 tests, 0 failures であることを確認する

## T5: CHANGELOG.md に v99.9.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v99.9.0]` エントリを追加する

## T6: versions/current.md 最新安定版の更新

- [x] `最終更新:` ヘッダーを `v99.9.0` に更新する
- [x] 最新安定版を `v99.9.0` に更新する（テスト数 4,275）

<!-- MILESTONE.md 更新は宣言版（v100.0.0）で対応予定（patch version は対象外） -->

## code-reviewer 指摘対応（実装後）

| 優先度 | 指摘 | 対応 |
|--------|------|------|
| [LOW] | `current.md` の `cargo install --version "99.0.0"` が `"99.9.0"` に更新されていない | 対応不要 — Cargo.toml version はパッチ版で更新しない設計（宣言版 99.0.0 のまま） |

## T-last: CI 事前確認（T4 の `cargo test` 全 pass 確認後・T5/T6 完了後に実施すること。`cargo test` 再実行は不要。Clippy / fmt のみ確認する）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
