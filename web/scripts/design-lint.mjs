#!/usr/bin/env node
// Verificador do design system "Pokerum Cozy Pixel".
// Regras mecânicas de docs/design/REGRAS.md e docs/design-system/project/README.md.
// Uso: node scripts/design-lint.mjs [arquivos...]   (sem argumentos: todo web/src)
// Saída: uma linha por achado "arquivo:linha  [erro|aviso] regra: mensagem"; exit 1 se houver erro.
import { readFileSync, readdirSync, statSync, existsSync } from 'node:fs'
import { join, relative, resolve, dirname, extname } from 'node:path'
import { fileURLToPath } from 'node:url'

const WEB = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const ROOT = resolve(WEB, '..')
const SRC = join(WEB, 'src')
const STYLE = join(SRC, 'style.css')
const TOKENS = join(ROOT, 'docs/design-system/project/tokens.json')

const DISPLAY_SIZES = new Set([20, 24, 32, 40, 48, 64]) // §4 Pixelify
const LABEL_SIZES = new Set([16]) // §4 Silkscreen: só 16px
const PIXEL_SIZES = new Set([12, 14, 16, 20, 24, 32, 40, 48, 64]) // escala geral
const BODY_SIZES = new Set([14, 16, 18]) // mono 14 permitido
const findings = []
const add = (file, line, level, rule, msg) => findings.push({ file: relative(ROOT, file), line, level, rule, msg })

function walk(dir, out = []) {
  for (const n of readdirSync(dir)) {
    const p = join(dir, n)
    if (statSync(p).isDirectory()) walk(p, out)
    else if (['.vue', '.css', '.ts', '.html'].includes(extname(p))) out.push(p)
  }
  return out
}

const args = process.argv.slice(2).map((a) => resolve(process.cwd(), a)).filter((f) => existsSync(f))
const files = args.length ? args.filter((f) => ['.vue', '.css', '.ts', '.html'].includes(extname(f))) : walk(SRC)
if (!args.length && existsSync(join(WEB, 'index.html'))) files.push(join(WEB, 'index.html'))

// Partes de um .vue/.css/.html que contêm CSS (com o número da linha inicial).
function cssBlocks(file, text) {
  if (file.endsWith('.css')) return [{ start: 1, css: text }]
  const blocks = []
  const re = /<style[^>]*>([\s\S]*?)<\/style>/g
  let m
  while ((m = re.exec(text))) blocks.push({ start: text.slice(0, m.index).split('\n').length, css: m[1] })
  return blocks
}

// Linhas com `/* design-lint-ignore: motivo */` (ou `<!-- design-lint-ignore -->`) ficam de fora.
const ignored = (line) => /design-lint-ignore/.test(line)

const EMOJI = /[\u{1F300}-\u{1FAFF}\u{2600}-\u{26FF}\u{2700}-\u{27BF}](?<![✓✗✦⇄▤▶▼])/u
const HEX = /#[0-9a-fA-F]{3,8}\b/

for (const file of files) {
  const text = readFileSync(file, 'utf8')
  const lines = text.split('\n')
  const isStyle = file === STYLE
  const isPixelArt = /pixelart\.ts$|PixelArt\.vue$/.test(file)

  // ---- CSS ----
  for (const { start, css } of cssBlocks(file, text)) {
    const cl = css.split('\n')
    // estado por seletor: a família de fonte declarada no mesmo bloco { }
    let blockFont = null
    let inTokens = false
    cl.forEach((raw, i) => {
      const ln = start + i
      const line = raw.replace(/\/\*.*?\*\//g, '')
      if (ignored(raw)) return
      if (/\{/.test(line)) blockFont = null
      if (isStyle && /^\s*:root\s*\{|prefers-color-scheme/.test(line)) inTokens = true
      if (isStyle && inTokens && /^\}/.test(raw)) inTokens = false

      const radius = line.match(/border-radius\s*:\s*([^;]+)/)
      if (radius && !/^\s*(0(px)?|var\(--radius[^)]*\))\s*$/.test(radius[1]))
        add(file, ln, 'erro', '§3.2 raio', `border-radius "${radius[1].trim()}": o raio é sempre 0`)

      if (/blur\(/.test(line)) add(file, ln, 'erro', '§3.3 sombra', 'blur()/backdrop-filter proibido: sombras são duras')
      const sh = line.match(/box-shadow\s*:[^;]*?(?:^|[\s:,])-?\d+(?:px)?\s+-?\d+(?:px)?\s+([1-9]\d*px)/)
      if (sh) add(file, ln, 'erro', '§3.3 sombra', `sombra com desfoque ${sh[1]}: use deslocamento sem blur`)

      if (/\b(ease|ease-in|ease-out|ease-in-out|linear|cubic-bezier\([^)]*\))\b(?!-gradient)/.test(line) && /(transition|animation)/.test(line))
        add(file, ln, 'erro', '§7 movimento', 'easing suave: use steps(n)')
      const tr = line.match(/(?:transition|animation)\s*:\s*([^;]+)/)
      if (tr && !/steps\(|none|var\(--/.test(tr[1]) && /\d+m?s/.test(tr[1]) && !/^\s*[a-z-]+\s+[\d.]+m?s\s+steps/.test(tr[1]))
        add(file, ln, 'aviso', '§7 movimento', `"${tr[1].trim()}" sem steps(): confirme que anda em quadros`)

      const odd = line.match(/\b(?:border(?:-[a-z]+)?(?:-width)?|outline(?:-width)?|outline-offset)\s*:[^;]*?(?<![\d.])([13579])px/)
      if (odd) add(file, ln, 'erro', '§3.1 grade', `${odd[1]}px em borda/contorno: use múltiplos de 2px (calc(var(--px) * n))`)
      const frac = line.match(/(?<![\w.])\d*\.\d+px/)
      if (frac) add(file, ln, 'erro', '§3.1 grade', `valor fracionário ${frac[0]} borra a borda do pixel`)

      if (/Press Start|Bricolage/.test(line)) add(file, ln, 'erro', '§4 tipografia', 'fonte fora do sistema')
      const ff = line.match(/font-family\s*:\s*([^;]+)/)
      if (ff) blockFont = ff[1]
      const fs = line.match(/font-size\s*:\s*(\d+(?:\.\d+)?)px/)
      if (fs) {
        const n = Number(fs[1])
        const fam = blockFont || ''
        if (/font-display|Pixelify/.test(fam) && !DISPLAY_SIZES.has(n))
          add(file, ln, 'erro', '§4 tipografia', `Pixelify em ${n}px: só ${[...DISPLAY_SIZES].join('/')} (abaixo de 20px use Nunito)`)
        else if (/font-label|Silkscreen/.test(fam) && !LABEL_SIZES.has(n))
          add(file, ln, 'erro', '§4 tipografia', `Silkscreen em ${n}px: só 16px; rótulo pequeno usa .label (Nunito 800 14px)`)
        else if (/var\(--font\)|Nunito/.test(fam) && n < 14)
          add(file, ln, 'erro', '§4 tipografia', `Nunito em ${n}px: mínimo 14px (rótulos) e 16px (texto corrido)`)
        else if (!fam && n < 14) add(file, ln, 'erro', '§8 legibilidade', `${n}px é menor que o piso de 14px`)
        else if (!fam && !PIXEL_SIZES.has(n) && !BODY_SIZES.has(n))
          add(file, ln, 'aviso', '§4 tipografia', `${n}px fora da escala (14 16 18 20 24 32 40 48 64)`)
      }

      // cor de nota/acento como TEXTO falha contraste no tema claro (§2.3)
      const col = line.match(/(?<![-\w])color\s*:\s*var\(--(score-(?:high|mid|low)|accent)\)/)
      if (col) add(file, ln, 'erro', '§2 contraste', `color: var(--${col[1]}) no claro não passa 4,5:1; use --${col[1] === 'accent' ? 'accent-strong' : col[1] + '-text'}`)

      if (!isStyle && !isPixelArt && HEX.test(line) && !/#fff(fff)?\b/i.test(line) && !/mask/.test(line))
        add(file, ln, 'erro', '§2 tokens', `cor literal ${line.match(HEX)[0]}: use um token (var(--…))`)
      if (isStyle && !inTokens && HEX.test(line))
        add(file, ln, 'aviso', '§2 tokens', `cor literal ${line.match(HEX)[0]} fora do bloco de tokens`)
    })
  }

  // ---- template / script ----
  lines.forEach((raw, i) => {
    if (ignored(raw)) return
    const ln = i + 1
    if (!file.endsWith('.css') && EMOJI.test(raw)) add(file, ln, 'erro', '§5.10 ícones', 'emoji como ícone/decoração')
    if (/pok[eé]\s*-?ball|brand-ball/i.test(raw)) add(file, ln, 'erro', '§9 propriedade intelectual', 'Poké Ball proibida')
    if (/\b(heroicons|lucide|font-?awesome|material-icons)\b/i.test(raw))
      add(file, ln, 'erro', '§5.10 ícones', 'biblioteca de ícones vetoriais proibida')
    if (/\bHP:|\bLv\s?\d/.test(raw) && /\.vue$/.test(file)) add(file, ln, 'erro', '§9 propriedade intelectual', 'rótulo "HP:"/"Lv" copia tela de jogo')
    if (/\.vue$/.test(file) && /scoreColor\(/.test(raw) && /color\s*:\s*scoreColor|\{\s*color:\s*scoreColor/.test(raw))
      add(file, ln, 'erro', '§2 contraste', 'scoreColor() como cor de texto: use scoreTextColor()')
    if (/<img\b(?![^>]*\balt=)/.test(raw)) add(file, ln, 'erro', '§8 acessibilidade', '<img> sem alt')
  })
}

// ---- deriva entre style.css e tokens.json (uma só fonte de verdade) ----
if (!args.length && existsSync(TOKENS)) {
  const tokens = JSON.parse(readFileSync(TOKENS, 'utf8'))
  const css = readFileSync(STYLE, 'utf8')
  const light = css.slice(css.indexOf(':root {'), css.indexOf('@media (prefers-color-scheme: dark)'))
  const dark = css.slice(css.indexOf('@media (prefers-color-scheme: dark)'))
  const val = (block, name) => {
    const m = block.match(new RegExp(`--${name}\\s*:\\s*([^;]+);`))
    return m ? m[1].trim().toLowerCase() : null
  }
  const norm = (v) => v.replace(/^\{(.+)\}$/, 'var(--$1)').toLowerCase()
  for (const t of tokens.color.tokens) {
    for (const [theme, block] of [['light', light], ['dark', dark]]) {
      const want = norm(t.value[theme])
      const got = val(block, t.name) ?? (theme === 'dark' ? val(light, t.name) : null)
      if (got === null) add(STYLE, 1, 'erro', 'tokens', `--${t.name} não existe em style.css (${theme})`)
      else if (got !== want && !(theme === 'dark' && val(block, t.name) === null && want === norm(t.value.light)))
        add(STYLE, 1, 'erro', 'tokens', `--${t.name} (${theme}) é ${got} em style.css e ${want} em tokens.json`)
    }
  }
}

const order = { erro: 0, aviso: 1 }
findings.sort((a, b) => order[a.level] - order[b.level] || a.file.localeCompare(b.file) || a.line - b.line)
for (const f of findings) console.log(`${f.file}:${f.line}  [${f.level}] ${f.rule}: ${f.msg}`)
const errs = findings.filter((f) => f.level === 'erro').length
const warns = findings.length - errs
console.log(`\ndesign-lint: ${errs} erro(s), ${warns} aviso(s) em ${files.length} arquivo(s)`)
process.exit(errs ? 1 : 0)
