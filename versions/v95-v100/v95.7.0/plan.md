# Plan: v95.7.0 — バッチ部分失敗ハンドリング

## Step 1: `runes/sap-odata/batch.fav` に型と関数を追加

1. 既存の `batch_request_builder` 定義の末尾に追記する
2. `BatchItemResult<T>` 直和型を追加する
   - `| BatchSuccess(T)` — 成功ケース
   - `| BatchFailure(BatchError)` — 失敗ケース
3. `PartialSuccess<T>` レコード型を追加する
   - `succeeded: List<BatchItemResult<T>>`
   - `failed:    List<BatchItemResult<T>>`
   - `success_rate: Float`
4. `batch_with_partial<T>` スタブ関数を追加する
   - シグネチャ: `fn batch_with_partial<T>(cfg: SapConfig, req: BatchRequest<T>) -> Result<PartialSuccess<T>, String>`
   - 戻り値: `Result.err("not implemented")`
5. 各定義に `public` を付与する

## Step 2: `fav/src/driver.rs` にテストを追加

1. `mod v95600_tests` の直後に `#[cfg(test)] mod v95700_tests { ... }` を追加する
2. `batch_item_result_defined` テスト: `batch.fav` に `BatchItemResult` が含まれることを確認
3. `partial_success_defined` テスト: `batch.fav` に `PartialSuccess` が含まれることを確認
4. `batch_with_partial_defined` テスト: `batch.fav` に `fn batch_with_partial` が含まれることを確認

## Step 3: `cargo test` で全 pass 確認

- `cargo test 2>&1 | grep "test result"` を実行し、4,180 tests, 0 failures を確認する

## Step 4: CHANGELOG / current.md 更新

1. `CHANGELOG.md` の先頭に `[v95.7.0]` エントリを追加する
2. `versions/current.md` の最新安定版を `v95.7.0` に更新する

## Step 5: tasks.md 更新

- 本バージョンの `tasks.md` を COMPLETE ステータスに更新する
