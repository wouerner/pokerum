Um RPG de bolso aconchegante: papel creme, tinta escura, molduras de pixel, caixas de diálogo e menus com cursor, com texto confortável de ler. Toda tela do Pokerum é montada com os tokens e componentes deste sistema. As regras completas, com o porquê de cada uma, estão em `docs/design/REGRAS.md` no repositório; a pesquisa de referências, em `docs/design/PESQUISA.md`.

## Princípios

1. **Pixel na moldura, conforto no texto.** O pixel mora nas bordas, rótulos, números e ilustrações. Parágrafos são sempre `body` (Nunito).
2. **Paleta curta.** Quatro tons quentes (`ink`, `ink-2`, `line-soft`, `bg`) e um acento (`accent`). Cor nova precisa de motivo.
3. **Contraste é para o que importa.** O maior contraste vai para ação, nota e status, nunca para enfeite.
4. **Tudo na grade.** A unidade é `px` (2px). Bordas, sombras e deslocamentos são múltiplos inteiros dela. Nada de 0.5px ou 1.5px.
5. **Movimento em quadros.** Transições usam `steps()`, nunca `ease`.
6. **Aconchego.** Microcopy gentil, espaço generoso, luz quente, nada de neon.

## Conteúdo e voz

- Português do Brasil, segunda pessoa, tom de quem ajuda: "Você ficaria em 1º de 3 candidatos", "Seu currículo tem boas chances de passar pela triagem automática desta vaga."
- Frases normais em *sentence case*. MAIÚSCULAS só em `tag-px` (botões, abas: `PUBLICAR VAGA`, `FORTE CANDIDATO`) e em `label` (rótulos pequenos: `SIMULADOR DE ATS · ANÔNIMO`, `PESO 40%`).
- Erros dizem o que houve e o que fazer, sem culpar: "Código de exclusão inválido." Páginas de erro podem ter humor gentil: "Esta página fugiu para o mato."
- Sem emoji. Ícones são glifos pixel (▶ ▼ ✓ ✗) ou ilustrações deste sistema.
- O codinome é texto (ex.: `Skarmory`, apelido `Carlos Silva`). O retrato ao lado é um **monstrinho original** gerado a partir do nome, nunca o sprite, a silhueta ou a arte do Pokémon.

## Cor

- **Página:** `bg` com o pontilhado `bg-dots`. **Janelas:** `surface`. **Áreas rebaixadas** dentro de janelas (slots, chips neutros): `surface-2`.
- **Texto:** `ink` para principal, `ink-2` para secundário, `ink-3` para terciário (eyebrow, apelido, placeholder). Os três passam 4,5:1 sobre `bg` e `surface` nos dois temas.
- **Acento:** `accent` só em preenchimentos grandes (marca, carregador). Botão primário e texto vermelho usam `accent-strong`; o texto sobre ele é `accent-ink`.
- **Links:** só `accent-2`.
- **Nota:** a faixa da nota define três cores. ≥ 75 → `score-high`, 50–74 → `score-mid`, < 50 → `score-low`. Use `score-*` para **preencher** barras, `score-*-text` para **texto e contorno** e `score-*-bg` para **fundo suave**. No tema claro, `score-*` e `accent` nunca são cor de texto (o âmbar dá 2,1:1).
- **Molduras:** janelas usam `frame`, controles usam `frame-strong` (3:1 no escuro). A sombra é `shadow-color`.
- **Proibido:** neon, gradiente colorido, glow azul/roxo, "RGB gamer". `glow` (âmbar) é opcional, um por tela, só no claro.
- **Cor nunca sozinha:** encontrada/faltando leva ✓/✗ além da cor; faixa da nota leva o rótulo em texto.

## Tipografia

Pixel no que é grande e curto, Nunito no que se lê. A fonte pixel é a identidade, mas fica em no máximo 25% do texto de cada página (meta: 10–20%).

| Papel | Família | Estilos | Use para |
|---|---|---|---|
| Display | `display` (Pixelify Sans) | `h1` 48, `h1-mobile` 32, `h2` 24, `name` 20 | h1, h2, marca, codinomes |
| Números | `num` (Fredoka 600) | `score-xl` 64, `position` 40, `score` 20 | notas, posições, contagens em destaque (classe `.num`) |
| Sotaque | `label` (Silkscreen) | `tag-px` 16 | botões, navegação, abas, aba do diálogo, selo da faixa, nome do algoritmo |
| Texto | `body` (Nunito) | `h3` 20/800, `dialog` 18, `body` 16, `body-strong` 16, `label` 14/800 caixa alta, `chip` 14/700 | títulos de card, parágrafos, diálogos, rótulos pequenos, palavras-chave |
| Código | `mono` (JetBrains Mono) | `code` 16 | só o código de exclusão |

- Silkscreen só em 16px, MAIÚSCULAS, até ~3 palavras. Pixelify só a partir de 20px, em textos curtos.
- Rótulo pequeno (eyebrow, rótulo de campo, "PESO 40%", "/100", tipo, "padrão") é `label`: Nunito 800, 14px, caixa alta, `letter-spacing: .06em`. Nunca fonte pixel abaixo de 16px.
- Conteúdo digitado por pessoas (título e descrição de vaga, palavras-chave, currículo, empresa) nunca vai em fonte pixel. Títulos de vaga usam `h3` (Nunito 800 20px); palavras-chave mantêm a grafia original ("Google Cloud").
- Texto corrido nunca abaixo de 16px, entrelinha 1.6. Nunca use Press Start 2P.
- Fontes pixel só nos tamanhos da tabela e com `font-synthesis: none`. Números (notas, posições) nunca em fonte pixel: use `.num` (Fredoka 600, tabular), que tem dígitos abertos e não confunde 6, 8 e 0.

## Forma e profundidade

- `radius` é 0 em tudo, inclusive botões, chips, campos e o carregador.
- Toda janela, botão e campo usa a **moldura pixel** (`.px-frame`): borda de 2 `px` desenhada com `box-shadow` nos quatro lados (o canto fica "mordido") mais a sombra dura. Não reescreva a técnica em cada componente: use a classe e ajuste `--fc` (cor) e `--sh` (sombra).
- Sombras sempre deslocadas e sem desfoque: `shadow-sm` (padrão) e `shadow` (destaque). Nada de `blur` nem `backdrop-filter`.
- A moldura fica por fora da caixa: elementos emoldurados ficam a pelo menos `space-2` (8px) um do outro.

## Espaço e layout

- Gutter lateral `space-4` (16px) em toda largura; conteúdo até 1040px.
- Janelas: padding `space-4` no celular e `space-5` a partir de 600px. Grade de janelas com gap `space-6`.
- Topo da página `space-8`, rodapé `space-16`.
- Tudo funciona em 390px de largura, sem rolagem horizontal.

## Movimento

- `steps(n)` sempre. Hover: 100ms `steps(2)`, o elemento sobe 1 `px`. Pressionar: desce 2 `px` até a sombra. Abrir janela: 200ms em 3 quadros. Barras de nota: ~400ms, um bloco por quadro.
- Permitidos: cursor ▶/▼ piscando (1s, `steps(2)`), barras enchendo, carregador em grade, pulinho do 404.
- Proibidos: parallax, transição suave de cor, skeleton shimmer.
- Com `prefers-reduced-motion: reduce`, toda animação desliga e o estado final aparece direto.

## Acessibilidade

- Foco: contorno **tracejado** de 2 `px` em `ink`, afastado 2 `px` (4 `px` em elementos emoldurados), mais o cursor ▶ em itens de menu.
- Área de toque mínima 44×44px (botões e campos já têm `min-height: 44px`).
- Glifos ▶ ▼ e ilustrações são `aria-hidden`; o estado vem de `aria-current` / `aria-selected`.
- Zoom a 200% não pode quebrar a grade.

## Iconografia e ilustração

- **Glifos** (grupo *Glyphs*): `cursor` ▶, `down` ▼, `check` ✓, `cross` ✗, todos 8×8, usados como máscara CSS (`.pi .pi-cursor` etc.) para herdar `currentColor`. Os arquivos do grupo são pintados em `ink` para visualização.
- **Ilustrações** (grupo *Illustrations*): pixel art original em grade de 16×16 (o 404 é 20×8), sempre em SVG com `shape-rendering: crispEdges` e escala inteira ×2 ou ×3. No app são desenhadas pelo componente `PixelArt` a partir de `web/src/pixelart.ts`, com cores em tokens.
  - `pokerum-mark`: a marca (folha de currículo com selo ✓). Cabeçalho, favicon, retrato do Pokerum no diálogo.
  - `floppy`: área de upload. `plant` e `mug`: toque aconchegante da home (no máximo 2 por página, longe do conteúdo). `404`: página não encontrada.
- **Monstrinhos** (retrato do candidato): gerados por código em `web/src/creatures.ts`, 16×16, simétricos, nas cores do tipo do codinome. Não são assets: cada nome tem o seu. Veja `CodenameBadge`.
- Nunca use bibliotecas de ícones vetoriais arredondados (Heroicons, Lucide), emoji como ícone ou ícones de terceiros.

## Propriedade intelectual

O gênero "RPG de bolso" é livre. Não pode aparecer nada específico de Pokémon ou da Nintendo: Poké Ball ou variações (círculo meio vermelho, meio branco), sprites e silhuetas de Pokémon, logotipo e fonte da marca, o corpo ou a moldura do Game Boy (nem como moldura de página nem como ícone), telas de jogo reconhecíveis (caixa de batalha com "HP:" e "Lv"), músicas e sons. Assets de terceiros só com licença registrada em `web/public/CREDITS.md`.

## Componentes

`Window` (moldura base), `DialogBox`, `Button`, `MenuCursor`, `FormField`, `ScoreBar`, `ScoreGauge`, `CodenameBadge`, `KeywordChips`, `EngineBadge`, `Loader`. Cada um tem guia e prévia nesta página. No app eles são componentes Vue em `web/src/components/` e classes em `web/src/style.css`; as prévias aqui são renderizações estáticas com o mesmo CSS.

## Revisão

Toda mudança de interface passa pelo agente `design-reviewer` (`.claude/agents/design-reviewer.md`) e pelo verificador `npm run lint:design` em `web/`. O checklist de PR está em `docs/design/REGRAS.md` §11.
