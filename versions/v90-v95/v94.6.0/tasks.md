# Tasks: v94.6.0 — OSS 整備（SAP コミュニティ向けドキュメント）

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,152 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v94500_tests` が存在することを確認する（v94.5.0 完了済みの証拠）
- [x] `fav/src/bench.rs` が存在することを確認する（v94.5.0 完了済みの証拠）

## T1: `runes/sap-odata/README.md` を新規作成する

- [x] `runes/sap-odata/README.md` を新規作成する
- [x] タイトル・概要セクションを含める
- [x] `Setup` セクションを含める（テスト要件: `"Setup"` または `"setup"` が含まれること）
  - [x] `fav.toml` の `[sap]` 設定例
  - [x] 環境変数 `SAP_USER` / `SAP_PASS` の設定方法
  - [x] SSM SecureString パス（本番環境向け）の案内
- [x] `Usage` セクションを含める（`query<T>()` / `batch()` / `fav infer --from sap` の使い方）
- [x] `License` セクションを含める（MIT）

## T2: `CONTRIBUTING.md` に SAP セットアップ手順を追記する

- [x] 既存の `CONTRIBUTING.md` を読み、`## SAP テスト環境のセットアップ` セクションを追加する
- [x] SAP Gateway ES5 デモシステム / `fav.toml` 設定 / 環境変数設定 / 接続確認の手順を記載する

## T3: `.github/ISSUE_TEMPLATE/sap-bug.md` を新規作成する

- [x] `.github/ISSUE_TEMPLATE/` ディレクトリが存在することを確認した
- [x] `sap-bug.md` を新規作成する（YAML front matter + 環境 / 再現手順 / 期待動作 / ログ欄）

## T4: `driver.rs` に `mod v94600_tests` を追加する

- [x] `mod v94500_tests { ... }` の直後に `#[cfg(test)] mod v94600_tests { ... }` を追加する（2 テスト）
- [x] `sap_odata_rune_readme_exists`: `"../runes/sap-odata/README.md"` が存在することを確認する
- [x] `sap_odata_rune_readme_has_setup`: `README.md` に `"Setup"` が含まれることを確認する

## T5: `CHANGELOG.md` に v94.6.0 エントリを追記する

- [x] `CHANGELOG.md` の先頭に v94.6.0 エントリを追加する

## T6: `cargo build` でコンパイル確認

- [x] `cargo build` を実行し、コンパイルエラーがないことを確認する

## T7: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,154 tests, 0 failures であることを確認する

## T-last: CI 事前確認（T7 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## T8: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする
