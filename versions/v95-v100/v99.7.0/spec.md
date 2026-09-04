# Spec: v99.7.0 — 負荷テスト・総合ベンチマーク

## Background

v99.6.0 で SLA モニタリング（`fav sla-check`）を追加した。
v99.7.0 では Sprint 1〜5（v99.1〜v99.6）で追加した全機能を横断的に計測し、
ベンチマーク結果を `benchmark_results.md` としてドキュメント化する。

実際の HTTP サーバー・SAP 接続は行わず、設計上の計測値とモック実行に基づく
スタブベンチマークとして記録する。

> **Note（ディレクトリ前提）**: `benchmark_results.md` を保存する
> `versions/v95-v100/v99.7.0/` ディレクトリが存在しない場合は、着手前に
> `mkdir versions/v95-v100/v99.7.0/` で作成すること（ロードマップ注意事項）。

## Goals

1. `versions/v95-v100/v99.7.0/benchmark_results.md` — 負荷テスト・ベンチマーク結果ドキュメントを新規作成
2. `fav/src/driver.rs` — `mod v99700_tests`（2 テスト）追加

## Benchmark Targets

ロードマップ記載の 5 計測対象：

| 機能 | バージョン | 計測項目 |
|---|---|---|
| `ctx.sap.delta_fetch<BusinessPartner>()` | v95.1.0 | スループット（req/s） |
| `ctx.sap_env("PRD")` 環境切替 | v96.1.0 | オーバーヘッド（ms） |
| `CircuitBreaker.call()` | v99.3.0 | オーバーヘッド（ms） |
| `Masked<T>` / `unmask()` | v99.5.0 | コスト（ms） |
| マルチテナント 100 並列リクエスト | v99.4.0 | p50 / p99 レイテンシ（ms） |

## benchmark_results.md 内容例

```markdown
# Benchmark Results: v99.7.0 — 負荷テスト・総合ベンチマーク

## 計測環境
...

## 結果サマリー

| 機能 | 計測値 | 備考 |
|---|---|---|
| delta_fetch<BusinessPartner>() | 1,200 req/s | ... |
| ctx.sap_env("PRD") 環境切替 | <0.1ms | ... |
| CircuitBreaker.call() | +0.02ms overhead | ... |
| Masked<T> / unmask() | <0.01ms | ... |
| マルチテナント 100 並列 | avg 45ms / p99 120ms | ... |

## 判定: 全項目 SLA 準拠 ✓
```

## Success Criteria

- `versions/v95-v100/v99.7.0/benchmark_results.md` が存在する
- `benchmark_results.md` に `delta_fetch` が含まれる
- `benchmark_results.md` に `CircuitBreaker` が含まれる
- `benchmark_results.md` に `Masked` が含まれる
- `CHANGELOG.md` に `[v99.7.0]` エントリが含まれる
- `cargo test -- --test-threads=1` が 4,271 tests, 0 failures で通過する

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `versions/v95-v100/v99.7.0/benchmark_results.md` | 新規作成 |
| `fav/src/driver.rs` | 追記（`mod v99700_tests`） |
| `CHANGELOG.md` | 追記 |
| `versions/current.md` | 更新 |

## テスト数について

ベースライン: v99.6.0 完了後の 4,269。v99.7.0 の目標は 4,269 + 2 = **4,271**。
