# Tasks: v94.0.0 — SAP Metadata Infer 1.0 宣言 ★クリーンアップ

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,138 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v93900_tests` が存在することを確認する（v93.9.0 完了済みの証拠）
- [x] `fav/tmp/hello.fav` が存在することを確認する
- [x] `site/content/docs/cli/infer.mdx` と `site/content/docs/runes/sap-odata.mdx` は v93.8.0 で更新済みのため本バージョンでの MDX 変更は不要であることを確認する

## T1: `cargo clean` を実施する

- [x] `cargo clean` を実行してビルド成果物をリセットする（8.5GiB 削減）
- [x] `cargo clean` 後に `fav/tmp/hello.fav` が存在することを確認する（存在を確認済み）

## T2: `fav/Cargo.toml` バージョンを更新する

- [x] `version = "93.0.0"` を `version = "94.0.0"` に変更する

## T3: `CHANGELOG.md` を更新する（v94000_tests 追加より前に実施すること）

- [x] `CHANGELOG.md` に v94.0.0 の宣言エントリを追加する（`changelog_has_v94_0_0` テストのため先行実施）

## T4: `MILESTONE.md` を更新する

- [x] 先頭に v94.0.0 — SAP Metadata Infer 1.0 宣言ブロックを追加する（`milestone_has_sap_metadata_infer` テストのため `"SAP Metadata Infer"` を含める）

## T5: `README.md` を更新する

- [x] 既存の v93.0 セクションの前に v94.0 宣言セクションを追加する（`readme_mentions_metadata_infer` テストのため `"Metadata Infer"` を含める）

## T6: `versions/current.md` を更新する

- [x] 「最終更新」を v94.0.0（2026-08-30）に更新する
- [x] 「最新安定版」欄を v94.0.0 — SAP Metadata Infer 1.0 宣言、4,142 tests に更新する

## T7: `driver.rs` の全 `cargo_toml_version_is_X_0_0` テストを一括更新する

- [x] `sed -i 's/93\.0\.0/94.0.0/g' src/driver.rs` で assert 内バージョン文字列を一括置換する
- [x] 置換後に `cargo_toml_version_is_93_0_0` が `"94.0.0"` を assert するようになったことを確認する

## T8: `driver.rs` に `mod v94000_tests` を追加する

- [x] `mod v93900_tests { ... }` の直後に `#[cfg(test)] mod v94000_tests { ... }` を追加する（4 テスト）
- [x] `cargo_toml_version_is_94_0_0`: `Cargo.toml` に `version = "94.0.0"` が含まれる
- [x] `changelog_has_v94_0_0`: `../CHANGELOG.md` に `v94.0.0` が含まれる
- [x] `milestone_has_sap_metadata_infer`: `../MILESTONE.md` に `SAP Metadata Infer` が含まれる
- [x] `readme_mentions_metadata_infer`: `../README.md` に `Metadata Infer` が含まれる

## T9: `cargo build` でコンパイル確認

- [x] `cargo build` を実行し、コンパイルエラーがないことを確認する（fav v94.0.0）

## T9a: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,142 tests, 0 failures であることを確認する

## T-last: CI 事前確認（`cargo build` 完了後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## T10: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする
