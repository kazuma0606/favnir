import fs from 'fs'
import path from 'path'
import matter from 'gray-matter'

const CONTENT_DIR = path.join(process.cwd(), 'content', 'docs')

export interface DocFrontmatter {
  title: string
  order: number
  category: string
  description?: string
}

export interface Doc {
  slug: string
  frontmatter: DocFrontmatter
  content: string
}

export interface SidebarCategory {
  name: string
  items: { title: string; slug: string; href: string }[]
}

function getAllMdxFiles(dir: string, base = ''): string[] {
  const files: string[] = []
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const rel = base ? `${base}/${entry.name}` : entry.name
    if (entry.isDirectory()) {
      files.push(...getAllMdxFiles(path.join(dir, entry.name), rel))
    } else if (entry.name.endsWith('.mdx') || entry.name.endsWith('.md')) {
      files.push(rel)
    }
  }
  return files
}

export function getAllDocs(): Doc[] {
  if (!fs.existsSync(CONTENT_DIR)) return []
  const files = getAllMdxFiles(CONTENT_DIR)
  return files.map((file) => {
    const slug = file.replace(/\.(mdx|md)$/, '')
    const raw = fs.readFileSync(path.join(CONTENT_DIR, file), 'utf-8')
    const { data, content } = matter(raw)
    return {
      slug,
      frontmatter: data as DocFrontmatter,
      content,
    }
  })
}

export function getDocBySlug(slug: string): Doc | null {
  const mdxPath = path.join(CONTENT_DIR, `${slug}.mdx`)
  const mdPath = path.join(CONTENT_DIR, `${slug}.md`)
  const filePath = fs.existsSync(mdxPath) ? mdxPath : fs.existsSync(mdPath) ? mdPath : null
  if (!filePath) return null
  const raw = fs.readFileSync(filePath, 'utf-8')
  const { data, content } = matter(raw)
  return { slug, frontmatter: data as DocFrontmatter, content }
}

export function buildSidebar(): SidebarCategory[] {
  const docs = getAllDocs()
  const categoryOrder = ['はじめに', '言語仕様', '標準ライブラリ', 'Rune']
  const map = new Map<string, { title: string; slug: string; order: number }[]>()

  for (const doc of docs) {
    const cat = doc.frontmatter.category ?? 'その他'
    if (!map.has(cat)) map.set(cat, [])
    map.get(cat)!.push({
      title: doc.frontmatter.title,
      slug: doc.slug,
      order: doc.frontmatter.order ?? 99,
    })
  }

  for (const items of map.values()) {
    items.sort((a, b) => a.order - b.order)
  }

  const result: SidebarCategory[] = []
  for (const cat of categoryOrder) {
    if (map.has(cat)) {
      result.push({
        name: cat,
        items: map.get(cat)!.map((item) => ({
          title: item.title,
          slug: item.slug,
          href: `/docs/${item.slug}/`,
        })),
      })
    }
  }
  for (const [cat, items] of map.entries()) {
    if (!categoryOrder.includes(cat)) {
      result.push({
        name: cat,
        items: items.map((item) => ({
          title: item.title,
          slug: item.slug,
          href: `/docs/${item.slug}/`,
        })),
      })
    }
  }
  return result
}
