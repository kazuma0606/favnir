# Tasks: v95.7.0 — バッチ部分失敗ハンドリング

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v95.6.0` であることを確認する
- [x] `runes/sap-odata/rpc.fav` が存在することを確認する（v95.6.0 完了済みの証拠）
- [x] `fav/src/driver.rs` に `mod v95600_tests` が存在することを確認する（v95.6.0 完了済みの証拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,177 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `95.0.0` であることを確認する
  （スプリント内バージョンでは Cargo.toml を更新しない）

## T1: `batch.fav` に型と関数を追加

- [x] `BatchItemResult<T>` 直和型を追加する（`BatchSuccess(T)` / `BatchFailure(BatchError)`）
- [x] `PartialSuccess<T>` レコード型を追加する（succeeded / failed / success_rate フィールド）
- [x] `batch_with_partial<T>` スタブ関数を追加する
  （`cfg: SapConfig, req: BatchRequest<T>) -> Result<PartialSuccess<T>, String>`、戻り値 `Result.err("not implemented")`）
- [x] 各定義に `public` を付与する

## T2: `driver.rs` にテストを追加

- [x] `mod v95600_tests` の直後に `#[cfg(test)] mod v95700_tests { ... }` を追加する
- [x] `batch_item_result_defined` テストを追加する（`batch.fav` に `BatchItemResult` が含まれる）
- [x] `partial_success_defined` テストを追加する（`batch.fav` に `PartialSuccess` が含まれる）
- [x] `batch_with_partial_defined` テストを追加する（`batch.fav` に `fn batch_with_partial` が含まれる）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,180 tests, 0 failures であることを確認する

## T4: CHANGELOG / current.md 更新

- [x] `CHANGELOG.md` の先頭に `[v95.7.0]` エントリを追加する
- [x] `versions/current.md` の最新安定版を `v95.7.0` に更新する

## T5: tasks.md 更新

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする

## T-last: CI 事前確認（T3 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
