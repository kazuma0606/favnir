'use client'

import { useState } from 'react'
import { Header } from '@/components/landing/header'
import { Footer } from '@/components/landing/footer'
import { Button } from '@/components/ui/button'

const EXAMPLE_CODE = `// Favnir Playground へようこそ
// !Io エフェクトのみ利用可能です

type Point = { x: Int  y: Int }

fn distance(a: Point, b: Point) -> Float {
  let dx = a.x - b.x
  let dy = a.y - b.y
  Float.sqrt(Int.to_float(dx * dx + dy * dy))
}

public fn main() -> Unit !Io {
  let p1 = Point { x: 0  y: 0 }
  let p2 = Point { x: 3  y: 4 }
  IO.println(distance(p1, p2))
}`

export default function PlaygroundPage() {
  const [code, setCode] = useState(EXAMPLE_CODE)
  const [output, setOutput] = useState('')
  const [errors, setErrors] = useState<string[]>([])
  const [wasmReady] = useState(false)
  const [running, setRunning] = useState(false)

  const handleRun = async () => {
    if (!wasmReady) {
      setOutput('Playground WASM ランタイムは現在準備中です。\nPhase B 実装後に利用可能になります。')
      return
    }
    setRunning(true)
    setErrors([])
    // Phase B で @favnir/wasm を接続する
    setRunning(false)
  }

  const handleCheck = () => {
    if (!wasmReady) {
      setErrors(['WASM ランタイムが未準備です。Phase B 実装後に利用可能になります。'])
      return
    }
  }

  return (
    <div className="min-h-screen flex flex-col">
      <Header />
      <main className="flex-1 mx-auto w-full max-w-7xl px-6 pt-24 pb-8 lg:px-8">
        <div className="flex items-center justify-between mb-4 mt-4">
          <div>
            <h1 className="text-2xl font-bold text-foreground">Playground</h1>
            <p className="text-sm text-muted-foreground mt-1">
              ブラウザ内で Favnir を実行できます（<code className="text-primary">!Io</code> のみ）
            </p>
          </div>
          <div className="flex gap-3">
            <Button variant="outline" onClick={handleCheck} size="sm">
              型チェック
            </Button>
            <Button onClick={handleRun} disabled={running} size="sm" className="bg-primary text-primary-foreground">
              {running ? '実行中...' : '実行'}
            </Button>
          </div>
        </div>

        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2 h-[calc(100vh-260px)]">
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
            />
          </div>

          <div className="rounded-lg border border-border overflow-hidden flex flex-col">
            <div className="border-b border-border px-4 py-2 bg-card">
              <span className="text-xs text-muted-foreground font-mono">出力</span>
            </div>
            <div className="flex-1 p-4 font-mono text-sm overflow-auto bg-card/50">
              {errors.length > 0 ? (
                <div className="space-y-1">
                  {errors.map((err, i) => (
                    <div key={i} className="text-destructive-foreground">{err}</div>
                  ))}
                </div>
              ) : output ? (
                <pre className="text-foreground/90 whitespace-pre-wrap">{output}</pre>
              ) : (
                <span className="text-muted-foreground">
                  「実行」ボタンを押すと結果が表示されます
                </span>
              )}
            </div>
          </div>
        </div>

        <p className="mt-3 text-xs text-muted-foreground text-center">
          Playground の WASM ランタイムは Phase B で実装予定です
        </p>
      </main>
      <Footer />
    </div>
  )
}
