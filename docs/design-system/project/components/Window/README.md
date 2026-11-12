# Window

A janela é a moldura pixel que envolve todo card e painel do Pokerum.

## Quando usar
Todo bloco que agrupa conteúdo: cards de vaga, painéis de critérios, palavras-chave, formulários. É a base do `DialogBox`, dos botões e dos campos.

## O que você fornece
- O conteúdo. Título da janela em `h2` (Pixelify 24px); títulos de card e de vaga em `h3` (Nunito 800 20px); texto em `body`.
- Classe `.card` (fundo `surface` + padding `space-5`) ou `.px-frame` só para a moldura.

## Faça
- Ajuste a moldura por variáveis: `--fc` (cor) e `--sh` (tamanho da sombra).
- Deixe pelo menos 8px entre janelas: a moldura fica por fora da caixa.
- Áreas rebaixadas internas usam `surface-2` com borda de 1 `px`.

## Não faça
- `border-radius`, `border` comum no lugar da moldura, sombra com desfoque.
- `margin` na própria moldura (quebra elementos de largura total).
- Janela dentro de janela dentro de janela: no máximo um nível de aninhamento.
