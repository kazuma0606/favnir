import { notFound } from 'next/navigation'
import { getAllDocs, getDocBySlug } from '@/lib/docs'
import { MDXRemote } from 'next-mdx-remote/rsc'
import type { Metadata } from 'next'

interface Props {
  params: Promise<{ slug: string[] }>
}

export async function generateStaticParams() {
  const docs = getAllDocs()
  return docs.map((doc) => ({ slug: doc.slug.split('/') }))
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const { slug } = await params
  const doc = getDocBySlug(slug.join('/'))
  if (!doc) return {}
  return {
    title: doc.frontmatter.title,
    description: doc.frontmatter.description,
  }
}

export default async function DocPage({ params }: Props) {
  const { slug } = await params
  const doc = getDocBySlug(slug.join('/'))
  if (!doc) notFound()

  return (
    <>
      <h1 className="text-3xl font-bold text-foreground mb-2">{doc.frontmatter.title}</h1>
      {doc.frontmatter.description && (
        <p className="text-lg text-muted-foreground mb-8">{doc.frontmatter.description}</p>
      )}
      <hr className="border-border mb-8" />
      <MDXRemote source={doc.content} />
    </>
  )
}
