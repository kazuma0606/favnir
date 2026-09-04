# Spec: v99.4.0 — マルチテナント対応

## Background

v99.3.0 で Circuit Breaker 型（`CircuitBreaker<T>`）を追加した。
v99.4.0 では SaaS 製品での使用を想定し、テナントごとに SAP 接続を切り替える
`TenantContext` 型を追加する。

`SapEnvironment` 型は v96.1.0（`runes/sap-odata/types.fav`）で定義済みであり、
`TenantContext` はそれを参照する形で設計する。

> **Note**: ロードマップの使用例に `ctx.sap.for_tenant("CUSTOMER_A")` という
> `SapClient` interface へのメソッド追加が示されているが、`SapClient` interface の
> 変更は `runes/sap-odata/client.fav`（interface 定義）/ `runes/sap-odata/mock.fav`
> （`MockSapClient` 実装）/ `fav/src/middle/checker.rs`（`SapClient` 組み込み型）
> への波及が伴うため、本バージョンのスコープ外とする。
> 本バージョンでは型定義（`TenantId` / `TenantContext`）とモック関数
> （`tenant_context_mock`）のみを提供し、`ctx.sap.for_tenant()` の実装は
> 将来バージョンで対応する。
>
> `ctx.fav` への追加は、`Ctx.for_tenant_mock` ヘルパー関数として提供する。
> これにより `TenantContext` を使ったテストが即時可能になる。

## Goals

1. `runes/sap-odata/tenant.fav` — `TenantId` 型エイリアス + `TenantContext` 型 + `tenant_context_mock` 関数を新規作成
2. `runes/sap-odata/sap_odata.fav` — `use sap_odata.tenant` + 3 シンボル re-export 追加（`TenantId` / `TenantContext` / `tenant_context_mock`）
3. `runes/ctx/ctx.fav` — `Ctx.for_tenant_mock` ヘルパー関数を追加
4. `fav/src/driver.rs` — `mod v99400_tests`（2 テスト）追加

## Syntax / API Examples

### tenant.fav

```favnir
-- runes/sap-odata/tenant.fav
-- SAP マルチテナント型定義（v99.4.0）

use sap_odata.types

-- テナント識別子
public type TenantId = String

-- テナントコンテキスト（テナントごとの SAP 接続設定）
-- SapEnvironment は v96.1.0 で定義済み（Prd / Qas / Dev / Custom(String)）
public type TenantContext = {
    tenant_id: TenantId,
    sap_env:   types.SapEnvironment,
    schema:    String
}

-- テスト用モック: TenantId から TenantContext を生成する
public fn tenant_context_mock(tenant_id: TenantId) -> TenantContext {
    TenantContext {
        tenant_id: tenant_id,
        sap_env:   types.SapEnvironment.Dev,
        schema:    String.concat(["schema_", tenant_id])
    }
}
```

### ctx.fav への追加

```favnir
-- テスト用 TenantContext を生成する（v99.4.0）
public fn Ctx.for_tenant_mock(tenant_id: String) -> tenant.TenantContext {
    tenant.tenant_context_mock(tenant_id)
}
```

### 使用例（pipeline）

```favnir
import rune "sap-odata"

-- テナントごとに SAP 接続を切り替えるパイプライン（テスト用）
pipeline fetch_for_tenant {
    stage Init {
        bind tenant_ctx <- Ctx.for_tenant_mock("CUSTOMER_A")
    }
    |> stage Fetch {
        bind _result <- Result.ok(tenant_ctx.tenant_id)
    }
}
```

## Success Criteria

- `runes/sap-odata/tenant.fav` が存在する
- `tenant.fav` に `TenantId` が含まれる
- `tenant.fav` に `TenantContext` が含まれる
- `tenant.fav` に `tenant_context_mock` が含まれる
- `runes/sap-odata/sap_odata.fav` に `TenantId` / `TenantContext` / `tenant_context_mock` の re-export が含まれる
- `runes/ctx/ctx.fav` に `Ctx.for_tenant_mock` が含まれる
- `CHANGELOG.md` に `[v99.4.0]` エントリが含まれる
- `cargo test -- --test-threads=1` が 4,265 tests, 0 failures で通過する

> **Note**: `sap_odata.fav` の re-export 追加と `ctx.fav` の `Ctx.for_tenant_mock`
> 追加は driver.rs テストではなく目視確認（tasks.md T2 / T3）で検証する。
> テスト数は 4,265（+2）のまま維持する。

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `runes/sap-odata/tenant.fav` | 新規作成 |
| `runes/sap-odata/sap_odata.fav` | `use sap_odata.tenant` + 3 シンボル re-export 追加 |
| `runes/ctx/ctx.fav` | `use sap_odata.tenant` + `Ctx.for_tenant_mock` 関数追加 |
| `fav/src/driver.rs` | 追記（`mod v99400_tests`） |
| `CHANGELOG.md` | 追記 |
| `versions/current.md` | 更新 |

## テスト数について

ベースライン: v99.3.0 完了後の 4,263。v99.4.0 の目標は 4,263 + 2 = **4,265**。
