# Tasks: v86.8.0 — Rune Registry デプロイ（sap-odata）

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,967 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v86700_tests` が存在することを確認する（v86.7.0 完了済みの証拠）
- [x] `runes/sap-odata/rune.toml` の `version` が `86.8.0` 未満（まだ更新前）であることを確認する

## T1: `CHANGELOG.md` に v86.8.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に v86.8.0 エントリを追加する（テストモジュール追加より先に実施）

## T2: `runes/sap-odata/rune.toml` を更新

- [x] `version` を `86.8.0` に更新する

## T3: `mod v86800_tests` を追加

- [x] `mod v86700_tests { ... }` の直後に `#[cfg(test)] mod v86800_tests { ... }` を追加する
- [x] `sap_odata_rune_version_matches_cargo` テストを実装する
- [x] `sap_odata_rune_entry_file_is_sap_odata_fav` テストを実装する

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,969 tests, 0 failures であることを確認する

## T5: Rune Registry デプロイ（手動）

- [ ] `/deploy-registry` スキルを実行して `sap-odata` Rune を Registry にデプロイする（終了コード 0 を確認）
- [ ] DynamoDB (`favnir-rune-registry`) に `name = "sap-odata"` のエントリが存在することを確認する
- [ ] S3 (`favnir-rune-packages`) に `sap-odata/` 配下の `.fav` ファイルが存在することを確認する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## 修正事項（code-reviewer 指摘対応）

- [MED] `sap_odata_rune_version_matches_cargo` のアサーションを `content.contains("86.")` から `content.contains("version     = \"86.8.0\"")` に強化（曖昧な部分文字列マッチを排除）
- [LOW] `runes/sap-odata/rune.toml` に `effects = ["!Http"]` を追加（他の rune との一貫性）
