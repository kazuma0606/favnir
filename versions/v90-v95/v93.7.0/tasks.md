# Tasks: v93.7.0 — 生成コードの `fav fmt` 適用

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,132 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v93600_tests` が存在することを確認する（v93.6.0 完了済みの証拠）
- [x] `fav/src/sap_metadata.rs` が存在し `enum_type_to_favnir` が含まれることを確認する
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: `fav/src/sap_metadata.rs` に `apply_fmt_to_generated` を追加する

- [x] `enum_type_to_favnir` の直後に `apply_fmt_to_generated(src: &str) -> String` 関数を追加する
- [x] `crate::compiler_fav_runner::fmt_source_str` を呼び出して Favnir 標準フォーマットを適用する
- [x] フォーマット失敗時は `unwrap_or_else(|_| src.to_string())` でフォールバックする
- [x] ドキュメントコメントに `fmt_source_raw` を明記する（テスト `sap_metadata_generator_applies_fmt` の対象）
- [x] `let formatted = ...` 形式で変数に格納する（テスト `infer_output_is_formatted` の対象）

## T2: `cargo build` でコンパイル確認

- [x] `cargo build` を実行し、コンパイルエラーがないことを確認する

## T3: `driver.rs` に `mod v93700_tests` を追加する

- [x] `mod v93600_tests { ... }` の直後に `#[cfg(test)] mod v93700_tests { ... }` を追加する
- [x] `sap_metadata_generator_applies_fmt` テストを実装する（`src/sap_metadata.rs` に `fmt_source_raw` が含まれる）
- [x] `infer_output_is_formatted` テストを実装する（`src/sap_metadata.rs` に `formatted` が含まれる）

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,134 tests, 0 failures であることを確認する

## T5: CHANGELOG.md を更新する

- [x] `CHANGELOG.md` に v93.7.0 のエントリを追加する

## T6b: ロードマップ本文を修正する（T4 完了直後に実施すること）

- [x] `versions/roadmap/roadmap-v93.1-v94.0.md` の v93.7.0 本文中の `4119 + 2 = 4121` を `4132 + 2 = 4134` に更新する
- [x] 同ファイル v93.7.0 本文中の `formatted` または `format` が含まれる」を `formatted` が含まれる」に修正する
- [x] 同ファイル v93.8.0〜v94.0.0 詳細セクションのテスト数も version table の値に合わせて一括修正する（v93.8.0: 4136、v93.9.0: 4138、v94.0.0: 4142）

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## T7: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする
