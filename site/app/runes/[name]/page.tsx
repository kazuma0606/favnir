import { notFound } from 'next/navigation'
import Link from 'next/link'
import { Header } from '@/components/landing/header'
import { Footer } from '@/components/landing/footer'
import { getDocBySlug, getAllDocs } from '@/lib/docs'
import { MDXRemote } from 'next-mdx-remote/rsc'
import type { Metadata } from 'next'

interface Props {
  params: Promise<{ name: string }>
}

export async function generateStaticParams() {
  const docs = getAllDocs()
  return docs
    .filter((d) => d.slug.startsWith('runes/'))
    .map((d) => ({ name: d.slug.replace('runes/', '') }))
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const { name } = await params
  const doc = getDocBySlug(`runes/${name}`)
  if (!doc) return {}
  return {
    title: doc.frontmatter.title,
    description: doc.frontmatter.description,
  }
}

export default async function RuneDetailPage({ params }: Props) {
  const { name } = await params
  const doc = getDocBySlug(`runes/${name}`)
  if (!doc) notFound()

  return (
    <div className="min-h-screen">
      <Header />
      <main className="mx-auto max-w-3xl px-6 pt-32 pb-20 lg:px-8">
        <Link href="/runes/" className="text-sm text-muted-foreground hover:text-foreground">
          ← Rune カタログに戻る
        </Link>
        <article className="mt-8 prose prose-invert prose-pre:bg-card prose-pre:border prose-pre:border-border max-w-none">
          <h1 className="text-3xl font-bold text-foreground mb-2">{doc.frontmatter.title}</h1>
          {doc.frontmatter.description && (
            <p className="text-lg text-muted-foreground mb-8">{doc.frontmatter.description}</p>
          )}
          <hr className="border-border mb-8" />
          <MDXRemote source={doc.content} />
        </article>
      </main>
      <Footer />
    </div>
  )
}
