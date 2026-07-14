# v42.9.0 仕様書 — v43.0 前調整・安定化

## 概要

v43.0.0「Real-Time Power 宣言」に向けた最終準備バージョン。
**コードフリーズ（新規機能追加なし）**。
`site/content/docs/real-time-power.mdx` を新規作成し、v42.1〜v42.8 で整備したリアルタイム機能の全体像をドキュメント化する。
driver.rs に meta テスト 2 件を追加し、Cargo.toml を `42.9.0` にバンプする。

---

## 背景・動機

v42.1〜v42.8 でリアルタイム機能（CEP / Stream join / back-pressure / WebSocket Rune / fav monitor / cookbook）を整備した。
v43.0.0 の宣言前にこれらの機能を俯瞰するドキュメントを作成し、新規ユーザーが「Favnir でリアルタイムパイプラインを実装するには何を使えばいいか」を一覧できるようにする。
新規コード変更はなく、ドキュメント追加 + バージョン bump + テスト追加のみ。

---

## 実装スコープ

### 1. `site/content/docs/real-time-power.mdx`

v42.x のリアルタイム機能を一覧する概要ドキュメント。各機能へのリンクと最小コード例を含む。

内容:
- タイトル: "Real-Time Power — Favnir v42.x リアルタイム機能概要"
- **CEP**（v42.1.0）: `Cep.window` / `Cep.match` — イベントパターン検出
- **Stream join**（v42.4.0）: `Stream.join` — 時間窓 2 ストリーム結合
- **Back-pressure**（v42.5.0）: `#[max_inflight(n)]` — 宣言的 back-pressure（parser + AST のみ、runtime は v44.x）
- **WebSocket Rune**（v42.6.0）: `WebSocket.send` / `WebSocket.broadcast` — リアルタイム push sink（stub、実接続は v44.x）
- **fav monitor**（v42.7.0）: パイプライン監視コマンド stub（実測は v43.x）
- cookbook へのリンク: `cep-login-purchase.mdx` / `stream-join.mdx`

### 2. `driver.rs` — `v42900_tests` 追加（2 テスト）

```rust
// -- v42900_tests (v42.9.0) -- v43.0 前調整・安定化 --
mod v42900_tests {
    fn cargo_toml_version_is_42_9_0()
    fn real_time_power_docs_exists()    // real-time-power.mdx の内容を確認
}
```

`real_time_power_docs_exists`:
- `include_str!("../../site/content/docs/real-time-power.mdx")` が `"Stream.join"` を含む
- `include_str!("../../site/content/docs/real-time-power.mdx")` が `"Cep.window"` を含む
- `include_str!("../../site/content/docs/real-time-power.mdx")` が `"WebSocket"` を含む

---

## テスト計画

| テスト名 | 内容 |
|---|---|
| `cargo_toml_version_is_42_9_0` | Cargo.toml に "42.9.0" が含まれる |
| `real_time_power_docs_exists` | `real-time-power.mdx` が `"Stream.join"` と `"Cep"` を含む |

**推定テスト数**: 2894 + 2 = **2896**
ロードマップ記載の 2895 は旧 v42.8.0 基準（2893+2）の誤差。v42.8.0 実績が 2894 のため 2896 が正しい推定値。

---

## 影響範囲

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `site/content/docs/real-time-power.mdx` | 新規 | v42.x リアルタイム機能概要ドキュメント |
| `fav/src/driver.rs` | 変更 | `v42900_tests` 2 件追加 |
| `fav/Cargo.toml` | 変更 | version `42.8.0` → `42.9.0` |
| `CHANGELOG.md` | 変更 | `[v42.9.0]` エントリ追加 |
| `versions/current.md` | 変更 | 最新安定版 v42.9.0・次版 v43.0.0 に更新 |
| `versions/roadmap/roadmap-v42.1-v43.0.md` | 変更 | v42.9.0 を完了済みにマーク、推定テスト数を 2895 → 実績 2896 に修正 |
| `versions/v40-v45/v42.9.0/tasks.md` | 変更 | COMPLETE ステータスに更新 |

---

## 非スコープ

- 新規言語機能・VM 機能の追加（コードフリーズ）
- `fav monitor` の実際のメトリクス収集実装 — v43.x 以降
- `#[max_inflight]` runtime back-pressure — v44.x 以降
- WebSocket 実接続 — v44.x 以降
- v43.0.0 マイルストーン宣言（`MILESTONE.md` 更新 / `README.md` 更新）— v43.0.0 で実施
