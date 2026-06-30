# v28.6.0 Spec — grafana Rune 追加

## 概要

Grafana HTTP API を通じてアノテーション作成・ダッシュボード更新・スナップショット生成を行う
`runes/grafana` Rune を追加する。

すべての操作は **スタブ実装**（stdout へのログ出力のみ）。
実際の Grafana HTTP リクエストは v28.7+ の E2E デモフェーズで実装予定。

エフェクトは `!Io`（HTTP 送信）。`checker.fav` の `ns_to_effect` に `"Grafana"` → `"IO"` を追加する。

---

## VM Primitives（vm.rs に追加）

| Primitive | パラメータ | 説明 |
|---|---|---|
| `Grafana.create_annotation_raw` | `dashboard_id: String, text: String, tags: String` | アノテーション作成（デプロイ記録等）|
| `Grafana.push_dashboard_raw` | `json: String` | ダッシュボード定義の更新 |
| `Grafana.snapshot_raw` | `dashboard_id: String` | スナップショット作成（共有 URL 生成）|

すべて `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` の両アームを実装。
wasm32 アームは `Result.err("Grafana not supported on wasm32")` を返す。

---

## Favnir Rune（runes/grafana/grafana.fav）

```favnir
public fn create_annotation(dashboard_id: String, text: String, tags: String) -> Result<Unit, String> !Io =
    Grafana.create_annotation_raw(dashboard_id, text, tags)

public fn push_dashboard(json: String) -> Result<Unit, String> !Io =
    Grafana.push_dashboard_raw(json)

public fn snapshot(dashboard_id: String) -> Result<String, String> !Io =
    Grafana.snapshot_raw(dashboard_id)
```

`snapshot` のみ戻り値が `Result<String, String>`（スナップショット URL を返す）。

---

## checker.fav 更新（ns_to_effect）

v28.5.0 時点の末尾:

```favnir
if ns == "Sentry" {
    "IO"
} else {
    ""
}
```

v28.6.0 後の末尾（Sentry else ブロック内に Grafana を追加）:

```favnir
if ns == "Sentry" {
    "IO"
} else {
    if ns == "Grafana" {
        "IO"
    } else {
        ""
    }
}
```

---

## 追加ファイル

| ファイル | 内容 |
|---|---|
| `runes/grafana/grafana.fav` | 3 関数 Rune（`!Io` エフェクト）|
| `examples/observability/grafana_dashboard.fav` | `seq GrafanaDashboardDemo = RecordDeploy \|> UpdateDashboard` |
| `site/content/docs/runes/grafana.mdx` | API リファレンスドキュメント |
| `benchmarks/v28.6.0.json` | `{"version":"28.6.0","test_count":2281}` |
| `CHANGELOG.md` | `[v28.6.0]` セクション追加 |

---

## スタブ制約

- Grafana HTTP API（`/api/annotations`, `/api/dashboards/db`, `/api/snapshots`）への実際のリクエストは v28.7 以降
- API キー（`Authorization: Bearer <key>`）の設定は v28.7 で `fav.toml` の `[grafana]` セクションに追加予定
- `snapshot` が返す URL はスタブでは `"https://grafana.example.com/dashboard/snapshot/stub"` の固定文字列
- wasm32 ターゲットでは `Result.err("Grafana not supported on wasm32")` を返す

---

## テスト一覧（driver.rs v286000_tests）

| # | テスト名 | 確認内容 |
|---|---|---|
| 1 | `grafana_rune_has_create_annotation_fn` | `runes/grafana/grafana.fav` に `fn create_annotation(` を含む |
| 2 | `grafana_rune_has_push_dashboard_fn` | `runes/grafana/grafana.fav` に `fn push_dashboard(` を含む |
| 3 | `grafana_rune_has_snapshot_fn` | `runes/grafana/grafana.fav` に `fn snapshot(` を含む |
| 4 | `grafana_rune_uses_io_effect` | `runes/grafana/grafana.fav` に `!Io` を含む |
| 5 | `vm_has_grafana_create_annotation_raw` | `backend/vm.rs` に `Grafana.create_annotation_raw` を含む |
| 6 | `grafana_example_has_pipeline` | `examples/observability/grafana_dashboard.fav` に `GrafanaDashboardDemo` を含む |
| 7 | `checker_has_grafana_effect` | `fav/self/checker.fav` に `ns == "Grafana"` と `"IO"` を含む（AND 条件） |
| 8 | `changelog_has_v28_6_0` | `CHANGELOG.md` に `[v28.6.0]` または `## v28.6.0` を含む |
| 9 | `grafana_doc_exists` | `site/content/docs/runes/grafana.mdx` に `Grafana` を含む |

合計 9 テスト。`cargo test grafana` で 8 件以上 PASS（`changelog_has_v28_6_0` は "grafana" を含まないためスキップ）。
test_count: **2281**（2272 + 9）

---

## 完了条件チェックリスト

- [ ] `Cargo.toml` version = `28.6.0`
- [ ] `runes/grafana/grafana.fav` 存在（3 関数、`!Io` エフェクト）
- [ ] `Grafana.*_raw` 3 VM primitive 存在（`#[cfg]` ガード付き）
- [ ] `fav/self/checker.fav` `ns_to_effect` に `ns == "Grafana"` → `"IO"` あり
- [ ] `examples/observability/grafana_dashboard.fav` に `GrafanaDashboardDemo` seq あり
- [ ] `site/content/docs/runes/grafana.mdx` 存在
- [ ] `CHANGELOG.md` に `[v28.6.0]` セクションあり
- [ ] `benchmarks/v28.6.0.json` 存在（test_count: 2281）
- [ ] `cargo test --bin fav v286000` — 9/9 PASS
- [ ] `cargo test --bin fav grafana` — 8 件以上 PASS
- [ ] `cargo test --bin fav` — 2281 tests PASS
