import Link from 'next/link'
import Image from 'next/image'
import { ChevronRight, Github } from 'lucide-react'
import { Button } from '@/components/ui/button'

const CODE_EXAMPLE = `import rune "aws"
import rune "duckdb"

type Order = { id: Int  customer: String  amount: Float }
type Summary = { customer: String  total: Float }

public fn pipeline() -> Unit !Io !AWS {
  bind orders <- aws.s3.read_csv<Order>(
    "data-lake", "raw/orders.csv")
  bind conn   <- duckdb.open(":memory:")
  bind result <- duckdb.query<Summary>(conn,
    "SELECT customer, SUM(amount) AS total
     FROM orders GROUP BY customer
     ORDER BY total DESC LIMIT 10")
  IO.println(result)
}`

export function Hero() {
  return (
    <section className="relative overflow-hidden pt-32 pb-20 lg:pt-40 lg:pb-32">
      <div className="absolute inset-0 -z-10">
        <div className="absolute top-1/4 left-1/2 -translate-x-1/2 h-[500px] w-[800px] rounded-full bg-primary/10 blur-[120px]" />
      </div>

      <div className="mx-auto max-w-7xl px-6 lg:px-8">
        <div className="mx-auto max-w-3xl text-center">
          <div className="mb-8 flex justify-center">
            <Image
              src="/images/favnir-mascot.png"
              alt="Favnir Mascot"
              width={160}
              height={160}
              className="glow-primary rounded-lg"
              priority
            />
          </div>

          <div className="mb-6 inline-flex items-center gap-2 rounded-full border border-border bg-secondary/50 px-4 py-1.5 text-sm">
            <span className="text-primary">v5.0.0</span>
            <span className="text-muted-foreground">— Now Available</span>
          </div>

          <h1 className="text-balance text-4xl font-bold tracking-tight text-foreground sm:text-6xl">
            型安全な
            <span className="text-primary">データパイプライン</span>
            専用言語
          </h1>

          <p className="mt-6 text-pretty text-lg leading-relaxed text-muted-foreground lg:text-xl">
            Favnir はデータエンジニアのための言語です。エフェクト型システムにより副作用を明示し、
            Rune エコシステムで AWS・DuckDB・認証を型安全に統合します。
          </p>

          <div className="mt-10 flex flex-col items-center gap-4 sm:flex-row sm:justify-center">
            <Button size="lg" asChild className="w-full bg-primary text-primary-foreground hover:bg-primary/90 sm:w-auto">
              <Link href="/docs/introduction/">
                ドキュメントを見る
                <ChevronRight className="ml-2 h-4 w-4" />
              </Link>
            </Button>
            <Button
              size="lg"
              variant="outline"
              asChild
              className="w-full border-border bg-transparent text-foreground hover:bg-secondary sm:w-auto"
            >
              <Link href="https://github.com/yoshiask/favnir" target="_blank" rel="noopener noreferrer">
                <Github className="mr-2 h-4 w-4" />
                GitHub
              </Link>
            </Button>
          </div>

          <div className="mt-16 overflow-hidden rounded-lg border border-border bg-card text-left">
            <div className="flex items-center gap-2 border-b border-border px-4 py-3">
              <div className="flex gap-1.5">
                <div className="h-3 w-3 rounded-full bg-red-500/50" />
                <div className="h-3 w-3 rounded-full bg-yellow-500/50" />
                <div className="h-3 w-3 rounded-full bg-green-500/50" />
              </div>
              <span className="ml-2 text-xs text-muted-foreground font-mono">pipeline.fav</span>
            </div>
            <pre className="overflow-x-auto p-6">
              <code className="text-sm font-mono leading-relaxed text-foreground/90">
                {CODE_EXAMPLE.split('\n').map((line, i) => (
                  <span key={i} className="block">
                    {line
                      .replace(/(import rune|public fn|bind|type)/g, '<kw>$1</kw>')
                      .split(/(<kw>.*?<\/kw>)/)
                      .map((part, j) =>
                        part.startsWith('<kw>') ? (
                          <span key={j} className="text-primary">
                            {part.replace(/<\/?kw>/g, '')}
                          </span>
                        ) : (
                          <span key={j}>{part}</span>
                        )
                      )}
                  </span>
                ))}
              </code>
            </pre>
          </div>
        </div>
      </div>
    </section>
  )
}
