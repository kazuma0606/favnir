import fs from 'fs'
import path from 'path'

export interface ErrorEntry {
  code: string
  title: string
  category: string
  description: string
  example: string
  fix: string
}

const CATALOG_PATH = path.join(process.cwd(), 'content', 'errors', 'catalog.json')

function loadCatalog(): ErrorEntry[] {
  if (!fs.existsSync(CATALOG_PATH)) return []
  return JSON.parse(fs.readFileSync(CATALOG_PATH, 'utf-8')) as ErrorEntry[]
}

export function getAllErrors(): ErrorEntry[] {
  return loadCatalog()
}

export function getErrorByCode(code: string): ErrorEntry | undefined {
  return loadCatalog().find((e) => e.code === code)
}

export function getErrorCategories(): string[] {
  const cats = new Set(loadCatalog().map((e) => e.category))
  return Array.from(cats)
}
