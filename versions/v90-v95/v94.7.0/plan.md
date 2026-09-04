# Plan: v94.7.0 — E2E デモ更新（$batch + SnapStart 完全デモ）

## 実装ステップ

### Step 1: `infra/e2e-demo/sap-odata/pipeline_advanced.fav` を新規作成する

既存の `pipeline.fav`（シナリオ 1〜4）と同じスタイルで、シナリオ 5 を実装する。

注意点:
- `import rune "sap-odata"` / `import rune "s3"` を先頭に記載する
- `ctx.sap.batch(req)` の呼び出しを必ず含める（テスト要件）
- `bind x <- pure_fn()` パターン: 純粋計算（`List.map` / `batch_request_builder` 等）は
  `Result.ok()` でラップせず `bind x <- expr` のみで記述する
- `--` コメント形式を使用する（既存 pipeline.fav に合わせる）

実装内容:
```
シナリオ 5: $batch + QueryBuilder による取引先一括同期
1. BusinessPartnerFilter を組み立て ctx.sap.business_partners() で取得
2. S3 に JSON バックアップ保存
3. List.map で BatchUpdate 操作リストを作成
4. batch_request_builder で BatchRequest を作成
5. ctx.sap.batch(req) でバッチ送信
6. Result.ok(... 同期件数 ...) を返す
```

### Step 2: `fav/src/driver.rs` に `mod v94700_tests` を追加する

`mod v94600_tests { ... }` の直後に追加。

テスト 2 件:
- `pipeline_advanced_fav_exists`: `std::path::Path::new("../infra/e2e-demo/sap-odata/pipeline_advanced.fav").exists()` で存在確認
- `pipeline_advanced_uses_batch`: `std::fs::read_to_string("../infra/e2e-demo/sap-odata/pipeline_advanced.fav")` で
  `"ctx.sap.batch"` が含まれることを確認

### Step 3: `CHANGELOG.md` に v94.7.0 エントリを追記する

### Step 4: `cargo build` でコンパイル確認

### Step 5: `cargo test` で全 pass 確認

`cargo test 2>&1 | grep "test result"` で 4,156 tests, 0 failures を確認する。

### Step 6: CI 事前確認

- `cargo clippy --locked -- -D warnings`
- `./target/debug/fav fmt --check self/compiler.fav`
- `./target/debug/fav fmt --check self/checker.fav`
