// Monstrinhos ORIGINAIS do Pokerum (§5.6 / §9): cada codinome ganha uma criatura em pixel art
// 16×16, gerada por regras a partir de um hash do nome. Sempre a mesma criatura para o mesmo
// nome. A forma sai de sorteios genéricos (corpo, olhos, orelhas, antenas, chifres, barriga,
// pintas, pés) e NÃO tem relação com o personagem que dá nome ao codinome.
// Nunca adicione traços característicos de personagens existentes (bochechas vermelhas, caudas
// em raio, bulbos, cascos...).

export const CREATURE_SIZE = 16

/** Cores: `--h` (matiz do tipo) vem do elemento pai; contorno e olhos usam `--line`. */
export const CREATURE_PALETTE: Record<string, string> = {
  O: 'var(--line)', // contorno
  E: 'var(--line)', // olho
  B: 'hsl(var(--h) 62% 62%)', // corpo
  S: 'hsl(var(--h) 52% 44%)', // sombra, pintas, pés
  L: 'hsl(var(--h) 80% 86%)', // barriga, pontas
  W: 'hsl(0 0% 100%)', // brilho do olho
}

function hash(s: string): number {
  let h = 2166136261
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i)
    h = Math.imul(h, 16777619)
  }
  return h >>> 0
}

/** PRNG determinístico (mulberry32). */
function rng(seed: number) {
  let a = seed >>> 0
  const next = () => {
    a = (a + 0x6d2b79f5) >>> 0
    let t = a
    t = Math.imul(t ^ (t >>> 15), t | 1)
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61)
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296
  }
  const pick = <T,>(xs: readonly T[]): T => xs[Math.floor(next() * xs.length)]
  return { next, pick, chance: (p: number) => next() < p }
}

const cache = new Map<string, string[]>()

/** Linhas da criatura (16 strings de 16 caracteres; '.' = transparente). */
export function creatureRows(name: string): string[] {
  const key = name.toLowerCase()
  const hit = cache.get(key)
  if (hit) return hit

  const N = CREATURE_SIZE
  const r = rng(hash(`pokerum-creature:${key}`))
  const g: string[][] = Array.from({ length: N }, () => Array(N).fill('.'))
  // escreve um pixel e o seu espelho (criaturas simétricas)
  const set = (x: number, y: number, c: string) => {
    if (x < 0 || y < 0 || x >= N || y >= N) return
    g[y][x] = c
    g[y][N - 1 - x] = c
  }
  const get = (x: number, y: number) => (x < 0 || y < 0 || x >= N || y >= N ? '.' : g[y][x])

  // ---- corpo: elipse arredondada
  const rx = r.pick([4.5, 5, 5.5, 6])
  const ry = r.pick([4, 4.5, 5, 5.5])
  const cy = 14.5 - ry // a base do corpo encosta na linha 14
  for (let y = 0; y < N; y++)
    for (let x = 0; x < N / 2; x++) {
      const dx = (x + 0.5 - N / 2) / rx
      const dy = (y + 0.5 - cy) / ry
      if (dx * dx + dy * dy <= 1) set(x, y, 'B')
    }
  const rows = g.map((row) => row.some((c) => c === 'B'))
  const top = rows.indexOf(true)
  const bottom = rows.lastIndexOf(true)

  // sombra na parte de baixo
  for (let y = 0; y < N; y++)
    for (let x = 0; x < N / 2; x++)
      if (get(x, y) === 'B' && (y + 0.5 - cy) / ry > 0.55) set(x, y, 'S')

  // barriga clara
  if (r.chance(0.6)) {
    const bry = ry * 0.42
    const brx = rx * 0.48
    const bcy = cy + ry * 0.38
    for (let y = 0; y < N; y++)
      for (let x = 0; x < N / 2; x++) {
        const dx = (x + 0.5 - N / 2) / brx
        const dy = (y + 0.5 - bcy) / bry
        if (dx * dx + dy * dy <= 1 && get(x, y) !== '.') set(x, y, 'L')
      }
  }

  // pintas nas costas/topo
  if (r.chance(0.4)) {
    for (let i = 0; i < 2; i++) {
      const x = 3 + Math.floor(r.next() * 4)
      const y = top + 1 + Math.floor(r.next() * 2)
      if (get(x, y) === 'B') set(x, y, 'S')
    }
  }

  // ---- olhos
  const ey = Math.round(cy - ry * 0.2)
  const ex = r.pick([5, 6]) // coluna do olho esquerdo (o direito é espelhado)
  const eyes = r.pick(['dot', 'tall', 'big', 'happy'] as const)
  if (eyes === 'dot') set(ex, ey, 'E')
  else if (eyes === 'tall') {
    set(ex, ey - 1, 'E')
    set(ex, ey, 'E')
  } else if (eyes === 'big') {
    set(ex - 1, ey - 1, 'W')
    set(ex, ey - 1, 'E')
    set(ex - 1, ey, 'E')
    set(ex, ey, 'E')
  } else {
    // olhos fechados e felizes: ^ ^
    set(ex - 1, ey, 'E')
    set(ex, ey - 1, 'E')
    set(ex + 1, ey, 'E')
  }

  // boca
  const mouth = r.pick(['none', 'line', 'open'] as const)
  if (mouth === 'line') set(7, ey + 2, 'E')
  else if (mouth === 'open') {
    set(7, ey + 2, 'E')
    if (get(7, ey + 3) !== '.') set(7, ey + 3, 'S')
  }

  // ---- topo da cabeça
  const head = r.pick(['none', 'round-ears', 'point-ears', 'antennae', 'horns', 'tuft'] as const)
  const edgeX = (y: number) => {
    for (let x = 0; x < N / 2; x++) if (get(x, y) !== '.') return x
    return N / 2
  }
  if (head === 'round-ears') {
    const x = edgeX(top + 1) + 1
    set(x, top - 1, 'B')
    set(x + 1, top - 1, 'B')
    set(x, top - 2, 'B')
    set(x + 1, top - 2, 'L')
  } else if (head === 'point-ears') {
    const x = edgeX(top + 1) + 1
    set(x, top - 1, 'B')
    set(x + 1, top - 1, 'B')
    set(x, top - 2, 'L')
  } else if (head === 'antennae') {
    const x = r.pick([5, 6])
    set(x, top - 1, 'S')
    set(x - 1, top - 2, 'S')
    set(x - 1, top - 3, 'L')
  } else if (head === 'horns') {
    const x = edgeX(top + 1) + 2
    set(x, top - 1, 'L')
    set(x - 1, top - 2, 'L')
  } else if (head === 'tuft') {
    set(6, top - 1, 'S')
    set(7, top - 1, 'S')
    set(7, top - 2, 'S')
  }

  // ---- bracinhos e pés
  if (r.chance(0.5)) {
    const y = Math.round(cy + 1)
    set(edgeX(y) - 1, y, 'B')
  }
  if (bottom < N - 1) {
    const fx = r.pick([4, 5])
    set(fx, bottom + 1, 'S')
    set(fx + 1, bottom + 1, 'S')
  }

  // ---- contorno de 1px em volta de tudo
  const out = g.map((row) => row.slice())
  for (let y = 0; y < N; y++)
    for (let x = 0; x < N; x++)
      if (g[y][x] === '.' && [[1, 0], [-1, 0], [0, 1], [0, -1]].some(([dx, dy]) => !['.', undefined].includes(g[y + dy]?.[x + dx]) && g[y + dy]?.[x + dx] !== 'O'))
        out[y][x] = 'O'

  const result = out.map((row) => row.join(''))
  cache.set(key, result)
  return result
}
