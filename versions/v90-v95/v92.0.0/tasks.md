# Tasks: v92.0.0 — SAP OData Query 1.0 宣言 ★クリーンアップ

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,090 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v91900_tests` が存在することを確認する（v91.9.0 完了済みの証拠）
- [x] `runes/sap-odata/query_client.fav` が存在することを確認する（v91.8.0 完了済みの証拠）
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: `CHANGELOG.md` を更新する

- [x] `## [v91.0.0]` の前に `## [v92.0.0]` エントリを追加する
- [x] v92.0.0 エントリに宣言文・Changed・Added（4,094 tests）を記載する
- [x] v91.1.0〜v91.9.0 の各エントリを v92.0.0 の直後に追加する

## T2: `fav/Cargo.toml` のバージョンを更新する

- [x] `version = "91.0.0"` を `version = "92.0.0"` に変更する

## T3: `MILESTONE.md` を更新する

- [x] `## v91.0.0` の前に `## v92.0.0（2026-08-27）— SAP OData Query 1.0 宣言` を追加する
- [x] 宣言文・達成内容（5 型 / ODataQueryBuilder / SapQueryClient 等）を記載する

## T4: `README.md` を更新する

- [x] v92.0 セクションを v91.0 の前に追加する（OData Query / SapQueryClient への言及）

## T5: `versions/current.md` を更新する

- [x] 最終更新日・バージョンを v92.0.0 に更新する
- [x] 最新安定版を v92.0.0（4,094 tests）に更新する
- [x] マイルストーン進捗表の v92.0 を「完了」に更新する

## T6: `driver.rs` 内の旧バージョン文字列を一括置換する

- [x] `sed -i 's/"91\.0\.0"/"92.0.0"/g' fav/src/driver.rs` を実行する（クォート版）
- [x] `sed -i 's/91\.0\.0/92.0.0/g' fav/src/driver.rs` を実行する（include_str! 内のエスケープ版も含む全置換）
- [x] 置換後に `grep -c '91.0.0' driver.rs` が 0 件であることを確認する

## T7: `driver.rs` に `mod v92000_tests` を追加する

- [x] `mod v91900_tests { ... }` の直後に `#[cfg(test)] mod v92000_tests { ... }` を追加する
- [x] `cargo_toml_version_is_92_0_0` テストを実装する
- [x] `changelog_has_v92_0_0` テストを実装する
- [x] `milestone_has_sap_odata_query` テストを実装する
- [x] `readme_mentions_odata_query` テストを実装する

## T8: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,094 tests, 0 failures であることを確認する

## T8b: ロードマップ推移表の実測値を反映する（v91.9.0 引き継ぎ）

- [x] `roadmap-v91.1-v92.0.md` の推移表に v91.5.0〜v91.9.0 の実測値を記入する
- [x] v92.0.0 行を `4,094 | +4` に更新する

## T9: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする（T0 の全項目を含む）

## Note

> **sed 一括置換 2 段階**: `"91.0.0"` の sed はクォート付き置換（実行時文字列）をカバーするが、`include_str!` 内で `\"91.0.0\"` のようにエスケープされた文字列はカバーしない。2 回目の sed（クォートなし `91.0.0`）で残り 44 件を置換した。

> **CHANGELOG 更新順序**: T1（CHANGELOG）は T7（driver.rs テスト追加）より前に実施した（`changelog_has_v92_0_0` テストが CHANGELOG 内容を要求するため）。

> **ロードマップのテスト数**: 実測 4,090 + 4 = 4,094（計画値 4,085 + 4 = 4,089 より +5）。推移表に実測値を反映済み。

## T-last: CI 事前確認（cargo clean 前に実施）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
- [x] `cargo clean` を実行する（13.4 GiB 削除）
