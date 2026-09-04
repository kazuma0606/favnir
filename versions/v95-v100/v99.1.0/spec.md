# Spec: v99.1.0 — OAuth2 PKCE / SAP BTP Trust Configuration

## Background

v99.0.0 で SAP Analytics 1.0 を宣言した。v99.1.0 からは SAP Platform Era（v99.1〜v100.0）として
エンタープライズ本番対応機能を追加していく。第 1 弾として、本番 SAP 環境への認証を
OAuth2 PKCE フローで行う `BtpCredential` 型と `BtpToken` 型を追加する。
SAP BTP の Service Key / XSUAA トークン取得を Favnir から自動化するための型基盤を整備する。

> **Note**: ロードマップ記載の `ctx.sap_btp.acquire_token()` は ctx interface の追加が必要となるため
> 本バージョンのスコープ外。本バージョンでは型定義（`BtpCredential` / `BtpToken`）と
> テスト用モック関数（`acquire_token_mock`）のみを提供し、
> `ctx.sap_btp` interface の実装は将来バージョン（v99.x）で対応する。

## Goals

1. `runes/sap-odata/btp_auth.fav` — `BtpCredential` / `BtpToken` 型 + `acquire_token_mock` 関数を新規作成
2. `fav/src/driver.rs` — `mod v99100_tests`（2 テスト）追加

## Syntax / API Examples

### btp_auth.fav

```favnir
-- runes/sap-odata/btp_auth.fav
-- SAP BTP OAuth2 PKCE 認証型定義（v99.1.0）

-- BTP サービスキー認証情報
public type BtpCredential = {
    client_id:     String,
    client_secret: String,
    token_url:     String,
    scope:         List<String>
}

-- BTP アクセストークン
public type BtpToken = {
    access_token: String,
    expires_in:   Int,
    token_type:   String
}

-- テスト用モック: BtpCredential からダミー BtpToken を返す
public fn acquire_token_mock(cred: BtpCredential) -> BtpToken {
    BtpToken {
        access_token: String.concat(["mock_token_for_", cred.client_id]),
        expires_in:   3600,
        token_type:   "Bearer"
    }
}
```

### 使用例（pipeline）

```favnir
import rune "sap-odata"

-- SAP BTP トークン取得 → SAP OData API 呼び出し pipeline
pipeline fetch_with_btp_auth !SapOData {
    stage Auth {
        bind cred  <- Result.ok(BtpCredential {
            client_id:     "my-client-id",
            client_secret: "my-client-secret",
            token_url:     "https://my-tenant.authentication.eu10.hana.ondemand.com/oauth/token",
            scope:         ["API_BUSINESS_PARTNER"]
        })
        bind token <- Result.ok(acquire_token_mock(cred))
    }
    |> stage Fetch {
        bind _orders <- ctx.sap.sales_orders(SalesOrderFilter {
            date_from: Option.some("2026-09-03"),
            date_to:   Option.none(),
            top:       Option.some(100)
        })
    }
}
```

## Success Criteria

- `runes/sap-odata/btp_auth.fav` が存在する
- `btp_auth.fav` に `BtpCredential` が含まれる
- `cargo test -- --test-threads=1` が 4,259 tests, 0 failures で通過する

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `runes/sap-odata/btp_auth.fav` | 新規作成 |
| `runes/sap-odata/sap_odata.fav` | `use sap_odata.btp_auth` + `BtpCredential`/`BtpToken`/`acquire_token_mock` re-export 追加 |
| `fav/src/driver.rs` | 追記（`mod v99100_tests`） |
| `CHANGELOG.md` | 追記 |
| `versions/current.md` | 更新 |

## テスト数について

ロードマップ記載値（4,254）は v99.0.0 の実際のテスト増分を反映していない。
実際のベースラインは v99.0.0 完了後の 4,257 であるため、v99.1.0 の目標は 4,257 + 2 = **4,259**。
