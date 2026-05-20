import Link from 'next/link'
import { Header } from '@/components/landing/header'
import { Footer } from '@/components/landing/footer'
import type { Metadata } from 'next'

export const metadata: Metadata = {
  title: 'Rune カタログ',
  description: 'Favnir の標準 Rune 一覧',
}

const REGISTRY_URL = 'https://32qp3qwhdh.execute-api.ap-northeast-1.amazonaws.com'

interface RuneMeta {
  effect: string
  tags: string[]
}

const LOCAL_META: Record<string, RuneMeta> = {
  auth:        { effect: '!Auth',    tags: ['JWT', 'RBAC', 'OAuth2'] },
  aws:         { effect: '!AWS',     tags: ['S3', 'SQS', 'DynamoDB'] },
  csv:         { effect: '!Io',      tags: ['CSV', 'parse', 'write'] },
  db:          { effect: '!Db',      tags: ['SQLite', 'PostgreSQL', 'transaction'] },
  duckdb:      { effect: '!Db',      tags: ['Parquet', 'CSV', 'OLAP'] },
  env:         { effect: '!Env',     tags: ['.env', 'dotenv', 'Secrets Manager'] },
  gen:         { effect: '',         tags: ['test data', 'seed', 'Parquet'] },
  grpc:        { effect: '!Network', tags: ['gRPC', 'protobuf', 'streaming'] },
  http:        { effect: '!Network', tags: ['REST', 'retry', 'JSON'] },
  incremental: { effect: '!Io',      tags: ['delta', 'upsert', 'watermark'] },
  json:        { effect: '',         tags: ['JSON', 'parse', 'serialize'] },
  log:         { effect: '!Io',      tags: ['JSON', 'CloudWatch', 'EMF'] },
  parquet:     { effect: '!Io',      tags: ['Parquet', 'columnar', 'Arrow'] },
  stat:        { effect: '',         tags: ['mean', 'stddev', 'histogram'] },
  validate:    { effect: '',         tags: ['schema', 'constraints', 'error'] },
}

interface RegistryRune {
  name: string
  version: string
  description: string
}

async function fetchRunes(): Promise<RegistryRune[]> {
  const res = await fetch(`${REGISTRY_URL}/runes`, { cache: 'force-cache' })
  if (!res.ok) {
    throw new Error(`Registry fetch failed: ${res.status} ${res.statusText}`)
  }
  return res.json() as Promise<RegistryRune[]>
}

export default async function RunesPage() {
  const runes = await fetchRunes()
  runes.sort((a, b) => a.name.localeCompare(b.name))

  return (
    <div className="min-h-screen">
      <Header />
      <main className="mx-auto max-w-5xl px-6 pt-32 pb-20 lg:px-8">
        <h1 className="text-3xl font-bold text-foreground">Rune カタログ</h1>
        <p className="mt-4 text-muted-foreground">
          Favnir の標準 Rune は <code className="text-primary">import rune &quot;name&quot;</code> の 1 行で使えます。
        </p>

        <div className="mt-12 grid gap-4 sm:grid-cols-2">
          {runes.map((rune) => {
            const meta = LOCAL_META[rune.name] ?? { effect: '', tags: [] }
            return (
              <Link
                key={rune.name}
                href={`/runes/${rune.name}/`}
                className="group rounded-lg border border-border bg-card p-6 transition-all hover:border-primary/50"
              >
                <div className="flex items-start justify-between mb-3">
                  <h2 className="text-lg font-semibold text-foreground group-hover:text-primary transition-colors">
                    {rune.name}
                    <span className="ml-2 text-xs font-normal text-muted-foreground">v{rune.version}</span>
                  </h2>
                  {meta.effect && (
                    <span className="ml-2 shrink-0 font-mono text-xs text-primary bg-primary/10 px-2 py-1 rounded">
                      {meta.effect}
                    </span>
                  )}
                </div>
                <p className="text-sm text-muted-foreground leading-relaxed mb-4">{rune.description}</p>
                <div className="flex flex-wrap gap-2">
                  {meta.tags.map((tag) => (
                    <span key={tag} className="text-xs text-muted-foreground bg-secondary px-2 py-0.5 rounded">
                      {tag}
                    </span>
                  ))}
                </div>
              </Link>
            )
          })}
        </div>
      </main>
      <Footer />
    </div>
  )
}
