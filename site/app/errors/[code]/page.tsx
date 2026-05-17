import { notFound } from 'next/navigation'
import Link from 'next/link'
import { Header } from '@/components/landing/header'
import { Footer } from '@/components/landing/footer'
import { getAllErrors, getErrorByCode } from '@/lib/errors'
import type { Metadata } from 'next'

interface Props {
  params: Promise<{ code: string }>
}

export async function generateStaticParams() {
  const errors = getAllErrors()
  return errors.map((e) => ({ code: e.code }))
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const { code } = await params
  const error = getErrorByCode(code)
  if (!error) return {}
  return {
    title: `${error.code}: ${error.title}`,
    description: error.description,
  }
}

export default async function ErrorDetailPage({ params }: Props) {
  const { code } = await params
  const error = getErrorByCode(code)
  if (!error) notFound()

  return (
    <div className="min-h-screen">
      <Header />
      <main className="mx-auto max-w-3xl px-6 pt-32 pb-20 lg:px-8">
        <Link href="/errors/" className="text-sm text-muted-foreground hover:text-foreground">
          ← エラーカタログに戻る
        </Link>

        <div className="mt-6">
          <div className="flex items-center gap-4 mb-2">
            <span className="font-mono text-2xl font-bold text-primary">{error.code}</span>
            <span className="text-sm text-muted-foreground bg-secondary px-2 py-0.5 rounded">
              {error.category}
            </span>
          </div>
          <h1 className="text-2xl font-bold text-foreground">{error.title}</h1>
        </div>

        <section className="mt-8">
          <h2 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground mb-3">説明</h2>
          <p className="text-foreground leading-relaxed">{error.description}</p>
        </section>

        <section className="mt-8">
          <h2 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground mb-3">例</h2>
          <pre className="rounded-lg border border-border bg-card p-4 overflow-x-auto text-sm font-mono text-foreground/90">
            <code>{error.example}</code>
          </pre>
        </section>

        <section className="mt-8">
          <h2 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground mb-3">修正方法</h2>
          <p className="text-foreground leading-relaxed whitespace-pre-line">{error.fix}</p>
        </section>
      </main>
      <Footer />
    </div>
  )
}
