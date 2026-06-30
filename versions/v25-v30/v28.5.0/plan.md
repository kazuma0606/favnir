# v28.5.0 Plan — sentry Rune 追加

## Phase 概要

| Phase | 内容 | 依存 |
|---|---|---|
| Phase 0 | 事前確認 | — |
| Phase 1 | Cargo.toml バージョン bump | — |
| Phase 2 | vm.rs に Sentry primitive 5 件追加 | Phase 1 |
| Phase 3 | runes/sentry/sentry.fav 新規作成 | Phase 2 |
| Phase 4 | examples/observability/sentry_alerting.fav 新規作成 | Phase 3 |
| Phase 5 | site/content/docs/runes/sentry.mdx 新規作成 | — |
| Phase 6 | CHANGELOG.md 更新 | — |
| Phase 7 | benchmarks/v28.5.0.json 新規作成 | — |
| Phase 9a | checker.fav 更新（ns_to_effect に Sentry 追加） | Phase 3 |
| Phase 9b | driver.rs に v285000_tests 追加 | Phase 9a |
| Phase 9c | cargo test --bin fav v285000 — 9/9 PASS 確認 | Phase 9b |
| Phase 9d | cargo test --bin fav 全体 — 2271 PASS 確認 | Phase 9c |

---

## Phase 0 — 事前確認

```bash
grep '^version' fav/Cargo.toml          # "28.4.0" を確認
cargo test --bin fav 2>&1 | tail -1     # "2262 tests" を含むことを確認
grep 'v285000_tests' fav/src/driver.rs  # 存在しないことを確認
grep 'Sentry.capture_error_raw' fav/src/backend/vm.rs  # 存在しないことを確認
grep 'ns == "Sentry"' fav/self/checker.fav  # 存在しないことを確認
```

---

## Phase 2 — vm.rs に Sentry primitive 追加

OTel `end_span_raw` の wasm32 アームの直後に挿入する。

```rust
// ── Sentry primitives (v28.5.0) ──────────────────────────────────────────────
// Stub: Sentry SDK / Relay HTTP 送信は v28.x 以降
#[cfg(not(target_arch = "wasm32"))]
"Sentry.capture_error_raw" => {
    let mut it = args.into_iter();
    let _err = vm_string(it.next().ok_or("Sentry.capture_error_raw: missing err")?, "Sentry.capture_error_raw")?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"Sentry.capture_error_raw" => Ok(err_vm(VMValue::Str("Sentry not supported on wasm32".into()))),

#[cfg(not(target_arch = "wasm32"))]
"Sentry.capture_message_raw" => {
    let mut it = args.into_iter();
    let _level = vm_string(it.next().ok_or("Sentry.capture_message_raw: missing level")?, "Sentry.capture_message_raw")?;
    let _msg   = vm_string(it.next().ok_or("Sentry.capture_message_raw: missing msg")?,   "Sentry.capture_message_raw")?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"Sentry.capture_message_raw" => Ok(err_vm(VMValue::Str("Sentry not supported on wasm32".into()))),

#[cfg(not(target_arch = "wasm32"))]
"Sentry.set_user_raw" => {
    let mut it = args.into_iter();
    let _id    = vm_string(it.next().ok_or("Sentry.set_user_raw: missing id")?,    "Sentry.set_user_raw")?;
    let _email = vm_string(it.next().ok_or("Sentry.set_user_raw: missing email")?, "Sentry.set_user_raw")?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"Sentry.set_user_raw" => Ok(err_vm(VMValue::Str("Sentry not supported on wasm32".into()))),

#[cfg(not(target_arch = "wasm32"))]
"Sentry.set_tag_raw" => {
    let mut it = args.into_iter();
    let _key   = vm_string(it.next().ok_or("Sentry.set_tag_raw: missing key")?,   "Sentry.set_tag_raw")?;
    let _value = vm_string(it.next().ok_or("Sentry.set_tag_raw: missing value")?, "Sentry.set_tag_raw")?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"Sentry.set_tag_raw" => Ok(err_vm(VMValue::Str("Sentry not supported on wasm32".into()))),

#[cfg(not(target_arch = "wasm32"))]
"Sentry.set_extra_raw" => {
    let mut it = args.into_iter();
    let _key   = vm_string(it.next().ok_or("Sentry.set_extra_raw: missing key")?,   "Sentry.set_extra_raw")?;
    let _value = vm_string(it.next().ok_or("Sentry.set_extra_raw: missing value")?, "Sentry.set_extra_raw")?;
    Ok(ok_vm(VMValue::Unit))
}
#[cfg(target_arch = "wasm32")]
"Sentry.set_extra_raw" => Ok(err_vm(VMValue::Str("Sentry not supported on wasm32".into()))),
```

---

## Phase 3 — runes/sentry/sentry.fav 新規作成

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

---

## Phase 4 — examples/observability/sentry_alerting.fav 新規作成

`examples/observability/` ディレクトリはすでに存在（v28.1.0 で作成）。

```favnir
// examples/observability/sentry_alerting.fav — Sentry エラートラッキングデモ (v28.5.0)
import runes/sentry

stage ReportError: Unit -> Result<Unit, String> !Io = |_| {
    bind _ <- Sentry.capture_error("pipeline execution failed: connection timeout")
    bind _ <- Sentry.set_tag("pipeline", "etl")
    Result.ok(unit)
}

stage SetContext: Unit -> Result<Unit, String> !Io = |_| {
    bind _ <- Sentry.set_user("user-001", "ops@example.com")
    bind _ <- Sentry.set_extra("pipeline_version", "28.5.0")
    Result.ok(unit)
}

seq SentryAlertingDemo = ReportError |> SetContext
```

---

## Phase 5 — site/content/docs/runes/sentry.mdx 新規作成

```mdx
---
title: sentry Rune
description: Sentry エラートラッキング Rune（v28.5.0）
---

# sentry Rune

`runes/sentry` は Sentry API 経由でエラーイベント・メッセージを送信する Rune です。

## インポート

```favnir
import runes/sentry
```

## 関数一覧

| 関数 | シグネチャ | 説明 |
|---|---|---|
| `Sentry.capture_error` | `(err: String) -> Result<Unit, String> !Io` | エラーイベント送信 |
| `Sentry.capture_message` | `(level: String, msg: String) -> Result<Unit, String> !Io` | メッセージイベント送信 |
| `Sentry.set_user` | `(id: String, email: String) -> Result<Unit, String> !Io` | ユーザーコンテキスト設定 |
| `Sentry.set_tag` | `(key: String, value: String) -> Result<Unit, String> !Io` | タグ設定（フィルタリング用） |
| `Sentry.set_extra` | `(key: String, value: String) -> Result<Unit, String> !Io` | 追加情報設定 |

## 使用例

```favnir
import runes/sentry

stage HandleError: Unit -> Result<Unit, String> !Io = |_| {
    bind _ <- Sentry.set_user("user-001", "ops@example.com")
    bind _ <- Sentry.set_tag("pipeline", "etl")
    bind _ <- Sentry.capture_error("stage failed: timeout")
    Result.ok(unit)
}
```

## エフェクト

すべての関数は `!Io` エフェクトを持ちます（Sentry API への HTTP 送信）。

## 注記

- `#[on_error(report_to, level)]` アノテーションによる自動エラー送信は v28.9+ で実装予定
- DSN（Data Source Name）の設定は v28.9+ で `fav.toml` の `[sentry]` セクションに追加予定。v28.5.0 時点ではスタブのため DSN 設定は不要
- WASM ターゲットでは `Result.err("Sentry not supported on wasm32")` を返します
- PII（個人情報）の取り扱いには Sentry のデータスクラビング設定を利用してください
```

---

## Phase 6 — CHANGELOG.md 更新

`CHANGELOG.md` の先頭に追加:

```markdown
## [v28.5.0] — 2026-06-28

### Added
- `runes/sentry/sentry.fav` — Sentry エラートラッキング Rune（capture_error / capture_message / set_user / set_tag / set_extra）
- `Sentry.capture_error_raw` / `capture_message_raw` / `set_user_raw` / `set_tag_raw` / `set_extra_raw` VM primitive 追加
- `fav/self/checker.fav` `ns_to_effect` に `"Sentry" => "IO"` 追加
- `examples/observability/sentry_alerting.fav` — SentryAlertingDemo E2E デモ
- `site/content/docs/runes/sentry.mdx` — ドキュメント追加
```

---

## Phase 7 — benchmarks/v28.5.0.json 新規作成

```json
{
  "version": "28.5.0",
  "test_count": 2272,
  "timestamp": "2026-06-28"
}
```

---

## Phase 9a — checker.fav 更新

`fav/self/checker.fav` の `ns_to_effect` 関数内、OTel の `else { "" }` ブロックを置き換える。
現時点の末尾ネスト（v28.3.0 で OTel を追加した後）の構造は以下の通り:

```favnir
// ── 変更前（grep "ns == \"OTel\"" の周辺 ──
                                                                                    if ns == "OTel" {
                                                                                        "IO"
                                                                                    } else {
                                                                                        ""   // ← ここを置き換える
                                                                                    }

// ── 変更後 ──
                                                                                    if ns == "OTel" {
                                                                                        "IO"
                                                                                    } else {
                                                                                        if ns == "Sentry" {
                                                                                            "IO"
                                                                                        } else {
                                                                                            ""
                                                                                        }
                                                                                    }
```

> **重要**: `"IO"`（全大文字）— v28.1.0〜v28.3.0 同様の JSONL パターンに従う。

> **重要**: `"IO"`（全大文字）— v28.1.0〜v28.3.0 同様の JSONL パターンに従う。

---

## Phase 9b — driver.rs テスト追加

`v285000_tests` を `v284000_tests` の直前に追加（10 件、`set_extra_fn` を含む）。

```rust
// ── v285000_tests (v28.5.0) — sentry Rune 追加 ────────────────────────────
#[cfg(test)]
mod v285000_tests {
    #[test]
    fn sentry_rune_has_capture_error_fn() {
        let src = include_str!("../../runes/sentry/sentry.fav");
        assert!(src.contains("fn capture_error("), "sentry rune must define fn capture_error(");
    }
    #[test]
    fn sentry_rune_has_capture_message_fn() {
        let src = include_str!("../../runes/sentry/sentry.fav");
        assert!(src.contains("fn capture_message("), "sentry rune must define fn capture_message(");
    }
    #[test]
    fn sentry_rune_has_set_user_fn() {
        let src = include_str!("../../runes/sentry/sentry.fav");
        assert!(src.contains("fn set_user("), "sentry rune must define fn set_user(");
    }
    #[test]
    fn sentry_rune_has_set_tag_fn() {
        let src = include_str!("../../runes/sentry/sentry.fav");
        assert!(src.contains("fn set_tag("), "sentry rune must define fn set_tag(");
    }
    #[test]
    fn sentry_rune_has_set_extra_fn() {
        let src = include_str!("../../runes/sentry/sentry.fav");
        assert!(src.contains("fn set_extra("), "sentry rune must define fn set_extra(");
    }
    #[test]
    fn sentry_rune_uses_io_effect() {
        let src = include_str!("../../runes/sentry/sentry.fav");
        assert!(src.contains("!Io"), "sentry rune must use !Io effect");
    }
    #[test]
    fn vm_has_sentry_capture_error_raw() {
        let src = include_str!("backend/vm.rs");
        assert!(src.contains("Sentry.capture_error_raw"), "vm.rs must implement Sentry.capture_error_raw");
    }
    #[test]
    fn sentry_example_has_pipeline() {
        let src = include_str!("../../examples/observability/sentry_alerting.fav");
        assert!(src.contains("SentryAlertingDemo"), "sentry_alerting.fav must define SentryAlertingDemo seq");
    }
    #[test]
    fn checker_has_sentry_effect() {
        let src = include_str!("../../fav/self/checker.fav");
        assert!(
            src.contains("ns == \"Sentry\"") && src.contains("\"IO\""),
            "checker.fav ns_to_effect must contain 'ns == \"Sentry\"' mapped to \"IO\""
        );
    }
    #[test]
    fn changelog_has_v28_5_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v28.5.0]") || src.contains("## v28.5.0"), "CHANGELOG.md must contain '[v28.5.0]'");
    }
}
```

---

## Phase 9c / 9d — テスト確認

```bash
cargo test --bin fav v285000   # 10/10 PASS
cargo test --bin fav sentry    # 8 件以上 PASS（sentry を名前に含む 8 件がマッチ）
cargo test --bin fav           # 2272 tests PASS
```
