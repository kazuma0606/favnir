# v28.5.0 Spec — sentry Rune 追加

## 概要

エラートラッキングプラットフォーム Sentry の Rune を追加する。
`capture_error / capture_message / set_user / set_tag / set_extra` の 5 関数をスタブ実装し、
`!Io` エフェクトで型安全にエラーイベント・メッセージを送信できるようにする。

> **スコープ注記**: `#[on_error(report_to, level)]` アノテーション（ロードマップ v28.5 記載）は
> ast.rs / parser.rs / コンパイラへの複数ファイル変更が必要なため v28.9+ で実装予定。
> v28.5.0 では Rune 関数の公開のみを行う。

## ロードマップ参照

`versions/roadmap/roadmap-v28.1-v29.0.md` — v28.5 セクション

## 実装内容

### T1 — Cargo.toml バージョン bump
`28.4.0` → `28.5.0`

### T2 — VM primitive 追加（vm.rs）

以下の 5 primitive を `fav/src/backend/vm.rs` に追加（`#[cfg]` ガード付き）。
OTel primitives の直後に挿入。

| primitive | 引数 | 非 WASM 戻り値 | WASM 戻り値 |
|---|---|---|---|
| `Sentry.capture_error_raw` | (err: String) | `ok_vm(Unit)` | `err_vm("Sentry not supported on wasm32")` |
| `Sentry.capture_message_raw` | (level: String, msg: String) | `ok_vm(Unit)` | `err_vm("Sentry not supported on wasm32")` |
| `Sentry.set_user_raw` | (id: String, email: String) | `ok_vm(Unit)` | `err_vm("Sentry not supported on wasm32")` |
| `Sentry.set_tag_raw` | (key: String, value: String) | `ok_vm(Unit)` | `err_vm("Sentry not supported on wasm32")` |
| `Sentry.set_extra_raw` | (key: String, value: String) | `ok_vm(Unit)` | `err_vm("Sentry not supported on wasm32")` |

> `vm_has_sentry_capture_error_raw` テスト 1 件で 5 primitive の実装を代表確認する。

### T3 — checker.fav 更新（Phase 9a）

`fav/self/checker.fav` の `ns_to_effect` に `"Sentry" => "IO"` を追加。
OTel else ブロックの内側（最も深いネスト）に追加。

> **重要**: v28.1.0〜v28.3.0 の教訓から `"IO"`（全大文字）を使用すること。

### T4 — Rune ファイル作成（runes/sentry/sentry.fav）

```favnir
// runes/sentry/sentry.fav — Sentry エラートラッキング Rune (v28.5.0)
// エラーイベント・メッセージを Sentry API 経由で送信する。
// v28.5.0 stub — 実際の Sentry SDK 送信は v28.x 以降
public fn capture_error(err: String) -> Result<Unit, String> !Io {
    Sentry.capture_error_raw(err)
}
public fn capture_message(level: String, msg: String) -> Result<Unit, String> !Io {
    Sentry.capture_message_raw(level, msg)
}
public fn set_user(id: String, email: String) -> Result<Unit, String> !Io {
    Sentry.set_user_raw(id, email)
}
public fn set_tag(key: String, value: String) -> Result<Unit, String> !Io {
    Sentry.set_tag_raw(key, value)
}
public fn set_extra(key: String, value: String) -> Result<Unit, String> !Io {
    Sentry.set_extra_raw(key, value)
}
```

### T5 — example ファイル作成

`examples/observability/sentry_alerting.fav` — エラーレポートデモ:
- `stage ReportError: Unit -> Result<Unit, String> !Io` — エラーキャプチャ + タグ設定
- `stage SetContext: Unit -> Result<Unit, String> !Io` — ユーザー情報 + 追加情報設定
- `seq SentryAlertingDemo = ReportError |> SetContext`

### T6 — サイトドキュメント
`site/content/docs/runes/sentry.mdx` 新規作成。

### T7 — CHANGELOG 更新
`CHANGELOG.md` に `[v28.5.0]` セクション追加。

### T8 — ベンチマーク
`benchmarks/v28.5.0.json` 新規作成（test_count: 2271）。

### T9 — driver.rs テスト（Phase 9b）
`v285000_tests` モジュール（9 件）を `driver.rs` に追加。

### T10 — テスト全通過確認
`cargo test --bin fav` で 2271 tests PASS。

## エフェクト設計

| Rune 関数 | エフェクト | 理由 |
|---|---|---|
| capture_error / capture_message / set_user / set_tag / set_extra | `!Io` | Sentry API への HTTP 送信（ネットワーク I/O） |

## テスト数

- v28.4.0: 2262 tests
- v28.5.0: **2272 tests**（+10、`set_extra_fn` テストを追加して 5 関数すべてを個別確認）

## 完了条件

- [ ] `Cargo.toml` version = "28.5.0"
- [ ] `runes/sentry/sentry.fav` 存在（5 関数、`!Io` エフェクト）
- [ ] `Sentry.*_raw` 5 VM primitive 存在（`#[cfg]` ガード付き）
- [ ] `fav/self/checker.fav` `ns_to_effect` に `ns == "Sentry"` → `"IO"` あり
- [ ] `examples/observability/sentry_alerting.fav` に `SentryAlertingDemo` seq あり
- [ ] `site/content/docs/runes/sentry.mdx` 存在
- [ ] `CHANGELOG.md` に `[v28.5.0]` セクションあり
- [ ] `benchmarks/v28.5.0.json` 存在（test_count: 2271）
- [ ] `cargo test --bin fav v285000` — 10/10 PASS
- [ ] `cargo test --bin fav sentry` — 8 件以上 PASS（`v285000_tests` のうち `sentry` を名前に含む 8 件がマッチ。`checker_has_sentry_effect` と `changelog_has_v28_5_0` の 2 件は `sentry` フィルタでは**マッチしない**——`v285000` フィルタでのみヒット）
- [ ] `cargo test --bin fav` — 2272 tests PASS
