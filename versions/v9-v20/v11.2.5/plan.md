# Favnir v11.2.5 実装計画

作成日: 2026-06-06

---

## 実装順序

```
Phase A: クライアント側（rune_cmd.rs）修正
    ↓
Phase B: サーバー側（main.fav）修正
    ↓
Phase C: インフラ（Terraform）更新
    ↓
Phase D: バージョン更新・ドキュメント
    ↓
Phase E: テスト確認・コミット
```

---

## Phase A — rune_cmd.rs（クライアント）

### A-1: クライアントトークン定数追加

```rust
// rune_cmd.rs 先頭
const FAV_CLIENT_TOKEN: &str = "fav-client-v1-abc123xyz";
```

### A-2: `registry_get` / `registry_get_bytes` に `X-Fav-Token` ヘッダー追加

```rust
fn registry_get(path: &str) -> Result<String, String> {
    let url = format!("{}{}", REGISTRY_URL, path);
    match ureq::get(&url)
        .set("X-Fav-Token", FAV_CLIENT_TOKEN)  // 追加
        .call() { ... }
}
```

同様に `registry_get_bytes` も更新。

### A-3: `cmd_rune_publish` — 認証をenv varから取得

```rust
fn cmd_rune_publish() {
    // 現在: let creds = BASE64.encode("admin:adminuser");
    //       let auth = format!("Basic {}", creds);
    // 変更後:
    let token = match std::env::var("FAV_PUBLISH_TOKEN") {
        Ok(t) => t,
        Err(_) => {
            eprintln!("error: FAV_PUBLISH_TOKEN environment variable is not set");
            eprintln!("       Set it to your publish token before running rune publish");
            std::process::exit(1);
        }
    };
    let auth = format!("Bearer {}", token);
    // registry_post に X-Fav-Token ヘッダーも追加
}
```

`registry_post` にも `X-Fav-Token` ヘッダーを追加する。

---

## Phase B — main.fav（サーバー）

### B-1: `check_client_token` 関数追加

```fav
fn check_client_token(token_opt: Option<String>) -> Bool !Env {
  bind expected_r <- Env.get_raw("FAV_CLIENT_TOKEN");
  bind expected <- match expected_r { Err(_) => ""  Ok(v) => v };
  match token_opt {
    None => false
    Some(t) => t == expected
  }
}
```

### B-2: `route` 関数の先頭でトークンチェック

```fav
fn route(req: ...) -> ... !AWS !Env {
  bind fav_token_opt <- Map.get(req, "fav_token");
  bind valid <- check_client_token(fav_token_opt);
  if valid { ... 既存ルーティング ... } else { resp_text(401, "Unauthorized") }
}
```

`main` 関数で `FAV_FAV_TOKEN` 環境変数 → `fav_token` フィールドとして req に追加。

### B-3: `handle_publish` — `Http.check_basic_auth` を廃止

```fav
fn handle_publish(name: String, body: String, auth: String) -> ... !AWS !Env {
  bind expected_r <- Env.get_raw("FAV_ADMIN_TOKEN");
  bind expected   <- match expected_r { Err(_) => ""  Ok(v) => v };
  bind bearer     <- if String.starts_with(auth, "Bearer ") {
    String.slice(auth, 7, String.length(auth))
  } else { "" };
  if bearer == expected { ... 既存の保存処理 ... } else { resp_text(401, "Unauthorized") }
}
```

---

## Phase C — Terraform

### C-1: `lambda.tf` に environment ブロック追加

```hcl
resource "aws_lambda_function" "registry" {
  ...
  environment {
    variables = {
      FAV_CLIENT_TOKEN = var.fav_client_token
      FAV_ADMIN_TOKEN  = var.registry_admin_token
    }
  }
}
```

### C-2: `variables.tf` に変数追加

```hcl
variable "fav_client_token" {
  description = "Static token embedded in fav binary for registry access"
  type        = string
  sensitive   = true
}

variable "registry_admin_token" {
  description = "Admin token for rune publish operations"
  type        = string
  sensitive   = true
}
```

CI/CD では GitHub Actions secrets から `TF_VAR_fav_client_token` / `TF_VAR_registry_admin_token` を渡す。

---

## Phase D — バージョン更新

- `rune-registry/fav.toml`: version `"0.1.0"` → `"0.2.0"`
- `rune-registry/SPEC.md`: 認証仕様を更新（旧: Basic Auth admin:adminuser → 新: X-Fav-Token + Bearer token）
- `fav/Cargo.toml`: version → `"11.2.5"`
- `fav/Cargo.lock`: `cargo build` で更新

---

## Phase E — テスト・コミット

- `cargo test --lib` — 全件通過確認（既存テストに影響なし）
- `git commit` — "feat: rune registry auth — X-Fav-Token + FAV_PUBLISH_TOKEN"
- `git push` — CI 確認
- `terraform apply` を手動実行（Lambda env vars 追加のため）
- デプロイ後 `fav rune list` 動作確認（トークンあり: OK、なし: 401）

---

## 注意事項

- `check_client_token` は `!Env` エフェクトを持つため、`route` / `main` の effect 宣言に `!Env` を追加する必要がある
- `main.fav` で `FAV_FAV_TOKEN` env var を読んで `fav_token` フィールドに追加する（bootstrap で `X-Fav-Token` ヘッダーを `FAV_FAV_TOKEN` にマッピング済みか確認が必要）
- bootstrap スクリプトで `X-Fav-Token` ヘッダーを抽出して `FAV_FAV_TOKEN` env var にセットする処理を追加する
