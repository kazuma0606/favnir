import Link from 'next/link'
import { ChevronRight } from 'lucide-react'

const docsItems = [
  {
    category: 'はじめに',
    items: [
      { title: 'イントロダクション', href: '/docs/introduction/' },
      { title: 'インストール', href: '/docs/installation/' },
      { title: 'クイックスタート', href: '/docs/quickstart/' },
    ],
  },
  {
    category: '言語仕様',
    items: [
      { title: '型システム', href: '/docs/language/types/' },
      { title: 'エフェクト型', href: '/docs/language/effects/' },
      { title: 'パターンマッチ', href: '/docs/language/pattern-matching/' },
      { title: 'Rune', href: '/docs/language/runes/' },
    ],
  },
  {
    category: 'Rune カタログ',
    items: [
      { title: 'AWS Rune', href: '/runes/aws/' },
      { title: 'DuckDB Rune', href: '/runes/duckdb/' },
      { title: 'Auth Rune', href: '/runes/auth/' },
      { title: 'すべての Rune →', href: '/runes/' },
    ],
  },
]

export function DocsPreview() {
  return (
    <section className="border-t border-border py-24 lg:py-32">
      <div className="mx-auto max-w-7xl px-6 lg:px-8">
        <div className="grid gap-12 lg:grid-cols-3">
          {docsItems.map((category) => (
            <div key={category.category}>
              <h3 className="mb-4 text-sm font-semibold uppercase tracking-wider text-primary">
                {category.category}
              </h3>
              <ul className="space-y-3">
                {category.items.map((item) => (
                  <li key={item.title}>
                    <Link
                      href={item.href}
                      className="group flex items-center gap-2 text-muted-foreground transition-colors hover:text-foreground"
                    >
                      <ChevronRight className="h-4 w-4 text-muted-foreground/50 transition-transform group-hover:translate-x-1 group-hover:text-primary" />
                      {item.title}
                    </Link>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      </div>
    </section>
  )
}
