import Link from 'next/link'
import { ExternalLink } from 'lucide-react'
import { Button } from '@/components/ui/button'

export function CTA() {
  return (
    <section className="border-t border-border py-24 lg:py-32">
      <div className="mx-auto max-w-7xl px-6 lg:px-8">
        <div className="relative overflow-hidden rounded-2xl border border-border bg-card px-6 py-16 sm:px-16 lg:px-24">
          <div className="absolute inset-0 -z-10">
            <div className="absolute bottom-0 left-1/2 -translate-x-1/2 h-[300px] w-[600px] rounded-full bg-primary/10 blur-[100px]" />
          </div>

          <div className="mx-auto max-w-2xl text-center">
            <h2 className="text-3xl font-bold tracking-tight text-foreground sm:text-4xl">
              今すぐ始めましょう
            </h2>
            <p className="mt-4 text-lg text-muted-foreground">
              Favnir で次世代のデータパイプラインを構築しませんか？
            </p>
            <div className="mt-8 flex flex-col items-center gap-4 sm:flex-row sm:justify-center">
              <Button size="lg" asChild className="w-full bg-primary text-primary-foreground hover:bg-primary/90 sm:w-auto">
                <Link href="/playground/">
                  Playground を試す
                  <ExternalLink className="ml-2 h-4 w-4" />
                </Link>
              </Button>
              <Button
                size="lg"
                variant="outline"
                asChild
                className="w-full border-border bg-transparent text-foreground hover:bg-secondary sm:w-auto"
              >
                <Link href="/docs/introduction/">ドキュメントを見る</Link>
              </Button>
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}
