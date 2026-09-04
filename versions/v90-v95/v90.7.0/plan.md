# Plan: v90.7.0 — `Ctx.mock` に `sap: MockSapClient` を追加

## 依存関係

```
Step 1（現状確認）
    ↓
Step 2（MockSapClient.default を mock.fav に追加）
    ↓
Step 3（Ctx.mock を ctx.fav に追加）
    ↓
Step 4（driver.rs テスト追加）
    ↓
Step 5（cargo test）
    ↓
Step 6（CHANGELOG 更新）
    ↓
Step 7（CI 事前確認）
```

## Steps

### Step 1: 現状確認

- `runes/ctx/ctx.fav` を読み込み `Ctx.mock` が未実装であることを確認する
- `runes/sap-odata/mock.fav` を読み込み `MockSapClient.default` が未実装であることを確認する
- 現テスト数が 4,054 であることを確認する

### Step 2: `MockSapClient.default` を `mock.fav` に追加

`runes/sap-odata/mock.fav` の末尾に以下を追加する:

```favnir
-- MockSapClient のデフォルト値コンストラクタ（v90.7.0）
-- 全フィールドを Result.err("not implemented") で初期化する。
-- テスト側で必要なフィールドのみ上書きして使用する。
public fn MockSapClient.default() -> MockSapClient {
    MockSapClient {
        business_partners_result: Result.err("not implemented"),
        sales_orders_result:      Result.err("not implemented"),
        materials_result:         Result.err("not implemented"),
        journal_entries_result:   Result.err("not implemented")
    }
}
```

### Step 3: `Ctx.mock` を `ctx.fav` に追加

`runes/ctx/ctx.fav` の末尾に以下を追加する:

```favnir
// AppCtx をテスト用設定で構築する（v90.7.0）
// sap: MockSapClient を受け取り AppCtx を返す。
// 他フィールド（db / s3 / io）は vm.rs のプリミティブが提供するデフォルト値を使用する。
public fn Ctx.mock(sap: MockSapClient) -> AppCtx {
    AppCtx {
        sap: sap
    }
}
```

### Step 4: `driver.rs` に `mod v90700_tests` を追加

`mod v90600_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v90700_tests {
    use std::fs;

    #[test]
    fn ctx_mock_has_sap_field() {
        let content = fs::read_to_string("../runes/ctx/ctx.fav").unwrap();
        assert!(content.contains("Ctx.mock"));
        assert!(content.contains("sap:"));
    }

    #[test]
    fn mock_sap_client_default_exists() {
        let content = fs::read_to_string("../runes/sap-odata/mock.fav").unwrap();
        assert!(content.contains("MockSapClient.default"));
    }
}
```

### Step 5: `cargo test` で全 pass 確認

- `cargo test 2>&1 | grep "test result"` で 4,056 tests, 0 failures を確認する

### Step 6: `CHANGELOG.md` に v90.7.0 エントリを追加

- `## [v90.6.0]` の前に v90.7.0 エントリを追加する
- `Ctx.mock` / `MockSapClient.default` / `4,056` が含まれることを確認する

### Step 7: CI 事前確認

- `cargo clippy --locked -- -D warnings` が pass することを確認する
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
