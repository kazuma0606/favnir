# Roadmap v35.1.0 〜 v36.0.0 — Deployment Story

Date: 2026-07-06
Status: 骨格確定（v35.0 完了時点）

---

## 目標

v35.0「Production Ready」では `fav build --target native` で生成したバイナリを
**手動で** Lambda にデプロイする形での達成を宣言した。

このフェーズは **`fav deploy` CLI による自動化** を実現する。

> **Deployment Story の定義**
> 「`fav deploy --target lambda` で Lambda に自動デプロイでき、
>  `fav deploy --target docker` で Docker イメージを生成でき、
>  `fav ci init` で GitHub Actions ワークフローを自動生成できる。
>  デプロイ設定は `deploy.fav` に宣言的に記述し、
>  `fav deploy status` で状況確認、`fav rollback` でロールバックできる」

v35.0 との差分: **手動 → CLI 自動化**（バイナリ生成は v35.0 達成済み、ここでは包装・配布を自動化）

---

## 設計決定事項

| 項目 | 決定 |
|---|---|
| Lambda ランタイム | provided.al2（カスタムランタイム）、バイナリは `fav build --target native` 流用 |
| Docker ベースイメージ | `debian:bookworm-slim` |
| k8s Manifest 形式 | Deployment + Service + ConfigMap |
| CI プロバイダ | GitHub Actions（v37.x で GitLab CI 追加予定） |
| deploy.fav 構文 | Favnir DSL（`deploy { target: Lambda, ... }` ブロック） |

---

## バージョン計画

### v35.1.0 — `fav deploy --target lambda`

**テーマ**: `fav build --target native` 生成バイナリを Lambda に自動デプロイする。

**実装内容**:
- `fav/src/deploy/lambda.rs` — zip パッケージング（bootstrap + rune 依存物）
- `fav/src/main.rs` — `Some("deploy")` アーム追加
- AWS CLI 経由の `aws lambda update-function-code`（`std::process::Command`、SDK 依存なし）
- `examples/lambda-deploy/` デモ追加

**完了条件**:
- `fav deploy --target lambda --function my-fn` で Lambda が更新される
- Rust テスト 2 件

---

### v35.2.0 — `fav deploy --target docker`

**テーマ**: Docker イメージを自動生成する。

**実装内容**:
- `fav/src/deploy/docker.rs` — Dockerfile 自動生成 + `docker build` 実行
- `fav deploy --target docker --tag my-pipeline:latest`

**完了条件**:
- Dockerfile が生成され `docker build` が実行される
- Rust テスト 1 件（Dockerfile テンプレート生成テスト）

---

### v35.3.0 — `fav ci init`

**テーマ**: GitHub Actions ワークフローを自動生成する。

**実装内容**:
- `fav/src/ci/github_actions.rs` — CI YAML 生成
- `fav/src/main.rs` — `Some("ci")` アーム追加
- 生成ファイル: `.github/workflows/ci.yml`（check + test + lint ステップ）

**完了条件**:
- `.github/workflows/ci.yml` が生成される
- Rust テスト 2 件

---

### v35.4.0 — `fav deploy --target k8s`

**テーマ**: Kubernetes Manifest を生成する。

**実装内容**:
- `fav/src/deploy/k8s.rs` — Deployment + Service + ConfigMap YAML 生成
- `fav.toml` に `[deploy.k8s]` セクション追加

**完了条件**:
- `deployment.yaml` / `service.yaml` / `configmap.yaml` が生成される
- Rust テスト 1 件

---

### v35.5.0 — `deploy.fav` 宣言的デプロイ設定

**テーマ**: デプロイ設定を Favnir DSL で記述できるようにする。

**想定構文**:
```favnir
deploy {
  target: Lambda
  function: "my-pipeline"
  region: "ap-northeast-1"
  memory: 512
  timeout: 60
  env: {
    DB_URL: env("DATABASE_URL")
    LOG_LEVEL: "info"
  }
}
```

**完了条件**:
- `deploy.fav` が `fav deploy` に自動読み込みされる
- Rust テスト 2 件（config parse テスト）

---

### v35.6.0 — `fav deploy status` + `fav rollback`

**テーマ**: デプロイ状態確認とロールバックを追加する。

**実装内容**:
- `fav deploy status` — 最終デプロイ日時・バージョン・状態表示
- `fav rollback` / `fav rollback --version 3` — Lambda alias でバージョン切り戻し
- `.fav-deploy-history.json` でデプロイ履歴管理

**完了条件**:
- 両コマンドが動作する
- Rust テスト 2 件

---

### v35.7.0 — `fav deploy --dry-run`

**テーマ**: デプロイ前に変更内容を確認できるようにする。

**実装内容**:
- `fav deploy --dry-run` — 変更差分を表示して実行しない
- `fav deploy --diff` — 前回デプロイとの差分表示

**完了条件**:
- dry-run で変更内容が表示される
- Rust テスト 1 件

---

### v35.8.0 — デプロイ cookbook + ドキュメント

**テーマ**: デプロイ機能のドキュメントと cookbook を整備する。

**追加コンテンツ**:
- `site/content/docs/deploy/lambda.mdx`
- `site/content/docs/deploy/docker.mdx`
- `site/content/docs/deploy/k8s.mdx`
- `site/content/cookbook/lambda-deploy.mdx`
- `site/content/cookbook/docker-pipeline.mdx`
- `site/content/cookbook/github-actions-ci.mdx`

**完了条件**:
- 上記ファイルが存在する
- Rust テスト 1 件

---

### v35.9.0 — v36.0 前調整・安定化

E2E 動作確認・CHANGELOG 更新・v36.0 宣言文草案。

---

### v36.0.0 — Deployment Story マイルストーン宣言 ★クリーンアップ

**宣言文（暫定）**:

> 「`fav deploy --target lambda` で Lambda にデプロイし、
>  `fav deploy --target docker` で Docker イメージを生成し、
>  `fav ci init` で GitHub Actions CI を自動設定できる。
>  デプロイ設定は `deploy.fav` に型安全に記述し、
>  `fav rollback` で前バージョンに即座に戻せる。
>
>  これが Favnir v36.0 — Deployment Story の姿である。」

**完了条件**:
- v35.1〜v35.9 の全機能が動作する
- `examples/lambda-deploy/` が存在する
- テスト数 3500+（`cargo test --locked` 0 failures）
- GitHub Issues の P1/P2 ラベル付きオープンバグが **0 件**
- `★クリーンアップ` 完了

---

## 参考リンク

- マスタースケジュール: `versions/roadmap/roadmap-v35.1-v40.0.md`
- 前サブスプリント: `versions/roadmap/roadmap-v34.1-v35.0.md`
- 次サブスプリント: `versions/roadmap/roadmap-v36.1-v37.0.md`
