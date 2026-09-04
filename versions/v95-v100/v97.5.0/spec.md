# Spec: v97.5.0 — SAP BTP Integration Suite connector（`iFlowClient`）

## Background

SAP BTP Integration Suite は iFlow（統合フロー）を通じてシステム間のデータ連携を行う。
v97.4.0 で承認フロー条件分岐 pipeline が完成したため、次に BTP Integration Suite への
メッセージ送信を Favnir から型安全に行う `iFlowClient` / `IFlowMessage` を追加する。

## Goals

1. `runes/sap-odata/iflow.fav` を新規作成し、以下を定義する：
   - `IFlowClient` レコード型（`base_url` / `oauth_url` / `client_id`）
   - `IFlowMessage` レコード型（`headers: List<String>` / `body: String`）
   - `iflow_send` スタブ関数（iFlow への送信を型安全に表現）
2. `fav/src/driver.rs` に `mod v97500_tests` を追加（2 テスト）

## 型定義・API 例

```favnir
-- runes/sap-odata/iflow.fav

public type IFlowClient = {
    base_url:  String,
    oauth_url: String,
    client_id: String
}

public type IFlowMessage = {
    headers: List<String>,
    body:    String
}

-- iFlow にメッセージを送信するスタブ
-- iflow_id: 対象 iFlow の名前（例: "OrderSync_iFlow"）
-- 戻り値: SAP BTP からのレスポンス本文（スタブは固定文字列）
public fn iflow_send(client: IFlowClient, iflow_id: String, message: IFlowMessage) -> String {
    String.concat(["sent:", iflow_id])
}
```

### 利用側イメージ（pipeline から呼び出し）

```favnir
bind resp <- ctx.sap_iflow.send("OrderSync_iFlow", IFlowMessage {
    headers: ["Content-Type: application/json"],
    body:    Json.encode(order)
})
```

※ `ctx.sap_iflow` フィールドへの組み込みは後続バージョン（v97.6.0 以降）で対応。
  本バージョンは `iflow.fav` のスタブ定義のみを対象とする。

## Success Criteria

- `runes/sap-odata/iflow.fav` が存在する
- `iflow.fav` に `IFlowClient` が定義されている
- `iflow.fav` に `IFlowMessage` が定義されている
- `iflow.fav` に `iflow_send` が定義されている
- `mod v97500_tests` の全テストが pass する
- `cargo test` で 4,223 tests, 0 failures（+2）

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 種別 | 内容 |
|---|---|---|
| `runes/sap-odata/iflow.fav` | 新規 | `IFlowClient` / `IFlowMessage` / `iflow_send` |
| `fav/src/driver.rs` | 追記 | `mod v97500_tests`（2 テスト） |
| `CHANGELOG.md` | 追記 | v97.5.0 エントリ |
| `versions/current.md` | 更新 | 最新安定版を v97.5.0 に変更 |
