// Cliente tipado da API Pokerum — espelha CONTRACT.md

// ---- motores de avaliação (v2) ----
export type EngineKind = 'heuristic' | 'llm' | 'external_api' | 'database'
export type Execution = 'inline' | 'background'
export type EvaluationStatus = 'pending' | 'running' | 'done' | 'failed'

export interface Engine {
  id: string
  name: string
  kind: EngineKind
  description: string
  execution: Execution
  /** Ligado no banco. */
  enabled: boolean
  /** Configurado (ex.: API key presente). */
  available: boolean
  unavailable_reason: string | null
}

export interface Evaluation {
  engine_id: string
  engine_name: string
  engine_kind: EngineKind
  status: EvaluationStatus
  /** Só quando `done`. */
  score: number | null
  /** Posição no ranking deste motor (só `done`). */
  position: number | null
  total: number | null
  report: ScoreReport | null
  /** Só quando `failed`. */
  error: string | null
  updated_at: string
}

export const DEFAULT_ENGINE = 'heuristic-v1'

export function isEngineActive(e: Engine): boolean {
  return e.enabled && e.available
}

export function isEvaluationPending(e: Evaluation): boolean {
  return e.status === 'pending' || e.status === 'running'
}

export interface JobSummary {
  id: number
  slug: string
  title: string
  company: string | null
  created_at: string
  applicants: number
  /** Motor que ordena o ranking padrão (ausente em backends antigos). */
  ranking_engine?: string
  /** Até 3 melhores colocados da vaga. */
  top: TopCandidate[]
}

export interface TopCandidate {
  position: number
  codename: string
  alias: string
  score: number
}

export interface Job {
  id: number
  slug: string
  title: string
  company: string | null
  description: string
  created_at: string
  keywords: string[]
  ranking_engine?: string
  /** Motores ativos (ausente em backends antigos). */
  engines?: Engine[]
}

export interface NewJob {
  title: string
  company?: string
  description: string
  ranking_engine?: string
}

export interface CriterionScore {
  key: string
  label: string
  weight: number
  score: number
  detail: string
}

export interface ScoreReport {
  total: number
  breakdown: CriterionScore[]
  matched_keywords: string[]
  missing_keywords: string[]
  suggestions: string[]
  engine_version: string
}

export interface RankingEntry {
  position: number
  codename: string
  alias: string
  score: number
  breakdown: CriterionScore[]
  created_at: string
  engine_id?: string
}

export interface ApplyResponse {
  codename: string
  alias: string
  owner_token: string
  score: number
  position: number
  total: number
  report: ScoreReport
  anonymized_text: string
  primary_engine?: string
  evaluations?: Evaluation[]
}

export interface ResumeApplication {
  job_slug: string
  job_title: string
  score: number
  position: number
  total: number
}

export interface ResumeProfile {
  codename: string
  alias: string
  anonymized_text: string
  applications: ResumeApplication[]
}

export interface ApplicationResult {
  codename: string
  alias: string
  score: number
  position: number
  total: number
  report: ScoreReport
  primary_engine?: string
  evaluations?: Evaluation[]
}

export class ApiError extends Error {
  status: number
  constructor(message: string, status: number) {
    super(message)
    this.status = status
  }
}

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
  let res: Response
  try {
    res = await fetch(`/api${path}`, {
      ...init,
      headers: { Accept: 'application/json', ...(init.headers ?? {}) },
    })
  } catch {
    throw new ApiError('Não foi possível conectar ao servidor.', 0)
  }
  if (!res.ok) {
    let msg = `Erro ${res.status}`
    try {
      const body = await res.json()
      if (body && typeof body.error === 'string') msg = body.error
    } catch {
      /* corpo não-JSON */
    }
    throw new ApiError(msg, res.status)
  }
  if (res.status === 204) return undefined as T
  const text = await res.text()
  return (text ? JSON.parse(text) : undefined) as T
}

const enc = encodeURIComponent

export const api = {
  health: () => request<{ ok: boolean }>('/health'),
  listJobs: () => request<JobSummary[]>('/jobs'),
  createJob: (job: NewJob) =>
    request<Job>('/jobs', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(job),
    }),
  getJob: (slug: string) => request<Job>(`/jobs/${enc(slug)}`),
  getRanking: (slug: string, engine?: string | null) =>
    request<RankingEntry[]>(`/jobs/${enc(slug)}/ranking${engine ? `?engine=${enc(engine)}` : ''}`),
  listEngines: () => request<Engine[]>('/engines'),
  apply: (slug: string, input: { file?: File | null; text?: string; name?: string }) => {
    const fd = new FormData()
    if (input.file) fd.append('file', input.file)
    else if (input.text) fd.append('text', input.text)
    if (input.name && input.name.trim()) fd.append('name', input.name.trim())
    return request<ApplyResponse>(`/jobs/${enc(slug)}/apply`, { method: 'POST', body: fd })
  },
  getResume: (codename: string) => request<ResumeProfile>(`/resumes/${enc(codename)}`),
  getApplication: (slug: string, codename: string) =>
    request<ApplicationResult>(`/jobs/${enc(slug)}/applications/${enc(codename)}`),
  deleteResume: (codename: string, token: string) =>
    request<void>(`/resumes/${enc(codename)}`, {
      method: 'DELETE',
      headers: { 'X-Owner-Token': token },
    }),
}

// ---- owner tokens em localStorage ----
const TOKEN_KEY = 'pokerum:owner_tokens'

function readTokens(): Record<string, string> {
  try {
    return JSON.parse(localStorage.getItem(TOKEN_KEY) || '{}') ?? {}
  } catch {
    return {}
  }
}

export function saveOwnerToken(codename: string, token: string) {
  try {
    const all = readTokens()
    all[codename] = token
    localStorage.setItem(TOKEN_KEY, JSON.stringify(all))
  } catch {
    /* storage indisponível */
  }
}

export function getOwnerToken(codename: string): string | null {
  return readTokens()[codename] ?? null
}

export function removeOwnerToken(codename: string) {
  try {
    const all = readTokens()
    delete all[codename]
    localStorage.setItem(TOKEN_KEY, JSON.stringify(all))
  } catch {
    /* ignore */
  }
}

// ---- resultado recém-submetido (para mostrar o token uma única vez) ----
const fresh = new Map<string, ApplyResponse>()
export function stashFreshApply(slug: string, r: ApplyResponse) {
  fresh.set(`${slug}/${r.codename}`, r)
}
export function takeFreshApply(slug: string, codename: string): ApplyResponse | undefined {
  const key = `${slug}/${codename}`
  const r = fresh.get(key)
  fresh.delete(key)
  return r
}

// ---- cache da lista de motores (raramente muda) ----
let enginesPromise: Promise<Engine[]> | null = null
export function getEnginesCached(): Promise<Engine[]> {
  if (!enginesPromise) {
    enginesPromise = api.listEngines().catch((e) => {
      enginesPromise = null
      throw e
    })
  }
  return enginesPromise
}
