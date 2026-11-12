// Pixel art original do Pokerum (§5.10 / §9). Nada aqui reproduz arte de terceiros.
// Cada string é uma linha de 16 pixels; as letras são mapeadas para tokens de cor.

/** Marca: uma "ficha" (currículo) com canto dobrado e um selo com ✓. */
export const BRAND_MARK = {
  rows: [
    '................',
    '.XXXXXXXXX......',
    '.XWWWWWWWXX.....',
    '.XWWWWWWWXWX....',
    '.XWAAWWWWXXXX...',
    '.XWAAWWWWWWWX...',
    '.XWWWWWWWWWWX...',
    '.XWIIIIIIIWWX...',
    '.XWWWWWWWWWWX...',
    '.XWIIIIIXXXXXXX.',
    '.XWWWWWWXAAAAAX.',
    '.XWIIIIIXAAAAcX.',
    '.XWWWWWWXAAAcAX.',
    '.XWWWWWWXcAcAAX.',
    '.XWWWWWWXAcAAAX.',
    '.XXXXXXXXXXXXXX.',
  ],
  palette: {
    X: 'var(--ink)',
    W: 'var(--surface)',
    I: 'var(--ink-3)',
    A: 'var(--accent)',
    c: '#ffffff',
  },
}

/** Disquete (área de upload). */
export const FLOPPY = {
  rows: [
    '................',
    '.XXXXXXXXXXXXX..',
    '.XDDMMMMMMMDDXX.',
    '.XDDMMMMmmMDDDX.',
    '.XDDMMMMmmMDDDX.',
    '.XDDMMMMMMMDDDX.',
    '.XDDDDDDDDDDDDX.',
    '.XDDDDDDDDDDDDX.',
    '.XDLLLLLLLLLLDX.',
    '.XDLllllllllLDX.',
    '.XDLLLLLLLLLLDX.',
    '.XDLllllllLLLDX.',
    '.XDLLLLLLLLLLDX.',
    '.XDLLLLLLLLLLDX.',
    '.XXXXXXXXXXXXXX.',
    '................',
  ],
  palette: {
    X: 'var(--ink)',
    D: 'var(--ink-2)',
    M: 'var(--line-soft)',
    m: 'var(--ink)',
    L: 'var(--surface)',
    l: 'var(--accent)',
  },
}

/** Planta no vaso (decoração cozy, §6). */
export const PLANT = {
  rows: [
    '................',
    '.......G........',
    '......GGG...G...',
    '...G..GgG..GG...',
    '..GGG.GgG.GGG...',
    '..GgGGGgGGGg....',
    '...GGgGgGgGG....',
    '....GGgggGG.....',
    '......GgG.......',
    '...XXXXXXXXXX...',
    '...XPPPPPPPPX...',
    '...XXXXXXXXXX...',
    '....XPPPPPPX....',
    '....XPPPPPPX....',
    '.....XPPPPX.....',
    '.....XXXXXX.....',
  ],
  palette: {
    G: 'var(--score-high)',
    g: 'var(--score-high-text)',
    P: 'var(--accent)',
    X: 'var(--ink)',
  },
}

/** Caneca fumegante (decoração cozy, §6). */
export const MUG = {
  rows: [
    '................',
    '.....S...S......',
    '....S...S.......',
    '.....S...S......',
    '....S...S.......',
    '................',
    '..XXXXXXXXX.....',
    '..XCCCCCCCXXXX..',
    '..XMMMMMMMX..X..',
    '..XMMMMMMMX..X..',
    '..XAAAAAAAX..X..',
    '..XMMMMMMMXXXX..',
    '..XMMMMMMMX.....',
    '...XMMMMMX......',
    '....XXXXX.......',
    '................',
  ],
  palette: {
    S: 'var(--ink-3)',
    C: 'var(--ink-2)',
    M: 'var(--surface)',
    A: 'var(--accent)',
    X: 'var(--ink)',
  },
}

/** "404" em blocos (página não encontrada). Dígitos 5×7 com sombra deslocada de 1px. */
export const NOT_FOUND = {
  rows: [
    'X...X...XXX...X...X.',
    'X...Xs.X...X..X...Xs',
    'X...Xs.X...Xs.X...Xs',
    'XXXXXs.X...Xs.XXXXXs',
    '....Xs.X...Xs.....Xs',
    '....Xs.X...Xs.....Xs',
    '....Xs..XXX.s.....Xs',
    '.....s...sss.......s',
  ],
  palette: {
    X: 'var(--accent)',
    s: 'var(--ink)',
  },
}
