import type { Metadata } from 'next'
import { Geist, Geist_Mono } from 'next/font/google'
import './globals.css'

const geistSans = Geist({
  variable: '--font-geist-sans',
  subsets: ['latin'],
})

const geistMono = Geist_Mono({
  variable: '--font-geist-mono',
  subsets: ['latin'],
})

export const metadata: Metadata = {
  title: {
    template: '%s | Favnir',
    default: 'Favnir — 型安全なデータパイプライン専用言語',
  },
  description:
    'Favnir はデータエンジニアのための型安全なパイプライン専用言語です。エフェクト型・Rune エコシステム・AWS ネイティブ統合を備えます。',
  icons: {
    icon: '/images/favnir-mascot.png',
  },
}

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  return (
    <html lang="ja" className={`${geistSans.variable} ${geistMono.variable} dark`}>
      <body className="font-sans antialiased min-h-screen bg-background text-foreground">
        {children}
      </body>
    </html>
  )
}
