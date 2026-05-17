# Favnir v5.0.0 仕様書 — AWS 本番稼働 + リファレンスサイト

作成日: 2026-05-17

---

## 概要

v4.x で構築した全ピース（AWS SDK・fav deploy・Rune Registry・LSP・MCP・Notebook）を使い、
Favnir 自身を AWS 上に乗せる最初の本番リリース。

**優先フェーズ**: リファレンスサイト（Phase C）を最初に完成させ、公開する。
その後 WASM Playground（Phase B）→ CI/CD（Phase A）→ Dogfooding（Phase D）の順に進める。

---

## Phase A: CI/CD（GitHub Actions）

### 対象ファイル

```
.github/
  workflows/
    ci.yml       ← fav check + fav test（PR ごと）
    deploy.yml   ← build → ECR push → fav deploy（main merge）
```

### ci.yml

```yaml
name: CI
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Favnir
        run: cargo install --path fav/
      - name: fav check
        run: fav check fav/src/
      - name: fav test
        run: fav test fav/src/
```

### deploy.yml

```yaml
name: Deploy
on:
  push:
    branches: [main]
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          role-to-assume: ${{ secrets.AWS_ROLE_ARN }}
          aws-region: ap-northeast-1
      - name: Login to ECR
        id: login-ecr
        uses: aws-actions/amazon-ecr-login@v2
      - name: Build & push Docker image
        run: |
          docker build -t ${{ steps.login-ecr.outputs.registry }}/favnir-registry:$GITHUB_SHA .
          docker push ${{ steps.login-ecr.outputs.registry }}/favnir-registry:$GITHUB_SHA
      - name: Deploy to ECS
        run: fav deploy --env prod
      - name: Deploy reference site to S3
        run: |
          cd site && npm run build
          aws s3 sync out/ s3://${{ secrets.SITE_BUCKET }} --delete
          aws cloudfront create-invalidation \
            --distribution-id ${{ secrets.CF_DISTRIBUTION_ID }} \
            --paths "/*"
```

---

## Phase B: `@favnir/wasm` — ブラウザ内ランタイム

> **注**: 既存の `fav build --wasm`（Favnir プログラム → WASM）とは別物。
> Favnir の**ランタイム自体**を WASM にコンパイルし JS から呼び出す。

### ディレクトリ構成

```
crates/
  favnir-wasm/
    Cargo.toml   ← crate-type = ["cdylib"]
    src/
      lib.rs
```

### エクスポート API

```rust
// crates/favnir-wasm/src/lib.rs
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct RunResult {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

#[derive(Serialize)]
pub struct CheckResult {
    pub errors: Vec<CheckError>,
    pub success: bool,
}

#[derive(Serialize)]
pub struct CheckError {
    pub code: String,
    pub message: String,
    pub line: u32,
    pub col: u32,
}

#[wasm_bindgen]
pub fn fav_check(source: &str) -> JsValue {
    // checker を呼び出して CheckResult を返す
}

#[wasm_bindgen]
pub fn fav_run(source: &str) -> JsValue {
    // !Io のみ許可、!Aws / !Db / !File は拒否
    // stdout をキャプチャして RunResult を返す
}
```

### npm パッケージ設定

```toml
# crates/favnir-wasm/Cargo.toml
[package]
name    = "favnir-wasm"
version = "5.0.0"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
serde        = { version = "1", features = ["derive"] }
serde-wasm-bindgen = "0.6"
```

```json
// crates/favnir-wasm/package.json
{
  "name": "@favnir/wasm",
  "version": "5.0.0",
  "files": ["pkg/"],
  "main": "./pkg/favnir_wasm.js",
  "types": "./pkg/favnir_wasm.d.ts"
}
```

### ビルドコマンド

```bash
cd crates/favnir-wasm
wasm-pack build --target web --out-dir ../../site/public/wasm
```

### サンドボックスポリシー

| エフェクト | Playground | 理由 |
|-----------|-----------|------|
| `!Io` (stdout/println) | 許可 | 出力をキャプチャ |
| `!File` | 拒否 | ブラウザ環境 |
| `!Db` | 拒否 | サーバー不要設計 |
| `!Aws` | 拒否 | クレデンシャル不要 |
| `!Network` | 拒否 | CORS / セキュリティ |
| `!Auth` | 拒否 | シークレット不要 |

---

## Phase C: リファレンスサイト（最優先）

### 技術スタック

- **フレームワーク**: Next.js 16 (`output: 'export'` で静的生成)
- **UI**: shadcn/ui + Tailwind v4（`mock/` をベースに更新）
- **コンテンツ**: MDX（`@next/mdx` + `gray-matter`）
- **デプロイ**: AWS S3 + CloudFront

### ディレクトリ構成

```
site/
  package.json
  next.config.ts
  tailwind.config.ts
  components.json
  public/
    images/
      favnir-mascot.png
    wasm/              ← @favnir/wasm ビルド出力
  content/
    docs/
      introduction.mdx
      installation.mdx
      quickstart.mdx
      language/
        types.mdx
        effects.mdx
        runes.mdx
        pattern-matching.mdx
      stdlib/
        io.mdx
        list.mdx
        map.mdx
        string.mdx
        result.mdx
        option.mdx
    errors/
      E0001.mdx
      E0100.mdx
      ...
  app/
    layout.tsx
    page.tsx                     ← ランディングページ
    docs/
      layout.tsx                 ← サイドバー付きレイアウト
      [...slug]/
        page.tsx                 ← MDX コンテンツ表示
    errors/
      page.tsx                   ← エラーカタログ一覧
      [code]/
        page.tsx                 ← エラー詳細
    runes/
      page.tsx                   ← Rune カタログ
    playground/
      page.tsx                   ← Live Playground（@favnir/wasm 使用）
  components/
    layout/
      header.tsx                 ← mock/ の Header を更新
      footer.tsx
      sidebar.tsx                ← docs サイドバー
    landing/
      hero.tsx                   ← mock/ の Hero を更新（コード例を現行構文に）
      features.tsx               ← mock/ の Features を更新
      docs-preview.tsx
      cta.tsx
    docs/
      mdx-components.tsx         ← MDX レンダラー
      code-block.tsx             ← シンタックスハイライト
      toc.tsx                    ← 目次
    playground/
      editor.tsx                 ← コードエディタ（textarea ベース）
      output.tsx                 ← 実行出力表示
```

### ランディングページのコードプレビュー更新

mock/ の古い構文（`pipeline / source / transform / sink / redshift`）を
現行の Favnir 構文に書き換える:

```favnir
// 旧（mock/ の proto 構文）
pipeline user_analytics {
  source s3("s3://data-lake/raw/")
  transform filter(active_users)
  transform aggregate(daily_metrics)
  sink redshift("analytics.user_stats")
}

// 新（現行 Favnir 構文）
import rune "aws"
import rune "duckdb"

type Order = { id: Int customer: String amount: Float }

public fn pipeline() -> Unit !Io !Aws {
  bind orders <- aws.s3.read_csv<Order>("data-lake", "raw/orders.csv")
  bind conn   <- duckdb.open(":memory:")
  bind _      <- gen.load_into(conn, "orders", orders)
  bind result <- duckdb.query<Summary>(conn,
    "SELECT customer, SUM(amount) FROM orders GROUP BY customer")
  IO.println(result)
}
```

### ページ仕様

#### ランディングページ (`app/page.tsx`)

- **Header**: Favnir マスコット + ナビゲーション（ドキュメント・エラー・Rune・Playground・GitHub）
- **Hero**: キャッチコピー「型安全なデータパイプライン専用言語」+ コードプレビュー（現行構文）
- **Features**: 4 カード（型安全・エフェクト型・Rune エコシステム・AWS ネイティブ）
- **DocsPreview**: ドキュメントカテゴリ一覧リンク
- **CTA**: GitHub / ドキュメントボタン
- **Footer**: コピーライト + リンク

#### ドキュメント (`app/docs/[...slug]/page.tsx`)

```
はじめに
  - イントロダクション
  - インストール
  - クイックスタート

言語仕様
  - 型システム（プリミティブ型・Record・List・Option・Result）
  - エフェクト型（!Io / !File / !Db / !Network / !Auth / !AWS / !Env）
  - パターンマッチ
  - Rune（import rune / use / マルチファイル）

標準ライブラリ
  - IO / List / Map / String / Result / Option / Int / Float

Rune
  - DB Rune / HTTP Rune / DuckDB Rune / Gen Rune
  - AWS Rune / Auth Rune / Log Rune / Env Rune
```

MDX ファイルからフロントマターで順序・タイトルを管理:

```mdx
---
title: "イントロダクション"
order: 1
category: "はじめに"
---

# Favnir とは

Favnir はデータエンジニアのための型安全なパイプライン専用言語です。
```

#### エラーカタログ (`app/errors/[code]/page.tsx`)

`fav explain --format json` コマンド（新規追加）で全エラーを JSON 出力し、
ビルド時に静的ページを生成する。

```bash
# ビルドスクリプトで実行
fav explain --all --format json > content/errors/catalog.json
```

```json
{
  "E0001": {
    "code": "E0001",
    "title": "Undefined variable",
    "description": "...",
    "example": "...",
    "fix": "..."
  }
}
```

#### Rune カタログ (`app/runes/page.tsx`)

各 Rune の API を静的コンテンツとして表示:

```
runes/
  page.tsx         ← カタログ一覧
  [name]/
    page.tsx       ← Rune API リファレンス（MDX）
```

content/docs/runes/*.mdx から生成。

#### Playground (`app/playground/page.tsx`)

```typescript
'use client'
import init, { fav_check, fav_run } from '../../../public/wasm/favnir_wasm'

export default function Playground() {
  const [code, setCode] = useState(EXAMPLE_CODE)
  const [output, setOutput] = useState('')
  const [errors, setErrors] = useState([])
  const [wasmReady, setWasmReady] = useState(false)

  useEffect(() => {
    init().then(() => setWasmReady(true))
  }, [])

  const handleRun = () => {
    const result = fav_run(code)
    setOutput(result.stdout)
    setErrors([])
  }

  const handleCheck = () => {
    const result = fav_check(code)
    setErrors(result.errors)
  }

  // ...
}
```

サンプルコード（Playground のデフォルト値）:

```favnir
// Favnir Playground へようこそ
// !Io エフェクトのみ利用可能です

type Point = { x: Int y: Int }

fn distance(a: Point, b: Point) -> Float {
  let dx = a.x - b.x
  let dy = a.y - b.y
  Float.sqrt(Int.to_float(dx * dx + dy * dy))
}

public fn main() -> Unit !Io {
  let p1 = Point { x: 0 y: 0 }
  let p2 = Point { x: 3 y: 4 }
  IO.println(distance(p1, p2))
}
```

### `fav explain` コマンド（新規追加）

エラーカタログをビルド時に JSON 生成するため `fav explain` を追加:

```
fav explain E0001              # 単一エラーの詳細を表示
fav explain --all              # 全エラーを表示
fav explain --all --format json # JSON で出力（サイトビルド用）
```

### デプロイ構成

```
AWS
  S3 バケット: favnir-site-prod
    - パブリックアクセス: なし
    - バケットポリシー: CloudFront OAC のみ許可
  CloudFront ディストリビューション
    - オリジン: S3 バケット（OAC）
    - エラーページ: 404 → /404.html（Next.js 静的エクスポート対応）
    - キャッシュ: デフォルト TTL 86400s、/wasm/* は immutable
    - カスタムドメイン: favnir.dev（Route 53 + ACM）

next.config.ts:
  output: 'export'
  trailingSlash: true
  images: { unoptimized: true }   ← S3 静的配信では next/image 最適化不要
```

### Terraform リソース（最小構成）

```hcl
# infra/site/main.tf
resource "aws_s3_bucket" "site" {
  bucket = "favnir-site-prod"
}

resource "aws_cloudfront_distribution" "site" {
  origin {
    domain_name              = aws_s3_bucket.site.bucket_regional_domain_name
    origin_access_control_id = aws_cloudfront_origin_access_control.site.id
    origin_id                = "s3-favnir-site"
  }
  default_root_object = "index.html"
  # ... price_class, cache_behavior, etc.
}
```

---

## Phase D: Dogfooding

### Rune Registry サーバー（Favnir HTTP サービス）

v5.0.0 時点ではシンプルな HTTP サービスとして実装:

```
rune-registry/
  fav.toml
  src/
    main.fav       ← HTTP サーバー（http rune 使用）
    handlers.fav   ← list / info / publish / install ハンドラ
    storage.fav    ← S3 + DynamoDB 操作（aws rune 使用）
```

---

## `fav explain` コマンド仕様

### CLI

```
fav explain <code>           # E0001 など単一エラー詳細
fav explain --all            # 全エラー一覧（テキスト）
fav explain --all --format json > catalog.json
```

### 出力フォーマット（JSON）

```json
[
  {
    "code": "E0001",
    "title": "Undefined variable",
    "category": "name_resolution",
    "description": "変数が定義されていません。",
    "example_bad": "fn main() -> Unit !Io {\n  IO.println(x)\n}",
    "example_good": "fn main() -> Unit !Io {\n  let x = 42\n  IO.println(x)\n}",
    "fix": "変数を使用前に定義するか、スペルを確認してください。"
  }
]
```

### 実装箇所

`fav/src/driver.rs` に `cmd_explain(code: Option<&str>, all: bool, format: &str)`、
`fav/src/main.rs` に `Some("explain")` アームを追加。

エラー定義は `fav/src/middle/checker.rs` の既存エラーコード定数から自動収集。

---

## 完了条件

### Phase A（CI/CD）
- [ ] `.github/workflows/ci.yml` が PR ごとに `fav check` + `fav test` を実行する
- [ ] `.github/workflows/deploy.yml` が main merge 時に ECR push + ECS deploy + S3 sync を実行する

### Phase B（@favnir/wasm）
- [ ] `wasm-pack build` が成功し `site/public/wasm/` に出力される
- [ ] `fav_check(source)` が JS から呼び出せる
- [ ] `fav_run(source)` が stdout をキャプチャして返す（`!Io` のみ許可）
- [ ] npm パッケージとして `@favnir/wasm@5.0.0` が使える

### Phase C（リファレンスサイト）— 最優先
- [ ] `site/` が Next.js 16 プロジェクトとして `npm run build` で静的出力できる
- [ ] ランディングページが現行 Favnir 構文のコードプレビューを表示する
- [ ] ドキュメントページが MDX から生成される（最低 10 ページ）
- [ ] エラーカタログページが `fav explain --all --format json` から生成される
- [ ] Rune カタログページが全 8 Rune を表示する
- [ ] Playground ページが `@favnir/wasm` でブラウザ内実行できる
- [ ] `aws s3 sync` で S3 にデプロイされ CloudFront から配信される
- [ ] カスタムドメイン（favnir.dev）で HTTPS アクセスできる

### Phase D（Dogfooding）
- [ ] Rune Registry API サーバーが Favnir HTTP サービスとして AWS に稼働する
- [ ] `fav install csv` が AWS 上の Registry から動作する

### `fav explain` コマンド
- [ ] `fav explain E0001` が単一エラーの詳細を表示する
- [ ] `fav explain --all --format json` が全エラーの JSON を出力する
- [ ] テスト 5 件以上

---

## 実装メモ

- **mock/ との関係**: `mock/` のデザイン（ダークテーマ・マスコット・shadcn/ui 構成）を `site/` にそのまま移植し、内容を現行仕様に更新する。mock/ 自体は削除しない（ベースライン記録として保持）
- **Next.js バージョン**: mock/ が Next.js 16 を使用しているのでそのまま継承
- **画像アセット**: `mock/public/images/favnir-mascot.png` → `site/public/images/` にコピー
- **wasm-bindgen 版**: `0.2`（安定版）
- **`fav explain` の優先度**: Phase C（リファレンスサイト）の一部として実装する。エラーカタログページのビルドに必要
- **Terraform 範囲**: v5.0.0 では S3 + CloudFront の最小構成のみ。ECS / ECR / Secrets Manager の詳細設計は Phase D に合わせて策定
- **ドメイン**: favnir.dev（Route 53 + ACM）は Phase C 完了後に設定
