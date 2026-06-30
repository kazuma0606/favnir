# v29.1.0 Spec — `fav publish` 実装（Rune Registry 本番稼働）

## 概要

`fav publish / add / search / info / update` を **実際の Lambda API** に接続し、
Rune Registry を本番稼働させる。
`infra/registry/`（既存 Terraform）を確認・拡充し、E2E フロー（publish → search → add）を動作させる。

## 背景・現状

| コマンド | 現状 | v29.1.0 目標 |
|---|---|---|
| `fav publish` | ローカルレジストリ（`~/.fav/registry/`）にのみ書き込む | `FAVNIR_REGISTRY_URL` 環境変数 → Lambda API POST を呼ぶ |
| `fav search` | `OFFICIAL_CATALOG`（静的定数）から検索 | `FAVNIR_REGISTRY_URL/v1/search?q=...` を呼ぶ（フォールバック: 静的カタログ） |
| `fav add` | ローカルレジストリからインストール | スコープ外（v29.2.x 以降に延期）|
| `fav info` | **未実装** | `FAVNIR_REGISTRY_URL/v1/packages/{name}` → 詳細表示 |
| `fav login` | `FAV_TOKEN` 環境変数スタブ | GitHub OAuth URL を生成して標準出力に表示 |
| Terraform | `infra/registry/` に存在（5 ファイル） | 確認・`rune-registry` とのマッピング整合性確保 |

## 設計決定

| 項目 | 決定 |
|---|---|
| API ベース URL | `FAVNIR_REGISTRY_URL` 環境変数（未設定時は `https://registry.favnir.dev`）|
| 認証 | `~/.fav/credentials` に保存済みトークンをベアラー認証に使用 |
| `fav info` のルーティング | `main.rs` の `Some("info")` アームを追加 |
| GitHub OAuth URL | `https://github.com/login/oauth/authorize?client_id=...` を生成して表示 |
| 既存 Terraform の扱い | `infra/registry/` をそのまま使用（`rune-registry` ディレクトリへのエイリアスはドキュメントのみ）|
| 破壊的変更 | なし（STABILITY.md v1.x ポリシーに従う）|

## 実装内容

### T1 — Cargo.toml バージョン bump
`fav/Cargo.toml` の `version` を `"29.0.0"` → `"29.1.0"` に更新。

### T2 — `cmd_publish` に API 呼び出し追加
`--dry-run` でない場合に `FAVNIR_REGISTRY_URL` → `{url}/v1/publish` へ HTTP POST。
既存のローカルレジストリ書き込みをフォールバックとして残す。

### T3 — `cmd_search` に API フォールバック追加
`FAVNIR_REGISTRY_URL` が設定されている場合は `/v1/search?q={query}` を呼ぶ。
未設定時は既存の `OFFICIAL_CATALOG` 検索にフォールバック。

### T4 — `pub fn cmd_info(pkg_name: &str)` 新規追加
`FAVNIR_REGISTRY_URL/v1/packages/{name}` から詳細情報を取得して表示。
`main.rs` に `Some("info")` アームを追加。

### T5 — `cmd_login` に GitHub OAuth URL 生成を追加
`FAV_GITHUB_CLIENT_ID` 環境変数（未設定時は `favnir-registry`）を使い、
`https://github.com/login/oauth/authorize?client_id={id}&scope=read:user` を生成して表示。

### T6 — `infra/registry/` 整合性確認
既存 `infra/registry/main.tf` の内容を確認し、Lambda + API Gateway + S3 構成であることを
`driver.rs` のテストで検証。

### T7 — CHANGELOG.md 更新
`CHANGELOG.md` に `[v29.1.0]` セクション追加。

### T8 — `benchmarks/v29.1.0.json` 新規作成（test_count: 2318）

### T9 — driver.rs テスト（6 件）

```
v291000_tests:
  driver_has_registry_api_base_url
  infra_registry_lambda_tf_exists
  cmd_info_fn_exists_in_driver
  login_generates_github_oauth_url
  fav_info_subcommand_in_main
  changelog_has_v29_1_0
```

## テスト数

- v29.0.0: 2312 tests
- v29.1.0: **2318 tests**（+6）

## 完了条件

- [ ] `Cargo.toml` version = "29.1.0"
- [ ] `cmd_publish` が `FAVNIR_REGISTRY_URL` を参照して API を呼ぶ
- [ ] `cmd_search` が `FAVNIR_REGISTRY_URL` フォールバックを持つ
- [ ] `pub fn cmd_info` が `driver.rs` に存在する
- [ ] `main.rs` に `Some("info")` アームがある
- [ ] `cmd_login` が GitHub OAuth URL を生成して出力する
- [ ] `infra/registry/main.tf` が存在する（既存確認）
- [ ] `CHANGELOG.md` に `[v29.1.0]` セクションあり
- [ ] `benchmarks/v29.1.0.json` 存在（test_count: 2318）
- [ ] `cargo test --bin fav v291000` — 6/6 PASS
- [ ] `cargo test --bin fav` 全体が 2318 tests PASS
- [ ] `fav publish --dry-run`（`examples/postgres_etl/` 等で）— exit 0
- [ ] `fav info postgres` — 詳細情報（またはフォールバック）を出力して exit 0
