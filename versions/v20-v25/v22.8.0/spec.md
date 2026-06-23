# v22.8.0 仕様書 — `fav deploy` 強化（ECS / K8s / Fly.io 対応）

## 概要

現状の `fav deploy` は AWS Lambda のみをターゲットとしている（v4.11.0）。
v22.8.0 では `--target` フラグを拡張し、**AWS ECS Fargate・Kubernetes CronJob・Fly.io**
の 3 プラットフォームへのデプロイをサポートする。

デプロイ設定は `fav.toml` の `[deploy]` セクションで管理し、
CLI からは `--target` フラグで上書きできる。

**テーマ**: 「コンテナベース実行環境への Favnir パイプラインのデプロイを標準化する」

---

## ロードマップ完了条件との対応

v22.8.0 は Distributed Scale ロードマップ（v22.1〜v23.0）の第八弾。
ロードマップ v22.8「`fav deploy` 強化（ECS / EKS 対応）」を実装する。

---

## 機能仕様

### CLI

```bash
fav deploy --target ecs  src/pipeline.fav   # AWS ECS Fargate
fav deploy --target k8s  src/pipeline.fav   # Kubernetes CronJob
fav deploy --target fly  src/pipeline.fav   # Fly.io
fav deploy --target ecs  --dry-run src/pipeline.fav   # ドライラン
```

既存の Lambda ターゲット:
```bash
fav deploy --target aws-lambda src/pipeline.fav   # 既存（変更なし）
fav deploy src/pipeline.fav                        # デフォルト: aws-lambda（後方互換）
```

### `fav.toml` `[deploy]` セクション拡張

```toml
[deploy]
target = "ecs"              # "aws-lambda" | "ecs" | "k8s" | "fly"
region = "ap-northeast-1"   # AWS リージョン（ecs 用）
cpu    = "1024"             # vCPU 単位（ecs 用、256/512/1024/2048/4096）
memory = "2048"             # MB（ecs 用）
cluster = "my-cluster"      # ECS クラスター名（ecs 用、省略時 "fav-cluster"）
namespace = "default"       # K8s namespace（k8s 用）
schedule  = "0 2 * * *"    # cron 式（k8s 用、CronJob schedule）
app       = "my-fav-app"   # Fly.io app 名（fly 用）
```

既存フィールド（後方互換維持）:
```toml
[deploy]
target     = "aws-lambda"      # デフォルト（既存）
runtime    = "provided.al2023" # Lambda 用（既存）
memory     = 256               # Lambda 用 MB（既存）
timeout    = 30                # Lambda 用 s（既存）
s3_bucket  = "my-bucket"       # Lambda 用（既存）
role_arn   = "arn:aws:..."     # Lambda 用（既存）
region     = "us-east-1"       # 全ターゲット共通（既存）
```

### デプロイフロー

#### ECS Fargate

1. **Step 1**: プロジェクトを Docker イメージとしてパッケージ化（`Dockerfile` を生成）
2. **Step 2**: ECR にプッシュ（リポジトリ名: `{project_name}`、タグ: タイムスタンプ）
3. **Step 3**: ECS タスク定義を生成・登録（`RegisterTaskDefinition` 相当の JSON 出力）
4. **Step 4**: ECS サービス更新または ECS Run Task 実行

> v22.8.0 では実際の AWS API 呼び出しは行わず、**Dockerfile / タスク定義 JSON / ECS CLIコマンド**を標準出力に生成するドライラン形式とする（実 API 統合は v23.x）。

#### Kubernetes CronJob

1. **Step 1**: `Dockerfile` を生成（ECS と共通）
2. **Step 2**: `CronJob` マニフェスト YAML を生成（`{project_name}-cronjob.yaml`）
3. **Step 3**: 適用コマンド（`kubectl apply -f {file}`）を出力

> v22.8.0 では YAML 生成のみ。`kubectl` 実行は行わない。

#### Fly.io

1. **Step 1**: `fly.toml` を生成（app 名・リージョン）
2. **Step 2**: `Dockerfile` を生成
3. **Step 3**: デプロイコマンド（`flyctl deploy`）を出力

> v22.8.0 では設定ファイル生成のみ。`flyctl` 実行は行わない。

### 生成ファイル

| ターゲット | 生成ファイル | 出力先 |
|---|---|---|
| `ecs` | `Dockerfile`（共通）、`ecs-task-def.json` | `--out-dir`（デフォルト `.fav-deploy/`） |
| `k8s` | `Dockerfile`（共通）、`{name}-cronjob.yaml` | `.fav-deploy/` |
| `fly` | `Dockerfile`（共通）、`fly.toml` | `.fav-deploy/` |

### 生成される `Dockerfile` テンプレート

```dockerfile
FROM debian:bookworm-slim
WORKDIR /app
COPY . .
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
# fav binary（ビルド済み）をコピー
COPY ./fav /usr/local/bin/fav
RUN chmod +x /usr/local/bin/fav
ENTRYPOINT ["fav", "run", "pipeline.fav"]
```

### ECS タスク定義 JSON テンプレート（`ecs-task-def.json`）

```json
{
  "family": "{project_name}",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "{cpu}",
  "memory": "{memory}",
  "executionRoleArn": "{role_arn}",
  "containerDefinitions": [{
    "name": "{project_name}",
    "image": "{ecr_repo}:{tag}",
    "essential": true,
    "logConfiguration": {
      "logDriver": "awslogs",
      "options": {
        "awslogs-group": "/fav/{project_name}",
        "awslogs-region": "{region}",
        "awslogs-stream-prefix": "fav"
      }
    }
  }]
}
```

### K8s CronJob YAML テンプレート

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: {project_name}
  namespace: {namespace}
spec:
  schedule: "{schedule}"
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: {project_name}
            image: {image}:{tag}
            imagePullPolicy: Always
          restartPolicy: OnFailure
```

### Fly.io `fly.toml` テンプレート

```toml
app = "{app_name}"
primary_region = "{region}"

[build]
  dockerfile = "Dockerfile"

[http_service]
  internal_port = 8080
  force_https   = true
```

---

## アーキテクチャ

### `fav/src/toml.rs` 変更

`DeployConfig` に新フィールドを追加（すべて `Option`、既存フィールドは変更なし）:

```rust
pub struct DeployConfig {
    // 既存フィールド（変更なし）
    pub target: String,         // "aws-lambda" | "ecs" | "k8s" | "fly"
    // ...

    // v22.8.0 追加フィールド
    pub cpu:       Option<String>,  // ECS vCPU（"256"|"512"|"1024"|"2048"|"4096"）
    pub cluster:   Option<String>,  // ECS クラスター名
    pub namespace: Option<String>,  // K8s namespace
    pub schedule:  Option<String>,  // K8s CronJob schedule（cron 式）
    pub app:       Option<String>,  // Fly.io app 名
    pub out_dir:   Option<String>,  // 生成ファイル出力先（デフォルト ".fav-deploy/"）
}
```

### `fav/src/driver.rs` 変更

#### `cmd_deploy_ecs` 追加

```rust
#[cfg(not(target_arch = "wasm32"))]
pub fn cmd_deploy_ecs(
    project_name: &str,
    deploy_cfg: &DeployConfig,
    dry_run: bool,
    out_dir: &str,
)
```

#### `cmd_deploy_k8s` 追加

```rust
#[cfg(not(target_arch = "wasm32"))]
pub fn cmd_deploy_k8s(
    project_name: &str,
    deploy_cfg: &DeployConfig,
    dry_run: bool,
    out_dir: &str,
)
```

#### `cmd_deploy_fly` 追加

```rust
#[cfg(not(target_arch = "wasm32"))]
pub fn cmd_deploy_fly(
    project_name: &str,
    deploy_cfg: &DeployConfig,
    dry_run: bool,
    out_dir: &str,
)
```

#### `cmd_deploy` 修正

既存 `cmd_deploy` に `target` パラメータ追加、ターゲットで分岐:

```rust
pub fn cmd_deploy(
    env: Option<&str>,
    function_name: Option<&str>,
    region: Option<&str>,
    dry_run: bool,
    target: Option<&str>,  // v22.8.0: 追加。None の場合 fav.toml [deploy].target を使用
    out_dir: Option<&str>, // v22.8.0: 追加。生成ファイル出力先
)
```

ターゲット分岐:

```rust
match effective_target.as_str() {
    "ecs"        => cmd_deploy_ecs(project_name, &deploy_cfg, dry_run, out_dir),
    "k8s"        => cmd_deploy_k8s(project_name, &deploy_cfg, dry_run, out_dir),
    "fly"        => cmd_deploy_fly(project_name, &deploy_cfg, dry_run, out_dir),
    "aws-lambda" => { /* 既存 Lambda ロジック */ }
    other        => { eprintln!("error: unknown deploy target `{}`", other); process::exit(1); }
}
```

### ヘルパー関数（`driver.rs` 内）

#### `generate_dockerfile` — 全ターゲット共通

```rust
fn generate_dockerfile(project_name: &str) -> String
```

#### `generate_ecs_task_def` — ECS 専用

```rust
fn generate_ecs_task_def(project_name: &str, deploy_cfg: &DeployConfig) -> String
```

#### `generate_k8s_cronjob` — K8s 専用

```rust
fn generate_k8s_cronjob(project_name: &str, deploy_cfg: &DeployConfig) -> String
```

#### `generate_fly_toml` — Fly.io 専用

```rust
fn generate_fly_toml(project_name: &str, deploy_cfg: &DeployConfig) -> String
```

### `fav/src/main.rs` 変更

`Some("deploy")` ブランチに `--target` と `--out-dir` フラグを追加:

```rust
"--target" => { target = Some(args[i + 1].clone()); i += 2; }
"--out-dir" => { out_dir = Some(args[i + 1].clone()); i += 2; }
```

---

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/toml.rs` | 更新 | `DeployConfig` に `cpu`/`cluster`/`namespace`/`schedule`/`app`/`out_dir` 追加 |
| `fav/src/driver.rs` | 更新 | `cmd_deploy` 拡張（target 分岐）、`cmd_deploy_ecs`/`cmd_deploy_k8s`/`cmd_deploy_fly` 追加、ヘルパー 4 件 |
| `fav/src/main.rs` | 更新 | `--target` / `--out-dir` フラグ追加、`cmd_deploy` 呼び出し変更 |
| `fav/Cargo.toml` | 更新 | `version = "22.7.0"` → `"22.8.0"` |
| `CHANGELOG.md` | 更新 | v22.8.0 エントリ追加 |
| `benchmarks/v22.8.0.json` | 新規 | ベンチマーク結果 |
| `site/content/docs/cli/deploy.mdx` | 新規 | deploy コマンドドキュメント |

---

## テスト一覧（v228000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_22_8_0` | Cargo.toml に `version = "22.8.0"` が含まれる |
| `deploy_ecs_generates_dockerfile` | `cmd_deploy_ecs` が Dockerfile を生成し `FROM debian` を含む |
| `deploy_k8s_generates_cronjob_yaml` | `cmd_deploy_k8s` が `CronJob` YAML を生成する |
| `deploy_fly_generates_fly_toml` | `cmd_deploy_fly` が `fly.toml` を生成し app 名を含む |
| `changelog_has_v22_8_0` | CHANGELOG.md に `[v22.8.0]` が含まれる |

---

## スコープ外（v22.8.0 では実装しない）

- 実際の AWS API / ECR プッシュ（`aws ecr put-image` 等）の呼び出し
- `kubectl apply` の実行
- `flyctl deploy` の実行
- Docker ビルドの自動実行
- `fav.toml` の `[deploy.ecs]` / `[deploy.k8s]` サブセクション（単一 `[deploy]` で管理）
- ECS サービス（Service）の作成（Run Task のみ）
- Kubernetes Deployment（CronJob のみ）
- Helm chart 生成
- `fav new` テンプレートへの `[deploy]` セクション新フィールド反映（v23.x で対応予定）

---

## アーキテクチャ注意事項

### `#[cfg(not(target_arch = "wasm32"))]` ガード

以下のすべてに `#[cfg(not(target_arch = "wasm32"))]` が必要:
- `cmd_deploy_ecs` / `cmd_deploy_k8s` / `cmd_deploy_fly`
- `write_deploy_file`（`std::fs` を使用するため）
- `generate_dockerfile` / `generate_ecs_task_def` / `generate_k8s_cronjob` / `generate_fly_toml`
- `cmd_deploy` 内のターゲット分岐（`cfg!()` マクロを使用）

### `cmd_deploy` 内のターゲット分岐

`#[cfg]` アトリビュートは `match` 文全体に付けられないため、`cfg!()` マクロを使用する:

```rust
// v22.8.0: コンテナターゲットへの分岐（native only）
if cfg!(not(target_arch = "wasm32")) {
    match effective_target {
        "ecs" => { cmd_deploy_ecs(...); return; }
        "k8s" => { cmd_deploy_k8s(...); return; }
        "fly" => { cmd_deploy_fly(...); return; }
        _ => {} // fall through to Lambda logic
    }
}
```

### ECS タスク定義 JSON のプレースホルダー

`generate_ecs_task_def` で生成される JSON の image フィールドには
`<ECR_REPO>/{name}:latest` というプレースホルダーを使用する
（実 ECR URI の統合は v23.x で対応）。

### `tempfile` クレート

テストで `TempDir` を使用するが、`tempfile = "3"` は既に native-only dependencies と
dev-dependencies に存在するため追加不要。

---

## 完了条件

- [ ] `DeployConfig` に新フィールドが追加される（後方互換維持）
- [ ] `cmd_deploy_ecs` が Dockerfile + ECS タスク定義 JSON（project_name を family として含む）を `.fav-deploy/` に生成する
- [ ] `cmd_deploy_k8s` が Dockerfile + CronJob YAML を `.fav-deploy/` に生成する
- [ ] `cmd_deploy_fly` が Dockerfile + `fly.toml`（app 名を含む）を `.fav-deploy/` に生成する
- [ ] `cmd_deploy` が `--target ecs/k8s/fly` で各関数に分岐する（`cfg!()` マクロ使用）
- [ ] `--dry-run` 時はファイルを書かずコンソール出力のみ
- [ ] `main.rs` に `--target` / `--out-dir` フラグが追加される
- [ ] `fav deploy --help` 出力に `--target` / `--out-dir` が含まれる
- [ ] 既存の Lambda デプロイ（`fav deploy` デフォルト）が後方互換を維持する
- [ ] すべての新規関数に `#[cfg(not(target_arch = "wasm32"))]` ガードが付く
- [ ] `cargo test v228000 --bin fav` — 5/5 PASS
- [ ] `cargo test --bin fav` — リグレッションなし（1878 件以上合格）
- [ ] `CHANGELOG.md` に v22.8.0 エントリ
- [ ] `benchmarks/v22.8.0.json` 作成済み
- [ ] `site/content/docs/cli/deploy.mdx` 作成済み
