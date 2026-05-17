import Link from 'next/link'
import { buildSidebar } from '@/lib/docs'

export async function DocsSidebar({ currentSlug }: { currentSlug?: string }) {
  const sidebar = buildSidebar()

  return (
    <nav className="w-64 shrink-0">
      <div className="sticky top-20 space-y-6 py-6">
        {sidebar.map((category) => (
          <div key={category.name}>
            <p className="mb-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              {category.name}
            </p>
            <ul className="space-y-1">
              {category.items.map((item) => {
                const isActive = currentSlug === item.slug
                return (
                  <li key={item.slug}>
                    <Link
                      href={item.href}
                      className={`block rounded-md px-3 py-1.5 text-sm transition-colors ${
                        isActive
                          ? 'bg-primary/10 text-primary font-medium'
                          : 'text-muted-foreground hover:text-foreground hover:bg-secondary'
                      }`}
                    >
                      {item.title}
                    </Link>
                  </li>
                )
              })}
            </ul>
          </div>
        ))}
      </div>
    </nav>
  )
}
