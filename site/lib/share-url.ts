'use client'

// NOTE: このモジュールはブラウザ専用です。SSR 環境では使用不可。

function base64urlEncode(bytes: Uint8Array): string {
  // チャンク 8192 バイトで btoa — スプレッド展開は大配列でスタックオーバーフローするため禁止
  const CHUNK = 8192
  let binary = ''
  for (let i = 0; i < bytes.length; i += CHUNK) {
    binary += String.fromCharCode(...bytes.subarray(i, i + CHUNK))
  }
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')
}

function base64urlDecode(str: string): Uint8Array {
  const b64 = str.replace(/-/g, '+').replace(/_/g, '/')
  const padded = b64 + '='.repeat((4 - (b64.length % 4)) % 4)
  return Uint8Array.from(atob(padded), c => c.charCodeAt(0))
}

export async function encodeCode(code: string): Promise<string> {
  if (typeof window === 'undefined') throw new Error('share-url: client only')
  const bytes = new TextEncoder().encode(code)
  try {
    const cs = new CompressionStream('gzip')
    const writer = cs.writable.getWriter()
    writer.write(bytes)
    writer.close()
    const compressed = await new Response(cs.readable).arrayBuffer()
    return base64urlEncode(new Uint8Array(compressed))
  } catch {
    // フォールバック: CompressionStream 未対応ブラウザは非圧縮 base64url
    return base64urlEncode(bytes)
  }
}

export async function decodeCode(encoded: string): Promise<string> {
  if (typeof window === 'undefined') throw new Error('share-url: client only')
  const bytes = base64urlDecode(encoded)
  try {
    const ds = new DecompressionStream('gzip')
    const writer = ds.writable.getWriter()
    writer.write(bytes)
    writer.close()
    const decompressed = await new Response(ds.readable).arrayBuffer()
    return new TextDecoder().decode(decompressed)
  } catch {
    return new TextDecoder().decode(bytes)
  }
}

export function buildShareUrl(encoded: string): string {
  if (typeof window === 'undefined') throw new Error('share-url: client only')
  const url = new URL(window.location.href)
  url.searchParams.set('c', encoded)
  url.searchParams.delete('s')
  return url.toString()
}
