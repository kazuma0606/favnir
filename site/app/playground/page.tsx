'use client'

import { useState, useEffect, useRef } from 'react'
import { Header } from '@/components/landing/header'
import { Footer } from '@/components/landing/footer'
import { Button } from '@/components/ui/button'

const EXAMPLE_CODE = `// Favnir Playground — ブラウザ内で型チェックと実行
// bind x <- expr; で変数を束縛します

fn max(a: Int, b: Int) -> Int {
  if a > b { a } else { b }
}

fn min(a: Int, b: Int) -> Int {
  if a < b { a } else { b }
}

fn clamp(value: Int, lo: Int, hi: Int) -> Int {
  max(lo, min(value, hi))
}

public fn main() -> Unit !Io {
  bind x <- clamp(150, 0, 100);
  bind y <- clamp(-50, 0, 100);
  IO.println_int(x);
  IO.println_int(y)
}`

interface Diagnostic {
  code: string
  message: string
  line: number
  col: number
}

declare global {
  interface Window {
    __favnirCheck?: (source: string) => Diagnostic[]
    __favnirCompile?: (source: string) => Uint8Array | null
  }
}

export default function PlaygroundPage() {
  const [code, setCode] = useState('')
  const [diagnostics, setDiagnostics] = useState<Diagnostic[]>([])
  const [output, setOutput] = useState<string[]>([])
  const [wasmReady, setWasmReady] = useState(false)
  const [checking, setChecking] = useState(false)
  const [running, setRunning] = useState(false)
  const [activePanel, setActivePanel] = useState<'diagnostics' | 'output'>('diagnostics')
  const readyRef = useRef(false)

  // Set example code client-side only to avoid SSR/hydration mismatch
  // (EXAMPLE_CODE contains -> and <- which get HTML-escaped server-side)
  useEffect(() => {
    setCode(EXAMPLE_CODE)
  }, [])

  useEffect(() => {
    const script = document.createElement('script')
    script.type = 'module'
    script.innerHTML = `
      try {
        const mod = await import('/wasm/favnir.js');
        await mod.default();
        window.__favnirCheck = mod.fav_check;
        window.__favnirCompile = mod.fav_compile;
        document.dispatchEvent(new Event('favnir-wasm-ready'));
      } catch (e) {
        // WASM not available
      }
    `
    document.head.appendChild(script)

    const onReady = () => {
      readyRef.current = true
      setWasmReady(true)
    }
    document.addEventListener('favnir-wasm-ready', onReady)
    return () => document.removeEventListener('favnir-wasm-ready', onReady)
  }, [])

  const handleCheck = async () => {
    if (!window.__favnirCheck) return
    setChecking(true)
    await new Promise(r => setTimeout(r, 0))
    const result = window.__favnirCheck(code)
    setDiagnostics(result)
    setActivePanel('diagnostics')
    setChecking(false)
  }

  const handleRun = async () => {
    if (!window.__favnirCompile) return
    setRunning(true)
    setActivePanel('output')
    await new Promise(r => setTimeout(r, 0))

    const bytes = window.__favnirCompile(code)
    if (!bytes) {
      setOutput(['実行エラー: このプログラムはブラウザ実行非対応です。\n（構造体・クロージャ・文字列操作を含むプログラムにはサーバー実行が必要です）'])
      setRunning(false)
      return
    }

    const lines: string[] = []
    let moduleMemory: WebAssembly.Memory | null = null
    try {
      const imports = {
        fav_host: {
          io_println: (ptr: number, len: number) => {
            if (!moduleMemory) return
            const buf = new Uint8Array(moduleMemory.buffer, ptr, len)
            lines.push(new TextDecoder().decode(buf))
          },
          io_print: (ptr: number, len: number) => {
            if (!moduleMemory) return
            const buf = new Uint8Array(moduleMemory.buffer, ptr, len)
            lines.push(new TextDecoder().decode(buf))
          },
          io_println_int: (n: bigint) => {
            lines.push(String(n))
          },
          io_println_float: (f: number) => {
            lines.push(String(f))
          },
          io_println_bool: (b: number) => {
            lines.push(b !== 0 ? 'true' : 'false')
          },
        },
      }
      const result = await WebAssembly.instantiate(bytes, imports)
      const exports = result.instance.exports as Record<string, unknown>
      // The Favnir WASM module exports its own linear memory
      if (exports.memory instanceof WebAssembly.Memory) {
        moduleMemory = exports.memory
      }
      const mainFn = exports.main as (() => void) | undefined
      if (typeof mainFn === 'function') {
        mainFn()
      }
      setOutput(lines.length > 0 ? lines : ['(出力なし)'])
    } catch (e) {
      setOutput([`実行エラー: ${e instanceof Error ? e.message : String(e)}`])
    }
    setRunning(false)
  }

  const hasErrors = diagnostics.length > 0
  const statusColor = !wasmReady
    ? 'text-muted-foreground'
    : diagnostics.length === 0
    ? 'text-green-400'
    : hasErrors
    ? 'text-red-400'
    : 'text-yellow-400'

  return (
    <div className="min-h-screen flex flex-col">
      <Header />
      <main className="flex-1 mx-auto w-full max-w-7xl px-6 pt-24 pb-8 lg:px-8">
        <div className="flex items-center justify-between mb-4 mt-4">
          <div>
            <h1 className="text-2xl font-bold text-foreground">Playground</h1>
            <p className="text-sm text-muted-foreground mt-1">
              ブラウザ内で Favnir の型チェックと実行
            </p>
          </div>
          <div className="flex items-center gap-3">
            <span className={`text-xs font-mono ${statusColor}`}>
              {!wasmReady
                ? 'WASM 読み込み中...'
                : diagnostics.length === 0
                ? '✓ エラーなし'
                : `${diagnostics.length} 件の診断`}
            </span>
            <Button
              onClick={handleCheck}
              disabled={!wasmReady || checking}
              size="sm"
              variant="outline"
            >
              {checking ? 'チェック中...' : '型チェック'}
            </Button>
            <Button
              onClick={handleRun}
              disabled={!wasmReady || running}
              size="sm"
              className="bg-primary text-primary-foreground"
            >
              {running ? '実行中...' : '▶ 実行'}
            </Button>
          </div>
        </div>

        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2 h-[calc(100vh-260px)]">
          {/* Editor */}
          <div className="rounded-lg border border-border overflow-hidden flex flex-col">
            <div className="flex items-center gap-2 border-b border-border px-4 py-2 bg-card">
              <div className="flex gap-1.5">
                <div className="h-2.5 w-2.5 rounded-full bg-red-500/50" />
                <div className="h-2.5 w-2.5 rounded-full bg-yellow-500/50" />
                <div className="h-2.5 w-2.5 rounded-full bg-green-500/50" />
              </div>
              <span className="text-xs text-muted-foreground font-mono ml-2">main.fav</span>
            </div>
            <textarea
              value={code}
              onChange={(e) => setCode(e.target.value)}
              className="flex-1 resize-none bg-card p-4 font-mono text-sm text-foreground/90 focus:outline-none leading-relaxed"
              spellCheck={false}
              suppressHydrationWarning
            />
          </div>

          {/* Right panel: diagnostics / output */}
          <div className="rounded-lg border border-border overflow-hidden flex flex-col">
            <div className="border-b border-border px-4 py-2 bg-card flex items-center gap-4">
              <button
                onClick={() => setActivePanel('diagnostics')}
                className={`text-xs font-mono transition-colors ${activePanel === 'diagnostics' ? 'text-foreground' : 'text-muted-foreground hover:text-foreground'}`}
              >
                診断
                {diagnostics.length > 0 && (
                  <span className="ml-1.5 px-1 rounded bg-red-900/40 text-red-400">
                    {diagnostics.length}
                  </span>
                )}
              </button>
              <button
                onClick={() => setActivePanel('output')}
                className={`text-xs font-mono transition-colors ${activePanel === 'output' ? 'text-foreground' : 'text-muted-foreground hover:text-foreground'}`}
              >
                出力
              </button>
            </div>

            <div className="flex-1 p-4 font-mono text-sm overflow-auto bg-card/50">
              {activePanel === 'diagnostics' ? (
                diagnostics.length > 0 ? (
                  <div className="space-y-3">
                    {diagnostics.map((d, i) => (
                      <div key={i} className="rounded border border-border p-3 bg-card">
                        <div className="flex items-center gap-2 mb-1">
                          <span className="text-xs px-1.5 py-0.5 rounded bg-red-900/40 text-red-400 font-mono">
                            {d.code}
                          </span>
                          <span className="text-xs text-muted-foreground">
                            {d.line}:{d.col}
                          </span>
                        </div>
                        <p className="text-foreground/90 text-xs leading-relaxed">{d.message}</p>
                      </div>
                    ))}
                  </div>
                ) : wasmReady ? (
                  <span className="text-green-400 text-xs">
                    型エラーなし — 「型チェック」を押すか「▶ 実行」でコードを実行
                  </span>
                ) : (
                  <span className="text-muted-foreground text-xs">
                    WASM ランタイムを読み込み中...
                  </span>
                )
              ) : (
                output.length > 0 ? (
                  <div className="space-y-0.5">
                    {output.map((line, i) => (
                      <div key={i} className="text-xs text-foreground/90 whitespace-pre-wrap">
                        {line}
                      </div>
                    ))}
                  </div>
                ) : (
                  <span className="text-muted-foreground text-xs">
                    「▶ 実行」を押してコードを実行します
                  </span>
                )
              )}
            </div>
          </div>
        </div>

        <p className="mt-3 text-xs text-muted-foreground text-center">
          Favnir type checker + compiler running in-browser via WebAssembly
        </p>
      </main>
      <Footer />
    </div>
  )
}
