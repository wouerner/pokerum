# MenuCursor

Cursor ▶ que marca o item ativo, em foco ou em hover de menus, abas e listas navegáveis.

## Quando usar
Navegação do topo, abas de algoritmo, abas arquivo/texto do envio, linhas do ranking, cards da home, candidaturas na página do currículo.

## O que você fornece
- Item com classe `.menu-item` e, dentro, `<span class="cur pi pi-cursor" aria-hidden="true">` antes do texto.
- Estado por `aria-current="page"` (navegação) ou `aria-selected="true"` (abas). Nunca só por classe visual.

## Comportamento
O cursor aparece no hover e pisca em 2 quadros no item em foco ou ativo. A aba ativa ganha sublinhado de 2 `px`. Não troque a cor de fundo do item para indicar seleção.
