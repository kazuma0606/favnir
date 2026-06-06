# Rune Registry 仕様書

作成日: 2026-05-19

---

## 概要

Rune Registry は **Favnir 自身で書かれた HTTP サービス**（dogfooding）。
AWS Lambda + API Gateway で動作し、Rune（Favnir パッケージ）のメタデータ管理を担う。

---

## API

**Base URL**: `https://32qp3qwhdh.execute-api.ap-northeast-1.amazonaws.com`

| Method | Path            | 認証     | 説明               |
|--------|-----------------|----------|--------------------|
| GET    | `/runes`        | `X-Fav-Token` 必須 | Rune 一覧を返す     |
| GET    | `/runes/{name}` | `X-Fav-Token` 必須 | 指定 Rune の詳細   |
| GET    | `/runes/{name}/download` | `X-Fav-Token` 必須 | Rune ダウンロード |
| GET    | `/runes/{name}/versions` | `X-Fav-Token` 必須 | バージョン一覧 |
| POST   | `/runes/{name}` | `X-Fav-Token` + `Authorization: Bearer <token>` | Rune を publish   |

### リクエスト/レスポンス例

```bash
# 一覧（X-Fav-Token 必須）
GET /runes
X-Fav-Token: fav-registry-v1-dk9p2mxw4qhz
→ [{"name":"csv","version":"0.1.0","description":"CSV Rune"}, ...]
→ 401 "Unauthorized"  # トークンなし or 不正

# 詳細
GET /runes/csv
X-Fav-Token: fav-registry-v1-dk9p2mxw4qhz
→ {"name":"csv","version":"0.1.0","description":"CSV Rune"}
→ 404 "rune not found"  # 存在しない場合

# Publish（管理者のみ）
POST /runes/csv
X-Fav-Token: fav-registry-v1-dk9p2mxw4qhz
Authorization: Bearer <FAV_PUBLISH_TOKEN>
Content-Type: application/json
{"version":"0.1.0","description":"CSV Rune"}
→ 201 "published"
→ 401 "Unauthorized"   # トークン不正
→ 400 "invalid JSON body"
```

### 認証

**クライアントトークン（全リクエスト共通）**:
- `X-Fav-Token` ヘッダーを全リクエストに付与
- トークンは `fav` バイナリに静的埋め込み（`FAV_CLIENT_TOKEN` 定数）
- Lambda 環境変数 `FAV_CLIENT_TOKEN` と照合

**publish 管理者トークン**:
- `Authorization: Bearer <token>` ヘッダーを付与
- クライアント側は `FAV_PUBLISH_TOKEN` 環境変数から読む（未設定時はエラー終了）
- Lambda 環境変数 `FAV_ADMIN_TOKEN` と照合

---

## アーキテクチャ

```
クライアント
    │ HTTPS
    ▼
API Gateway HTTP API  (favnir-registry)
    │ Lambda Proxy統合
    ▼
Lambda  favnir-registry  (コンテナイメージ, 512MB, 30秒)
    │
    ├── bootstrap (bash)       ← Lambda Runtime API をポーリング
    │     └── fav run /app/src/main.fav  ← Favnir コードを毎回実行
    │
    ├── DynamoDB  favnir-rune-registry   ← Rune メタデータ
    └── S3        favnir-rune-packages   ← Rune パッケージ本体
```

---

## リクエスト処理フロー

```
1. API Gateway → Lambda にイベント投入
2. bootstrap が Lambda Runtime API をポーリング (GET /invocation/next)
3. イベント JSON から method / path / body / Authorization を jq で抽出
4. 環境変数 FAV_METHOD / FAV_PATH / FAV_BODY / FAV_AUTH にセット
5. fav run /app/src/main.fav を実行
   └── Env.require_raw で環境変数を読む
   └── route() でルーティング
   └── DynamoDB / S3 を操作 (!AWS エフェクト)
   └── IO.println でレスポンス Map を JSON 出力
6. bootstrap が stdout を受け取り Lambda レスポンス形式に変換
   {"status":"201","body":"published","content_type":"text/plain"}
   → {"statusCode":201,"headers":{"Content-Type":"text/plain"},"body":"published"}
7. Lambda Runtime API に POST (invocation/{id}/response)
```

---

## データモデル

### DynamoDB テーブル: `favnir-rune-registry`

| フィールド    | 型     | 説明             |
|---------------|--------|------------------|
| `name`        | String | パーティションキー（Rune 名） |
| `version`     | String | セマンティックバージョン |
| `description` | String | 説明文           |

### S3 バケット: `favnir-rune-packages`

- キー: Rune 名（例: `csv`）
- 値: POST ボディの JSON 文字列をそのまま保存

---

## AWS リソース一覧

| リソース          | 名前                          | 備考                      |
|-------------------|-------------------------------|---------------------------|
| Lambda            | `favnir-registry`             | コンテナイメージ、ap-northeast-1 |
| ECR               | `favnir-registry`             | Docker イメージ置き場     |
| API Gateway       | `favnir-registry`             | HTTP API, $default ステージ |
| DynamoDB          | `favnir-rune-registry`        | PAY_PER_REQUEST           |
| S3                | `favnir-rune-packages`        | 非公開、SSE-AES256        |
| IAM Role          | `favnir-registry-lambda`      | DynamoDB + S3 アクセス権  |

すべて Terraform (`infra/registry/`) で管理。

---

## CI/CD ワークフロー

トリガー: `rune-registry/**`, `fav/**`, `infra/registry/**` への master push

```
GitHub Actions (ubuntu-latest)
    │
    ├── OIDC → AWS 認証 (secrets.AWS_DEPLOY_ROLE_ARN)
    ├── ECR ログイン
    ├── docker build (repo root をコンテキスト)
    │     └── rune-registry/Dockerfile
    │           Stage 1: rust:1.88-slim → cargo build --release --bin fav
    │           Stage 2: debian:bookworm-slim → fav バイナリ + bootstrap + Favnir ソース
    ├── docker push (:{sha} と :latest の 2 タグ)
    └── aws lambda update-function-code --image-uri :{sha}
          aws lambda wait function-updated
```

初回デプロイのみ手動で `terraform apply` が必要（Lambda, API Gateway 等の作成）。
以降の更新は push だけで自動反映。

---

## ソースファイル構成

```
rune-registry/
├── src/main.fav   Favnir HTTP サービス本体
├── bootstrap      Lambda Custom Runtime ポーリングスクリプト (bash)
├── Dockerfile     マルチステージビルド
├── fav.toml       Favnir プロジェクト設定
└── SPEC.md        本ドキュメント

infra/registry/
├── main.tf        ECR / DynamoDB / S3 / IAM / API Gateway
├── lambda.tf      Lambda + API Gateway 統合 + パーミッション
├── providers.tf   AWS provider 設定
├── variables.tf   変数定義
└── outputs.tf     api_url 等の出力

.github/workflows/
└── deploy-registry.yml   CI/CD パイプライン
```

---

## 既知の制約

- **レスポンス速度**: `fav run` がリクエストのたびにソースをコンパイル・実行するため約 4〜6 秒かかる（dogfooding 目的のため許容）
- **Lambda Function URL**: このアカウントでは NONE auth が機能しないため API Gateway 経由
- **クライアントトークン**: バイナリに静的埋め込みのため、デコンパイルすれば取得可能（完全なゼロトラストではない）
- **publish トークン**: 管理者が `FAV_PUBLISH_TOKEN` 環境変数にセットして実行する必要がある
