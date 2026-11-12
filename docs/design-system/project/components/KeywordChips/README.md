# KeywordChips

Lista de palavras-chave em chips retangulares com glifo e cor.

## O que você fornece
`keywords`, `variant` (`neutral`, `match`, `miss`) e o texto de vazio.

## Regras
`match` usa ✓ e `score-high-*`; `miss` usa ✗ e `score-low-*`; os dois têm texto oculto para leitor de tela ("encontrada:", "faltando:"). Texto em `chip` (Nunito 700 14px) na grafia original da palavra; palavras-chave são conteúdo, nunca fonte pixel. Gap `space-2`. A cor nunca é o único sinal.
