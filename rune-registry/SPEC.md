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
| GET    | `/runes`        | 不要     | Rune 一覧を返す     |
| GET    | `/runes/{name}` | 不要     | 指定 Rune の詳細   |
| POST   | `/runes/{name}` | Basic 認証 | Rune を publish   |

### リクエスト/レスポンス例

```bash
# 一覧
GET /runes
→ [{"name":"csv","version":"0.1.0","description":"CSV Rune"}, ...]

# 詳細
GET /runes/csv
→ {"name":"csv","version":"0.1.0","description":"CSV Rune"}
→ 404 "rune not found"  # 存在しない場合

# Publish（管理者のみ）
POST /runes/csv
Authorization: Basic YWRtaW46YWRtaW51c2Vy   # admin:adminuser
Content-Type: application/json
{"version":"0.1.0","description":"CSV Rune"}
→ 201 "published"
→ 401 "Unauthorized"   # 認証失敗
→ 400 "invalid JSON body"
```

### 認証

HTTP Basic Auth。`admin:adminuser` のみ publish 可。
認証ロジックは Favnir コード内 (`Http.check_basic_auth`) で完結。

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
- **認証情報**: admin:adminuser はハードコード（本番化する場合は Secrets Manager 等に移行が必要）
