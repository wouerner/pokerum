# ScoreGauge

Medidor principal da nota: número grande, barra segmentada larga e selo da faixa.

## O que você fornece
`score` (0–100) e `size` (largura em px). A faixa define cor e rótulo: `FORTE CANDIDATO` (≥ 75), `NA DISPUTA` (50–74), `EM RISCO` (< 50).

## Regras
Número em `score-xl` (Fredoka 600 64px) com `score-*-text`; "/100" em `label` `ink-2`; selo em `tag-px` sobre `score-*-bg` com contorno na cor do texto. Um medidor por tela, no topo do resultado. Nada de arco vetorial liso.
