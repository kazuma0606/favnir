# v75.6.0 仕様書 — Stream freshness monitoring

Date: 2026-08-15
Status: 計画中

---

## Background

Kafka / Kinesis などのストリーミングシステムでは、最後のイベントから一定時間以上経過した場合にパイプラインが止まっている（またはプロデューサーが沈黙している）と判断できる。この「ストリームの遅延（lag）」を型で表現し、テスト可能な形で監視する基盤を v75.6.0 で追加する。

v75.1.0 の `FreshnessPolicy`（バッチデータ鮮度）と対をなす、ストリーム専用の鮮度監視型。

---

## Goals

1. `StreamFreshnessMonitor` 構造体（source: String, max_lag_secs: u64）を追加する
2. `StreamLagResult` 構造体（lag_secs: u64, exceeded: bool, source: String）を追加する
3. `check_stream_lag(last_event_ts: i64, now: i64, monitor: &StreamFreshnessMonitor) -> StreamLagResult` を追加する
4. `format_stream_lag_report(result: &StreamLagResult) -> String` を追加する
5. Rust テスト 2 件を追加し 3704 tests に到達する

---

## 型・関数仕様

### `StreamFreshnessMonitor` 構造体

```rust
#[derive(Debug, Clone)]
pub struct StreamFreshnessMonitor {
    pub source:       String,  // ストリームソース識別子（例: "kafka://orders-topic"）
    pub max_lag_secs: u64,     // 許容最大遅延（秒）
}
```

---

### `StreamLagResult` 構造体

```rust
#[derive(Debug, Clone)]
pub struct StreamLagResult {
    pub lag_secs: u64,    // 実際の遅延（秒）。未来タイムスタンプの場合は 0
    pub exceeded: bool,   // lag_secs > max_lag_secs のとき true
    pub source:   String, // StreamFreshnessMonitor.source のコピー
}
```

---

### `check_stream_lag`

```rust
pub fn check_stream_lag(
    last_event_ts: i64,
    now:           i64,
    monitor:       &StreamFreshnessMonitor,
) -> StreamLagResult
```

**判定ロジック:**
- `lag_secs = max(now - last_event_ts, 0) as u64`
  — `now < last_event_ts`（未来タイムスタンプ）は lag = 0 として扱う
  — `saturating_sub` + `.max(0)` で安全な計算（v75.5.0 の `apply_retention_check` と同方針）
- `exceeded = lag_secs > monitor.max_lag_secs`（開区間。ちょうど max_lag_secs 秒は exceeded=false）

---

### `format_stream_lag_report`

```rust
pub fn format_stream_lag_report(result: &StreamLagResult) -> String
```

**出力フォーマット:**
- 正常時: `"[OK] source={source} lag={lag_secs}s"`
- 超過時: `"[EXCEEDED] source={source} lag={lag_secs}s"`

---

## Favnir コード例

```favnir
// ストリーム遅延監視
fn monitor_stream(ctx: AppCtx) -> Result<Unit, String> {
    bind monitor <- StreamFreshnessMonitor {
        source: "kafka://orders-topic",
        max_lag_secs: 30
    }
    bind lag <- check_stream_lag(last_event_ts, ctx.now_secs(), monitor)
    if lag.exceeded {
        ctx.io.println(format_stream_lag_report(lag))
    }
    Result.ok(Unit)
}
```

---

## Success Criteria

- `StreamFreshnessMonitor` 構造体が定義されている（source: String, max_lag_secs: u64）
- `StreamLagResult` 構造体が定義されている（lag_secs: u64, exceeded: bool, source: String）
- `check_stream_lag` が正しい `StreamLagResult` を返す
- `format_stream_lag_report` が `[OK]` / `[EXCEEDED]` フォーマットを生成する
- `cargo test` が 3704 tests all pass
- `CHANGELOG.md` の先頭に v75.6.0 エントリが存在する

---

## テスト仕様

### `stream_lag_within_threshold`

- `monitor = StreamFreshnessMonitor { source: "kafka://orders-topic".to_string(), max_lag_secs: 30 }`
- `last_event_ts=100, now=120`（lag=20 秒）→ `lag_secs=20, exceeded=false`
- `last_event_ts=100, now=130`（lag=30 秒 = ちょうど境界）→ `exceeded=false`（開区間）
- `format_stream_lag_report` の結果が `"[OK]"` を含む

### `stream_lag_exceeded_detected`

- `monitor = StreamFreshnessMonitor { source: "kafka://orders-topic".to_string(), max_lag_secs: 30 }`
- `last_event_ts=100, now=131`（lag=31 秒）→ `lag_secs=31, exceeded=true`
- `format_stream_lag_report` の結果が `"[EXCEEDED]"` を含む
- `last_event_ts=100, now=99`（未来タイムスタンプ）→ `lag_secs=0, exceeded=false`

---

## 変更ファイル

- `fav/src/driver.rs` — `StreamFreshnessMonitor`, `StreamLagResult`, `check_stream_lag`, `format_stream_lag_report`, `v756000_tests` を追加
- `CHANGELOG.md` — v75.6.0 エントリを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `75.5.0` → `75.6.0` に更新

---

## 対象外

- 実際の Kafka / Kinesis API 呼び出し（Rune での実装は将来バージョン）
- `source` フィールドの URL バリデーション（呼び出し側の責任）
- `lag_secs` の精度（秒単位のみ、ミリ秒は将来拡張）
