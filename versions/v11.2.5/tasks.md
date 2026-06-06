# Favnir v11.2.5 Tasks

Date: 2026-06-06
Theme: Rune Registry 認証強化 — X-Fav-Token + FAV_PUBLISH_TOKEN

---

## Phase A — rune_cmd.rs（クライアント）

- [ ] A-1: `FAV_CLIENT_TOKEN` 定数を `rune_cmd.rs` 先頭に追加
- [ ] A-2: `registry_get` に `.set("X-Fav-Token", FAV_CLIENT_TOKEN)` を追加
- [ ] A-3: `registry_get_bytes` に `.set("X-Fav-Token", FAV_CLIENT_TOKEN)` を追加
- [ ] A-4: `registry_post` に `.set("X-Fav-Token", FAV_CLIENT_TOKEN)` を追加
- [ ] A-5: `cmd_rune_publish` — `BASE64.encode("admin:adminuser")` を廃止
  - `FAV_PUBLISH_TOKEN` env var から取得
  - 未設定時は明確なエラーメッセージを出して `exit(1)`
  - auth を `Bearer <token>` 形式に変更

---

## Phase B — main.fav（サーバー）

- [ ] B-1: `check_client_token(token_opt: Option<String>) -> Bool !Env` 追加
  - `Env.get_raw("FAV_CLIENT_TOKEN")` と照合
- [ ] B-2: `route` の effect 宣言に `!Env` を追加
- [ ] B-3: `route` 先頭で `fav_token` フィールドを取得し `check_client_token` でチェック
  - 不正 → `resp_text(401, "Unauthorized")` を即返す
- [ ] B-4: `handle_publish` の `Http.check_basic_auth` を廃止
  - `Env.get_raw("FAV_ADMIN_TOKEN")` と `Bearer <token>` を照合
  - effect 宣言に `!Env` を追加
- [ ] B-5: `main` 関数で `FAV_FAV_TOKEN` env var を読んで req に `fav_token` フィールドとして追加
  - `main` の effect 宣言に `!Env` を追加（既に `!Env` あり）

---

## Phase C — bootstrap スクリプト更新

- [ ] C-1: `rune-registry/bootstrap` で `X-Fav-Token` ヘッダーを `FAV_FAV_TOKEN` env var にマッピング
  ```bash
  FAV_FAV_TOKEN=$(echo "$EVENT_DATA" | jq -r '.headers["x-fav-token"] // .headers["X-Fav-Token"] // ""')
  export FAV_FAV_TOKEN
  ```

---

## Phase D — Terraform 更新

- [ ] D-1: `infra/registry/lambda.tf` の `aws_lambda_function.registry` に `environment` ブロック追加
  ```hcl
  environment {
    variables = {
      FAV_CLIENT_TOKEN = var.fav_client_token
      FAV_ADMIN_TOKEN  = var.registry_admin_token
    }
  }
  ```
- [ ] D-2: `infra/registry/variables.tf` に `fav_client_token` / `registry_admin_token` 変数追加（`sensitive = true`）

---

## Phase E — バージョン更新・ドキュメント

- [ ] E-1: `rune-registry/fav.toml` version → `"0.2.0"`
- [ ] E-2: `rune-registry/SPEC.md` 認証仕様を更新
  - 旧: `Authorization: Basic YWRtaW46YWRtaW51c2Vy`
  - 新: `X-Fav-Token: <client-token>` (全リクエスト) + `Authorization: Bearer <admin-token>` (publish のみ)
- [ ] E-3: `fav/Cargo.toml` version → `"11.2.5"`
- [ ] E-4: `cargo build` で `Cargo.lock` 更新

---

## Phase F — テスト・デプロイ

- [ ] F-1: `cargo test --lib` — 全件通過確認
- [ ] F-2: `git commit & push` — CI 確認
- [ ] F-3: `terraform apply` 手動実行（Lambda env vars 追加）
- [ ] F-4: デプロイ後の動作確認
  - `fav rune list` — トークンあり → OK
  - `curl /runes` — トークンなし → 401

---

## 完了条件サマリー

| 確認項目 | 状態 |
|---|---|
| `fav rune list` が `X-Fav-Token` を送信している | |
| `curl /runes`（トークンなし）→ 401 | |
| `curl /runes -H "X-Fav-Token: <token>"` → 200 | |
| `fav rune publish`（`FAV_PUBLISH_TOKEN` 未設定）→ エラー終了 | |
| `fav rune publish`（`FAV_PUBLISH_TOKEN` セット済み）→ 201 | |
| `cargo test --lib` 全件通過 | |
