# Plan: v85.6.0 — `SapError` 型 + エラーハンドリング（4xx / 5xx / ネットワーク）

## Step 1: 前提確認

- `cargo test` を実行し、3,941 tests, 0 failures を確認する
- `fav/src/driver.rs` に `mod v85500_tests` が存在することを確認する（v85.5.0 完了済みの証拠）
- `runes/sap-odata/types.fav` に `ODataParams` が存在することを確認する（v85.5.0 追加済み）

## Step 2: `runes/sap-odata/types.fav` に `SapErrorCode` と `SapError` を追加

既存ファイルの末尾に以下を追記する。

```favnir
-- SAP OData エラーコード列挙型（v85.6.0）
-- HTTP ステータスコードとのマッピング:
--   400 → BadRequest / 401 → Unauthorized / 403 → Forbidden
--   404 → NotFound / 5xx → ServerError / 接続失敗 → NetworkError
type SapErrorCode = NotFound | Unauthorized | Forbidden | BadRequest | ServerError | NetworkError

-- SAP OData エラー型（v85.6.0）
-- OData v4 エラーレスポンス（{"error": {"code": "...", "message": "..."}}）に対応
type SapError = {
    code:    SapErrorCode,
    message: String,
    detail:  Option<String>
}
```

## Step 3: `fav/src/driver.rs` に `mod v85600_tests` を追加

`mod v85500_tests { ... }` の直後に追加する。

```rust
#[cfg(test)]
mod v85600_tests {
    #[test]
    fn sap_error_type_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/types.fav")
            .expect("runes/sap-odata/types.fav should exist");
        assert!(
            content.contains("SapError"),
            "types.fav should define SapError type"
        );
    }

    #[test]
    fn sap_error_code_variants_exist() {
        let content = std::fs::read_to_string("../runes/sap-odata/types.fav")
            .expect("runes/sap-odata/types.fav should exist");
        assert!(
            content.contains("SapErrorCode"),
            "types.fav should define SapErrorCode type"
        );
    }
}
```

## Step 4: `cargo test` で全 pass 確認

```
cargo test 2>&1 | grep "test result"
# 期待: 3943 tests, 0 failures
```

## Step 5: CHANGELOG 更新

`CHANGELOG.md` の先頭に v85.6.0 エントリを追加する。

## Step 6: CI 事前確認

以下はすべて `fav/` ディレクトリで実行する。

```
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
