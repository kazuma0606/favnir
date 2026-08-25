# Tasks: v81.0.0 — Test-Driven Data 1.0 宣言 ★クリーンアップ

> ロードマップのテスト数（3831）と実際の完了条件（3841）が **10 件** ずれているが、
> v80.2.0〜v80.9.0 の code-reviewer 対応で累積 10 件追加されたことが原因。
> （計算: ロードマップ想定 3827 + 実際の drift +10 = 3837 がベース → +4 = 3841）
>
> **重要**: `changelog_has_v81_0_0` テストが含まれるため、
> T3（CHANGELOG・ドキュメント更新）を T5（`cargo test`）より **前** に実施すること。
>
> **追加判明事項**: 過去の `cargo_toml_version_is_XX` テスト群は `include_str!("../Cargo.toml")`
> で現行 Cargo.toml を読み込み、宣言バージョン文字列を一斉チェックしている。
> Cargo.toml を 81.0.0 に更新した後、driver.rs 内の `"80.0.0"` を `"81.0.0"` に一括置換（`replace_all: true`）が必要。

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3837 tests, 0 failures を確認する
- [x] `Cargo.toml` バージョンが `80.0.0` であることを確認する
  （v80.x マイナーバージョンは Cargo.toml を更新しない慣例。宣言バージョンで一括更新する）
- [x] `fav/src/driver.rs` に `mod v80900_tests` が存在することを確認する（v80.9.0 完了済みの証拠）

## T1: `Cargo.toml` バージョン更新

- [x] `fav/Cargo.toml` の `version = "80.0.0"` を `version = "81.0.0"` に変更する

## T2: `fav/src/driver.rs` に `mod v81000_tests` を追加

- [x] `mod v80900_tests { ... }` の直後に `#[cfg(test)] mod v81000_tests { ... }` を追加する
- [x] `use std::fs;` でインポートする
- [x] `cargo_toml_version_is_81_0_0` テストを実装する（`Cargo.toml` に `"81.0.0"` が含まれることを確認）
- [x] `changelog_has_v81_0_0` テストを実装する（`../CHANGELOG.md` に `"v81.0.0"` が含まれることを確認）
- [x] `milestone_has_test_driven_data` テストを実装する（`../MILESTONE.md` に `"Test-Driven Data"` が含まれることを確認）
- [x] `readme_mentions_fav_test` テストを実装する（`../README.md` に `"fav test"` が含まれることを確認）
- [x] driver.rs 内の `"80.0.0"` を `"81.0.0"` に一括置換する（過去の cargo_toml_version_is_XX テスト群が一斉 FAIL するため）

## T3: ドキュメント更新（T5 の `cargo test` より前に実施）

- [x] `CHANGELOG.md` の先頭に v81.0.0 エントリを追加する
- [x] `MILESTONE.md` に Test-Driven Data 1.0 達成宣言を追加する（`"Test-Driven Data"` を含めること）
- [x] `README.md` に `fav test` コマンドの言及を追加する
- [x] `versions/current.md` を v81.0.0 に更新する
- [x] `versions/roadmap/roadmap-v80.1-v85.0.md` の Sprint 1 テーブルを全行「完了」に更新する

## T4: `cargo clean` + `fav/tmp/hello.fav` 復元

- [x] `cargo clean` を実行してビルドキャッシュをリセットする（宣言バージョン慣例）
- [x] `fav/tmp/hello.fav` を復元する（`cargo clean` で消える既知問題）
  → 今回は `fav/tmp/` は `target/` 外のため自動的に残存していた

## T5: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3841 tests, 0 failures であることを確認する

## T-last: CI 事前確認

`cargo test` 完了後（`fav/` ディレクトリで実行。`target/debug/fav` バイナリが存在することを前提）に実行する。

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
