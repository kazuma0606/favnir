# Spec: v52.5.0 — SLA 監視 Rune

Status: PLANNED
Date: 2026-07-21

---

## 目的

v52.4.0 でリネージ HTML レポートを実装した。
v52.5.0 では `runes/sla/sla.fav` を新規作成し、
`check_freshness` / `check_latency` / `alert` の 3 関数を提供する SLA 監視 Rune を追加する。

SLA 違反時は `Err` を返し、呼び出し側が `bind _ <- sla.check_freshness(...)` パターンで
fail-fast パイプラインを構築できるようにする。

外部アラート基盤（Prometheus / Datadog 等）との連携は `alert` 関数が担う（v52.5.0 はスタブ実装）。

---

## 使用例

```favnir
import sla

stage CheckFreshness: DataBatch -> Result<DataBatch> = |batch| {
  bind _ <- sla.check_freshness(batch.timestamp, max_age_seconds: 3600)
  bind _ <- sla.check_latency(stage: "Parse", threshold_ms: 200)
  Ok(batch)
}
```

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `runes/sla/sla.fav` | 新規作成（`check_freshness` / `check_latency` / `alert`） |
| `fav/src/driver.rs` | `v52500_tests` モジュール追加（2 件） |
| `fav/Cargo.toml` | version → `"52.5.0"` |
| `CHANGELOG.md` | v52.5.0 エントリ追加 |
| `versions/current.md` | v52.5.0（3146 tests）に更新 |
| `versions/roadmap/roadmap-v52.1-v53.0.md` | v52.5.0 実績欄を更新 |

---

## 詳細仕様

### 1. `runes/sla/sla.fav`

作成場所: `/c/Users/yoshi/favnir/runes/sla/sla.fav`

既存 Rune（`prometheus.fav` / `datadog.fav`）と同じスタブパターンを使用。
各関数は `Sla.*_raw` primitive を呼び出す。

```favnir
// runes/sla/sla.fav — SLA 監視 Rune (v52.5.0)
//
// 使い方:
//   import sla
//
// SLA 違反時は Err を返す（呼び出し側が bind で fail-fast を実現）。
// アラート発火は !Observe エフェクト経由（v52.5.0 はスタブ実装）。

// データの鮮度を確認する。
// timestamp（Unix 秒）が現在時刻 - max_age_seconds より古い場合 Err を返す。
public fn check_freshness(timestamp: Int, max_age_seconds: Int) -> Result<Unit, String> {
    Sla.check_freshness_raw(timestamp, max_age_seconds)
}

// ステージの実行レイテンシが threshold_ms ミリ秒以内か確認する。
// 超過した場合は Err を返す。
public fn check_latency(stage: String, threshold_ms: Int) -> Result<Unit, String> {
    Sla.check_latency_raw(stage, threshold_ms)
}

// SLA 違反アラートを送信する。
// 外部アラート基盤（Prometheus / Datadog 等）へ転送するスタブ（v52.5.0）。
public fn alert(message: String) -> Result<Unit, String> {
    Sla.alert_raw(message)
}
```

**設計方針**:
- 既存 Rune（`prometheus.fav` / `kafka.fav`）と同じ `Namespace.fn_raw` スタブパターン
- `!Observe` エフェクトはコメントで言及（Effect enum への追加は将来バージョン）
- `check_freshness` の `timestamp` / `max_age_seconds` はキーワード引数（named parameter）も可
- `check_latency` の `stage` / `threshold_ms` はキーワード引数（named parameter）も可

### 2. テスト（2 件）

追加先: `driver.rs` の `v52500_tests` モジュール（`v52400_tests` の直前）

#### `sla_rune_latency_check`

```rust
#[test]
fn sla_rune_latency_check() {
    let src = include_str!("../../runes/sla/sla.fav");
    assert!(src.contains("fn check_latency("), "sla rune must define fn check_latency(");
    assert!(src.contains("threshold_ms"), "check_latency must have threshold_ms parameter");
}
```

#### `sla_rune_freshness_check`

```rust
#[test]
fn sla_rune_freshness_check() {
    let src = include_str!("../../runes/sla/sla.fav");
    assert!(src.contains("fn check_freshness("), "sla rune must define fn check_freshness(");
    assert!(src.contains("max_age_seconds"), "check_freshness must have max_age_seconds parameter");
}
```

---

## テスト数

- ベース: **3144** tests（v52.4.0 完了時点）
- `v52400_tests` に version テストなし → 削除 0 件
- 追加: `v52500_tests` 2 件（`sla_rune_latency_check` + `sla_rune_freshness_check`）
- **合計: 3146 tests**

---

## 完了条件

- `cargo test` 3146 passed, 0 failed
- `cargo clippy -- -D warnings` クリーン
- `runes/sla/sla.fav` が存在し `check_freshness` / `check_latency` / `alert` を定義している
  - `alert` 関数はテストで自動確認しないため、目視で存在を確認すること
- `include_str!("../../runes/sla/sla.fav")` がコンパイル時に解決できる（パス確認）
- `v52500_tests` 2 件（`sla_rune_latency_check` + `sla_rune_freshness_check`）が pass する
