# FormField

Campo de formulário: rótulo `label` (Nunito 800 14px, caixa alta) acima, campo com moldura pixel, dica em `body` abaixo.

## O que você fornece
- `<div class="field">` com `<label for>` e o `input`/`textarea`/`select` com `id` estável.
- Dica opcional em `.hint` (`ink-2`).
- Select dentro de `.select-wrap` para ganhar o ▼ pixel.
- Upload: área `.upload` (borda tracejada de 2 `px`, fundo `surface-2`) com a ilustração `floppy`.

## Regras
Texto do campo 16px em `body`; código de exclusão em `mono`. Placeholder em `ink-3`. Em foco, a sombra aparece. Erros de validação aparecem num `DialogBox` `error` compacto logo abaixo, nunca só em vermelho no rótulo.
