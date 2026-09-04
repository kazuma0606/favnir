# Tasks: v91.9.0 — 安定化・コードフリーズ

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,088 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v91800_tests` が存在することを確認する（v91.8.0 完了済みの証拠）
- [x] `runes/sap-odata/query.fav` に `public type ODataQueryBuilder` が含まれることを確認する
- [x] `runes/sap-odata/query_client.fav` が存在することを確認する
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: `driver.rs` に `mod v91900_tests` を追加

- [x] `mod v91800_tests { ... }` の直後に `#[cfg(test)] mod v91900_tests { ... }` を追加する
- [x] `odata_query_smoke_all_query_types` テストを実装する（`query.fav` に 5 クエリ型すべてが含まれることを確認）
- [x] `odata_filter_expr_serializable` テストを実装する（`query.fav` に `"public fn filter_to_odata_string"` が含まれることを確認）

## T2: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,090 tests, 0 failures であることを確認する

## T3: tasks.md を COMPLETE に更新

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする（T0 の全項目を含む）

## Note

> 実装完了後は本ファイルの Status を `COMPLETE` に変更し、T0〜T-last の全チェックボックスを `[x]` にすること（T0 の全項目を含む）。

> **CHANGELOG**: v91.9.0 は中間スプリントのため、CHANGELOG.md への記録は v92.0.0 宣言時にまとめて行う。

> **ロードマップのテスト数**: ロードマップは 4,088 + 2 = 4,090 に更新済み。一覧表・推移表の実測値反映は v92.0.0 宣言時に実施する。

> **バグ修正のみ**: v91.9.0 は安定化スプリントのため、スモークテスト 2 件以外の新規追加は行わない。

> **v92.0.0 への引き継ぎ**: v92.0.0 tasks.md 作成時に以下を含めること:
> - ロードマップ推移表の全実測値反映（v91.5.0〜v91.9.0）
> - `PurchaseOrderQuery` のロードマップ記述を 5 型に修正済みであることの確認

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
