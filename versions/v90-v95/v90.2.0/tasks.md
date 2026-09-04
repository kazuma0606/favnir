# Tasks: v90.2.0 — `AppCtx` に `sap: SapClient` フィールドを追加

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,043 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v90100_tests` が存在することを確認する（v90.1.0 完了済みの証拠）
- [x] `runes/sap-odata/types.fav` に `interface SapClient` が含まれることを確認する（v90.1.0 完了済みの証拠）
- [x] `runes/ctx/ctx.fav` が存在しないことを確認する（本バージョンで新規作成するファイル）

## T1: 既存 ctx rune ファイルの確認

- [x] `runes/ctx/db.fav` の記述形式を確認する（`//` コメント・`interface DbCtx` の形式を採用）
- [x] `runes/ctx/io.fav` の記述形式を確認する（`//` コメント・`interface IoCtx` の形式を採用）

## T2: `runes/ctx/ctx.fav` を新規作成

- [x] `runes/ctx/ctx.fav` を新規作成する
- [x] `AppCtx` type 定義に `s3: StorageCtx`・`db: DbCtx`・`io: IoCtx` の既存フィールドを含める（型名は実在する interface 名に合わせる）
- [x] `sap: SapClient` フィールドを追加する（v90.2.0 追加であることをコメントで明記）
- [x] ファイルヘッダーコメント（`//` スタイル、バージョン・用途説明）を追加する

## T3: `mod v90200_tests` を `driver.rs` に追加

- [x] `mod v90100_tests { ... }` の直後に `#[cfg(test)] mod v90200_tests { ... }` を追加する
- [x] `app_ctx_has_sap_field` テストを実装する（`ctx.fav` に `sap` が含まれることを確認）
- [x] `sap_field_type_is_sap_client` テストを実装する（`ctx.fav` に `sap: SapClient` が含まれることを確認）

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,045 tests, 0 failures であることを確認する

## T5: `CHANGELOG.md` に v90.2.0 エントリを追加

- [x] `CHANGELOG.md` の先頭（`## [v90.1.0]` の前）に v90.2.0 エントリを追加する
- [x] `v90.2.0`・`AppCtx`・`sap: SapClient`・テスト数 `4,045` が含まれることを確認する
> 本バージョンは `changelog_has_v90_2_0` Rust テストを含まないため T4 後の追加で問題ない。
> 将来 CHANGELOG テスト（`changelog_has_vXX`）を追加する場合は T3 より前に移動すること。

> 上記テスト全 pass 後、CI 事前確認（T-last）に進む。

## Note

> 実装完了後は本ファイルの Status を `COMPLETE` に変更し、T0〜T-last の全チェックボックスを `[x]` にすること。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
