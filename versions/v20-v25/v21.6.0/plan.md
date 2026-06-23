# v21.6.0 実装計画 — Playground v2

## タスク順序

```
T1: share-url.ts — encodeCode / decodeCode ユーティリティ（サーバーレス URL）
T2: playground-templates.ts — 6 テンプレート定義
T3: page.tsx 更新 — Share ボタン・テンプレート UI・実行統計（T1, T2 後）
T4: share Lambda — infra/share/ Terraform + handlers/share.js
T5: share-api.ts — Lambda API クライアント（T4 後）
T6: page.tsx に Lambda 短縮 URL 統合（T3, T5 後）
T7: Cargo.toml バージョン更新
T8: driver.rs v216000_tests 追加
T9: CHANGELOG + playground.mdx
```

T1, T2, T4 は並列可
T3 は T1, T2 後
T5 は T4 後
T6 は T3, T5 後
T7〜T9 は最後

---

## T1: `site/lib/share-url.ts` — URL エンコード/デコード

### ファイル作成先

`site/lib/share-url.ts`

### 実装

```typescript
// CompressionStream を使った gzip + base64url エンコード
export async function encodeCode(code: string): Promise<string> {
  const bytes = new TextEncoder().encode(code)
  try {
    const cs = new CompressionStream('gzip')
    const writer = cs.writable.getWriter()
    writer.write(bytes)
    writer.close()
    const compressed = await new Response(cs.readable).arrayBuffer()
    return base64urlEncode(new Uint8Array(compressed))
  } catch {
    // フォールバック: 非圧縮 base64url
    return base64urlEncode(bytes)
  }
}

export async function decodeCode(encoded: string): Promise<string> {
  const bytes = base64urlDecode(encoded)
  try {
    const ds = new DecompressionStream('gzip')
    const writer = ds.writable.getWriter()
    writer.write(bytes)
    writer.close()
    const decompressed = await new Response(ds.readable).arrayBuffer()
    return new TextDecoder().decode(decompressed)
  } catch {
    return new TextDecoder().decode(bytes)
  }
}

export function buildShareUrl(encoded: string): string {
  const url = new URL(window.location.href)
  url.searchParams.set('c', encoded)
  url.searchParams.delete('s')  // slug パラメータを除去
  return url.toString()
}

// base64url (RFC 4648 §5: +→-, /→_, = 除去)
// NOTE: スプレッド展開ではなくチャンク処理で大配列のスタックオーバーフローを防ぐ
function base64urlEncode(bytes: Uint8Array): string {
  const CHUNK = 8192
  let binary = ''
  for (let i = 0; i < bytes.length; i += CHUNK) {
    binary += String.fromCharCode(...bytes.subarray(i, i + CHUNK))
  }
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')
}

function base64urlDecode(str: string): Uint8Array {
  const b64 = str.replace(/-/g, '+').replace(/_/g, '/')
  const padded = b64 + '='.repeat((4 - b64.length % 4) % 4)
  return Uint8Array.from(atob(padded), c => c.charCodeAt(0))
}
```

### 注意事項

- `CompressionStream` は Next.js の SSR で利用不可（`window` が undefined）
- `'use client'` コンポーネントからのみ呼び出す。`encodeCode`/`decodeCode` の先頭に以下のガードを追加:
  ```typescript
  if (typeof window === 'undefined') throw new Error('share-url: client only')
  ```
- `encodeCode` / `decodeCode` は `async`（Promise を返す）

---

## T2: `site/lib/playground-templates.ts` — テンプレート定義

### テンプレート 6 種

```typescript
export interface PlaygroundTemplate {
  id: string
  name: string
  description: string
  code: string
}

export const PLAYGROUND_TEMPLATES: PlaygroundTemplate[] = [
  {
    id: 'hello-world',
    name: 'Hello World',
    description: 'IO.println で文字列を出力',
    code: `public fn main() -> Unit !Io {\n  IO.println("Hello, Favnir!")\n}`,
  },
  {
    id: 'pipeline-basic',
    name: 'パイプライン基礎',
    description: 'stage + seq + |> の組み合わせ',
    code: `stage Double: Int -> Int = |n| n * 2\nstage AddOne: Int -> Int = |n| n + 1\n\nseq Transform = Double |> AddOne\n\npublic fn main() -> Unit !Io {\n  IO.println_int(Transform(5))\n}`,
  },
  {
    id: 'list-transform',
    name: 'List 操作',
    description: 'map / filter / fold の活用',
    code: `public fn main() -> Unit !Io {\n  bind nums <- [1, 2, 3, 4, 5]\n  bind evens <- List.filter(nums, |n| n % 2 == 0)\n  bind doubled <- List.map(evens, |n| n * 2)\n  bind total <- List.fold(doubled, 0, |acc, n| acc + n)\n  IO.println_int(total)\n}`,
  },
  {
    id: 'result-handling',
    name: 'Result 型',
    description: 'Result<T, E> + ? 演算子によるエラーハンドリング',
    code: `fn parse_int(s: String) -> Result<Int, String> =\n  match Int.parse(s) {\n    Some(n) => Result.ok(n)\n    None    => Result.err(f"parse error: {s}")\n  }\n\npublic fn main() -> Unit !Io {\n  bind n <- parse_int("42")?\n  IO.println_int(n)\n}`,
  },
  {
    id: 'record-type',
    name: 'レコード型',
    description: 'type 定義 + フィールドアクセス',
    // NOTE: Favnir レコード型のフィールドはコンマでなく改行区切り
  code: `type User = {\n  name: String\n  age: Int\n}\n\nfn greet(u: User) -> String =\n  f"Hello, {u.name}! Age: {u.age}"\n\npublic fn main() -> Unit !Io {\n  bind user <- User {\n    name: "Alice"\n    age: 30\n  }\n  IO.println(greet(user))\n}`,
  },
  {
    id: 'fstring-format',
    name: 'f-string フォーマット',
    description: 'f-string による文字列補間',
    code: `public fn main() -> Unit !Io {\n  bind name <- "Favnir"\n  bind version <- 21\n  IO.println(f"Welcome to {name} v{version}!")\n}`,
  },
]
```

---

## T3: `site/app/playground/page.tsx` 更新

### 追加する状態

```typescript
const [shareStatus, setShareStatus] = useState<'idle' | 'copied'>('idle')
const [execStats, setExecStats] = useState<{ ms: number; memMB: number | null } | null>(null)
const [showTemplates, setShowTemplates] = useState(false)
```

### URL 復元（useEffect）

```typescript
useEffect(() => {
  const params = new URLSearchParams(window.location.search)
  const encoded = params.get('c')
  if (encoded) {
    decodeCode(encoded).then(decoded => {
      setCode(decoded)
      // バナー表示は useState<boolean> で管理
      setFromShare(true)
    })
  } else {
    setCode(EXAMPLE_CODE)
  }
}, [])
```

### Share ボタン

```typescript
const handleShare = async () => {
  const encoded = await encodeCode(code)
  const url = buildShareUrl(encoded)
  await navigator.clipboard.writeText(url)
  setShareStatus('copied')
  setTimeout(() => setShareStatus('idle'), 2000)
}
```

### 実行統計（handleRun 内）

計測範囲は `window.__favnirCompile(code)` 呼び出しから `mainFn()` 完了まで（コンパイル + WASM instantiate + 実行を含む）。これがユーザーの体感時間に最も近い。

```typescript
const t0 = performance.now()                // ← コンパイル前に開始
const bytes = window.__favnirCompile(code)  // コンパイル（既存コード）
if (!bytes) { /* エラー処理 */ }
const result = await WebAssembly.instantiate(bytes, imports)  // instantiate
// ... moduleMemory 設定 ...
mainFn()                                    // 実行
const ms = Math.round(performance.now() - t0)
const memMB = (performance as PerformanceWithMemory).memory
  ? Math.round((performance as PerformanceWithMemory).memory.usedJSHeapSize / 1024 / 1024 * 10) / 10
  : null
setExecStats({ ms, memMB })
```

`PerformanceWithMemory` は Chrome 専用拡張のため型定義を追加:

```typescript
interface PerformanceWithMemory extends Performance {
  memory?: { usedJSHeapSize: number }
}
```

### テンプレート選択

```typescript
const handleSelectTemplate = (template: PlaygroundTemplate) => {
  setCode(template.code)
  setShowTemplates(false)
  setExecStats(null)
  setDiagnostics([])
}
```

### UI レイアウト変更

ヘッダー右のボタン行:
```
[ テンプレート ▼ ]  [ 型チェック ]  [ ▶ 実行 ]  [ 📋 共有 ]
```

出力パネルの下部:
```tsx
{execStats && (
  <div className="text-xs text-muted-foreground mt-2 pt-2 border-t border-border">
    実行時間: {execStats.ms}ms
    {execStats.memMB !== null && `  |  メモリ: ${execStats.memMB}MB`}
  </div>
)}
```

フロムシェアバナー（エディタ上部）:
```tsx
{fromShare && (
  <div className="text-xs text-blue-400 px-4 py-1 bg-blue-900/20 border-b border-border">
    共有コードを読み込みました — 自由に編集できます
    <button onClick={() => setFromShare(false)} className="ml-2 opacity-60">✕</button>
  </div>
)}
```

---

## T4: `infra/share/` — Lambda + S3 Terraform

### ディレクトリ構成

```
infra/share/
├── main.tf          # S3 バケット + Lifecycle ルール
├── lambda.tf        # Lambda 関数 + API Gateway + CORS
├── providers.tf     # AWS provider
├── variables.tf     # region, environment
├── outputs.tf       # api_url 出力
└── handlers/
    └── share.js     # Lambda ハンドラ（Node.js 20.x）
```

### `handlers/share.js`

```javascript
const { S3Client, PutObjectCommand, GetObjectCommand } = require('@aws-sdk/client-s3')

const s3 = new S3Client({ region: process.env.AWS_REGION })
const BUCKET = process.env.SHARE_BUCKET

// slug: 6文字 alphanumeric (a-z0-9)
function generateSlug() {
  const chars = 'abcdefghijklmnopqrstuvwxyz0123456789'
  return Array.from({ length: 6 }, () => chars[Math.floor(Math.random() * chars.length)]).join('')
}

exports.handler = async (event) => {
  const method = event.httpMethod || event.requestContext?.http?.method

  if (method === 'POST') {
    const body = JSON.parse(event.body || '{}')
    const code = body.code ?? ''
    if (!code || code.length > 32768) {
      return { statusCode: 400, body: JSON.stringify({ error: 'invalid code' }) }
    }
    // slug 衝突チェック: GetObject で存在確認し衝突時は最大 5 回再生成
    let slug = ''
    for (let attempt = 0; attempt < 5; attempt++) {
      slug = generateSlug()
      try {
        await s3.send(new GetObjectCommand({ Bucket: BUCKET, Key: `shares/${slug}.fav` }))
        // オブジェクトが存在 → 衝突 → 再試行
      } catch {
        break // 404 = 衝突なし → このスラグを使う
      }
    }
    await s3.send(new PutObjectCommand({
      Bucket: BUCKET,
      Key: `shares/${slug}.fav`,
      Body: code,
      ContentType: 'text/plain',
    }))
    const url = `https://play.favnir.dev/playground?s=${slug}`
    return {
      statusCode: 201,
      headers: { 'Content-Type': 'application/json', 'Access-Control-Allow-Origin': '*' },
      body: JSON.stringify({ slug, url }),
    }
  }

  if (method === 'GET') {
    const slug = event.pathParameters?.slug
    if (!slug || !/^[a-z0-9]{6}$/.test(slug)) {
      return { statusCode: 400, body: JSON.stringify({ error: 'invalid slug' }) }
    }
    try {
      const resp = await s3.send(new GetObjectCommand({ Bucket: BUCKET, Key: `shares/${slug}.fav` }))
      const code = await resp.Body.transformToString()
      return {
        statusCode: 200,
        headers: { 'Content-Type': 'application/json', 'Access-Control-Allow-Origin': '*' },
        body: JSON.stringify({ code }),
      }
    } catch {
      return { statusCode: 404, body: JSON.stringify({ error: 'not found' }) }
    }
  }

  // OPTIONS (CORS preflight)
  return {
    statusCode: 200,
    headers: { 'Access-Control-Allow-Origin': '*', 'Access-Control-Allow-Methods': 'GET,POST,OPTIONS' },
    body: '',
  }
}
```

### S3 セキュリティ + Lifecycle

```hcl
# Public Access Block（必須）
resource "aws_s3_bucket_public_access_block" "shares" {
  bucket                  = aws_s3_bucket.shares.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

# サーバーサイド暗号化（AES256）
resource "aws_s3_bucket_server_side_encryption_configuration" "shares" {
  bucket = aws_s3_bucket.shares.id
  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

# Lifecycle: 90日 TTL
resource "aws_s3_bucket_lifecycle_configuration" "shares_ttl" {
  bucket = aws_s3_bucket.shares.id
  rule {
    id     = "expire-shares"
    status = "Enabled"
    filter { prefix = "shares/" }
    expiration { days = 90 }
  }
}
```

### Lambda デプロイパッケージ

Node.js 20.x ランタイムには AWS SDK v3 が含まれないため、`package.json` + `npm install` + zip が必要:

```
infra/share/handlers/
├── package.json       # { "dependencies": { "@aws-sdk/client-s3": "^3" } }
├── package-lock.json  # npm install 後に生成
└── share.js
```

`lambda.tf` の `filename` は `${path.module}/handlers/share.zip` を参照。
デプロイ前に `cd infra/share/handlers && npm install && zip -r share.zip .` を実行。
または `null_resource` + `local-exec` で自動化。

---

## T5: `site/app/playground/share-api.ts` — Lambda クライアント

```typescript
const SHARE_API = process.env.NEXT_PUBLIC_SHARE_API_URL ?? ''

export interface ShareResult {
  slug: string
  url: string
}

export async function shareCode(code: string): Promise<ShareResult | null> {
  if (!SHARE_API) return null
  try {
    const resp = await fetch(`${SHARE_API}/share`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ code }),
    })
    if (!resp.ok) return null
    return await resp.json() as ShareResult
  } catch {
    return null
  }
}

export async function loadSharedCode(slug: string): Promise<string | null> {
  if (!SHARE_API) return null
  try {
    const resp = await fetch(`${SHARE_API}/share/${slug}`)
    if (!resp.ok) return null
    const data = await resp.json() as { code: string }
    return data.code
  } catch {
    return null
  }
}
```

---

## T6: page.tsx に Lambda 短縮 URL 統合

`handleShare` を更新して Lambda API を優先、フォールバックはサーバーレス URL:

```typescript
const handleShare = async () => {
  const lambdaResult = await shareCode(code)
  const url = lambdaResult?.url ?? buildShareUrl(await encodeCode(code))
  await navigator.clipboard.writeText(url)
  setShareStatus('copied')
  setTimeout(() => setShareStatus('idle'), 2000)
}
```

`?s=<slug>` パラメータの復元（useEffect に追加）:

```typescript
const slug = params.get('s')
if (slug) {
  loadSharedCode(slug).then(decoded => {
    if (decoded) { setCode(decoded); setFromShare(true) }
    else { setCode(EXAMPLE_CODE) }
  })
}
```

---

## T7: Cargo.toml バージョン更新

`version = "21.5.0"` → `"21.6.0"`
`v215000_tests::version_is_21_5_0` に `#[ignore]` を追加

---

## T8: `fav/src/driver.rs` — v216000_tests 追加（8件）

v21.6 は主に TypeScript/インフラの変更だが、以下をテスト:

| テスト名 | 内容 |
|----------|------|
| `version_is_21_6_0` | Cargo.toml に `"21.6.0"` が含まれる |
| `playground_share_url_file_exists` | `site/lib/share-url.ts` が存在する |
| `playground_templates_file_exists` | `site/lib/playground-templates.ts` が存在する |
| `playground_share_api_file_exists` | `site/app/playground/share-api.ts` が存在する |
| `infra_share_main_tf_exists` | `infra/share/main.tf` が存在する |
| `infra_share_lambda_handler_exists` | `infra/share/handlers/share.js` が存在する |
| `playground_templates_count` | `playground-templates.ts` に 6 テンプレート（`id:` が 6 個）含まれる |
| `changelog_has_v21_6_0` | `CHANGELOG.md` に `[v21.6.0]` エントリが含まれる |

### テスト実装パターン

```rust
#[test]
fn playground_share_url_file_exists() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .join("site/lib/share-url.ts");
    assert!(path.exists(), "share-url.ts が見つかりません: {:?}", path);
}
```

---

## T9: CHANGELOG + playground.mdx

### CHANGELOG エントリ

`CHANGELOG.md` の先頭に v21.6.0 エントリを追加。

### `site/content/docs/tools/playground.mdx`

- Playground v2 機能一覧
- 共有 URL の使い方（スクリーンショット用プレースホルダー）
- テンプレートギャラリーの紹介
- フォーク（共有 URL からの読み込み）

---

## 実装リスクと対策

| リスク | 対策 |
|--------|------|
| `CompressionStream` が SSR で使えない | `typeof window === 'undefined'` ガードで early throw。`'use client'` 限定で呼び出す |
| `btoa(String.fromCharCode(...bytes))` スタックオーバーフロー | 8192 バイトのチャンク処理で回避（T1 コードスニペット参照） |
| base64url デコードで URL 特殊文字が壊れる | base64url は URL-safe（`+→-`, `/→_`）なので `encodeURIComponent` は不要 |
| Lambda cold start でシェア URL 生成が遅い | サーバーレス URL（`?c=`）をデフォルトとし、Lambda はオプション強化 |
| S3 slug 衝突 | `GetObject` で最大 5 回衝突確認・再生成ループ（T4 実装コード参照） |
| `performance.memory` が undefined（Chrome 以外） | `?.` オプショナルチェーンで安全に参照 |
| `NEXT_PUBLIC_SHARE_API_URL` が未設定 | `null` を返す分岐で graceful degradation。設定手順は T5 に記載 |
| Node.js 20.x に AWS SDK v3 が含まれない | `handlers/package.json` + `npm install` + zip バンドル（T4 参照） |
| record-type テンプレートのコンマ区切り誤り | Favnir は改行区切り。テンプレートに改行区切りを使用（T2 コードスニペット参照） |
| `terraform validate` + LocalStack で Lifecycle ルールが動かない | Lifecycle は AWS 本番のみ確認。ローカルでは TTL テストをスキップ |

## `NEXT_PUBLIC_SHARE_API_URL` 設定手順

```bash
# 1. Terraform でデプロイして API URL を取得
cd infra/share
terraform apply
export API_URL=$(terraform output -raw api_url)

# 2. ローカル開発
echo "NEXT_PUBLIC_SHARE_API_URL=${API_URL}" >> site/.env.local

# 3. CI/CD（GitHub Actions等）
# NEXT_PUBLIC_SHARE_API_URL を repository secret / environment variable に設定

# 4. 未設定時の動作: shareCode() が null を返す → サーバーレス URL にフォールバック
```
