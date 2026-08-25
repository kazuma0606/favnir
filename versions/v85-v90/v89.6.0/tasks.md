# Tasks: v89.6.0 — `site/content/docs/runes/sap-odata.mdx` ドキュメント

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,029 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v89500_tests` が存在することを確認する（v89.5.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `89.0.0` であることを確認する
- [x] `site/content/docs/runes/snowflake.mdx` が存在することを確認する（参照パターン）

## T1: `site/content/docs/runes/sap-odata.mdx` を作成

- [x] フロントマター（title / order / category / description）を記述する
- [x] `import rune "sap-odata"` 構文と概要を追加する
- [x] `fav.toml [sap]` セクションの設定例を追加する
- [x] 環境変数一覧（SAP_BASE_URL / SAP_USERNAME / SAP_PASSWORD / SAP_CLIENT）を追加する
- [x] `BusinessPartner` サンプルコードを追加する
- [x] `SalesOrder` サンプルコードを追加する
- [x] `Material` サンプルコードを追加する
- [x] `JournalEntry` サンプルコードを追加する
- [x] `## 業務シナリオ` セクション（4 シナリオのテーブル）を追加する
- [x] `## Rune Registry` セクション（`import rune "sap-odata"` + Registry URL）を追加する
- [x] `## Docker Compose モックサーバー` セクション（SAP 接続環境変数 + `AWS_ENDPOINT_URL` の設定例）を追加する

## T2: `mod v89600_tests` を `driver.rs` に追加

- [x] `mod v89500_tests { ... }` の直後に `#[cfg(test)] mod v89600_tests { ... }` を追加する
- [x] `docs_sap_odata_mdx_exists` テストを実装する（`"../site/content/docs/runes/sap-odata.mdx"` の存在確認）
- [x] `docs_sap_odata_contains_business_partner_section` テストを実装する（`"BusinessPartner"` を含むことを確認）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,031 tests, 0 failures であることを確認する

> 上記テスト全 pass 後、CI 事前確認（T-last）に進む。

## Note

> 実装完了後は本ファイルの Status を `COMPLETE` に変更し、T0〜T-last の全チェックボックスを `[x]` にすること。

CHANGELOG / MILESTONE 更新は v90.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
