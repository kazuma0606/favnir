# v29.1.0 Tasks — `fav publish` 実装（Rune Registry 本番稼働）

**状態**: COMPLETE
**開始日**: 2026-06-28
**完了日**: 2026-06-28

---

## 事前確認（T0）

- [x] `Cargo.toml` の version が `29.0.0` であること
- [x] `cargo test --bin fav 2>&1 | grep "^test result"` が `2312 passed` を含むこと
- [x] `driver.rs` に `mod v291000_tests` が存在しないこと
- [x] `driver.rs` に `pub fn cmd_publish` が存在すること（v17.x で実装済み）
- [x] `driver.rs` に `pub fn cmd_info` が存在しないこと（v29.1.0 で追加）
- [x] `infra/registry/main.tf` が存在すること

---

## タスク一覧

| タスク | 内容 | 状態 |
|---|---|---|
| T1 | `Cargo.toml` version `29.0.0` → `29.1.0` | [x] |
| T2 | `driver.rs`: `cmd_publish` に `FAVNIR_REGISTRY_URL` API 呼び出し追加 | [x] |
| T3 | `driver.rs`: `cmd_search` に `FAVNIR_REGISTRY_URL` フォールバック追加 | [x] |
| T4 | `driver.rs`: `pub fn cmd_info(pkg_name: &str)` 新規追加 | [x] |
| T5 | `main.rs`: `Some("info")` アーム追加（`cmd_info` 呼び出し） | [x] |
| T6 | `driver.rs`: `cmd_login` に GitHub OAuth URL 生成追加 | [x] |
| T7 | `CHANGELOG.md` に `[v29.1.0]` セクション追加 | [x] |
| T8 | `benchmarks/v29.1.0.json` 新規作成（test_count: 2318） | [x] |
| T9 | `driver.rs` に `v291000_tests` 6 件追加 | [x] |
| T9.5 | `cargo test --bin fav v291000` — 6/6 PASS 確認 | [x] |
| T10 | `cargo test --bin fav` 全体 — 2318 tests PASS 確認 | [x] |
| T11 | `fav publish --dry-run`（`fav/examples/csv_demo/` で実行）— exit 0 確認 | [x] |
| T11.5 | `fav info postgres` — フォールバック表示して exit 0 確認 | [x] |
| T12 | tasks.md を COMPLETE に更新 | [x] |

---

## テスト詳細（T9）

```rust
// ── v291000_tests (v29.1.0) — fav publish 実装（Rune Registry 本番稼働）────────────────────────────
#[cfg(test)]
mod v291000_tests {
    // include_str! のみ使用のため use super::* 不要
    #[test]
    fn driver_has_registry_api_base_url() {
        let src = include_str!("driver.rs");
        assert!(
            src.contains("FAVNIR_REGISTRY_URL") && src.contains("std::env::var"),
            "cmd_publish must read FAVNIR_REGISTRY_URL via std::env::var"
        );
    }
    #[test]
    fn infra_registry_lambda_tf_exists() {
        let src = include_str!("../../infra/registry/lambda.tf");
        assert!(
            src.contains("aws_lambda_function") || src.contains("aws_apigatewayv2_api"),
            "infra/registry/lambda.tf must define aws_lambda_function or aws_apigatewayv2_api"
        );
    }
    #[test]
    fn cmd_info_fn_exists_in_driver() {
        let src = include_str!("driver.rs");
        assert!(src.contains("pub fn cmd_info"), "driver.rs must define pub fn cmd_info");
    }
    #[test]
    fn login_generates_github_oauth_url() {
        let src = include_str!("driver.rs");
        assert!(
            src.contains("github.com/login/oauth/authorize") || src.contains("github_oauth"),
            "cmd_login must generate a GitHub OAuth URL"
        );
    }
    #[test]
    fn fav_info_subcommand_in_main() {
        let src = include_str!("main.rs");
        assert!(src.contains("\"info\""), "main.rs must route `fav info` subcommand");
    }
    #[test]
    fn changelog_has_v29_1_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(
            src.contains("[v29.1.0]") || src.contains("## v29.1.0"),
            "CHANGELOG.md must contain '[v29.1.0]'"
        );
    }
}
```

---

## 完了条件チェックリスト

- [x] `Cargo.toml` version = "29.1.0"
- [x] `driver.rs` に `FAVNIR_REGISTRY_URL` または `registry.favnir.dev` の参照あり
- [x] `infra/registry/lambda.tf` が存在し `aws_lambda_function` / `aws_apigatewayv2_api` を含む
- [x] `driver.rs` に `pub fn cmd_info` 関数あり
- [x] `cmd_login` に GitHub OAuth URL 生成あり（`github.com/login/oauth/authorize`）
- [x] `main.rs` に `"info"` サブコマンドの分岐あり
- [x] `CHANGELOG.md` に `[v29.1.0]` セクションあり
- [x] `benchmarks/v29.1.0.json` 存在（test_count: 2318）
- [x] `cargo test --bin fav v291000` — 6/6 PASS
- [x] `cargo test --bin fav` — 2318 tests PASS
- [x] `fav publish --dry-run`（`fav/examples/csv_demo/` で実行）— exit 0
- [x] `fav info postgres` — exit 0（フォールバック表示）

---

## コードレビュー指摘対応

### spec-reviewer 指摘（実装前）
- [HIGH] `driver_has_registry_api_base_url` テストの偽陽性 → `registry.favnir.dev` がすでに存在するため `FAVNIR_REGISTRY_URL && std::env::var` の AND 条件に変更
- [HIGH] `infra_registry_main_tf_exists` → `main.tf` に `aws_lambda_function` はないため `lambda.tf` に変更

### code-reviewer 指摘（実装後）
- [MED] `cmd_info`: パッケージ未発見時の exit code が 0 → `eprintln!` + `std::process::exit(1)` に変更（`fav info nonexistent` → exit 1 確認済み）
- [LOW] `cmd_search` URL 未エスケープ — HTTP 有効化時に対応予定（現状は HTTP 未送信のためスコープ外）
- [LOW] `FAV_GITHUB_CLIENT_ID` デフォルト値の説明なし — 本番では ENV 必須（コメントで明記済み）
