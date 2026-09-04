# Plan: v94.1.0 — `BatchRequest<T>` 型定義

## 実装ステップ

### Step 1: `runes/sap-odata/batch.fav` を新規作成する

`runes/sap-odata/` ディレクトリに `batch.fav` を作成する。

定義する型:
1. `BatchOperation<T>` — ADT（BatchCreate / BatchUpdate / BatchDelete）
2. `BatchRequest<T>` — record（entity_set: String, operations: List<BatchOperation<T>>）
3. `BatchResponse<T>` — record（succeeded: List<T>, failed: List<BatchError>）
4. `BatchError` — record（index: Int, message: String）

注意事項:
- Favnir 構文: `bind` を使う（`let` は Rust のみ）
- ジェネリック型パラメータは `<T>` 表記
- ADT の variant は `| VariantName(args)` 構文

### Step 2: `fav/src/driver.rs` に `mod v94100_tests` を追加する

`mod v93900_tests { ... }` の直後（または最後の `mod` ブロックの直後）に追加。

テスト 2 件:
- `sap_batch_file_exists`: `std::path::Path::new("../runes/sap-odata/batch.fav").exists()` を assert
- `batch_request_type_defined`: `std::fs::read_to_string("../runes/sap-odata/batch.fav")` で `BatchRequest` の存在を確認

### Step 3: `cargo build` でコンパイル確認

`cd fav && cargo build` を実行し、エラーがないことを確認する。

### Step 4: `cargo test` で全 pass 確認

`cargo test 2>&1 | grep "test result"` を実行し、4,144 tests, 0 failures であることを確認する。

### Step 5: CI 事前確認

- `cargo clippy --locked -- -D warnings`
- `./target/debug/fav fmt --check self/compiler.fav`
- `./target/debug/fav fmt --check self/checker.fav`
