# Pokerum

Ranqueamento **público e anônimo** de currículos contra uma vaga específica.
Você cola a descrição da vaga, os candidatos enviam o currículo, e o sistema
estima a pontuação que um motor ATS com IA daria — mostrando em que posição a
pessoa ficaria e o que melhorar.

- **Anonimização**: o nome real vira um nome aleatório e cada currículo ganha um
  codinome de Pokémon único. E-mail, telefone, links, CPF/RG/CNPJ, endereço e
  data de nascimento são removidos. O texto original **não é salvo**.
- **Vários algoritmos de avaliação**: cada currículo é avaliado por todos os motores
  ativos e cada motor tem o seu ranking. Por enquanto só o `heuristic-v1` está ativo:
  hard skills com sinônimos, similaridade TF-IDF, anos de experiência, senioridade,
  formação/idiomas e formato ATS. Novos motores entram com o tempo.

  A arquitetura já suporta LLMs, APIs externas e motores baseados em banco, com fila em
  segundo plano, retentativas e timeout. Para adicionar um motor, veja
  [`backend/api/src/engines/README.md`](backend/api/src/engines/README.md).

## Design

Visual **"cozy pixel"**, de RPG de bolso da era Game Boy, com a paleta creme/tinta/vermelho.
- Regras obrigatórias para qualquer mudança de UI: [`docs/design/REGRAS.md`](docs/design/REGRAS.md)
  (inclui checklist de PR).
- Pesquisa de referências (Dribbble, Pinterest, Godly, Lospec): [`docs/design/PESQUISA.md`](docs/design/PESQUISA.md).

## Stack

| Parte | Tecnologia |
|---|---|
| `backend/core` | Rust — anonimização + motor de pontuação (sem I/O) |
| `backend/api`  | Rust — axum + sqlx + SQLite, migrations em `backend/api/migrations` |
| `web`          | Vue 3 + Vite + TypeScript |

## Docker

### Desenvolvimento (hot reload)

```bash
docker compose -f docker-compose.dev.yml up --build
# frontend (Vite + HMR) → http://localhost:5173
# api (cargo-watch)     → http://localhost:8080
```

- O código de `backend/` e `web/` é montado como volume: salvar um `.rs` recompila
  e reinicia a API; salvar um `.vue`/`.ts` atualiza o navegador.
- Cache do cargo, `target/`, `node_modules` e o banco de dev ficam em volumes nomeados.
- O banco de dev já sobe com dados de exemplo (`POKERUM_SEED=1`).
- Portas ocupadas? `API_PORT=18081 WEB_PORT=15173 docker compose -f docker-compose.dev.yml up`.
- Zerar o banco de dev: `docker compose -f docker-compose.dev.yml down -v`.

### Produção

```bash
docker compose up -d --build        # → http://localhost:8080
```

- Imagem multi-stage (~100 MB): build do Vue + build release do Rust em uma imagem
  `debian-slim` com um único binário que serve a API e a SPA.
- O processo roda como usuário não-root; o SQLite fica no volume `pokerum-data` (`/data`).
- As migrations rodam sozinhas no start, e há healthcheck em `/api/health`.
- Variáveis: `POKERUM_PORT` (porta no host, padrão 8080) e `POKERUM_SEED=1` (dados de exemplo).
- Backup do banco: `docker compose cp app:/data/pokerum.db ./backup.db`.

## Rodando sem Docker

```bash
# backend (porta 8080) — aplica migrations automaticamente; --seed cria dados de exemplo
cd backend && cargo run -p pokerum_api -- --seed

# frontend (porta 5173, proxy /api -> 8080)
cd web && npm install && npm run dev
```

## Produção (um único binário servindo a SPA)

```bash
cd web && npm run build
cd ../backend && cargo build --release -p pokerum_api
POKERUM_WEB_DIST=../web/dist DATABASE_URL="sqlite://pokerum.db?mode=rwc" ./target/release/pokerum_api
```

## Produção (Coolify)

Publicado em **https://pokerum.wouerner.dev** pelo Coolify, com o build pack Docker Compose e
`docker-compose.coolify.yml` (mesma imagem do `Dockerfile`, sem porta publicada, banco SQLite no
volume `pokerum-data`). Passo a passo, DNS na Cloudflare, backup e conferência em
[docs/deploy.md](docs/deploy.md). Variáveis em `.env.example`.

**Google Analytics:** com `POKERUM_GA_ID=G-XXXXXXXXXX` o servidor injeta a tag do GA4 no
`index.html`, com aviso de cookies (LGPD, Consent Mode): sem aceite, o GA não grava cookies.
Detalhes em [docs/deploy.md](docs/deploy.md#google-analytics).

## Migrations

Ficam em `backend/api/migrations/NNNN_descricao.sql` e rodam no start da API.
**Nunca edite uma migration já aplicada** — para mudar o banco, crie o próximo
arquivo numerado (ex. `0003_add_job_location.sql`).

## Motores de avaliação (CLI)

```bash
docker compose exec app /app/pokerum_api engines list             # produção
cd backend && cargo run -p pokerum_api -- engines list             # local
# também: enable <id> | disable <id> | backfill <id> | retry <id>
```

## Testes

```bash
cd backend && cargo test
cd web && npm run build   # type-check + build
```
