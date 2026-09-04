# Plan: v90.2.0 — `AppCtx` に `sap: SapClient` フィールドを追加

## 実装ステップ

### Step 1: 既存 ctx rune ファイルの確認

`runes/ctx/` 内の既存ファイル（`db.fav`・`io.fav`・`http.fav`・`stream.fav`・`mock_db.fav`）の
形式を参照し、`ctx.fav` の記述スタイルを統一する。

### Step 2: `runes/ctx/ctx.fav` を新規作成

```favnir
// runes/ctx/ctx.fav — AppCtx 型定義（v90.2.0）
// AppCtx は Favnir パイプライン関数の依存注入コンテナ。
// Ctx.build で本番インスタンスを、Ctx.mock でテスト用インスタンスを生成する。
// 実行時は vm.rs の AppCtx プリミティブが実体を提供する。

type AppCtx = {
    s3:  StorageCtx,     // S3 等のオブジェクトストレージアクセス（v13.5.0）
    db:  DbCtx,          // DB アクセス（runes/ctx/db.fav: DbCtx）（v13.5.0）
    io:  IoCtx,          // 標準 IO / ファイル IO（runes/ctx/io.fav: IoCtx）（v13.5.0）
    sap: SapClient       // SAP S/4HANA OData アクセス（v90.2.0 追加）
}
```

### Step 3: `driver.rs` に `mod v90200_tests` を追加

`mod v90100_tests { ... }` の直後に以下を挿入する:

```rust
#[cfg(test)]
mod v90200_tests {
    // use super::* は不要（std::fs のみ使用）
    #[test]
    fn app_ctx_has_sap_field() {
        let content = std::fs::read_to_string("../runes/ctx/ctx.fav")
            .expect("ctx.fav should exist");
        assert!(
            content.contains("sap"),
            "ctx.fav AppCtx should have sap field"
        );
    }

    #[test]
    fn sap_field_type_is_sap_client() {
        let content = std::fs::read_to_string("../runes/ctx/ctx.fav")
            .expect("ctx.fav should exist");
        assert!(
            content.contains("sap: SapClient"),
            "ctx.fav should declare sap field as SapClient"
        );
    }
}
```

### Step 4: `cargo test` で全 pass 確認

```bash
cd fav && cargo test 2>&1 | grep "test result"
```

期待結果: `test result: ok. 4045 passed; 0 failed`

### Step 5: `CHANGELOG.md` に v90.2.0 エントリを追加

### Step 6: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
