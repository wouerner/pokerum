# Deploy em produção

Como o Pokerum vai para o ar em **https://pokerum.wouerner.dev**, no mesmo servidor Coolify dos
sites Vagas (veja `vagas-php/docs/deploy.md`, que segue o mesmo desenho).

Nada sigiloso entra neste arquivo: senhas, tokens, IP do servidor e IDs internos do Coolify e
da Cloudflare ficam só nos painéis.

## Visão geral

```
visitante ──HTTPS──▶ Cloudflare (proxy) ──HTTPS──▶ Traefik (Coolify) ──HTTP:8080──▶ serviço app
                                                                                   (API Rust + site Vue)
                                                                                         │
                                                                              volume pokerum-data (SQLite)
```

| Peça | O que é |
|---|---|
| Domínio | **https://pokerum.wouerner.dev** |
| DNS e HTTPS público | Cloudflare, zona `wouerner.dev` |
| Servidor e deploy | Coolify, build pack Docker Compose (`docker-compose.coolify.yml`) |
| Proxy reverso | Traefik do Coolify, com certificado Let's Encrypt |
| Imagem | `Dockerfile` da raiz: build do site (Node) + build da API (Rust) + runtime Debian slim |
| Banco | SQLite em `/data/pokerum.db`, no volume nomeado `pokerum-data`. Migrations rodam sozinhas no start |

## Primeira publicação

1. **Repositório.** O Coolify publica a partir de um repositório Git (GitHub). O projeto precisa
   estar num repositório com a branch `main`.
2. **Cloudflare.** Na zona `wouerner.dev`, crie o registro **A** `pokerum` apontando para o IP do
   servidor do Coolify, primeiro como **DNS only** (nuvem cinza), para o Let's Encrypt emitir o
   certificado.
   - SSL/TLS da zona já está em **Full** (não use Flexible: o Traefik redireciona HTTP→HTTPS e
     entraria em loop).
3. **Coolify.**
   - *New Resource → Public/Private Repository* (o deste projeto), branch `main`.
   - Build pack **Docker Compose**, *Docker Compose Location* `/docker-compose.coolify.yml`.
   - Domínio do serviço **`app`**: `https://pokerum.wouerner.dev` (sempre com `https://`; com
     `http://` o Traefik não pede certificado). A porta interna (8080) vem de
     `SERVICE_URL_APP_8080` no compose, então não vai no domínio.
   - *Environment Variables*: só se quiser mudar algum padrão do `.env.example`. Em produção,
     `POKERUM_SEED` fica `0` (sem vagas fictícias), a não ser que queira a vitrine com exemplos.
   - Deploy. O build compila Rust em modo release: a primeira vez leva alguns minutos.
4. **Certificado e proxy.** Quando `https://pokerum.wouerner.dev` responder com certificado do
   Let's Encrypt, ligue o proxy da Cloudflare (**Proxied**, nuvem laranja).
5. **Conferir** (abaixo).

## Atualizar

**O deploy não é automático:** um push na `main` não publica nada. Faça o deploy pelo painel do
Coolify (ou pela API).

- As migrations (`backend/api/migrations/`) rodam sozinhas no start. Nunca edite uma já aplicada.
- O servidor encerra limpo com SIGTERM. Avaliações em segundo plano interrompidas pelo deploy são
  retomadas no próximo start.
- Novo motor de avaliação inline faz backfill sozinho no start; motores com custo (LLM/API) só
  com `backfill` explícito (`backend/api/src/engines/README.md`).

## Google Analytics

Defina `POKERUM_GA_ID=G-XXXXXXXXXX` em *Environment Variables* no Coolify e faça redeploy (não
precisa rebuild: a tag é injetada pelo servidor no `index.html`). Vazio desliga.

- **LGPD (Consent Mode):** o consentimento padrão é negado. O site mostra um aviso de cookies
  (`ConsentBar`); só com "Aceitar" o GA grava cookies. Sem aceite, o GA recebe apenas sinais
  anônimos (modelagem de conversões). A escolha fica em `localStorage["pokerum-ga-consent"]` e pode
  ser revista pelo link "Cookies" no rodapé.
- **Rotas do Vue:** o GA4 conta as trocas de página do SPA pela medição otimizada (*Enhanced
  measurement → Page changes based on browser history events*, ligada por padrão no fluxo da web).
- **Conferir:** `curl -s https://pokerum.wouerner.dev/ | grep -o 'gtag/js?id=[^"]*'` e o relatório
  *Tempo real* do GA com o aviso aceito.

## Operação

Pelo terminal do serviço `app` no Coolify:

```bash
/app/pokerum_api engines list            # motores e estado
/app/pokerum_api engines backfill <id>   # avalia candidaturas antigas com um motor
```

**Backup:** o banco inteiro é o arquivo `/data/pokerum.db` (com `-wal`/`-shm` ao lado). Use o
backup de volume do Coolify ou, no servidor:

```bash
docker run --rm -v <volume>:/data -v "$PWD":/out alpine \
  sh -c 'apk add -q sqlite && sqlite3 /data/pokerum.db ".backup /out/pokerum-$(date +%F).db"'
```

(`<volume>` é o nome que o Coolify deu ao `pokerum-data`, visível em `docker volume ls`.)

## Conferir depois de um deploy

```bash
curl -sI https://pokerum.wouerner.dev/ | grep -iE '^(HTTP|server)'                  # HTTP/2 200
curl -s -o /dev/null -w '%{http_code} %{redirect_url}\n' http://pokerum.wouerner.dev/   # redireciona para https
curl -s https://pokerum.wouerner.dev/api/health                                     # {"ok":true}
curl -s https://pokerum.wouerner.dev/api/engines | head -c 200                      # heuristic-v1
curl -s -o /dev/null -w '%{http_code}\n' https://pokerum.wouerner.dev/vagas/qualquer  # 200 (fallback do SPA)
```

## Antes de divulgar

O site aceita envios anônimos (vagas e currículos), então vale proteger contra abuso:

- **Limite de envios:** uma regra de *Rate limiting* na Cloudflare para `POST /api/*`
  (ex.: 20 por minuto por IP) resolve sem mudar código.
- **Moderação:** não há painel para remover vagas abusivas; hoje isso é feito direto no SQLite.
- **Marca:** o nome "Pokerum" e os codinomes usam nomes de Pokémon (marca registrada). As imagens
  são todas originais (`docs/design/REGRAS.md` §9), mas o risco de marca existe se o projeto crescer.
