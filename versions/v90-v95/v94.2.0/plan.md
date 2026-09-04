# Plan: v94.2.0 — `ChangeSet` + `ctx.sap.batch()` 実装

## 実装ステップ

### Step 1: `runes/sap-odata/batch.fav` に `ChangeSet<T>` と `batch_request_builder` を追加する

既存の `batch.fav`（v94.1.0 で作成）に以下を追記する。

1. `public type ChangeSet<T> = { operations: List<BatchOperation<T>> }`
2. `public fn batch_request_builder<T>(entity_set: String, ops: List<BatchOperation<T>>) -> BatchRequest<T>`

注意:
- Favnir 構文: ヘルパー関数内では `bind` を使う（`let` は Rust のみ）
- `batch_request_builder` は `BatchRequest { entity_set, operations }` をそのまま返す単純な構築関数

### Step 2: `runes/sap-odata/types.fav` の `SapClient` interface に `batch` メソッドを追加する

`SapClient` interface の末尾（`}` の直前）に以下を追加する。

```favnir
    fn batch(ctx: SapClient, req: BatchRequest<String>) -> Result<BatchResponse<String>, String>
```

注意:
- `BatchRequest<String>` / `BatchResponse<String>` の型パラメータは `String`（汎用 JSON ペイロード）
- `batch.fav` の型を参照するため、import/use 構文は不要（同一 rune ディレクトリ内）

### Step 3: `fav/src/driver.rs` に `mod v94200_tests` を追加する

`mod v94100_tests { ... }` の直後に追加。

テスト 2 件:
- `change_set_type_defined`: `std::fs::read_to_string("../runes/sap-odata/batch.fav")` で `ChangeSet` の存在を確認
- `sap_client_has_batch_method`: `std::fs::read_to_string("../runes/sap-odata/types.fav")` で `batch` の存在を確認

### Step 4: `cargo build` でコンパイル確認

`cargo build` でエラーがないことを確認する。

### Step 5: `cargo test` で全 pass 確認

`cargo test 2>&1 | grep "test result"` で 4,146 tests, 0 failures を確認する。

### Step 6: CI 事前確認

- `cargo clippy --locked -- -D warnings`
- `./target/debug/fav fmt --check self/compiler.fav`
- `./target/debug/fav fmt --check self/checker.fav`
