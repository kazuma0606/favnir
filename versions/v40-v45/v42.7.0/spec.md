# v42.7.0 仕様書 — `fav monitor`

## 概要

実行中パイプラインのスループット・イベント数・レイテンシをターミナルに表示する `fav monitor` コマンドを追加する。
v42.7.0 では `cmd_monitor` 関数を `main.rs` に追加し、stub 出力（メトリクス収集は v43.x 以降）を実装する。
`driver.rs` に `v42700_tests` 2 件を追加し、Cargo.toml を `42.7.0` にバンプする。

---

## 背景・動機

v42.1〜v42.6 で CEP・Stream join・back-pressure・WebSocket Rune を整備した。
リアルタイムパイプラインの運用フェーズで必要となる「パイプライン監視」機能の入口として `fav monitor` を追加する。
v42.7.0 では CLI エントリポイントとコマンドルーティングを整備し、実際のメトリクス収集は v43.x 以降で実装する。

---

## 実装スコープ

### 1. `main.rs` — `"monitor"` アーム追加

`Some("watch")` アームの直後（または `Some("profile")` の近傍）に追加:

```rust
Some("monitor") => cmd_monitor(&args),
```

### 2. `main.rs` — `cmd_monitor` 関数追加

```rust
/// v42.7.0: fav monitor — パイプライン監視（stub）
/// 実際のメトリクス収集は v43.x 以降で実装。
/// 引数はすべて無視する（未知引数もエラーにしない）。v43.x で --interval オプション追加時に引数解析を実装する。
fn cmd_monitor(_args: &[String]) {
    println!("fav monitor — pipeline metrics (stub)");
    println!("Throughput, event count, and latency monitoring will be available in v43.x.");
}
```

### 3. `driver.rs` — `v42700_tests` 追加（2 テスト）

```rust
// -- v42700_tests (v42.7.0) -- fav monitor --
mod v42700_tests {
    fn cargo_toml_version_is_42_7_0()
    fn monitor_cmd_exists()           // main.rs に "monitor" と "cmd_monitor" が含まれることを確認
}
```

`monitor_cmd_exists`:
- `include_str!("../src/main.rs")` が `"cmd_monitor(&args)"` を含む（ルーティングアームとして機能していることを確認）
- `include_str!("../src/main.rs")` が `"cmd_monitor"` を含む

---

## テスト計画

| テスト名 | 内容 |
|---|---|
| `cargo_toml_version_is_42_7_0` | Cargo.toml に "42.7.0" が含まれる |
| `monitor_cmd_exists` | `main.rs` が `"monitor"` と `"cmd_monitor"` を含む |

**推定テスト数**: 2891 + 2 = **2893**
ロードマップ記載の 2892 は旧 v42.6.0 基準（2890+2）の誤差。v42.6.0 実績が 2891 のため 2893 が正しい推定値。ロードマップは実装完了後に実績値で修正する。

---

## 影響範囲

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/main.rs` | 変更 | `Some("monitor")` アーム追加、`cmd_monitor` 関数追加 |
| `fav/src/driver.rs` | 変更 | `v42700_tests` 2 件追加 |
| `fav/Cargo.toml` | 変更 | version `42.6.0` → `42.7.0` |
| `CHANGELOG.md` | 変更 | `[v42.7.0]` エントリ追加 |
| `versions/current.md` | 変更 | 最新安定版 v42.7.0・次版 v42.8.0 に更新 |
| `versions/roadmap/roadmap-v42.1-v43.0.md` | 変更 | v42.7.0 を完了済みにマーク、推定テスト数を 2892 → 実績 2893 に修正 |

---

## 非スコープ

- 実際のパイプラインメトリクス収集（スループット・イベント数・レイテンシの実測）— v43.x 以降
- ターミナル UI のリアルタイム更新（TUI ライブラリ利用等）— v43.x 以降
- `fav monitor --interval <seconds>` オプション — v43.x 以降（stub では引数を全無視）
- `!Monitor` エフェクト追加 — v43.x 以降
- Prometheus/Datadog 等への外部メトリクス送信 — v43.x 以降
- `site/content/docs/tools/monitor.mdx` 新規作成 — v43.x 以降（stub 段階のため省略）
