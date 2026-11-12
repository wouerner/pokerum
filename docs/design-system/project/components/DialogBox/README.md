# DialogBox

Caixa de diálogo de RPG: janela com moldura dupla, aba de quem fala, retrato opcional e ▼ piscando.

## Quando usar
Obrigatória para: aviso de privacidade, veredito da nota, estados vazios, erros da API, "Carregando…", "Avaliando…" de algoritmos em segundo plano e o aviso do código de exclusão.

## O que você fornece
- `speaker`: quem fala, em `tag-px` (Silkscreen 16px), MAIÚSCULAS curtas (`POKERUM`, `SISTEMA`, `ERRO`, `ATENÇÃO`, `ATS CLÁSSICO`).
- O texto, em `dialog` (18px, entrelinha 1.7), até ~70 caracteres por linha.
- `more`: mostra o ▼ quando há continuação ou ação.
- `variant`: `default`, `info` (`surface-2`), `warn` (`score-mid-*`), `error` (`score-low-*`).
- `portrait` (opcional): ilustração original em 56×56. `pokerum-mark` quando fala o Pokerum.
- `compact`: texto 16px para espaços estreitos.

## Não faça
- Texto em fonte pixel dentro do diálogo.
- Retrato com sprite ou personagem de terceiros.
- Efeito de digitação além do primeiro diálogo da página, ou que não pule com clique.
