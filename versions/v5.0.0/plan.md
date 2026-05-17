# Favnir v5.0.0 実装計画 — AWS 本番稼働 + リファレンスサイト

作成日: 2026-05-17

実装順序: Phase C（リファレンスサイト）→ Phase B（WASM）→ Phase A（CI/CD）→ Phase D（Dogfooding）

---

## Phase C: リファレンスサイト（最優先）

### Step C-0: `fav explain` コマンド追加

リファレンスサイトのエラーカタログページに必要なため最初に実装する。

**`fav/src/driver.rs`** に追加:

```rust
pub fn cmd_explain(code_arg: Option<&str>, all: bool, format: &str) {
    // エラーカタログを収集
    let catalog = build_error_catalog();

    if all {
        if format == "json" {
            println!("{}", serde_json::to_string_pretty(&catalog).unwrap());
        } else {
            for entry in &catalog {
                println!("{}: {}", entry.code, entry.title);
                println!("  {}", entry.description);
            }
        }
        return;
    }

    if let Some(code) = code_arg {
        if let Some(entry) = catalog.iter().find(|e| e.code == code) {
            println!("[{}] {}", entry.code, entry.title);
            println!("{}", entry.description);
            if !entry.fix.is_empty() {
                println!("\nFix: {}", entry.fix);
            }
        } else {
            eprintln!("Unknown error code: {}", code);
            std::process::exit(1);
        }
        return;
    }

    eprintln!("Usage: fav explain <code> | --all [--format json]");
    std::process::exit(1);
}

#[derive(serde::Serialize)]
pub struct ErrorEntry {
    pub code: String,
    pub title: String,
    pub category: String,
    pub description: String,
    pub example_bad: String,
    pub example_good: String,
    pub fix: String,
}

fn build_error_catalog() -> Vec<ErrorEntry> {
    // checker.rs の定数からハードコードで収集
    // E0001〜E0313 の主要エラーを網羅
    vec![
        ErrorEntry {
            code: "E0001".to_string(),
            title: "Undefined variable".to_string(),
            category: "name_resolution".to_string(),
            description: "変数が定義されていません。".to_string(),
            example_bad: "fn main() -> Unit !Io {\n  IO.println(x)\n}".to_string(),
            example_good: "fn main() -> Unit !Io {\n  let x = 42\n  IO.println(x)\n}".to_string(),
            fix: "変数を使用前に定義するか、スペルを確認してください。".to_string(),
        },
        // ... 主要エラー分
    ]
}
```

**`fav/src/main.rs`** に追加:

```rust
Some("explain") => {
    let mut code_arg = None;
    let mut all = false;
    let mut format = "text".to_string();
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--all" => all = true,
            "--format" => { i += 1; if i < rest.len() { format = rest[i].clone(); } }
            arg if !arg.starts_with('-') => code_arg = Some(arg),
            _ => {}
        }
        i += 1;
    }
    driver::cmd_explain(code_arg, all, &format);
}
```

---

### Step C-1: `site/` プロジェクト初期化

```
site/
  package.json
  next.config.ts
  tsconfig.json
  tailwind.config.ts
  components.json        ← shadcn/ui 設定
  postcss.config.mjs
  public/
    images/
      favnir-mascot.png  ← mock/public/images/ からコピー
  app/
    globals.css
    layout.tsx
    page.tsx
  components/
    ui/                  ← shadcn/ui コンポーネント（mock/ からコピー）
```

**`site/package.json`** (mock/package.json をベースに調整):

```json
{
  "name": "favnir-site",
  "version": "5.0.0",
  "private": true,
  "scripts": {
    "dev": "next dev",
    "build": "next build",
    "start": "next start",
    "lint": "eslint ."
  },
  "dependencies": {
    "next": "16.2.6",
    "react": "^19",
    "react-dom": "^19",
    "@next/mdx": "^16.0.0",
    "gray-matter": "^4.0.3",
    "next-mdx-remote": "^5.0.0",
    "shiki": "^1.0.0",
    "lucide-react": "^0.564.0",
    "class-variance-authority": "^0.7.1",
    "clsx": "^2.1.1",
    "tailwind-merge": "^3.3.1",
    "@radix-ui/react-dialog": "1.1.15",
    "@radix-ui/react-slot": "1.2.4",
    "@radix-ui/react-tabs": "1.1.13",
    "@radix-ui/react-tooltip": "1.2.8",
    "@radix-ui/react-scroll-area": "1.2.10",
    "@radix-ui/react-separator": "1.1.8"
  },
  "devDependencies": {
    "@tailwindcss/postcss": "^4.2.0",
    "@types/node": "^22",
    "@types/react": "^19",
    "@types/react-dom": "^19",
    "postcss": "^8.5",
    "tailwindcss": "^4.2.0",
    "tw-animate-css": "1.3.3",
    "typescript": "5.7.3"
  }
}
```

**`site/next.config.ts`**:

```typescript
import type { NextConfig } from 'next'
import createMDX from '@next/mdx'

const withMDX = createMDX({})

const nextConfig: NextConfig = {
  output: 'export',
  trailingSlash: true,
  pageExtensions: ['js', 'jsx', 'ts', 'tsx', 'md', 'mdx'],
  images: {
    unoptimized: true,
  },
}

export default withMDX(nextConfig)
```

---

### Step C-2: コンポーネント移植（mock/ → site/）

mock/components/landing.tsx の各コンポーネントを site/components/landing/ に分割移植。
更新箇所:

1. **Hero コードプレビュー**: 旧 `pipeline/source/transform/sink/redshift` 構文を現行 Favnir 構文に書き換え
2. **Header ナビゲーション**: `ドキュメント / エラー / Rune / Playground / GitHub` に変更
3. **Features**: 「宣言的構文」→「エフェクト型システム」、「高速実行」→「Rune エコシステム」に更新
4. **Badge**: `v0.1.0 Beta` → `v5.0.0` に更新

---

### Step C-3: ドキュメント MDX コンテンツ

`site/content/docs/` に MDX ファイルを作成。最低限の構成:

```
content/docs/
  introduction.mdx           ← Favnir とは / 設計思想
  installation.mdx           ← cargo install / fav --help
  quickstart.mdx             ← Hello World → DuckDB pipeline まで
  language/
    types.mdx                ← プリミティブ / Record / List / Option / Result
    effects.mdx              ← !Io / !File / !Db / !Network / !AWS / !Auth / !Env
    pattern-matching.mdx     ← match / Option / Result パターン
    runes.mdx                ← import rune / use / マルチファイル rune
  stdlib/
    io.mdx                   ← IO.println / IO.read_line
    list.mdx                 ← List.map / filter / fold / any / len
    map.mdx                  ← Map.get / set / keys
    string.mdx               ← String.split / contains / format
    result.mdx               ← Result.ok / err / map / and_then
    option.mdx               ← Option.some / none / unwrap_or
  runes/
    db.mdx
    duckdb.mdx
    http.mdx
    aws.mdx
    auth.mdx
    log.mdx
    env.mdx
    gen.mdx
```

フロントマター形式:

```mdx
---
title: "イントロダクション"
order: 1
category: "はじめに"
description: "Favnir はデータエンジニアのための型安全なパイプライン専用言語です"
---
```

ドキュメントページの実装:

```typescript
// site/app/docs/[...slug]/page.tsx
import { getDocBySlug, getAllDocs } from '@/lib/docs'
import { MDXContent } from '@/components/docs/mdx-components'

export async function generateStaticParams() {
  const docs = await getAllDocs()
  return docs.map(doc => ({ slug: doc.slug.split('/') }))
}

export default async function DocPage({ params }) {
  const { slug } = await params
  const doc = await getDocBySlug(slug.join('/'))
  return (
    <div className="prose prose-invert max-w-none">
      <h1>{doc.frontmatter.title}</h1>
      <MDXContent source={doc.content} />
    </div>
  )
}
```

---

### Step C-4: エラーカタログページ

ビルド前に `fav explain --all --format json > site/content/errors/catalog.json` を実行し、
それを静的ページ生成に使う。

```typescript
// site/lib/errors.ts
import catalog from '@/content/errors/catalog.json'

export function getAllErrors() {
  return catalog as ErrorEntry[]
}

export function getErrorByCode(code: string) {
  return catalog.find((e: ErrorEntry) => e.code === code)
}
```

```typescript
// site/app/errors/[code]/page.tsx
import { getAllErrors, getErrorByCode } from '@/lib/errors'
import { CodeBlock } from '@/components/docs/code-block'

export async function generateStaticParams() {
  return getAllErrors().map(e => ({ code: e.code }))
}

export default async function ErrorPage({ params }) {
  const { code } = await params
  const entry = getErrorByCode(code)
  // ...
}
```

---

### Step C-5: Rune カタログページ

各 Rune の API を MDX から生成。`site/app/runes/[name]/page.tsx`:

```typescript
export async function generateStaticParams() {
  return ['db', 'duckdb', 'http', 'aws', 'auth', 'log', 'env', 'gen']
    .map(name => ({ name }))
}
```

---

### Step C-6: S3 + CloudFront デプロイ（Terraform）

`infra/site/` に Terraform ファイルを作成:

```
infra/
  site/
    main.tf
    variables.tf
    outputs.tf
    providers.tf
```

**`infra/site/main.tf`**:

```hcl
terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
  backend "s3" {
    bucket = "favnir-terraform-state"
    key    = "site/terraform.tfstate"
    region = "ap-northeast-1"
  }
}

provider "aws" {
  region = var.aws_region
}

# CloudFront には us-east-1 の ACM が必要
provider "aws" {
  alias  = "us_east_1"
  region = "us-east-1"
}

resource "aws_s3_bucket" "site" {
  bucket = var.site_bucket_name
}

resource "aws_s3_bucket_public_access_block" "site" {
  bucket = aws_s3_bucket.site.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_cloudfront_origin_access_control" "site" {
  name                              = "favnir-site-oac"
  origin_access_control_origin_type = "s3"
  signing_behavior                  = "always"
  signing_protocol                  = "sigv4"
}

resource "aws_cloudfront_distribution" "site" {
  enabled             = true
  default_root_object = "index.html"
  price_class         = "PriceClass_100"  # 北米 + 欧州 + 日本

  origin {
    domain_name              = aws_s3_bucket.site.bucket_regional_domain_name
    origin_id                = "s3-favnir-site"
    origin_access_control_id = aws_cloudfront_origin_access_control.site.id
  }

  default_cache_behavior {
    allowed_methods        = ["GET", "HEAD"]
    cached_methods         = ["GET", "HEAD"]
    target_origin_id       = "s3-favnir-site"
    viewer_protocol_policy = "redirect-to-https"
    compress               = true

    forwarded_values {
      query_string = false
      cookies { forward = "none" }
    }

    min_ttl     = 0
    default_ttl = 86400
    max_ttl     = 31536000
  }

  # WASM ファイルはキャッシュを長期に
  ordered_cache_behavior {
    path_pattern     = "/wasm/*"
    allowed_methods  = ["GET", "HEAD"]
    cached_methods   = ["GET", "HEAD"]
    target_origin_id = "s3-favnir-site"
    viewer_protocol_policy = "redirect-to-https"

    forwarded_values {
      query_string = false
      cookies { forward = "none" }
    }

    min_ttl     = 31536000
    default_ttl = 31536000
    max_ttl     = 31536000
  }

  custom_error_response {
    error_code         = 404
    response_code      = 404
    response_page_path = "/404.html"
  }

  custom_error_response {
    error_code         = 403
    response_code      = 404
    response_page_path = "/404.html"
  }

  restrictions {
    geo_restriction { restriction_type = "none" }
  }

  viewer_certificate {
    cloudfront_default_certificate = true
    # カスタムドメイン設定後に ACM ARN を追加
  }
}

# S3 バケットポリシー（CloudFront OAC のみ許可）
resource "aws_s3_bucket_policy" "site" {
  bucket = aws_s3_bucket.site.id
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Service = "cloudfront.amazonaws.com" }
      Action    = "s3:GetObject"
      Resource  = "${aws_s3_bucket.site.arn}/*"
      Condition = {
        StringEquals = {
          "AWS:SourceArn" = aws_cloudfront_distribution.site.arn
        }
      }
    }]
  })
}
```

**`infra/site/outputs.tf`**:

```hcl
output "cloudfront_domain" {
  value = aws_cloudfront_distribution.site.domain_name
}

output "site_bucket" {
  value = aws_s3_bucket.site.bucket
}

output "distribution_id" {
  value = aws_cloudfront_distribution.site.id
}
```

---

## Phase B: `@favnir/wasm`

### Step B-1: `crates/favnir-wasm/` 作成

```toml
# crates/favnir-wasm/Cargo.toml
[package]
name    = "favnir-wasm"
version = "5.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen       = "0.2"
serde              = { version = "1", features = ["derive"] }
serde-wasm-bindgen = "0.6"
# favnir コア（workspace member）
fav = { path = "../../fav", default-features = false }
```

### Step B-2: `lib.rs` 実装

```rust
use wasm_bindgen::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
pub struct RunResult {
    pub stdout: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct CheckResult {
    pub errors: Vec<DiagnosticItem>,
    pub success: bool,
}

#[derive(Serialize)]
pub struct DiagnosticItem {
    pub code: String,
    pub message: String,
    pub line: u32,
    pub col: u32,
}

#[wasm_bindgen]
pub fn fav_check(source: &str) -> JsValue {
    let result = fav::check_source(source);
    serde_wasm_bindgen::to_value(&result).unwrap()
}

#[wasm_bindgen]
pub fn fav_run(source: &str) -> JsValue {
    let result = fav::run_source_sandboxed(source);
    serde_wasm_bindgen::to_value(&result).unwrap()
}
```

### Step B-3: `fav` crate に WASM 向けエントリポイント追加

`fav/src/lib.rs`（新規）:

```rust
// WASM 向け公開 API
pub fn check_source(source: &str) -> wasm_types::CheckResult { ... }
pub fn run_source_sandboxed(source: &str) -> wasm_types::RunResult {
    // !File / !Db / !Aws / !Network / !Auth を拒否
    // stdout をキャプチャ（thread_local バッファ）
    ...
}
```

### Step B-4: ビルドスクリプト

```bash
# scripts/build-wasm.sh
cd crates/favnir-wasm
wasm-pack build --target web --out-dir ../../site/public/wasm --release
```

---

## Phase A: CI/CD

### Step A-1: `.github/workflows/ci.yml`

PR ごとに実行:
1. `cargo build`
2. `cargo test` (937 件)
3. `cd site && npm ci && npm run build`（サイトビルド確認）

### Step A-2: `.github/workflows/deploy.yml`

main merge 時に実行:
1. AWS credentials（OIDC）
2. ECR login → docker build → push
3. `fav deploy --env prod`（ECS rolling update）
4. `scripts/build-wasm.sh`（WASM ビルド）
5. `cd site && npm run build`
6. `aws s3 sync out/ s3://$SITE_BUCKET --delete`
7. CloudFront キャッシュ無効化

---

## Phase D: Dogfooding

### Step D-1: `rune-registry/` サービス

Favnir HTTP サービスとして Rune Registry API を実装:

```
rune-registry/
  fav.toml
  src/
    main.fav    ← Http.serve + ルーティング
    handlers.fav
    storage.fav
```

```toml
# rune-registry/fav.toml
[rune]
name    = "rune-registry"
version = "1.0.0"

[deploy]
target  = "ecs"
region  = "ap-northeast-1"
cluster = "favnir-prod"
```

---

## 実装順序まとめ

```
Phase C（最優先）
  C-0: fav explain コマンド（fav/src/driver.rs + main.rs）
  C-1: site/ プロジェクト初期化
  C-2: コンポーネント移植（mock/ → site/components/）
  C-3: MDX ドキュメントコンテンツ作成
  C-4: エラーカタログページ
  C-5: Rune カタログページ
  C-6: Terraform S3 + CloudFront 構築 → デプロイ

Phase B
  B-1: crates/favnir-wasm/ 作成
  B-2: lib.rs 実装（fav_check / fav_run）
  B-3: fav/src/lib.rs に WASM エントリポイント追加
  B-4: Playground ページ実装
  B-5: wasm-pack build → site/public/wasm/ 出力確認

Phase A
  A-1: .github/workflows/ci.yml
  A-2: .github/workflows/deploy.yml

Phase D
  D-1: rune-registry/ Favnir HTTP サービス実装
  D-2: ECS デプロイ
```
