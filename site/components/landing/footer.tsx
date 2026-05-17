import Link from 'next/link'
import Image from 'next/image'
import { Github } from 'lucide-react'

export function Footer() {
  return (
    <footer className="border-t border-border py-12">
      <div className="mx-auto max-w-7xl px-6 lg:px-8">
        <div className="flex flex-col items-center justify-between gap-6 sm:flex-row">
          <div className="flex items-center gap-3">
            <Image
              src="/images/favnir-mascot.png"
              alt="Favnir Logo"
              width={28}
              height={28}
              className="rounded"
            />
            <span className="text-lg font-semibold text-foreground">Favnir</span>
          </div>
          <div className="flex items-center gap-6">
            <Link href="/docs/introduction/" className="text-sm text-muted-foreground hover:text-foreground">
              ドキュメント
            </Link>
            <Link href="/errors/" className="text-sm text-muted-foreground hover:text-foreground">
              エラーカタログ
            </Link>
            <Link href="/runes/" className="text-sm text-muted-foreground hover:text-foreground">
              Rune
            </Link>
            <Link
              href="https://github.com/yoshiask/favnir"
              target="_blank"
              rel="noopener noreferrer"
              className="text-muted-foreground hover:text-foreground"
            >
              <Github className="h-5 w-5" />
            </Link>
          </div>
        </div>
        <div className="mt-8 text-center text-sm text-muted-foreground">
          © 2026 Favnir. MIT License.
        </div>
      </div>
    </footer>
  )
}
