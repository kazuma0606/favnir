# Spec: v97.7.0 — `MockWorkflowClient`（承認フローのオフラインテスト）

## Background

v97.3.0 で `ApprovalClient` interface が定義されたが、テスト時に本物の承認サービスへの
接続なしで pipeline を実行できるモック実装がまだない。
本バージョンで `MockWorkflowClient` を追加し、承認フローのオフラインテストを可能にする。

## Goals

1. `runes/sap-odata/mock.fav` に以下を追加する：
   - `MockWorkflowClient` レコード型（`auto_approve: Bool` / `reject_reason: Option<String>`）
   - `impl ApprovalClient for MockWorkflowClient`（`request_approval` スタブ）
2. `runes/ctx/ctx.fav` に `Ctx.mock_workflow` 関数を追加する：
   - 既存 `Ctx.mock(sap: MockSapClient)` との共存のため新関数として追加
3. `fav/src/driver.rs` に `mod v97700_tests` を追加する（2 テスト）

## 型定義・API 例

```favnir
-- runes/sap-odata/mock.fav に追加

public type MockWorkflowClient = {
    auto_approve:  Bool,
    reject_reason: Option<String>    -- Reject 時の理由（auto_approve = false の場合に使用）
}

impl ApprovalClient for MockWorkflowClient {
    fn request_approval(client: MockWorkflowClient, subject: String, context: String)
        -> TaskDecision {
        match client.auto_approve {
            true  -> Approve
            false -> Reject(Option.get_or(client.reject_reason, "rejected"))
        }
    }
}
```

```favnir
-- runes/ctx/ctx.fav に追加

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

> **設計注記**: ロードマップの `Ctx.mock(MockWorkflowClient {...})` という表記は意図を示す
> サンプルであり、実装では既存の `Ctx.mock(sap: MockSapClient)` との共存のため
> `Ctx.mock_workflow` という別関数として提供する。

### 利用側イメージ

```favnir
-- 常に承認するモック
bind ctx <- Ctx.mock_workflow(MockWorkflowClient {
    auto_approve:  true,
    reject_reason: Option.none()
})

-- 特定理由で却下するモック
bind ctx <- Ctx.mock_workflow(MockWorkflowClient {
    auto_approve:  false,
    reject_reason: Option.some("予算超過")
})
```

## Success Criteria

- `runes/sap-odata/mock.fav` に `MockWorkflowClient` が定義されている
- `mock.fav` に `impl ApprovalClient for MockWorkflowClient` が含まれている
- `runes/ctx/ctx.fav` に `Ctx.mock_workflow` が定義されている
- `mod v97700_tests` の全テストが pass する
- `cargo test` で 4,227 tests, 0 failures（+2）

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 種別 | 内容 |
|---|---|---|
| `runes/sap-odata/mock.fav` | 追記 | `MockWorkflowClient` 型 + `impl ApprovalClient` |
| `runes/ctx/ctx.fav` | 追記 | `Ctx.mock_workflow` 関数 |
| `fav/src/driver.rs` | 追記 | `mod v97700_tests`（2 テスト） |
| `CHANGELOG.md` | 追記 | v97.7.0 エントリ |
| `versions/current.md` | 更新 | 最新安定版を v97.7.0 に変更 |
