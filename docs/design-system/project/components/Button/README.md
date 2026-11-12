# Button

Botão retangular com moldura pixel e rótulo `tag-px` (Silkscreen 16px) em MAIÚSCULAS.

## Variantes
- **Primário** `.btn-primary`: fundo `accent-strong`, texto `accent-ink`. Um por tela, para a ação principal ("Testar meu currículo").
- **Secundário** `.btn`: fundo `surface`, texto `ink`.
- **Perigo** `.btn-danger`: fundo `score-low-bg`, texto `score-low-text`. Só ações destrutivas ("Excluir definitivamente").
- **Fantasma** `.btn-ghost`: sem moldura; sublinha no hover. Ações secundárias ("Cancelar").
- `.small`: rótulo `label` (Nunito 800 14px, caixa alta) para ações secundárias dentro de janelas. Silkscreen nunca abaixo de 16px.

## O que você fornece
Rótulo de até ~4 palavras que diz exatamente o que acontece. `type`, `disabled` e o handler.

## Comportamento
Hover sobe 1 `px` e a sombra cresce; pressionar desce até a sombra; tudo em `steps(2)`. Altura mínima 44px. Foco tracejado afastado 4 `px`.

## Não faça
Cantos arredondados, ícones vetoriais, emoji, dois primários lado a lado, rótulo em frase longa.
