# Pokerum

Ranking público e anônimo de currículos contra vagas. Backend Rust (`backend/`), frontend Vue 3 (`web/`). Contrato da API em `CONTRACT.md`.

## Design system (obrigatório em qualquer mudança de interface)

- Antes de mexer em `web/`, leia `docs/design-system/project/README.md` e o `README.md` dos componentes que vai usar em `docs/design-system/project/components/`. As regras numeradas (§) estão em `docs/design/REGRAS.md`.
- Use os tokens de `web/src/style.css` (espelho de `docs/design-system/project/tokens.json`) e os componentes existentes antes de criar algo novo. Nada de cor literal, raio, sombra borrada, easing suave ou fonte fora do sistema.
- Mudou um token? Altere `style.css` **e** `tokens.json` juntos; `npm run lint:design` acusa divergência.
- Ao terminar uma mudança de UI: rode `cd web && npm run lint:design && npm run build` e chame o agente `design-reviewer` sobre os arquivos alterados. Só considere pronto com parecer APROVADO (ou ressalvas aceitas pelo usuário).
- Versão navegável do design system: https://claude.ai/artifact/7FP4KGb3LBi5duEcCBbJsr (publicada a partir de `docs/design-system/`; republique ao mudar tokens, componentes ou ilustrações).

## Deploy

- Produção: https://pokerum.wouerner.dev via Coolify (`docker-compose.coolify.yml`); detalhes em `docs/deploy.md`. Deploy é manual pelo painel. Variável nova precisa entrar no `.env.example` **e** no `docker-compose.coolify.yml`.

## Ambiente

- Disco quase cheio: compile e teste Rust dentro do container de dev (`docker compose -f docker-compose.dev.yml exec -T api cargo test`), não no host.
- Dev: `API_PORT=18081 WEB_PORT=15173 docker compose -f docker-compose.dev.yml up -d` (web em :15173 com hot reload).
