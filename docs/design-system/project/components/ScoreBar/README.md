# ScoreBar

Barra de nota segmentada em 10 blocos de 10 pontos, como uma barra de vida de RPG.

## O que você fornece
- `score` de 0 a 100. A cor vem da faixa: ≥ 75 `score-high`, 50–74 `score-mid`, < 50 `score-low`.
- `showValue`: número à direita em `score` (Fredoka 600 20px), colorido com `score-*-text`.
- `height`: altura dos blocos, sempre par (padrão 10px).

## Regras
O último bloco parcial é preenchido em proporção, na grade de 2px. A barra enche um bloco por quadro (~400ms). Tem `role="meter"` com o valor. Não use rótulos "HP:" ou "Lv" nem copie o layout de barra de um jogo.
