import { Header } from '@/components/landing/header'
import { Footer } from '@/components/landing/footer'
import { DocsSidebar } from '@/components/docs/sidebar'

export default function DocsLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="min-h-screen">
      <Header />
      <div className="mx-auto max-w-7xl px-6 pt-24 pb-16 lg:px-8">
        <div className="flex gap-12">
          <DocsSidebar />
          <main className="min-w-0 flex-1">
            <article className="prose prose-invert prose-pre:bg-card prose-pre:border prose-pre:border-border max-w-none">
              {children}
            </article>
          </main>
        </div>
      </div>
      <Footer />
    </div>
  )
}
