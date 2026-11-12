# Loader

Carregador em grade 2×2 de pixels que acendem em sequência, no sentido horário.

## Quando usar
Botões enviando, atualização automática, avaliações em segundo plano. Carregamento de página inteira usa um `DialogBox` "Carregando…" com o carregador.

## Regras
Classe `.spinner` (20px) ou `.spinner-xs` (14px); `accent` sobre `line-soft`; troca de quadro a cada 200ms; sempre `aria-hidden` com o texto de estado ao lado. Nunca spinner circular, barra indeterminada lisa ou skeleton shimmer.
