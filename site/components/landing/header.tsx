'use client'

import Link from 'next/link'
import Image from 'next/image'
import { useState } from 'react'
import { Menu, X, Github } from 'lucide-react'
import { Button } from '@/components/ui/button'

const navigation = [
  { name: 'ドキュメント', href: '/docs/introduction/' },
  { name: 'エラー', href: '/errors/' },
  { name: 'Rune', href: '/runes/' },
  { name: 'Playground', href: '/playground/' },
]

export function Header() {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false)

  return (
    <header className="fixed top-0 left-0 right-0 z-50 border-b border-border bg-background/80 backdrop-blur-xl">
      <nav className="mx-auto flex max-w-7xl items-center justify-between px-6 py-4 lg:px-8">
        <div className="flex items-center gap-3">
          <Link href="/" className="flex items-center gap-3">
            <Image
              src="/images/favnir-mascot.png"
              alt="Favnir Logo"
              width={32}
              height={32}
              className="rounded"
            />
            <span className="text-xl font-semibold tracking-tight text-foreground">Favnir</span>
          </Link>
        </div>

        <div className="hidden md:flex md:items-center md:gap-8">
          {navigation.map((item) => (
            <Link
              key={item.name}
              href={item.href}
              className="text-sm text-muted-foreground transition-colors hover:text-foreground"
            >
              {item.name}
            </Link>
          ))}
        </div>

        <div className="hidden md:flex md:items-center md:gap-4">
          <Link
            href="https://github.com/yoshiask/favnir"
            target="_blank"
            rel="noopener noreferrer"
            className="text-muted-foreground transition-colors hover:text-foreground"
          >
            <Github className="h-5 w-5" />
            <span className="sr-only">GitHub</span>
          </Link>
          <Button asChild className="bg-primary text-primary-foreground hover:bg-primary/90">
            <Link href="/docs/introduction/">Get Started</Link>
          </Button>
        </div>

        <div className="flex md:hidden">
          <button
            type="button"
            className="text-muted-foreground"
            onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
          >
            <span className="sr-only">メニューを開く</span>
            {mobileMenuOpen ? <X className="h-6 w-6" /> : <Menu className="h-6 w-6" />}
          </button>
        </div>
      </nav>

      {mobileMenuOpen && (
        <div className="md:hidden">
          <div className="space-y-1 px-6 pb-4 pt-2">
            {navigation.map((item) => (
              <Link
                key={item.name}
                href={item.href}
                className="block py-2 text-base text-muted-foreground hover:text-foreground"
                onClick={() => setMobileMenuOpen(false)}
              >
                {item.name}
              </Link>
            ))}
            <div className="mt-4">
              <Button asChild className="w-full bg-primary text-primary-foreground">
                <Link href="/docs/introduction/">Get Started</Link>
              </Button>
            </div>
          </div>
        </div>
      )}
    </header>
  )
}
