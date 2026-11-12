# CodenameBadge

Ficha do candidato: tile quadrado com um monstrinho original gerado a partir do codinome, codinome, apelido e selo de "tipo".

## O que você fornece
`codename`, `alias` opcional, `size` (`sm` 36px, `md` 40px, `lg` 64px) e `showType`.

## Regras
- O tipo (16 opções, como Fogo, Água, Fada) e o matiz vêm de um hash determinístico do codinome; a luminosidade vem do tema, para passar 4,5:1 nos dois temas.
- Codinome em `display`; apelido em `body` `ink-3`; tipo em `label` (Nunito 800 14px, caixa alta).
- O retrato é o monstrinho 16×16 de `creatureRows(codename)` (`web/src/creatures.ts`), escala ×2 (sm/md) ou ×3 (lg), com contorno em `line` e corpo no matiz do tipo. O mesmo nome gera sempre a mesma criatura.
- Nunca sprite, silhueta ou arte do Pokémon que dá nome ao codinome, nem traços característicos de personagens existentes.
