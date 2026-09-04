# Spec: v99.2.0 — `!Audit` エフェクトマーカー + 監査ログ ctx interface

## Background

v99.1.0 で SAP BTP OAuth2 認証型（`BtpCredential` / `BtpToken`）を追加した。
v99.2.0 では SAP データへのアクセスを監査ログとして記録する `AuditEvent` 型・
`AuditTrail` 型・`AuditClient` ctx interface を追加する。

ctx パターンに従い、`AppCtx` に `audit: AuditClient` フィールドを追加する。

> **Note**: `!Audit` pipeline マーカー（Rust `Effect` enum への `Audit` 追加と
> `checker.fav` の exhaustive match 更新）はロードマップに記載されているが、
> 本バージョンでは実装しない。理由: `!SapOData` / `!SapAnalytics` のような
> 既存のエフェクトマーカーと異なり、`!Audit` は「監査証跡が必要な pipeline」
> を型レベルで示すマーカーであり、Rust checker が `!Audit` を未知エフェクトとして
> 扱うと E0016/E0017 等のエラーが発生する。Rust `Effect` enum への追加は
> 段階的実装の複雑さを考慮し、v99.4.0 以降の宣言版で対応する。
> 本バージョンのパイプライン使用例は `ctx.audit.log()` を `!SapOData` pipeline 内で
> 呼び出すパターンとし、`!Audit` マーカーを pipeline 署名には付与しない。
>
> `AuditTrail` 型はロードマップに明記されており、本バージョンで定義する。

## Goals

1. `runes/sap-odata/audit.fav` — `AuditEvent` 型 + `AuditTrail` 型 + `AuditClient` interface + `log_audit_event_mock` 関数を新規作成
2. `runes/sap-odata/sap_odata.fav` — `use sap_odata.audit` + 3 シンボル re-export 追加（`AuditEvent` / `AuditTrail` / `log_audit_event_mock`）
3. `runes/ctx/ctx.fav` — `AppCtx` に `audit: AuditClient` フィールドを追加
4. `fav/src/driver.rs` — `mod v99200_tests`（2 テスト）追加

## Syntax / API Examples

### audit.fav

```favnir
-- runes/sap-odata/audit.fav
-- SAP 監査ログ型定義（v99.2.0）

-- 監査イベント（SAP データアクセス 1 件につき 1 レコード）
public type AuditEvent = {
    actor:     String,
    action:    String,
    resource:  String,
    timestamp: String,
    result:    String
}

-- 監査証跡（複数 AuditEvent のコレクション）
public type AuditTrail = {
    events:    List<AuditEvent>,
    pipeline:  String,
    started_at: String
}

-- 監査ログクライアント interface（本番: CloudWatch / Splunk 等）
public interface AuditClient {
    fn log(event: AuditEvent) -> Result<Unit, String>
}

-- テスト用モック: 受け取った AuditEvent を受理して Result.ok を返す
public fn log_audit_event_mock(event: AuditEvent) -> Result<Unit, String> {
    Result.ok(Unit)
}
```

### ctx.fav への追加

```favnir
type AppCtx = {
    -- ... 既存フィールド ...
    audit: AuditClient,  -- 監査ログクライアント（runes/sap-odata/audit.fav: AuditClient）（v99.2.0 追加）
}
```

### 使用例（pipeline）

```favnir
import rune "sap-odata"

-- SAP BusinessPartner 取得 + 監査ログ記録 pipeline
-- !Audit マーカーは v99.4.0 以降で追加予定。現バージョンは !SapOData のみ。
pipeline fetch_bp_with_audit !SapOData {
    stage Fetch {
        bind bp <- ctx.sap.business_partner_by_id("10000001", false)
        bind _  <- ctx.audit.log(AuditEvent {
            actor:     "pipeline-user",
            action:    "READ",
            resource:  "BusinessPartner/10000001",
            timestamp: "2026-09-03T00:00:00Z",
            result:    "success"
        })
    }
}
```

## Success Criteria

- `runes/sap-odata/audit.fav` が存在する
- `audit.fav` に `AuditEvent` が含まれる
- `audit.fav` に `AuditTrail` が含まれる
- `audit.fav` に `AuditClient` が含まれる
- `runes/sap-odata/sap_odata.fav` に `AuditEvent` / `AuditTrail` / `log_audit_event_mock` の re-export が含まれる
- `runes/ctx/ctx.fav` の `AppCtx` に `audit: AuditClient` フィールドが存在する
- `cargo test -- --test-threads=1` が 4,261 tests, 0 failures で通過する

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `runes/sap-odata/audit.fav` | 新規作成 |
| `runes/sap-odata/sap_odata.fav` | `use sap_odata.audit` + `AuditEvent` / `AuditTrail` / `log_audit_event_mock` re-export 追加 |
| `runes/ctx/ctx.fav` | `AppCtx` に `audit: AuditClient` フィールドを追加 |
| `fav/src/driver.rs` | 追記（`mod v99200_tests`） |
| `CHANGELOG.md` | 追記 |
| `versions/current.md` | 更新 |

## テスト数について

ベースライン: v99.1.0 完了後の 4,259。v99.2.0 の目標は 4,259 + 2 = **4,261**。
