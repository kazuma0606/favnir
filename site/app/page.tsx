import { Header } from '@/components/landing/header'
import { Hero } from '@/components/landing/hero'
import { Features } from '@/components/landing/features'
import { DocsPreview } from '@/components/landing/docs-preview'
import { CTA } from '@/components/landing/cta'
import { Footer } from '@/components/landing/footer'

export default function HomePage() {
  return (
    <div className="min-h-screen">
      <Header />
      <main>
        <Hero />
        <Features />
        <DocsPreview />
        <CTA />
      </main>
      <Footer />
    </div>
  )
}
