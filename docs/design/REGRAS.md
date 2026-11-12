# Regras de design: Pokerum "Cozy Pixel GB"

Fonte das decisões: [`PESQUISA.md`](./PESQUISA.md).

Estas regras são **normativas**. Elas usam três níveis:
- **DEVE**: obrigatório.
- **NÃO DEVE**: proibido.
- **PODE**: opcional.

Toda tela nova ou alterada segue este documento. Os tokens vivem em `web/src/style.css`.

**Em uma frase:** *um RPG de bolso aconchegante*. Papel creme, tinta escura, molduras de pixel,
caixas de diálogo e menus com cursor, mas com texto confortável de ler e acessível.

---

## 1. Princípios

1. **Pixel na moldura, conforto no texto.** O pixel está nas bordas, nos rótulos, nos números e
   nos ícones. Parágrafos são sempre numa fonte arredondada e legível.
2. **Paleta curta.** Quatro tons quentes mais um acento. Cada cor nova precisa de justificativa.
3. **Contraste é para o que importa.** O maior contraste fica para ações, nota e status, nunca
   para enfeite.
4. **Tudo na grade.** A unidade é `--px: 2px`. Medidas, bordas, sombras e deslocamentos são
   múltiplos inteiros dela.
5. **Movimento em quadros.** Nada desliza suave; tudo "pula" em poucos quadros, como num jogo de 8 bits.
6. **Aconchego.** Microcopy gentil, espaço generoso entre seções, luz e cores quentes, sem neon.

## 2. Cor

A paleta atual **fica**. Ela é organizada como uma rampa quente de 4 tons (estilo
[HoneyGB](https://lospec.com/palette-list/honeygb)) mais um acento.

### 2.1 Rampa de tela (4 tons)
| Token | Claro | Escuro | Papel |
|---|---|---|---|
| `--gb-0` / `--ink` | `#1b1a22` | `#f2eee6` | tinta: texto, molduras, sombras |
| `--gb-1` / `--ink-2` | `#55525f` | `#b9b4c4` | texto secundário |
| `--gb-2` / `--line-soft` | `#d9d1c0` | `#34313f` | divisórias, trilho das barras, bordas internas |
| `--gb-3` / `--bg` | `#f6f1e7` | `#14131b` | papel / fundo da tela |

**Superfícies:**
- `--surface` (`#fffdf8` / `#1d1c27`): janelas.
- `--surface-2` (`#f1ebdd` / `#272535`): áreas rebaixadas.

### 2.2 Acento ("cor do jogador")
- `--accent` `#e5432f`: **preenchimentos** (barra do 1º lugar, destaques grandes, marca).
- `--accent-strong` `#cc3621`: **botão primário** com texto branco (5,1:1) e texto vermelho sobre
  o papel (4,5:1). No escuro, o acento `#ff5c47` passa como texto (6,0:1).
- `--accent-2` `#2f5be5`: **só links**. Não usar como cor de decoração.

### 2.3 Status de nota (barras de "vida")
| Faixa | Preenchimento | Texto no claro | Fundo suave |
|---|---|---|---|
| ≥ 75 | `--score-high` `#1f9d55` | `--score-high-text` `#17733f` | `--score-high-bg` |
| 50–74 | `--score-mid` `#e09a12` | `--score-mid-text` `#8a5a00` | `--score-mid-bg` |
| < 50 | `--score-low` `#d93a2b` | `--score-low-text` `#b42d20` | `--score-low-bg` |

**Regras de contraste:**
- **DEVE:** texto com no mínimo **4,5:1**; números grandes (≥ 24px) e ícones com no mínimo **3:1**.
  Medido no claro:
  - `--ink-3` `#8a8695` no papel dá 3,2:1 e **NÃO DEVE** ser usado em texto. Use o novo `--ink-3`
    `#6e6a79` (4,7:1).
  - O âmbar `#e09a12` como texto dá 2,1:1, então **NÃO DEVE** colorir texto no claro. Use
    `--score-*-text`.
- **NÃO DEVE:** neon, gradientes coloridos, "RGB gamer", glow azul ou roxo.
- **PODE:** um brilho **âmbar** suave (`--glow: #ffe4a8`) para foco ou estado de "carregando",
  com no máximo um por tela e **só no modo claro** (no escuro ele estoura o contraste).

## 3. Grade, forma e profundidade

### 3.1 Unidade
- `--px: 2px`. **Todas** as medidas de borda, sombra, deslocamento e degrau são `calc(var(--px) * n)`.
  Espaçamentos de layout usam múltiplos de 4px.
- **NÃO DEVE** haver valores fracionários (0.5px, 1.5px), porque borram a borda do pixel.

### 3.2 Cantos
- **Raio zero em tudo, sem exceção:** `--radius: 0`. Nada de `border-radius` vetorial, inclusive
  em botões, chips, inputs e no indicador de carregamento (ver §5.12).
- Janelas e botões têm **cantos chanfrados de 1 degrau**, com uma de duas técnicas:
  - (a) `clip-path` com polígono em degraus (preferível para elementos com fundo);
  - (b) moldura feita de `box-shadow` em 4 lados, sem borda (o canto fica "mordido").
- Use a classe utilitária `.px-frame`. **NÃO DEVE** repetir a técnica à mão em cada componente.

```css
/* moldura pixel: borda de 2 unidades com canto chanfrado + sombra deslocada */
.px-frame {
  --b: calc(var(--px) * 2);              /* espessura (4px) */
  border: 0;
  background: var(--surface);
  box-shadow:
    0 calc(var(--b) * -1) 0 0 var(--line),   /* topo */
    0 var(--b) 0 0 var(--line),              /* base */
    calc(var(--b) * -1) 0 0 0 var(--line),   /* esquerda */
    var(--b) 0 0 0 var(--line),              /* direita */
    calc(var(--b) * 2) calc(var(--b) * 2) 0 0 var(--shadow-color); /* sombra deslocada */
}
```
- A moldura fica **por fora** da caixa (box-shadow). Por isso, elementos com moldura **DEVEM** ter
  pelo menos 8px de distância entre si (gap do layout). **NÃO DEVE** usar `margin` na própria
  moldura, porque ela quebra elementos de largura total.
- Implementação: `.px-frame` em `web/src/style.css`. `.card`, `.btn` e os campos de formulário
  reaproveitam a mesma regra, com espessura e sombra ajustáveis por `--fc` e `--sh`.

### 3.3 Profundidade
- **Sombra sempre deslocada, sem desfoque** (`Xpx Ypx 0`), na cor da tinta. Os tamanhos são
  `--shadow-sm` (2 unidades) e `--shadow` (4 unidades). Isso já é a linguagem atual e fica.
- **Hover:** o elemento sobe 1 unidade (`translate(-2px,-2px)`) e a sombra cresce 1 unidade.
- **Pressionar:** desce até a sombra (`translate(4px,4px)`, sombra 0), como o botão de um console.
- **NÃO DEVE** usar `blur` em sombras nem `backdrop-filter`.

## 4. Tipografia

**Princípio: pixel no que é grande e curto, Nunito no que se lê.** A fonte pixel é a identidade, mas
em excesso cansa: em 12/11/2026 ela ocupava 38% do texto das páginas, e a Silkscreen em 12px
maiúsculo (chips, "peso", rótulos) era o principal ponto de ilegibilidade. A dosagem abaixo a levou
para 6–21%.

| Papel | Fonte | Tamanhos (px inteiros) | Uso |
|---|---|---|---|
| Display | **Pixelify Sans** 700 | 20, 24, 32, 40, 48, 64 | h1, h2, marca, codinomes |
| Números | **Fredoka** 600, `tabular-nums` | 20, 24, 40, 64 | notas, posições ("1º"), contagens em destaque. Classe `.num`, token `--font-num` |
| Sotaque | **Silkscreen** 400 | **16** (só) | botões, navegação, abas, aba de quem fala no diálogo, selo da faixa (FORTE CANDIDATO), nome do algoritmo. Classe `.tag-px` |
| Título de card | **Nunito** 800 | 20 | `h3`: títulos de vaga, de seção dentro de janela. É conteúdo, quebra linha |
| Rótulo | **Nunito** 800, MAIÚSCULAS, `letter-spacing: .06em` | 14 | eyebrows, rótulos de campo, "peso 40%", "/100", tipo do codinome, "padrão", contagens, botão pequeno. Classes `.label` / `.eyebrow` |
| Chip | **Nunito** 700, grafia original | 14 | palavras-chave ("Google Cloud", não "GOOGLE CLOUD") |
| Texto | **Nunito** 400/600/700 | 16 (mín.), 18 | parágrafos, descrições, sugestões, diálogos, formulários |
| Código | JetBrains Mono 400 | 14, 16 | só o código de exclusão e ids técnicos |

**DEVE:**
- Fonte pixel **≤ 25% dos caracteres visíveis** de cada página (meta: 10–20%).
- Silkscreen **só em 16px, MAIÚSCULAS, até ~3 palavras**. Nunca em 12px.
- Pixelify **só a partir de 20px** e em textos curtos (uma linha de título, um nome, um número).
- Fontes pixel só nos tamanhos da tabela (tamanhos intermediários borram a grade) e com `font-synthesis: none`.
- Texto corrido em Nunito, ≥ 16px, entrelinha 1.6, *sentence case*.

**NÃO DEVE:**
- **Conteúdo digitado por pessoas** (título e descrição de vaga, palavras-chave, texto do currículo,
  nome de empresa) em fonte pixel. A exceção é o codinome, que é curto e gerado pelo sistema.
- Parágrafo, sugestão, lista ou mensagem de erro em fonte pixel.
- Rótulo pequeno (< 16px) em fonte pixel: use `.label`.
- Press Start 2P: muito "fliperama", agressiva para o tom cozy.

**Números (notas, posições, contagens):** em **Fredoka 600** (`.num` / `--font-num`), nunca em fonte
pixel: dígitos pixelados se confundem (6/8/0) e cansam na leitura de notas. A Fredoka é arredondada,
combina com a Nunito e mantém o tom de jogo. Use `tabular-nums` quando estiverem alinhados em colunas. Contagem dentro de rótulo ("3 CANDIDATOS") segue o rótulo.

## 5. Componentes

### 5.1 Janela (`.px-frame`)
- **Uso:** todo card ou painel vira uma "janela de RPG": fundo `--surface`, moldura `.px-frame`,
  padding de 6 a 8 unidades.
- **Título:** `h2` em Pixelify Sans; títulos menores dentro da janela são `h3` (Nunito 800). Pode ter uma **aba** no topo-esquerdo (Silkscreen 16 sobre
  fundo de tinta), como o nome de quem fala num diálogo.

### 5.2 Caixa de diálogo (`DialogBox`)
- **Uso obrigatório:**
  - aviso de privacidade;
  - veredito da nota ("Seu currículo tem boas chances…");
  - estados vazios;
  - erros da API;
  - "avaliando…" de motores em segundo plano;
  - aviso do código de exclusão.
- **Forma:** janela larga com **moldura dupla** (a moldura pixel mais uma linha interna de 1
  unidade a 2 unidades da borda), aba de "quem fala" (ex.: `POKERUM`, `SISTEMA`, `ATS CLÁSSICO`) e
  um **▼ piscando** no canto inferior direito quando houver ação ou continuação.
- **Retrato:** PODE ter um retrato à esquerda (slot `portrait`, tile de 56px). Use a marca pixel
  quando quem fala é o Pokerum e a ficha do codinome quando é o candidato. Retrato é sempre arte
  **original**.
- **Texto:** Nunito 16–18px, entrelinha 1.7, no máximo ~70 caracteres por linha.
- **Efeito "máquina de escrever":** PODE ser usado só no **primeiro** diálogo da página, deve ser
  pulável com clique e **não pode** rodar com `prefers-reduced-motion`.

### 5.3 Menus e cursor ▶
- **Itens com o cursor:** listas navegáveis (menu do topo, abas de algoritmo, linhas do ranking,
  opções de select próprio). O item em hover, foco ou ativo ganha um **cursor ▶** à esquerda, em
  vez de mudar a cor de fundo.
- **Cursor:** um glifo pixel (SVG 8×8 ou o caractere ▶ em Silkscreen) que **pisca em 2 quadros**
  no item focado.
- **Abas:** o cursor ▶ fica antes do nome do item ativo, que também ganha sublinhado de 2 unidades.

### 5.4 Barra de nota ("vida")
- **Forma:** trilho com moldura de 1 unidade e **10 segmentos** (cada um vale 10 pontos),
  preenchidos com a cor de status. O último segmento parcial é preenchido em proporção, dentro da
  grade de 2px.
- **Rótulo:** número à direita em Fredoka (`.num`). Um prefixo curto `.label` (`PTS`) PODE
  aparecer.
- **Animação:** a barra **enche em quadros** (`steps(10)`, cerca de 400ms) ao entrar na tela.
- **NÃO DEVE** copiar o layout de barra de vida de um jogo específico (ver §9).

### 5.5 Medidor principal (nota de 0 a 100)
- **Forma:** número grande (Pixelify 64) com uma **barra segmentada** grande abaixo, ou um arco
  **segmentado** em blocos (sem arco vetorial liso).
- **Rótulo de faixa:** "FORTE CANDIDATO", "NA DISPUTA" ou "EM RISCO", em Silkscreen sobre o
  fundo suave da faixa.

### 5.6 Ficha do candidato (codinome)
- **Retrato:** um **monstrinho original** em pixel art 16×16, gerado por regras a partir de um
  hash do codinome (`web/src/creatures.ts`), num **tile quadrado** com a cor do "tipo". O mesmo nome
  gera sempre a mesma criatura. Tamanhos do tile: 36px (pequeno), 40px (médio) e 64px (grande); o
  sprite usa escala ×2 (32px) ou ×3 (48px).
- **Gerador:** corpo arredondado simétrico, olhos, boca, topo da cabeça (orelhas, antenas, chifres,
  topete), barriga, pintas e pés, sorteados sem relação com o personagem que dá nome ao codinome.
  Cores: corpo/sombra/barriga no matiz do tipo, contorno e olhos em `--line`. **NÃO DEVE** ganhar
  traços característicos de personagens existentes (bochechas vermelhas, cauda em raio, bulbo nas
  costas, casco, chama na cauda…).
- **Ao lado:** codinome (Pixelify 24–32) e apelido (Nunito).
- **Selo de tipo:** chip `.label` (Nunito 800 14px) com a cor vinda do hash.
- **NÃO DEVE:** sprite, silhueta, arte oficial ou desenho reconhecível do Pokémon que dá nome ao
  codinome. O nome é só texto; o retrato é sempre o monstrinho gerado.

### 5.7 Botões
- **Formato:** retangulares, cantos chanfrados, rótulo Silkscreen 16 (o tamanho pequeno usa `.label`, Nunito 800 14px).
- **Primário:** fundo `--accent-strong` com texto branco. **Secundário:** fundo `--surface` com
  texto de tinta. **Perigo:** fundo `--score-low-bg` com texto `--score-low-text`.
- **Comportamento:** hover e pressionar conforme §3.3. O foco segue §8.

### 5.8 Chips e etiquetas
- **Formato:** retângulos de 1 unidade de borda, Nunito 700 14px na grafia original.
- **Encontrada / faltando:** usar ✓ / ✗ em pixel **e** a cor. A cor nunca é o único sinal.

### 5.9 Formulários
- **Inputs e textarea:** moldura pixel, fundo `--surface`, texto Nunito 16, rótulo `.label` (Nunito 800 14px)
  acima.
- **Upload:** área tracejada em degraus (borda `dashed` de 2 unidades) com um ícone pixel de disquete.

### 5.10 Ícones
- **Desenho:** pixel art própria, em **SVG 16×16 ou 8×8** com `shape-rendering: crispEdges`,
  escalado só ×2 ou ×3.
- **Imagens bitmap:** usar `image-rendering: pixelated`.
- **NÃO DEVE:** emoji como ícone, bibliotecas de ícones vetoriais arredondados (Heroicons, Lucide) ou
  ícones de Pokémon.

### 5.11 Janelas sobrepostas (modais e gavetas)
- **Fundo:** o cenário atrás **escurece em tom quente**, com `--ink` a 45% de opacidade e sem blur.
  A página continua visível, como num menu de jogo aberto sobre o mapa.
- **Abas:** ficam **no topo da janela**, encostadas na moldura, e a ativa recebe o cursor ▶.
- **Abertura:** em 3 quadros (§7), com foco preso dentro da janela e **Esc** para fechar.

### 5.12 Carregamento
- **Forma:** **grade 2×2 de pixels** que acendem em sequência (sentido horário, `--accent` sobre
  `--line-soft`), trocando de quadro a cada 200ms. Classe `.spinner`, com tamanhos de 20px e 14px.
- **Mensagem:** carregamentos longos usam a caixa de diálogo "Carregando… ▼" (§5.2).
- **NÃO DEVE:** spinner circular, barra de progresso indeterminada lisa ou *skeleton shimmer*.

### 5.13 Erros e páginas vazias
- **Página de erro (404/500):** PODE ter uma **ilustração pixel lúdica e original** (ex.: o número
  em blocos que "pulam" em 2 quadros), seguida de uma caixa de diálogo com o caminho de volta.
- **Tom:** humor gentil ("Esta página fugiu para o mato."), nunca culpar a pessoa.

### 5.14 Listas como inventário
- PODE apresentar coleções curtas (candidaturas de um currículo, palavras-chave) como **grade de
  slots** quadrados em `--surface-2` com moldura de 1 unidade, como num inventário.
- Cada slot DEVE ter texto legível (a grade nunca substitui o rótulo).

## 6. Fundo, textura e elementos cozy
- **Fundo:** pontilhado em grade de 4px (o atual), em `--bg-dots`. É o nosso "dithering".
- **Elementos cozy:** PODE haver **até 2 elementos decorativos** pixel por página (planta no vaso,
  caneca, gato dormindo, luminária), **originais**, com `aria-hidden="true"` e longe do conteúdo.
- **NÃO DEVE:** scanlines ou "efeito CRT" sobre o texto (prejudica a leitura), nem textura sobre
  as janelas.

## 7. Movimento
- **Curva:** sempre `steps(n)`, nunca `ease`/`cubic-bezier`. Durações curtas:
  - hover: 80–120ms, `steps(2)`;
  - abrir painel: 160–240ms, `steps(4)`;
  - barras: ~400ms, `steps(10)`.
- **Animações permitidas:** cursor ▶/▼ piscando (1s, `steps(2)`), barras enchendo, "pulo" de
  hover, entrada de janela em 3 quadros (escala 0.96 → 1), digitação do 1º diálogo.
- **DEVE:** sob `prefers-reduced-motion: reduce`, desligar piscar, digitação e entradas (o estado
  final aparece direto).
- **NÃO DEVE:** parallax, transições suaves de cor, *skeleton shimmer* (use o diálogo "Carregando…▼").

## 8. Acessibilidade (inegociável)
- **Contraste:** conforme §2.
- **Foco visível:** contorno **tracejado** de 2 unidades em `--ink`, deslocado 2 unidades, **mais**
  o cursor ▶ quando for item de menu.
- **Leitura:** corpo ≥ 16px; rótulos nunca abaixo de 14px; fontes pixel nunca abaixo de 16px (Silkscreen) e 20px (Pixelify); zoom a 200% sem quebrar a grade.
- **Ícones:** decorativos com `aria-hidden`. Os glifos ▶/▼ são decorativos, e o estado vem de
  `aria-current` / `aria-selected`.
- **Toque:** área mínima de 44×44px.
- **Cor:** nunca é o único indicador (ver §5.8).

## 9. Propriedade intelectual
O estilo "RPG de bolso" é um gênero e pode ser usado. **NÃO DEVE** aparecer nada que seja
específico de Pokémon ou da Nintendo:
- a **Poké Ball** e variações (círculo metade vermelha, metade branca, com faixa e botão central);
- sprites, silhuetas e retratos de Pokémon ou personagens;
- logotipo, fonte do logotipo, o nome "Pokémon" estilizado;
- a moldura ou o corpo do Game Boy (inclusive como "moldura" da página ou ícone do aparelho), o
  logo "Nintendo" ou "GAME BOY";
- telas e layouts reconhecíveis dos jogos (ex.: caixa de batalha com "HP:", "Lv" e a barra na
  posição exata do jogo);
- músicas e sons.

**Regras relacionadas:**
- **Marca do Pokerum:** um símbolo **original** em pixel (ex.: um cartucho ou uma "ficha" com um
  ✓, ou um coração pixel com uma estrela).
- **Codinomes:** continuam sendo nomes em texto.
- **Assets de terceiros:** só com licença verificada e registrada em `web/public/CREDITS.md`.

## 10. Tokens (resumo)

```css
:root {
  --px: 2px;
  --radius: 0; --radius-sm: 0;
  --font-display: 'Pixelify Sans', var(--font);
  --font-label: 'Silkscreen', var(--font);
  --font-num: 'Fredoka', var(--font);
  --font: 'Nunito', ui-sans-serif, system-ui, sans-serif;
  --mono: 'JetBrains Mono', ui-monospace, monospace;
  --ink-3: #6e6a79;              /* era #8a8695 (3,2:1); no escuro #958fa3 (era #7f7a8c, 4,1:1) */
  --accent-strong: #cc3621;
  --score-low-text: #b42d20; --score-mid-text: #8a5a00; --score-high-text: #17733f;
  --shadow-color: var(--line);
  --glow: #ffe4a8;
  --step-fast: 100ms; --step-mid: 200ms; --step-slow: 400ms;
}
/* modo escuro: as cores atuais continuam; os *-text usam as próprias cores de nota */
```

## 11. Checklist de revisão (PR de UI)
- [ ] Nenhum `border-radius` > 0, nenhum valor fracionário de px, nenhuma sombra com blur.
- [ ] Fonte pixel ≤ 25% do texto: Pixelify ≥ 20px em títulos/números/codinomes, Silkscreen só 16px em botões/abas; rótulos pequenos e conteúdo digitado em Nunito.
- [ ] Contraste ≥ 4,5:1 em texto (claro e escuro); a cor de nota não é usada como texto no claro.
- [ ] Avisos, erros e estados vazios usam `DialogBox`.
- [ ] Animações com `steps()` e desligadas em `prefers-reduced-motion`.
- [ ] Foco visível tracejado; alvos ≥ 44px.
- [ ] Nada de §9 (Poké Ball, sprites, layouts de jogo).
- [ ] Testado a 390px de largura e no modo escuro.

## 12. Rastreabilidade: de onde vem cada regra
Detalhes e links em [`PESQUISA.md`](./PESQUISA.md).

| Regra | Referência |
|---|---|
| §2 Paleta como rampa de 4 tons quentes + 1 acento | Lospec **HoneyGB**; Pinterest "Retro Gameboy Style Art" e "Selected Gameboy palette" (4–5 valores); Dribbble Barbara Jura (quente + 1 acento frio) |
| §2 Entorno dessaturado, saturação só no acento e no status | Pinterest "Cozy Game … UI shop" (`#a59991 #8b6e69`); Dribbble Dusan Ko; daisyUI "RPG pixel art" |
| §2 Sem neon/RGB; luz e cor quentes | "Cozy Gamer"; Dribbble AWAIS (taverna); Recent "Casino Hero" como contra-exemplo |
| §3 Raio zero, cantos em degrau, grade inteira, sombra dura | daisyUI "Pixel art"; Pinterest "HUD Dialogue Boxes" (9-slice) |
| §4 Pixel só em títulos e rótulos; corpo em Nunito; ≤ 30% | Guias de pares de fontes pixel; "Cozy Gamer"; Dribbble Trevor Brennan (legibilidade × tamanho) |
| §5.2 Caixa de diálogo com moldura dupla, aba, retrato e ▼ | Pinterest "HUD Dialogue Boxes" e pin de diálogo com retrato; daisyUI "RPG pixel art" |
| §5.3 Menus com cursor ▶ | Convenção de RPG de bolso (daisyUI "RPG pixel art"; boards "Gameboy UI") |
| §5.4/5.5 Barras segmentadas para nota | daisyUI "RPG pixel art" (barras com moldura e cantos pixel) |
| §5.11 Fundo escurecido e quente, abas no topo da janela | Pinterest "Cozy Game … UI shop" |
| §5.12 Loader em grade de pixels | Godly/Recent "Subway Surfers Loading State" (pixel-grid loader) |
| §5.13 404 lúdico em pixel | Godly/Recent "404 Chicken Flock Animation" |
| §5.14 Listas como inventário | Pinterest "UI shop" (slots); Dribbble Humble Pixel "Pocket Inventory" |
| §6 Elementos cozy (planta, caneca), dithering só no fundo | "Cozy Gamer"; Pinterest "Cozy pixel art" e "Selected Gameboy palette" |
| §7 Movimento em quadros (`steps`) | daisyUI "Pixel art" / "RPG pixel art"; Recent (micro-interações) |
| §9 Não usar o aparelho nem a marca de terceiros | Dribbble ArmKum e pin do console (contra-exemplos); Recent "Displace" (metáfora retrô sem imitar aparelho) |

## 13. Onde está implementado
| Regra | Arquivo |
|---|---|
| Tokens, `.px-frame`, botões, campos, cursor ▶, foco, movimento | `web/src/style.css` |
| Caixa de diálogo com moldura dupla e retrato (§5.2) | `web/src/components/DialogBox.vue` |
| Loader em grade de pixels (§5.12) | `.spinner` em `web/src/style.css` |
| 404 em pixel (§5.13) | `web/src/views/NotFoundView.vue` + `NOT_FOUND` em `pixelart.ts` |
| Barra de nota (§5.4) e medidor (§5.5) | `ScoreBar.vue`, `ScoreGauge.vue` |
| Ficha do candidato (§5.6) | `CodenameBadge.vue` |
| Pixel art original: marca, disquete, planta, caneca (§5.10, §6, §9) | `web/src/pixelart.ts` + `PixelArt.vue`, `web/src/assets/px/*.svg` |
| Cor de texto acessível por nota (§2.3) | `scoreTextColor()` em `web/src/util.ts` |
| Créditos de fontes e assets | `web/public/CREDITS.md` |
