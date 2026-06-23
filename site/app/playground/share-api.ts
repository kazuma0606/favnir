const SHARE_API = process.env.NEXT_PUBLIC_SHARE_API_URL ?? ''

export interface ShareResult {
  slug: string
  url: string
}

export async function shareCode(code: string): Promise<ShareResult | null> {
  if (!SHARE_API) return null
  try {
    const resp = await fetch(`${SHARE_API}/share`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ code }),
    })
    if (!resp.ok) return null
    return await resp.json() as ShareResult
  } catch {
    return null
  }
}

export async function loadSharedCode(slug: string): Promise<string | null> {
  if (!SHARE_API) return null
  try {
    const resp = await fetch(`${SHARE_API}/share/${slug}`)
    if (!resp.ok) return null
    const data = await resp.json() as { code: string }
    return data.code
  } catch {
    return null
  }
}
