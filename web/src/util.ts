import type { EngineKind } from './api'

const ENGINE_KIND_LABELS: Record<EngineKind, string> = {
  heuristic: 'Heurístico',
  llm: 'LLM',
  external_api: 'API externa',
  database: 'Banco de dados',
}

export function engineKindLabel(kind: string | null | undefined): string {
  return (kind && ENGINE_KIND_LABELS[kind as EngineKind]) || kind || 'Motor'
}

export function scoreTier(score: number): 'low' | 'mid' | 'high' {
  if (score >= 75) return 'high'
  if (score >= 50) return 'mid'
  return 'low'
}

/** Cor de preenchimento (barras, blocos). Não usar em texto no modo claro (§2.3). */
export function scoreColor(score: number): string {
  return `var(--score-${scoreTier(score)})`
}

/** Cor de texto da faixa de nota, com contraste ≥ 4,5:1 (§2.3). */
export function scoreTextColor(score: number): string {
  return `var(--score-${scoreTier(score)}-text)`
}

export function fmtScore(n: number): string {
  return Math.round(n).toString()
}

export function fmtDate(iso: string): string {
  // SQLite CURRENT_TIMESTAMP vem como "YYYY-MM-DD HH:MM:SS" (UTC)
  const norm = /^\d{4}-\d{2}-\d{2} \d/.test(iso) ? iso.replace(' ', 'T') + 'Z' : iso
  const d = new Date(norm)
  if (isNaN(d.getTime())) return iso
  return d.toLocaleDateString('pt-BR', { day: '2-digit', month: 'short', year: 'numeric' })
}

function hash(s: string): number {
  let h = 2166136261
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i)
    h = Math.imul(h, 16777619)
  }
  return h >>> 0
}

export interface CodenameType {
  label: string
  hue: number
}

// "Tipos" fictícios, só texto + cor — derivados de forma determinística do hash do nome.
const TYPES: CodenameType[] = [
  { label: 'Fogo', hue: 14 },
  { label: 'Água', hue: 205 },
  { label: 'Planta', hue: 120 },
  { label: 'Elétrico', hue: 48 },
  { label: 'Psíquico', hue: 320 },
  { label: 'Gelo', hue: 185 },
  { label: 'Pedra', hue: 35 },
  { label: 'Fantasma', hue: 265 },
  { label: 'Dragão', hue: 245 },
  { label: 'Fada', hue: 340 },
  { label: 'Lutador', hue: 0 },
  { label: 'Venenoso', hue: 285 },
  { label: 'Terra', hue: 28 },
  { label: 'Voador', hue: 225 },
  { label: 'Inseto', hue: 80 },
  { label: 'Aço', hue: 210 },
]

export function codenameType(name: string): CodenameType {
  return TYPES[hash(name.toLowerCase()) % TYPES.length]
}

export function ordinal(n: number): string {
  return `${n}º`
}
