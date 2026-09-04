# Plan: v90.8.0 — サイトドキュメント更新（ctx.sap パターンガイド）

## 依存関係

```
Step 1（現状確認）
    ↓
Step 2（sap-odata.mdx 更新）
    ↓
Step 3（driver.rs テスト追加）
    ↓
Step 4（cargo test）
    ↓
Step 5（CHANGELOG 更新）
    ↓
Step 6（CI 事前確認）
```

## Steps

### Step 1: 現状確認

- `site/content/docs/runes/sap-odata.mdx` を読み込み、旧スタイル（`sap_config_from_env`）が残っていることを確認する
- 現テスト数が 4,056 であることを確認する

### Step 2: `sap-odata.mdx` を更新

以下の 2 点を実施する:

#### 2-1: 既存コード例を `ctx.sap.*` スタイルに書き換える

各エンティティセクション（BusinessPartner / SalesOrder / Material / JournalEntry）のコード例を更新:
- `bind cfg <- sap_odata.sap_config_from_env()` の行を削除
- `sap_odata.METHOD(cfg, filter)` → `ctx.sap.METHOD(filter)` に書き換え
- 関数シグネチャを `fn xxx(ctx: AppCtx) ->` に更新

#### 2-2: 以下の 3 セクションを末尾（`## Docker Compose モックサーバー` の前）に追加する

**`## ctx.sap パターン`**
- AppCtx 経由で SAP にアクセスする方法の説明
- `fn sync_business_partners(ctx: AppCtx) -> Result<Int, String>` のコード例

**`## MockSapClient でユニットテスト`**
- `Ctx.mock(MockSapClient { ... })` の使用例
- `MockSapClient.default()` の使用例

**`## Ctx.build 自動設定注入`**
- `Ctx.build()` が `fav.toml [sap]` / 環境変数から自動設定することの説明
- 本番コードでは `bind ctx <- Ctx.build()` → `ctx.sap.METHOD(filter)` の流れを説明

### Step 3: `driver.rs` に `mod v90800_tests` を追加

`mod v90700_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v90800_tests {
    #[test]
    fn docs_sap_odata_mentions_ctx_sap() {
        let content = std::fs::read_to_string(
            "../site/content/docs/runes/sap-odata.mdx",
        )
        .expect("sap-odata.mdx should exist");
        assert!(
            content.contains("ctx.sap"),
            "sap-odata.mdx should mention ctx.sap"
        );
    }

    #[test]
    fn docs_sap_odata_mentions_mock_sap_client() {
        let content = std::fs::read_to_string(
            "../site/content/docs/runes/sap-odata.mdx",
        )
        .expect("sap-odata.mdx should exist");
        assert!(
            content.contains("MockSapClient"),
            "sap-odata.mdx should mention MockSapClient"
        );
    }
}
```

### Step 4: `cargo test` で全 pass 確認

- `cargo test 2>&1 | grep "test result"` で 4,058 tests, 0 failures を確認する

### Step 5: `CHANGELOG.md` に v90.8.0 エントリを追加

- `## [v90.7.0]` の前に v90.8.0 エントリを追加する
- `ctx.sap` / `MockSapClient` / `Ctx.build` / `4,058` が含まれることを確認する

### Step 6: CI 事前確認

- `cargo clippy --locked -- -D warnings` が pass することを確認する
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
