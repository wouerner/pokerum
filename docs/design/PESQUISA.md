# Pesquisa de design: "cozy pixel art", estilo RPG de Game Boy

**Objetivo:** dar ao Pokerum uma cara **aconchegante (cozy)** e de **pixel art**, no espírito dos RPGs
de bolso da era Game Boy / Game Boy Color, **mantendo a paleta atual**.

**Fontes pedidas:** Pinterest, Godly (hoje Recent) e Dribbble.
**Fontes complementares:** Lospec (paletas), tendências de UI e guias técnicos de CSS.
**Data:** 12/11/2026.

> As regras normativas que saíram desta pesquisa estão em [`REGRAS.md`](./REGRAS.md).

---

## 1. O que cada fonte entregou

### Dribbble
**Método:** o Dribbble bloqueia leitura automatizada (as páginas voltam vazias), e o bloqueio não
foi contornado. A análise usa o que o próprio Dribbble publica de cada shot nos índices de busca:
título, descrição, cores extraídas e tags.

| Shot | O que mostra | O que aproveitamos |
|---|---|---|
| [Game Boy Landing Page](https://dribbble.com/shots/4832767-Game-Boy-Landing-Page), Barbara Jura | Landing com paleta **quente**: `#E96F00 #F7A705 #BC8105 #A36806 #5B3E0E #9E3404`, quase preto `#0D0C09` e um único acento frio `#2B5870` | Paleta quente + 1 acento; tinta quase preta em vez de preto puro |
| [Gameboy color website (concept)](https://dribbble.com/shots/4879426-Gameboy-color-website-concept), ArmKum | Site construído **dentro do corpo do Game Boy**, com título em fundo amarelo | **Contra-exemplo:** usar o aparelho como moldura é *trade dress* da Nintendo (§9 das regras) |
| [Pocket Inventory: Adventure Book](https://dribbble.com/shots/26034942-Pocket-Inventory-Series-1-Adventure-Book-v1-4) e [Pixel Console](https://dribbble.com/shots/23038357-Pocket-Inventory-Series-9-Pixel-Console), Humble Pixel | Inventário pixel em formato de livro animado; **perfil + inventário** num só painel | Página do currículo como "perfil + inventário" (candidaturas como itens) |
| [5x8 bitmap font for Gameboy](https://dribbble.com/shots/2220192-5x8-bitmap-font-for-Gameboy), Trevor Brennan | Fonte bitmap mono "equilibrando legibilidade máxima e tamanho mínimo" | Piso de 12px para a fonte de rótulo; fonte pixel só para textos curtos |
| [Pixel Art Platform Game Landing Page](https://dribbble.com/shots/25600104-Pixel-Art-Platform-Game-Landing-Page-UI-UX-Web-Design), Dusan Ko | Landing com paleta terrosa e dessaturada e acento ferrugem | Dessaturar o entorno e reservar a saturação para o acento |
| [8-Bit Worlds](https://dribbble.com/shots/25976628-8-Bit-Worlds-Fantasy-Sci-Fi-and-Retro-Adventures-in-Pixel-Art), AWAIS | Taverna 8-bit "quente e convidativa", com mesas de madeira | Tom *cozy*: madeira, luz quente, objetos do dia a dia |
| [Game Boy Pixelart Detail](https://dribbble.com/shots/6708504-Game-Boy-Pixelart-Detail), Victor Heijler | Cidade com só 2–3 tons | Hierarquia por **luminosidade**, não por matiz |

### Pinterest
**Método:** os boards bloqueiam leitura automatizada, e o bloqueio não foi contornado. Usei a
**API pública de oEmbed** do Pinterest (a mesma usada para incorporar pins em sites) para obter
título, autor e miniatura de pins selecionados. As miniaturas foram **analisadas visualmente**,
inclusive com extração das cores dominantes. As imagens não foram copiadas para o repositório.

| Pin | O que mostra | Cores dominantes | O que aproveitamos |
|---|---|---|---|
| [Pixel Art Cozy Game Plant Decor UI shop](https://www.pinterest.com/pin/922675042398487201/) | Janela de loja sobre o cenário **escurecido**, **abas** no topo da janela, grade de **slots** creme com preço, barra de rolagem | `#a59991 #8b6e69 #978d87 #cebcae` (quentes e **dessaturados**) | Fundo escurecido e quente atrás de janelas sobrepostas; abas em cima da janela; grade de slots |
| [HUD: Dialogue Boxes INDEX](https://www.pinterest.com/pin/366902700873176270/), Sprite Might | Kits de molduras "9-slice" de diálogo em 3 cores, com **borda dupla** e cantos ornamentados | `#523a40 #9a8186 #c5bcbb` | Moldura dupla (linha interna) na caixa de diálogo |
| [Diálogo com retrato](https://www.pinterest.com/pin/191121577932080798/) (board "gameboy") | **Retrato à esquerda**, caixa de texto clara com borda dupla e **▼** no canto inferior direito | `#010101 #efefff #534f52` | Espaço de retrato na caixa de diálogo e ▼ de continuação |
| [Retro Gameboy Style Art](https://www.pinterest.com/pin/785315253742940376/) | Mapa de cidade com 4 tons de oliva, tiles de 16px | `#c4d0a2 #96a179 #858f6b #535a42` | Rampa de 4 tons com saltos fortes de luminosidade |
| [Selected Gameboy palette pixel art](https://www.pinterest.com/pin/317926054920709578/) | Cena com 4–5 verdes e **dithering** nas transições | `#b8dfa8 #8ac072 #417651 #14302b` | Dithering só como textura de fundo ou decoração |
| [Paleta quente](https://www.pinterest.com/pin/2040762327123438/) (The Gnome Company) | `C64304 FBE2A9 F5BE71 B26F18 F16B05` | — | Confirma o creme-pêssego + vermelho-laranja |
| [Paleta cozy](https://www.pinterest.com/pin/1196337398057591/) | `#CABCAB #AD7C59 #D4B4A1 #B85C48 #80221E` | — | Vermelhos terrosos e bege, compatíveis com o nosso acento |
| [Game Boy pixel art](https://cl.pinterest.com/pin/15058980000345013/) | Ícone do console em pixel | — | **Contra-exemplo:** o aparelho em si não pode aparecer (§9) |

**Boards de contexto:** [Game Boy Style Pixel Art](https://www.pinterest.com/abbeybricks/game-boy-style-pixel-art/) (≈610 pins) · [Gameboy UI](https://de.pinterest.com/bensauerball/gameboy-ui/) · [Cozy UI](https://www.pinterest.com/alrightlook/cozy-ui/) · [Cozy pixel art](https://ph.pinterest.com/mimimossie/cozy-pixel-art/) (≈440 pins) · [Pixel Game Dialogue](https://www.pinterest.com/ideas/pixel-game-dialogue/897633391179/).

### Godly (hoje "Recent")
**Situação:** o **Godly foi incorporado ao Recent**. O próprio `godly.website` aparece no índice com
o título "Recent — Design Inspiration", e todas as páginas antigas (`/website/...`) redirecionam
para `recent.design/?ref=godly`. A pesquisa foi feita no **acervo do Recent**, que mantém a mesma
curadoria e as tags de estilo (Retro, Playful, Minimal…).

| Item | Tags | O que aproveitamos |
|---|---|---|
| [Subway Surfers Loading State](https://recent.design/i/bjomut6-subway-surfers-loading-state), Shane Levine | Interface · Playful · Microinteraction · Loading Animation. Tem **"pixel-grid shimmer loader"** | **Loader em grade de pixels** no lugar do spinner circular |
| [404 Chicken Flock Animation](https://recent.design/i/uuhoew2-404-chicken-flock-animation), Ana Howard | Motion · Playful · Kinetic Type. O "404" é **formado por personagens** que se reorganizam | **404 lúdico** desenhado em blocos de pixel que "pulam" |
| [Displace](https://recent.design/i/yqc9ady-displace) | Retro · Interactive · Minimal. A home vira um **desktop** de sistema operacional retrô | Metáfora de interface retrô (janelas, abas, cursor) em vez de página "corporativa"; **sem** imitar um aparelho real |
| [Casino Hero](https://recent.design/i/dtnuyvc-casino-hero-section) | Retro · Dark · Grain | **Contra-exemplo:** retrô escuro e de alto contraste não é *cozy* |
| [Avara](https://recent.design/i/bimlbnk-avara) · [Frig App Icon](https://recent.design/i/v9qpcdh-frig-app-icon) | Playful · Character | Personagens e objetos amigáveis como toque de calor (aqui, em pixel e originais) |

**Conclusão sobre o Godly/Recent:** quase não há sites inteiros em pixel no acervo. O que ele
oferece é a camada de **micro-interações lúdicas**: carregamento, 404 e metáforas de interface.
Pixel art aparece como **detalhe de interação**, não como tema de página inteira, e isso reforça
a regra "pixel na moldura, conforto no texto".

### Complementares
- **Lospec, paletas "GB" e "cozy":**
  - **HoneyGB** (`#3e3a42 #877286 #f0b695 #e9f5da`): "focada em cores quentes, do branco-amarelado
    ao roxo escuro… cores aconchegantes em poucos tons". É a mais próxima do nosso caso.
  - Também: EarthGB, Coldfire GB, Pumpkin Patch 15, Cozy Tones.
- **daisyUI Trends, "Pixel art" e "RPG pixel art":** convenções consolidadas:
  - grade inteira e escala só por múltiplos inteiros;
  - paleta indexada com tons explícitos de luz, meio-tom, sombra e status;
  - cantos em degrau, sem anti-aliasing;
  - fontes bitmap em tamanhos inteiros;
  - movimento em quadros;
  - caixas de diálogo emolduradas com entrelinha generosa;
  - **cor de maior contraste reservada para o que é interativo e para status**.
- **"Cozy Gamer" (guia de estilo):**
  - luz quente, texturas e plantas;
  - fonte pixel **só em rótulos** e o corpo numa sans **arredondada** (Nunito);
  - entrelinha de 1.6–1.8 e contraste ≥ 4.5:1;
  - **sem neon/RGB "gamer"**.
- **Guias de tipografia pixel:**
  - fontes pixel são de display;
  - devem cobrir só 20–30% do texto da página;
  - pares sugeridos: Pixelify Sans + Nunito;
  - Press Start 2P é "muito alta, de fliperama".
- **Técnica (CSS):**
  - bordas em degrau com `box-shadow` ou "9-slice";
  - `image-rendering: pixelated`;
  - evitar valores fracionários;
  - SVG com `crispEdges` em vez de centenas de `box-shadow`.

## 2. Síntese: o que é "cozy pixel GB"

| Eixo | Game Boy / pixel | Cozy | Pokerum (síntese) |
|---|---|---|---|
| Cor | 4 tons + 1 acento | quentes, papel, luz âmbar | **paleta atual** organizada como rampa quente de 4 tons + vermelho de acento |
| Forma | cantos em degrau, molduras | macio, sem quinas agressivas | molduras em pixel com **cantos chanfrados** (degrau de 1 unidade), sem raio vetorial |
| Tipo | bitmap em tamanhos inteiros | arredondada, entrelinha alta | **Pixelify Sans** (títulos, números), **Silkscreen** (rótulos curtos), **Nunito** (texto) |
| Componentes | diálogo, menu com cursor ▶, barra de vida | cartões, adesivos, plantas | caixa de diálogo, menus com cursor, barras segmentadas, "ficha" do candidato |
| Movimento | quadros (steps), cursor piscando | lento, gentil | `steps()`, durações curtas, cursor ▼ piscando, respeitando *reduced motion* |
| Textura | dithering, grade | papel, madeira | fundo pontilhado em grade de 4px (já existe) |
| Densidade | HUD compacto | espaço generoso | densidade média: menus compactos dentro de páginas espaçosas |

## 3. O que manter do site atual
- **Cores:** creme `#f6f1e7`, tinta `#1b1a22`, vermelho `#e5432f`, verde, âmbar e vermelho de nota,
  e o modo escuro.
- **Bordas e sombras:** bordas grossas e **sombra deslocada sem desfoque**, que já é linguagem pixel.
- **Fundo:** o pontilhado, que já parece dithering.

## 4. O que muda
- **Forma:** raio arredondado (14px) vira **cantos em pixel**.
- **Tipografia:** Bricolage Grotesque e JetBrains Mono viram Pixelify Sans, Silkscreen e Nunito.
  O mono fica só para o código de exclusão.
- **Componentes:** avisos e estados viram **caixas de diálogo**; listas e abas ganham **cursor ▶**;
  barras de nota viram **barras segmentadas** tipo vida/XP.
- **Movimento:** transições suaves viram **quadros** (`steps`).
- **Logo:** o logo atual lembra uma **Poké Ball** (marca registrada) e deve virar um símbolo
  **original**.

## 5. Cuidados de propriedade intelectual
"Estilo RPG de Game Boy" é um **gênero visual** (paleta curta, pixels, diálogos, menus com cursor)
e pode ser usado livremente. Já os elementos **específicos de Pokémon** não podem:
- sprites e personagens;
- a Poké Ball;
- logotipo e fonte da marca;
- telas e layouts reconhecíveis dos jogos, como a caixa de batalha com "HP" e "Lv" exatamente
  como no jogo.

Os codinomes continuam sendo **só texto**. Ver regra §9 em `REGRAS.md`.

---

## Fontes
- Dribbble: [Game Boy Landing Page, Barbara Jura](https://dribbble.com/shots/4832767-Game-Boy-Landing-Page) · [Coleção "Gameboy", Jake Yem](https://dribbble.com/jakeyem/collections/5637773-Gameboy) · [Pixel Art Platform Game Landing Page, Dusan Ko](https://dribbble.com/shots/25600104-Pixel-Art-Platform-Game-Landing-Page-UI-UX-Web-Design) · [Game Boy Pixelart Detail, Victor Heijler](https://dribbble.com/shots/6708504-Game-Boy-Pixelart-Detail) · [Game boy. Pixel art., Gora Gorskiy](https://dribbble.com/shots/7358043-Game-boy-Pixel-art) · [busca "pixel gameboy"](https://dribbble.com/search/pixel-gameboy)
- Pinterest: [Game Boy Style Pixel Art](https://www.pinterest.com/abbeybricks/game-boy-style-pixel-art/) · [Gameboy UI](https://de.pinterest.com/bensauerball/gameboy-ui/) · [Pixel Art UI and Gameboy Color](https://www.pinterest.com/hummel4726/pixel-art-ui/) · [Green/90s Gameboy Pixels](https://www.pinterest.com/nito_placing_pixels/green90s-gameboy-pixels/) · [Cozy pixel art](https://ph.pinterest.com/mimimossie/cozy-pixel-art/) · [Cozy game plant decor UI](https://www.pinterest.com/pin/922675042398487201/) · [Gameboy Pixel Art Background](https://www.pinterest.com/ideas/gameboy-pixel-art-background/923739407368/)
- Godly (hoje Recent): [godly.website → recent.design](https://recent.design/?ref=godly) · [Subway Surfers Loading State](https://recent.design/i/bjomut6-subway-surfers-loading-state) · [404 Chicken Flock Animation](https://recent.design/i/uuhoew2-404-chicken-flock-animation) · [Displace](https://recent.design/i/yqc9ady-displace) · [Casino Hero](https://recent.design/i/dtnuyvc-casino-hero-section) · [Avara](https://recent.design/i/bimlbnk-avara)
- Sites em pixel (complementar): · [Sage: Retro web design / pixel art](https://sage.agency/blog/websites-that-use-cool-pixel-art-design/) · [Framer: sites anos 90](https://www.framer.com/blog/90s-websites/) · [Webflow: retro websites](https://webflow.com/blog/retro-website) · [Codrops: 8-bit universe](https://tympanus.net/codrops/2025/12/23/building-a-nostalgic-8-bit-universe-with-modern-tech-a-vibe-coding-journey/)
- Paletas: [Lospec HoneyGB](https://lospec.com/palette-list/honeygb) · [Lospec tag "gb"](https://lospec.com/palette-list/tag/gb) · [Lospec tag "cozy"](https://lospec.com/palette-list/tag/cozy)
- Convenções: [daisyUI Trends, Pixel art](https://trends.daisyui.com/trend/pixel-art/) · [daisyUI Trends, RPG pixel art](https://trends.daisyui.com/trend/rpg-pixel-art/) · [Cozy Gamer design reference](https://chrislemke.github.io/website_designs/examples/Cozy_Gamer.html)
- Assets de referência (só olhar, não usar sem checar licença): [Game Boy dialogue frames](https://2bitcrook.itch.io/gameboy-dialogue-frames) · [Cozy Pixel UI Essentials](https://hazardpay.itch.io/cozy-ui-essentials-pack) · [Pixel UI cozy style](https://ldcmleo.itch.io/pixel-ui-interface-cozy-style) · [GB menu CC0](https://opengameart.org/node/180090)
- Tipografia e técnica: [pares de fontes pixel](https://pixelfontpicks-u4.pages.dev/pixel-font-pairing-guide-for-modern-web-layouts-pixel-fonts-for-web-design) · [LogRocket: pixel art com CSS](https://blog.logrocket.com/designing-pixel-art-using-css-only/) · [CSS-Tricks: box-shadow](https://css-tricks.com/?p=255501) · [image-rendering](https://modern-css.com/reference/properties/image-rendering/)
