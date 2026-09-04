# Spec: v99.6.0 — SLA モニタリング + `fav sla-check`

## Background

v99.5.0 で GDPR データマスキング（`Masked<T>`）を追加した。
v99.6.0 では SAP API の応答時間 SLA を定義し、違反を検出する `SlaDefinition` / `SlaViolation`
型と `fav sla-check` CLI コマンドを追加する。

> **Note（実装パターン）**: ロードマップには `fav/src/main.rs` と `fav/src/driver.rs` を
> 修正ファイルとして挙げているが、v95.8.0（`fav sap-mock`）の実装パターンに倣い、
> `SlaDefinition` / `SlaViolation` 構造体と `cmd_sla_check` 関数は `driver.rs` に定義する。
> `main.rs` は `sla-check` サブコマンドのルーティング（`cmd_sla_check` 呼び出し）のみを担う。
>
> v99.6.0 の `cmd_sla_check` はスタブ実装（stdout にフォーマット済み文字列を出力するのみ）。
> 実際の SLA 測定・TOML 設定ファイル解析は将来バージョンで実施する。

## Goals

1. `fav/src/driver.rs` — `SlaDefinition` / `SlaViolation` 構造体 + `cmd_sla_check` 関数を追加
2. `fav/src/main.rs` — `sla-check` サブコマンドのルーティングを追加
3. `fav/src/driver.rs` — `mod v99600_tests`（2 テスト）追加

## Syntax / API Examples

### SlaDefinition / SlaViolation（Rust）

```rust
/// SLA 定義（v99.6.0）
#[derive(Debug, Clone)]
pub struct SlaDefinition {
    pub endpoint: String,
    pub max_latency_ms: u32,
    pub availability: f64,   // 0.999 = 99.9%
}

/// SLA 違反（v99.6.0）
#[derive(Debug, Clone)]
pub struct SlaViolation {
    pub sla: SlaDefinition,
    pub actual_ms: u32,
    pub timestamp: String,
}
```

### cmd_sla_check（Rust）

```rust
/// `fav sla-check` コマンド（v99.6.0）
/// SLA 準拠チェックを実行し、違反レポートを返す。
/// v99.6.0 はスタブ実装。実際の SLA 測定は後続バージョンで実施。
pub fn cmd_sla_check(config: &str, from: &str, to: &str) -> String {
    format!(
        "SLA check: config={config}, from={from}, to={to}\nNo violations detected."
    )
}
```

### CLI 使用例

```bash
$ fav sla-check --config sla.toml --from 2026-08-01 --to 2026-08-31
SLA check: config=sla.toml, from=2026-08-01, to=2026-08-31
No violations detected.
```

### Favnir 型参照例（将来バージョン向けイメージ）

```favnir
-- SlaDefinition / SlaViolation は将来 runes/sap-odata/sla.fav で定義予定
-- v99.6.0 ではスタブ CLI コマンドのみを提供する
type SlaDefinition = {
    endpoint:       String,
    max_latency_ms: Int,
    availability:   Float    -- 0.999 = 99.9%
}

type SlaViolation = {
    sla:         SlaDefinition,
    actual_ms:   Int,
    timestamp:   String
}
```

## Success Criteria

- `fav/src/driver.rs` に `SlaDefinition` 構造体が定義されている
- `fav/src/driver.rs` に `SlaViolation` 構造体が定義されている
- `fav/src/driver.rs` に `cmd_sla_check` 関数が定義されている
- `fav/src/main.rs` に `sla-check` サブコマンドのルーティングが追加されている（目視確認）
- `CHANGELOG.md` に `[v99.6.0]` エントリが含まれる
- `cargo test -- --test-threads=1` が 4,269 tests, 0 failures で通過する

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `fav/src/driver.rs` | `SlaDefinition` / `SlaViolation` 構造体 + `cmd_sla_check` 関数 + `mod v99600_tests` 追記 |
| `fav/src/main.rs` | `sla-check` サブコマンドルーティング追加 |
| `CHANGELOG.md` | 追記 |
| `versions/current.md` | 更新 |

## テスト数について

ベースライン: v99.5.0 完了後の 4,267。v99.6.0 の目標は 4,267 + 2 = **4,269**。
