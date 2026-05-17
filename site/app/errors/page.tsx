import Link from 'next/link'
import { Header } from '@/components/landing/header'
import { Footer } from '@/components/landing/footer'
import { getAllErrors } from '@/lib/errors'
import type { Metadata } from 'next'

export const metadata: Metadata = {
  title: 'エラーカタログ',
  description: 'Favnir コンパイラエラーコードの一覧と解説',
}

const CATEGORY_LABELS: Record<string, string> = {
  pipeline: 'パイプライン / 構造',
  types: '型エラー',
  effects: 'エフェクト',
  modules: 'モジュール / スキーマ',
  runtime: 'ランタイム',
  deprecated: '廃止キーワード',
}

export default function ErrorsPage() {
  const errors = getAllErrors()
  const byCategory = errors.reduce<Record<string, typeof errors>>((acc, e) => {
    if (!acc[e.category]) acc[e.category] = []
    acc[e.category].push(e)
    return acc
  }, {})

  const categoryOrder = ['pipeline', 'types', 'effects', 'modules', 'runtime', 'deprecated']

  return (
    <div className="min-h-screen">
      <Header />
      <main className="mx-auto max-w-4xl px-6 pt-32 pb-20 lg:px-8">
        <h1 className="text-3xl font-bold text-foreground">エラーカタログ</h1>
        <p className="mt-4 text-muted-foreground">
          Favnir コンパイラが出力するすべてのエラーコードとその解説です。
        </p>

        <div className="mt-12 space-y-12">
          {categoryOrder.map((cat) => {
            const items = byCategory[cat]
            if (!items?.length) return null
            return (
              <section key={cat}>
                <h2 className="text-xl font-semibold text-foreground mb-4">
                  {CATEGORY_LABELS[cat] ?? cat}
                </h2>
                <div className="rounded-lg border border-border overflow-hidden">
                  {items.map((error, i) => (
                    <Link
                      key={error.code}
                      href={`/errors/${error.code}/`}
                      className={`flex items-center gap-4 px-5 py-4 transition-colors hover:bg-secondary ${
                        i < items.length - 1 ? 'border-b border-border' : ''
                      }`}
                    >
                      <span className="font-mono text-sm text-primary w-16 shrink-0">{error.code}</span>
                      <span className="text-sm text-foreground">{error.title}</span>
                    </Link>
                  ))}
                </div>
              </section>
            )
          })}
        </div>
      </main>
      <Footer />
    </div>
  )
}
