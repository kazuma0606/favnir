# Favnir v11.2.5 仕様書

作成日: 2026-06-06
テーマ: Rune Registry 認証強化

---

## 背景と目的

現在の Rune Registry は以下の問題を抱えている:

| 問題 | 現状 |
|---|---|
| GET /runes (一覧・詳細・ダウンロード) | 認証なし。誰でも curl でアクセス可能 |
| POST /runes/{name} (publish) | `admin:adminuser` をクライアントソースにハードコード |

v11.2.5 では以下を実現する:

1. **クライアントトークン認証** — `fav` バイナリにトークンを埋め込み、全リクエストに `X-Fav-Token` ヘッダーを付与。Lambda 側で検証し、`fav` インストール済みユーザーのみアクセス可能にする。
2. **publish 認証のハードコード廃止** — Lambda 環境変数 `FAV_ADMIN_TOKEN` と照合する方式に変更。クライアントは `FAV_PUBLISH_TOKEN` 環境変数から読む。

---

## 認証設計

### クライアントトークン（全リクエスト共通）

```
fav バイナリ
  └── const FAV_CLIENT_TOKEN: &str = "fav-client-v1-<token>"
        ↓ X-Fav-Token ヘッダーとして送信
Lambda (main.fav)
  └── Env.get_raw("FAV_CLIENT_TOKEN") と照合
        → 不一致 or 未送信 → 401 Unauthorized
```

- トークンはバイナリに同梱（静的共有秘密）
- 野良 curl / スクレイピングを防止
- `fav` バイナリを持つユーザー全員がアクセス可能（全ユーザー共通トークン）

### publish 管理者トークン

```
管理者（Favnir maintainer）
  └── 環境変数 FAV_PUBLISH_TOKEN=<admin-secret> を設定してから実行
        ↓ Authorization: Bearer <token>
Lambda (main.fav)
  └── Env.get_raw("FAV_ADMIN_TOKEN") と照合
        → 不一致 or 未送信 → 401 Unauthorized
```

- ハードコードの `admin:adminuser` を廃止
- Lambda 環境変数 `FAV_ADMIN_TOKEN` に設定
- クライアント側は `FAV_PUBLISH_TOKEN` env var から読む（未設定時はエラー終了）

---

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/rune_cmd.rs` | `FAV_CLIENT_TOKEN` 定数追加、全リクエストにヘッダー付与、publish 認証を env var 参照に変更 |
| `rune-registry/src/main.fav` | 全ルートでクライアントトークンチェック追加、publish 認証を `FAV_ADMIN_TOKEN` env var 参照に変更 |
| `infra/registry/lambda.tf` | Lambda `environment` ブロックに `FAV_CLIENT_TOKEN` / `FAV_ADMIN_TOKEN` 追加 |
| `infra/registry/variables.tf` | `fav_client_token` / `registry_admin_token` 変数追加 |
| `rune-registry/fav.toml` | version → `"0.2.0"` |
| `rune-registry/SPEC.md` | 認証仕様更新 |
| `fav/Cargo.toml` | version → `"11.2.5"` |

---

## API 変更後の仕様

**Base URL**: `https://32qp3qwhdh.execute-api.ap-northeast-1.amazonaws.com`

| Method | Path | 認証 | 説明 |
|---|---|---|---|
| GET | `/runes` | `X-Fav-Token` 必須 | Rune 一覧 |
| GET | `/runes/{name}` | `X-Fav-Token` 必須 | Rune 詳細 |
| GET | `/runes/{name}/download` | `X-Fav-Token` 必須 | Rune ダウンロード |
| GET | `/runes/{name}/versions` | `X-Fav-Token` 必須 | バージョン一覧 |
| POST | `/runes/{name}` | `X-Fav-Token` + `Authorization: Bearer <admin>` | Rune publish |

---

## SPEC.md に記載する既知の制約

- クライアントトークンはバイナリに静的埋め込みのため、デコンパイルすれば取得可能（完全なゼロトラストではない）
- publish トークンは管理者が `FAV_PUBLISH_TOKEN` 環境変数にセットする必要がある
- Terraform apply は初回のみ手動。以降は CI が Lambda 環境変数を更新
