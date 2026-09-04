# Spec: v99.3.0 — Rate Limiting / Circuit Breaker

## Background

v99.2.0 で監査ログ型（`AuditEvent` / `AuditTrail` / `AuditClient`）を追加した。
v99.3.0 では SAP API の過負荷保護と障害時の自動 fallback を提供する
`CircuitBreaker<T>` 型を実装する。

`RetryPolicy` 型は v96.8.0（`runes/sap-odata/connection.fav`）で定義済みであり、
ロードマップでは「`CircuitBreaker<T>` はそれを内包する形で実装する」と記載されているが、
本バージョンでは以下の理由により `retry_policy` フィールドと `fallback` フィールドを
スコープ外とする:

- `retry_policy: RetryPolicy` — `connection.fav` の `RetryPolicy` を `resilience.fav` から
  `use` すると依存関係が複雑化する。`RetryPolicy` の組み込みは v99.7.0 の
  総合ベンチマーク時に `resilience.fav` を拡張して対応する予定。
- `fallback: Option<fn() -> T>` — Favnir は高階関数（`fn` 型の値）を
  型フィールドとして格納する構文（`Option<fn() -> T>` 等）が未サポートのため、
  本バージョンではスコープ外とする。

> **Note**: ロードマップの使用例に `CircuitBreaker.call(ctx.sap_circuit, ...)` という
> ctx フィールド参照があるが、`ctx.sap_circuit` を `AppCtx` に追加するには
> ジェネリック型パラメータの扱いを慎重に設計する必要がある。
> 本バージョンでは型定義・ヘルパー関数（`circuit_breaker_default` /
> `circuit_breaker_call_mock`）のみを提供し、`ctx.sap_circuit` の `AppCtx` への
> 組み込みは将来バージョンで対応する。

## Goals

1. `runes/sap-odata/resilience.fav` — `CircuitState` 列挙型 + `CircuitBreaker<T>` 型 + `circuit_breaker_default` / `circuit_breaker_call_mock` 関数を新規作成
2. `runes/sap-odata/sap_odata.fav` — `use sap_odata.resilience` + 4 シンボル re-export 追加（`CircuitState` / `CircuitBreaker` / `circuit_breaker_default` / `circuit_breaker_call_mock`）
3. `fav/src/driver.rs` — `mod v99300_tests`（2 テスト）追加

## Syntax / API Examples

### resilience.fav

```favnir
-- runes/sap-odata/resilience.fav
-- SAP API Rate Limiting / Circuit Breaker 型定義（v99.3.0）

-- Circuit Breaker の状態
public type CircuitState =
    | Closed     -- 通常動作
    | Open       -- トリップ中（リクエストを遮断）
    | HalfOpen   -- 復旧試行中

-- Circuit Breaker 設定
public type CircuitBreaker<T> = {
    state:            CircuitState,
    failure_count:    Int,
    threshold:        Int,
    reset_timeout_ms: Int,
    tag:              String
}

-- デフォルト設定の CircuitBreaker を生成する
public fn circuit_breaker_default<T>(tag: String) -> CircuitBreaker<T> {
    CircuitBreaker {
        state:            Closed,
        failure_count:    0,
        threshold:        5,
        reset_timeout_ms: 30000,
        tag:              tag
    }
}

-- テスト用モック: Closed 状態なら value を返す、Open なら Result.err を返す
public fn circuit_breaker_call_mock<T>(cb: CircuitBreaker<T>, value: T) -> Result<T, String> {
    match cb.state {
        Closed   -> Result.ok(value)
        HalfOpen -> Result.ok(value)
        Open     -> Result.err(String.concat(["circuit open: ", cb.tag]))
    }
}
```

### 使用例

```favnir
import rune "sap-odata"

-- Circuit Breaker で SAP API を保護するパイプライン
-- circuit_breaker_call_mock はモック関数のため !SapOData は不要
pipeline fetch_with_circuit_breaker {
    stage Init {
        bind cb <- circuit_breaker_default<List<BusinessPartner>>("sap-bp-api")
    }
    |> stage Fetch {
        bind result <- circuit_breaker_call_mock(cb, List.empty())
    }
}
```

## Success Criteria

- `runes/sap-odata/resilience.fav` が存在する
- `resilience.fav` に `CircuitState` が含まれる
- `resilience.fav` に `CircuitBreaker` が含まれる
- `resilience.fav` に `circuit_breaker_default` が含まれる
- `resilience.fav` に `circuit_breaker_call_mock` が含まれる
- `runes/sap-odata/sap_odata.fav` に `CircuitState` / `CircuitBreaker` / `circuit_breaker_default` / `circuit_breaker_call_mock` の re-export が含まれる
- `CHANGELOG.md` に `[v99.3.0]` エントリが含まれる
- `cargo test -- --test-threads=1` が 4,263 tests, 0 failures で通過する

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `runes/sap-odata/resilience.fav` | 新規作成 |
| `runes/sap-odata/sap_odata.fav` | `use sap_odata.resilience` + 4 シンボル re-export 追加 |
| `fav/src/driver.rs` | 追記（`mod v99300_tests`） |
| `CHANGELOG.md` | 追記 |
| `versions/current.md` | 更新 |

## テスト数について

ベースライン: v99.2.0 完了後の 4,261。v99.3.0 の目標は 4,261 + 2 = **4,263**。
