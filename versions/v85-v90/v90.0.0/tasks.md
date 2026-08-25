# Tasks: v90.0.0 — SAP Integration 1.0 宣言

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,037 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v89900_tests` が存在することを確認する（v89.9.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `89.0.0` であることを確認する
- [x] `fav/tmp/hello.fav` が存在することを確認する（cargo clean 後に消えないことを確認）

## T1: `cargo clean`

- [x] `cargo clean` を実行してビルドキャッシュを削除する（22,636 files, 12.1GiB 削除）
- [x] `fav/tmp/hello.fav` がまだ存在することを確認する（cargo clean では消えない）

## T2: `Cargo.toml` バージョン更新

- [x] `fav/Cargo.toml` の `version = "89.0.0"` を `version = "90.0.0"` に変更する

## T3: `driver.rs` の `"89.0.0"` 文字列を一括更新

- [x] `grep -c "89\.0\.0" src/driver.rs` で置換対象件数を確認する（実測: 42 件）
- [x] `sed -i 's/89\.0\.0/90.0.0/g' src/driver.rs` で一括置換する
- [x] 置換後に `grep "89\.0\.0" src/driver.rs` がヒットしないことを確認する（0 件）

## T4: `CHANGELOG.md` に v90.0.0 エントリを追加

- [x] `CHANGELOG.md` の先頭（`## [v89.0.0]` の前）に v90.0.0 エントリを追加する
- [x] `v90.0.0`・`SAP Integration 1.0 宣言`・テスト数 `4,041` が含まれることを確認する

## T5: `MILESTONE.md` に SAP Integration 1.0 を追加

- [x] `MILESTONE.md` に `SAP Integration 1.0` セクションを追加する
- [x] `SAP Integration` という文字列が含まれることを確認する

## T6: `README.md` に SAP Integration 言及を追加

- [x] `README.md` の機能一覧または最新リリースセクションに `SAP Integration` を追加する

## T7: `versions/current.md` を v90.0.0 に更新

- [x] `versions/current.md` を v89.0.0 → v90.0.0 に更新する

## T7.5: `roadmap-v85.1-v90.0.md` の全エントリを完了マークに更新

- [x] `versions/roadmap/roadmap-v85.1-v90.0.md` の Status を「完了」に更新する
- [x] バージョン一覧表の「未着手」を全て「完了」に更新する

## T8: `mod v90000_tests` を `driver.rs` に追加

- [x] `mod v89900_tests { ... }` の直後に `#[cfg(test)] mod v90000_tests { ... }` を追加する
- [x] `cargo_toml_version_is_90_0_0` テストを実装する
- [x] `changelog_has_v90_0_0` テストを実装する
- [x] `milestone_has_sap_integration` テストを実装する
- [x] `readme_mentions_sap_integration` テストを実装する

## T9: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,041 tests, 0 failures であることを確認する
  - 初回実行で `v9120_tests` の 2 件が競合（テンプファイル共用による既存の断続的問題）
  - 再実行で 4,041 tests, 0 failures を確認

> 上記テスト全 pass 後、CI 事前確認（T-last）に進む。

## Note

> 実装完了後は本ファイルの Status を `COMPLETE` に変更し、T0〜T-last の全チェックボックスを `[x]` にすること。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
