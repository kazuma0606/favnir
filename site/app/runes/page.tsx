import Link from 'next/link'
import { Header } from '@/components/landing/header'
import { Footer } from '@/components/landing/footer'
import type { Metadata } from 'next'

export const metadata: Metadata = {
  title: 'Rune カタログ',
  description: 'Favnir の標準 Rune 一覧',
}

const runes = [
  {
    name: 'aws',
    title: 'AWS Rune',
    description: 'S3 / SQS / DynamoDB を型安全に操作。LocalStack で開発し本番 AWS へ切り替え。',
    effect: '!AWS',
    tags: ['S3', 'SQS', 'DynamoDB'],
  },
  {
    name: 'db',
    title: 'DB Rune',
    description: 'SQLite / PostgreSQL 接続・クエリ・トランザクション・マイグレーション。',
    effect: '!Db',
    tags: ['SQLite', 'PostgreSQL', 'transaction'],
  },
  {
    name: 'duckdb',
    title: 'DuckDB Rune',
    description: 'ブラウザ不要の組み込み OLAP。Parquet / CSV を SQL で直接クエリ。',
    effect: '!Db',
    tags: ['Parquet', 'CSV', 'OLAP'],
  },
  {
    name: 'http',
    title: 'HTTP Rune',
    description: 'GET / POST / PUT / DELETE・リトライ・Bearer 認証・JSON レスポンス。',
    effect: '!Network',
    tags: ['REST', 'retry', 'JSON'],
  },
  {
    name: 'auth',
    title: 'Auth Rune',
    description: 'JWT 検証・RBAC・OAuth2・API キー生成。Cognito / ALB との統合。',
    effect: '!Auth',
    tags: ['JWT', 'RBAC', 'OAuth2'],
  },
  {
    name: 'log',
    title: 'Log Rune',
    description: '構造化ログ・ログコード体系・CloudWatch EMF メトリクス出力。',
    effect: '!Io',
    tags: ['JSON', 'CloudWatch', 'EMF'],
  },
  {
    name: 'env',
    title: 'Env Rune',
    description: '環境変数の取得・型変換・.env ファイルロード・Secrets Manager フォールバック。',
    effect: '!Env',
    tags: ['.env', 'dotenv', 'Secrets Manager'],
  },
  {
    name: 'gen',
    title: 'Gen Rune',
    description: 'シード指定の再現可能なテストデータ生成。フィールド名ヒントでリアルなデータを出力。',
    effect: '',
    tags: ['test data', 'seed', 'Parquet'],
  },
]

export default function RunesPage() {
  return (
    <div className="min-h-screen">
      <Header />
      <main className="mx-auto max-w-5xl px-6 pt-32 pb-20 lg:px-8">
        <h1 className="text-3xl font-bold text-foreground">Rune カタログ</h1>
        <p className="mt-4 text-muted-foreground">
          Favnir の標準 Rune は <code className="text-primary">import rune &quot;name&quot;</code> の 1 行で使えます。
        </p>

        <div className="mt-12 grid gap-4 sm:grid-cols-2">
          {runes.map((rune) => (
            <Link
              key={rune.name}
              href={`/runes/${rune.name}/`}
              className="group rounded-lg border border-border bg-card p-6 transition-all hover:border-primary/50"
            >
              <div className="flex items-start justify-between mb-3">
                <h2 className="text-lg font-semibold text-foreground group-hover:text-primary transition-colors">
                  {rune.title}
                </h2>
                {rune.effect && (
                  <span className="ml-2 shrink-0 font-mono text-xs text-primary bg-primary/10 px-2 py-1 rounded">
                    {rune.effect}
                  </span>
                )}
              </div>
              <p className="text-sm text-muted-foreground leading-relaxed mb-4">{rune.description}</p>
              <div className="flex flex-wrap gap-2">
                {rune.tags.map((tag) => (
                  <span key={tag} className="text-xs text-muted-foreground bg-secondary px-2 py-0.5 rounded">
                    {tag}
                  </span>
                ))}
              </div>
            </Link>
          ))}
        </div>
      </main>
      <Footer />
    </div>
  )
}
