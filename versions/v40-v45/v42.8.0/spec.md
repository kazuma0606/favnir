# v42.8.0 仕様書 — Real-Time Power cookbook

## 概要

v42.1〜v42.7 で整備したリアルタイム機能（CEP / Stream join / back-pressure / WebSocket / fav monitor）の使用例として、
`site/content/cookbook/` に 2 件の MDX cookbook 記事を追加する。

---

## 背景・動機

v42.x スプリントで整備した機能を実際のユースケースで示すドキュメントが不足している。
v42.8.0 では CEP を使ったログイン〜購入検出パイプラインと、Stream.join を使った 2 ストリーム結合のサンプルを提供し、
v43.0.0 の「Real-Time Power 宣言」に向けた実用例コレクションを整備する。

---

## 実装スコープ

### 1. `site/content/cookbook/cep-login-purchase.mdx`

CEP Rune（v42.1.0 で追加）を使い、「ログインイベントの直後に購入イベントが発生した」セッションを検出するパイプライン例。

内容:
- タイトル: "CEP: ログイン→購入セッション検出"
- `Cep.window` で 5 分ウィンドウを設定
- `Cep.match` で login イベントの後に purchase イベントが続くパターンを定義
- `WebSocket.send` で検出結果を push（v42.6.0 WebSocket Rune 使用例）

### 2. `site/content/cookbook/stream-join.mdx`

`Stream.join`（v42.4.0 で追加）を使い、ユーザーイベントストリームと商品ストリームを time-window で結合するパイプライン例。

内容:
- タイトル: "Stream join: 2 ストリームの時間窓結合"
- `Stream.from` でそれぞれのストリームを定義
- `Stream.join(users, products, |u, p| u.product_id == p.id, 60)` で 60 秒窓で結合
- `Stream.to_list` で結果を取得し処理

### 3. `driver.rs` — `v42800_tests` 追加（1 テスト）

```rust
// -- v42800_tests (v42.8.0) -- Real-Time Power cookbook --
mod v42800_tests {
    fn realtime_cookbook_mdx_exists()
}
```

`realtime_cookbook_mdx_exists`:
- `include_str!("../../site/content/cookbook/cep-login-purchase.mdx")` が `"Cep.window"` を含む
- `include_str!("../../site/content/cookbook/stream-join.mdx")` が `"Stream.join"` を含む

---

## テスト計画

| テスト名 | 内容 |
|---|---|
| `realtime_cookbook_mdx_exists` | `cep-login-purchase.mdx` が `"Cep"` を含み、`stream-join.mdx` が `"Stream.join"` を含む |

**推定テスト数**: 2893 + 1 = **2894**
ロードマップ記載の 2893 は旧 v42.7.0 基準（2892+1）の誤差。v42.7.0 実績が 2893 のため 2894 が正しい推定値。

---

## 影響範囲

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `site/content/cookbook/cep-login-purchase.mdx` | 新規 | CEP ログイン→購入検出 cookbook |
| `site/content/cookbook/stream-join.mdx` | 新規 | Stream join 2 ストリーム結合 cookbook |
| `fav/src/driver.rs` | 変更 | `v42800_tests` 1 件追加 |
| `fav/Cargo.toml` | 変更 | version `42.7.0` → `42.8.0` |
| `CHANGELOG.md` | 変更 | `[v42.8.0]` エントリ追加 |
| `versions/current.md` | 変更 | 最新安定版 v42.8.0・次版 v42.9.0 に更新 |
| `versions/roadmap/roadmap-v42.1-v43.0.md` | 変更 | v42.8.0 を完了済みにマーク、推定テスト数を 2893 → 実績 2894 に修正 |

---

## 非スコープ

- `cep-login-purchase.mdx` で使用する Cep Rune の新機能追加 — v42.x 完了済みの範囲内で記述
- `stream-join.mdx` で使用する Stream.join の新機能追加 — v42.4.0 実装済みの範囲内で記述
- フロントエンドでの cookbook ページナビゲーション更新 — v43.0.0 以降
- `fav monitor` の使用例 cookbook — v43.x（monitor 実装完了後）
- `#[max_inflight]` back-pressure の cookbook — v44.x（runtime 実装完了後）
