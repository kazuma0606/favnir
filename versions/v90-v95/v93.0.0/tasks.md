# Tasks: v93.0.0 — SAP QueryBuilder 1.0 宣言 ★クリーンアップ

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,116 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v92900_tests` が存在することを確認する（v92.9.0 完了済みの証拠）
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: `cargo clean`

- [x] `cargo clean` を実行する（13408 files, 8.4GiB 削除）
- [x] `fav/tmp/hello.fav` が消えていないことを確認する（target/ 外のため影響なし）

## T2: `Cargo.toml` バージョン更新

- [x] `fav/Cargo.toml` の `version = "92.0.0"` を `version = "93.0.0"` に変更する

## T3: `driver.rs` の旧バージョン参照を一括更新

- [x] `sed -i 's/92\.0\.0/93.0.0/g'` で `92.0.0` を `93.0.0` に一括置換する（48 箇所）

## T4: `CHANGELOG.md` を更新

- [x] `CHANGELOG.md` の先頭に v93.0.0 エントリを追加する（宣言文・Changed・Added・テスト数）

## T5: `MILESTONE.md` を更新

- [x] `MILESTONE.md` に SAP QueryBuilder 1.0 宣言セクションを追加する

## T6: `README.md` を更新

- [x] `README.md` に `QueryBuilder` の言及を追加する（v93.0 セクション追加）

## T7: `versions/current.md` を更新

- [x] `versions/current.md` の最新安定版を v93.0.0 に更新する
- [x] `v93.0` マイルストーン行を「計画中」→「完了」に更新し W020→W060 を修正

## T8: `mod v93000_tests` を `driver.rs` に追加する

- [x] ファイル末尾の `mod v92900_tests { ... }` の直後に `#[cfg(test)] mod v93000_tests { ... }` を追加する
- [x] `cargo_toml_version_is_93_0_0` テストを実装する
- [x] `changelog_has_v93_0_0` テストを実装する
- [x] `milestone_has_sap_query_builder` テストを実装する
- [x] `readme_mentions_query_builder` テストを実装する

## T9: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,120 tests, 0 failures であることを確認する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## T10: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする
