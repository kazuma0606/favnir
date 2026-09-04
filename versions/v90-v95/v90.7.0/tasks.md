# Tasks: v90.7.0 — `Ctx.mock` に `sap: MockSapClient` を追加

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,054 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v90600_tests` が存在することを確認する（v90.6.0 完了済みの証拠）
- [x] `runes/sap-odata/mock.fav` に `MockSapClient` が含まれることを確認する（v90.3.0 完了済みの証拠）
- [x] `runes/ctx/ctx.fav` に `Ctx.build` が含まれることを確認する（v90.4.0 完了済みの証拠）
- [x] `runes/ctx/ctx.fav` に `Ctx.mock` が含まれないことを確認する（本バージョンで追加するため）

## T1: 現状確認

- [x] `runes/ctx/ctx.fav` を読み込み `Ctx.mock` が未実装であることを確認する
- [x] `runes/sap-odata/mock.fav` を読み込み `MockSapClient.default` が未実装であることを確認する

## T2: `MockSapClient.default` を `mock.fav` に追加

- [x] `runes/sap-odata/mock.fav` の末尾に `MockSapClient.default()` 関数を追加する
- [x] 全フィールドを `Result.err("not implemented")` で初期化することを確認する
- [x] コメントスタイルが `--`（sap-odata ディレクトリの慣例）であることを確認する

## T3: `Ctx.mock` を `ctx.fav` に追加

- [x] `runes/ctx/ctx.fav` の末尾に `Ctx.mock(sap: MockSapClient) -> AppCtx` を追加する
- [x] `AppCtx { sap: sap }` を返すことを確認する
- [x] コメントスタイルが `//`（ctx ディレクトリの慣例）であることを確認する

## T4: `mod v90700_tests` を `driver.rs` に追加

- [x] `mod v90600_tests { ... }` の直後に `#[cfg(test)] mod v90700_tests { ... }` を追加する
- [x] `ctx_mock_has_sap_field` テストを実装する（`ctx.fav` に `Ctx.mock` と `sap:` が含まれる）
- [x] `mock_sap_client_default_exists` テストを実装する（`mock.fav` に `MockSapClient.default` が含まれる）

## T5: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,056 tests, 0 failures であることを確認する

## T6: `CHANGELOG.md` に v90.7.0 エントリを追加

- [x] `CHANGELOG.md` の先頭（`## [v90.6.0]` の前）に v90.7.0 エントリを追加する
- [x] `v90.7.0`・`Ctx.mock`・`MockSapClient.default`・テスト数 `4,056` が含まれることを確認する
> 本バージョンは `changelog_has_v90_7_0` Rust テストを含まないため T5 後の追加で問題ない。

> 上記テスト全 pass 後、CI 事前確認（T-last）に進む。

## Note

> 実装完了後は本ファイルの Status を `COMPLETE` に変更し、T0〜T-last の全チェックボックスを `[x]` にすること。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
