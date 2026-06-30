# v29.6.0 Spec — pagerduty Rune 追加

**バージョン**: 29.6.0
**日付**: 2026-06-30
**フェーズ**: Ecosystem Maturity (phase 6)
**前バージョン**: v29.5.0 (github Rune 追加)

---

## 概要

インシデント管理ツール PagerDuty を Favnir パイプラインに統合する。
`#[on_error]` アノテーションと組み合わせることで、クリティカルな `stage` の失敗を
自動でエスカレーションできるようになる。

> **ポジショニング**: GitHub Rune（v29.5）でデータ品質を DevOps に接続した次のステップ。
> Favnir パイプラインの障害対応（インシデント通知 → 確認 → 解決）を一貫して自動化する。
> `fav run alert.fav` でクリティカル障害が PagerDuty に即時エスカレーション。

---

## 対象コンポーネント

| コンポーネント | 内容 |
|---|---|
| `runes/pagerduty/pagerduty.fav` | PagerDuty Rune 実装（4 関数）|
| `runes/pagerduty/rune.toml` | Rune メタデータ |
| `fav/src/driver.rs` | `v296000_tests` 6 件追加 |
| `fav/Cargo.toml` | version 29.5.0 → 29.6.0 |
| `CHANGELOG.md` | `[v29.6.0]` セクション追加 |
| `benchmarks/v29.6.0.json` | ベンチマーク記録 |
| `site/content/docs/runes/pagerduty.mdx` | PagerDuty Rune ドキュメント |

---

## PagerDuty Rune API

### 実装関数

| 関数 | シグネチャ | 内容 |
|---|---|---|
| `PagerDuty.create_incident` | `(config: String, title: String, severity: String, key: String) -> Result<String, String> !Http` | インシデント作成（incident_key を返す）|
| `PagerDuty.resolve` | `(config: String, incident_key: String) -> Result<Unit, String> !Http` | インシデント解決 |
| `PagerDuty.acknowledge` | `(config: String, incident_key: String) -> Result<Unit, String> !Http` | インシデント確認（対応中マーク）|
| `PagerDuty.add_note` | `(config: String, incident_key: String, note: String) -> Result<Unit, String> !Http` | インシデントにノート追加 |

### 設定

| 環境変数 | 説明 |
|---|---|
| `PAGERDUTY_ROUTING_KEY` | PagerDuty Events API v2 のルーティングキー（必須。HTTP 有効化後にリクエストに付与予定）|
| `PAGERDUTY_BASE_URL` | API ベース URL（デフォルト: `https://events.pagerduty.com`）|

### 使用例

```favnir
import runes/pagerduty

// クリティカルな stage が失敗したら PagerDuty アラートを作成
stage AlertOnFailure: String -> Unit !Http = |error_msg| {
  bind _ <- PagerDuty.create_incident(
    config.pagerduty,
    "[CRITICAL] Pipeline Failure: " ++ error_msg,
    "critical",
    "pipeline-failure-" ++ Gen.uuid()
  )
  Result.ok(unit)
}

// インシデント解決
stage ResolveIncident: String -> Unit !Http = |incident_key| {
  PagerDuty.resolve(config.pagerduty, incident_key)
}
```

---

## テスト戦略

### v296000_tests（6 件）

| テスト名 | 検証内容 |
|---|---|
| `pagerduty_rune_file_exists` | `runes/pagerduty/pagerduty.fav` が存在し `PagerDuty.create_incident` を含む |
| `pagerduty_resolve_fn_exists` | `pagerduty.fav` に `PagerDuty.resolve` が存在する |
| `pagerduty_acknowledge_fn_exists` | `pagerduty.fav` に `PagerDuty.acknowledge` が存在する |
| `pagerduty_add_note_fn_exists` | `pagerduty.fav` に `PagerDuty.add_note` が存在する |
| `pagerduty_rune_toml_exists` | `runes/pagerduty/rune.toml` が存在し `pagerduty` を含む |
| `changelog_has_v29_6_0` | `CHANGELOG.md` に `[v29.6.0]` が存在する |

検証関数カバレッジ: `create_incident`, `resolve`, `acknowledge`, `add_note`（4/4 関数 = 100%）

テスト数: 2342 → **2348**（+6）

---

## 完了条件

- [ ] `Cargo.toml` version = "29.6.0"
- [ ] `runes/pagerduty/pagerduty.fav` に 4 関数が実装されている
- [ ] `runes/pagerduty/rune.toml` が存在する（`[rune]` セクションのみ）
- [ ] `CHANGELOG.md` に `[v29.6.0]` セクションあり
- [ ] `benchmarks/v29.6.0.json` 存在（test_count: 2348）
- [ ] `site/content/docs/runes/pagerduty.mdx` 存在
- [ ] `cargo test --bin fav v296000` — 6/6 PASS
- [ ] `cargo test --bin fav` — 2348 tests PASS

---

## 実装上の制約

### `PagerDuty.add_note` について

PagerDuty のノート追加は本来 REST API（`POST /incidents/{id}/notes`）が必要。
ただし REST API は incident ID（UUID）を要求するが、Events API v2 で返るのは `dedup_key` のみ。

現実装では Events API v2（`/v2/enqueue`）に `event_action: "trigger"` + note 内容を `summary` に埋め込む
プレースホルダーとして動作する。`Http.patch` primitive 実装後に REST API へ移行予定。

## スコープ外

- PagerDuty Events API v2 への実際の HTTP 接続 — インフラ稼働後に有効化
- `#[on_error]` アノテーションとの自動統合 — Favnir のアノテーション評価機構拡張が必要
- PagerDuty REST API（インシデント一覧・ユーザー管理等）— Events API v2 のみ実装
- `add_note` の本実装（REST API `/incidents/{id}/notes`）— `Http.patch` 実装後に移行
- Webhook 受信（PagerDuty → Favnir）— v30.x+ で対応
