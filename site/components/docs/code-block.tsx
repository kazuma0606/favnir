import { createHighlighter, type Highlighter } from 'shiki'
import favnirGrammar from '@/lib/favnir-grammar.json'

interface Props {
  children?: React.ReactNode
  className?: string
  [key: string]: unknown
}

function extractText(node: React.ReactNode): string {
  if (typeof node === 'string') return node
  if (typeof node === 'number') return String(node)
  if (Array.isArray(node)) return node.map(extractText).join('')
  if (node && typeof node === 'object' && 'props' in node) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return extractText((node as any).props?.children)
  }
  return ''
}

// Singleton — one highlighter instance per Node.js process
let highlighterInstance: Highlighter | null = null
async function getHighlighter(): Promise<Highlighter> {
  if (!highlighterInstance) {
    highlighterInstance = await createHighlighter({
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      langs: [favnirGrammar as any, 'typescript', 'javascript', 'bash', 'json', 'toml', 'rust'],
      themes: ['one-dark-pro'],
    })
  }
  return highlighterInstance
}

const LANG_MAP: Record<string, string> = {
  favnir: 'fav',
  fav: 'fav',
  ts: 'typescript',
  js: 'javascript',
  sh: 'bash',
}

const SUPPORTED = new Set(['fav', 'typescript', 'javascript', 'bash', 'json', 'toml', 'rust'])

export async function CodeBlock({ children, className }: Props) {
  const raw = className?.replace(/^language-/, '') ?? ''
  const lang = LANG_MAP[raw] ?? raw
  const code = extractText(children).trimEnd()

  if (!code) return null

  let html: string
  try {
    const hl = await getHighlighter()
    const safeLang = SUPPORTED.has(lang) ? lang : 'bash'
    html = hl.codeToHtml(code, { lang: safeLang, theme: 'one-dark-pro' })
  } catch {
    const escaped = code
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
    html = `<pre style="background:#282c34;padding:1rem;border-radius:0.5rem;overflow-x:auto;font-size:0.875em"><code style="color:#abb2bf">${escaped}</code></pre>`
  }

  return (
    <div
      className="my-4 rounded-lg overflow-hidden [&>pre]:p-4 [&>pre]:overflow-x-auto [&>pre]:leading-relaxed [&>pre]:text-sm"
      dangerouslySetInnerHTML={{ __html: html }}
    />
  )
}
