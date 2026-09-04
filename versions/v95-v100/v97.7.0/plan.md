# Plan: v97.7.0 — `MockWorkflowClient`（承認フローのオフラインテスト）

## 実装ステップ

### Step 1: `runes/sap-odata/mock.fav` に `MockWorkflowClient` を追加

既存ファイルの末尾（`MockSapClient.default()` の後）に追加する。

1. `use sap_odata.workflow` を mock.fav の use 宣言に追加する
   （`ApprovalClient` と `TaskDecision` が `workflow.fav` で定義されているため必須）

2. `MockWorkflowClient` レコード型を定義する：
   - `auto_approve: Bool`
   - `reject_reason: Option<String>`

3. `impl ApprovalClient for MockWorkflowClient` を定義する：
   - シグネチャ: `fn request_approval(client: MockWorkflowClient, subject: String, context: String) -> TaskDecision`
     （`workflow.fav` の interface 定義 `fn request_approval(client: ApprovalClient, ...)` に対応）
   - `auto_approve` を `match` して `true → Approve` / `false → Reject(Option.get_or(reject_reason, "rejected"))` を返す
   - `subject` / `context` はスタブ（v97.x で実際の承認 API に使用予定）コメント付き

### Step 2: `runes/ctx/ctx.fav` に `Ctx.mock_workflow` を追加

1. `use sap_odata.mock` を ctx.fav の use 宣言に追加する
   （`MockWorkflowClient` は `sap_odata.mock` モジュールに定義されるため必須）

2. 既存 `Ctx.mock` の直後に追加する：

```favnir
-- テスト用 AppCtx を MockWorkflowClient で構築する（v97.7.0）
-- Ctx.mock(sap: MockSapClient) との共存のため別関数として提供する。
-- sap / db / s3 / io / sap_event フィールドは vm.rs のプリミティブが
-- 提供するデフォルト値を使用する（Ctx.mock と同様）。
public fn Ctx.mock_workflow(workflow: MockWorkflowClient) -> AppCtx {
    AppCtx {
        approval: workflow
    }
}
```

**注意**: ロードマップ（行 222）の `Ctx.mock(MockWorkflowClient {...})` という表現は、
意図を示すサンプルであり、実際の実装では既存の `Ctx.mock(sap: MockSapClient)` との
共存のため `Ctx.mock_workflow` という別関数として提供する。

### Step 3: `fav/src/driver.rs` に `mod v97700_tests` を追加

`mod v97600_tests` の直後に追加（`std::fs::read_to_string` パターン — `mod v97600_tests` と同じ方式）:

```rust
#[cfg(test)]
mod v97700_tests {
    // use super::* は不要（std::fs のみ使用）
    #[test]
    fn mock_fav_has_mock_workflow_client() {
        let content = std::fs::read_to_string("../runes/sap-odata/mock.fav")
            .expect("runes/sap-odata/mock.fav should exist");
        assert!(
            content.contains("MockWorkflowClient"),
            "mock.fav should define MockWorkflowClient"
        );
    }
    #[test]
    fn ctx_fav_has_mock_workflow() {
        let content = std::fs::read_to_string("../runes/ctx/ctx.fav")
            .expect("runes/ctx/ctx.fav should exist");
        assert!(
            content.contains("mock_workflow"),
            "ctx.fav should define Ctx.mock_workflow"
        );
    }
}
```

### Step 4: `cargo test` で全 pass 確認

テスト数: 4,225 + 2 = 4,227

### Step 5: `CHANGELOG.md` に v97.7.0 エントリを追加

先頭に追加。

### Step 6: `versions/current.md` 更新

- `最終更新:` ヘッダーを `v97.7.0` に更新
- 最新安定版を `v97.7.0 — 4,227 tests` に更新

### Step 7: CI 事前確認（Clippy / Self-fmt）

- `cargo clippy --locked -- -D warnings` が pass することを確認する
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
