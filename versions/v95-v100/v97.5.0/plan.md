# Plan: v97.5.0 — SAP BTP Integration Suite connector（`iFlowClient`）

## 実装ステップ

### Step 1: `runes/sap-odata/iflow.fav` 新規作成

`runes/sap-odata/` に `iflow.fav` を作成する。

定義内容:
1. `IFlowClient` レコード型（`base_url: String` / `oauth_url: String` / `client_id: String`）
2. `IFlowMessage` レコード型（`headers: List<String>` / `body: String`）
3. `iflow_send` スタブ関数（`client: IFlowClient`, `iflow_id: String`, `message: IFlowMessage` → `String`）
   - スタブ実装: `String.concat(["sent:", iflow_id])`（実際の BTP API 呼び出しは将来対応）
   - 未使用引数 `client` / `message` にはスタブコメントを付与

### Step 2: `fav/src/driver.rs` に `mod v97500_tests` を追加

`mod v97400_tests` の直後に追加:

```rust
#[cfg(test)]
mod v97500_tests {
    #[test]
    fn iflow_fav_exists() {
        let _ = std::fs::read_to_string("../runes/sap-odata/iflow.fav")
            .expect("runes/sap-odata/iflow.fav should exist (v97.5.0)");
    }
    #[test]
    fn iflow_fav_has_iflow_client() {
        let content = std::fs::read_to_string("../runes/sap-odata/iflow.fav")
            .expect("runes/sap-odata/iflow.fav should exist");
        assert!(
            content.contains("IFlowClient"),
            "iflow.fav should define IFlowClient"
        );
    }
}
```

### Step 3: `cargo test` で全 pass 確認

テスト数: 4,221 + 2 = 4,223

### Step 4: `CHANGELOG.md` に v97.5.0 エントリを追加

先頭に追加。

### Step 5: `versions/current.md` 更新

- `最終更新:` ヘッダーを `v97.5.0` に更新
- 最新安定版を `v97.5.0 — 4,223 tests` に更新

### Step 6: CI 事前確認（Clippy / Self-fmt）

- `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
