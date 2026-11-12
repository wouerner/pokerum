# Pokerum — contrato entre módulos

Ranqueamento público e anônimo de currículos contra uma vaga, simulando como
motores ATS (com IA) pontuariam cada candidato.

## Layout

```
backend/            Cargo workspace
  core/  (crate `pokerum_core`, lib)  — anonimização + pontuação. SEM I/O, SEM banco.
  api/   (crate `pokerum_api`, bin)   — axum + sqlx(SQLite) + migrations, usa pokerum_core
web/                Vue 3 + Vite + TypeScript (SPA), consome /api
```

## Interface pública de `pokerum_core` (obrigatória)

```rust
// pokerum_core::anonymize
pub struct AnonymizedResume {
    pub codename: String,        // nome de Pokémon, ex: "Bulbasaur"
    pub alias: String,           // nome aleatório de pessoa, ex: "Marina Duarte"
    pub text: String,            // texto com PII removida/substituída
    pub redactions: Vec<Redaction>,
}
pub struct Redaction { pub kind: RedactionKind, pub count: usize }
pub enum RedactionKind { Name, Email, Phone, Url, Document /*CPF/RG/CNPJ*/, Address /*CEP, rua*/, BirthDate }

/// `taken_codenames`: codinomes já usados (devem ser únicos globalmente).
/// `declared_name`: nome real informado pelo candidato no formulário (opcional) —
/// também é usado para encontrar e substituir ocorrências no texto.
pub fn anonymize(raw_text: &str, declared_name: Option<&str>,
                 taken_codenames: &std::collections::HashSet<String>,
                 rng: &mut impl rand::Rng) -> AnonymizedResume;
pub fn pokemon_names() -> &'static [&'static str];   // >= 151 nomes

// pokerum_core::scoring
pub struct JobProfile { ... }   // derivado da descrição da vaga
pub fn analyze_job(title: &str, description: &str) -> JobProfile;
pub fn score(job: &JobProfile, resume_text: &str) -> ScoreReport;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScoreReport {
    pub total: f64,                       // 0..=100
    pub breakdown: Vec<CriterionScore>,   // ver critérios abaixo
    pub matched_keywords: Vec<String>,
    pub missing_keywords: Vec<String>,    // palavras importantes da vaga ausentes
    pub suggestions: Vec<String>,         // dicas em PT-BR para melhorar o currículo
    pub engine_version: String,           // ex "heuristic-v1"
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CriterionScore { pub key: String, pub label: String, pub weight: f64, pub score: f64 /*0..=100*/, pub detail: String }
```

`JobProfile` também deve ser `Serialize + Deserialize` (a api guarda em JSON).

### Critérios de pontuação (heuristic-v1)
| key | peso | descrição |
|---|---|---|
| `hard_skills` | 0.40 | skills técnicas da vaga presentes no CV (dicionário de skills + sinônimos, ex: "js"↔"javascript", "postgres"↔"postgresql") |
| `semantic` | 0.20 | similaridade TF-IDF/cosseno entre vaga e CV (stopwords PT+EN, normalização de acentos) |
| `experience` | 0.15 | anos de experiência exigidos vs. detectados (datas "2019 - 2023", "atual", "5 anos de experiência") |
| `seniority_title` | 0.10 | aderência de cargo/senioridade (júnior/pleno/sênior/lead, título da vaga no CV) |
| `education_lang` | 0.05 | formação/certificações e idiomas pedidos (inglês etc.) |
| `ats_format` | 0.10 | legibilidade ATS: seções reconhecíveis (experiência, formação, skills), tamanho, bullets, ausência de excesso de símbolos |

Suporte bilíngue PT-BR e EN.

## Banco (SQLite, migrations em `backend/api/migrations/`, sqlx `migrate!`)

Migrations numeradas `0001_init.sql`, `0002_...` — nunca editar migration já aplicada; mudanças futuras = nova migration.

```
jobs(id INTEGER PK, slug TEXT UNIQUE, title TEXT, company TEXT NULL, description TEXT,
     profile_json TEXT, created_at TEXT DEFAULT CURRENT_TIMESTAMP)
resumes(id INTEGER PK, codename TEXT UNIQUE, alias TEXT, anonymized_text TEXT,
        redactions_json TEXT, owner_token_hash TEXT, created_at TEXT)
applications(id INTEGER PK, job_id FK jobs ON DELETE CASCADE, resume_id FK resumes ON DELETE CASCADE,
        score REAL, report_json TEXT, engine_version TEXT, created_at TEXT,
        UNIQUE(job_id, resume_id))
```
O texto ORIGINAL do currículo nunca é persistido. O `owner_token` (aleatório, devolvido
uma única vez ao candidato) permite ao dono ver/excluir seu currículo; só o hash (sha256) é salvo.

## API REST (prefixo `/api`, JSON, porta 8080)

| método | rota | corpo / resposta |
|---|---|---|
| GET | `/api/health` | `{ "ok": true }` |
| GET | `/api/jobs` | `[JobSummary]` com `{id, slug, title, company, created_at, applicants, top: [{position, codename, alias, score}]}` (top = até 3 melhores) |
| POST | `/api/jobs` | `{title, company?, description}` → `Job` |
| GET | `/api/jobs/:slug` | `Job` = `{id, slug, title, company, description, created_at, keywords: [String]}` |
| GET | `/api/jobs/:slug/ranking` | `[RankingEntry]` = `{position, codename, alias, score, breakdown, created_at}` ordenado desc |
| POST | `/api/jobs/:slug/apply` | multipart: `file` (pdf/txt/md) OU campo `text`; `name?` (nome real, só p/ anonimizar) → `ApplyResponse` |
| GET | `/api/resumes/:codename` | `{codename, alias, anonymized_text, applications:[{job_slug, job_title, score, position, total}]}` |
| GET | `/api/jobs/:slug/applications/:codename` | `{codename, alias, score, position, total, report: ScoreReport}` |
| DELETE | `/api/resumes/:codename` | header `X-Owner-Token` → 204 |

`ApplyResponse = { codename, alias, owner_token, score, position, total, report: ScoreReport, anonymized_text }`

Erros: `{ "error": "mensagem" }` com status 4xx/5xx.
Limite upload 5 MB. CORS liberado em dev. Em produção a api serve `web/dist` como estático (fallback SPA para index.html) se a pasta existir (env `POKERUM_WEB_DIST`).

Env: `DATABASE_URL` (default `sqlite://pokerum.db?mode=rwc`), `PORT` (default 8080).

---

## Motores de avaliação (multi-algoritmo) — v2

O sistema suporta vários **motores de avaliação** (scoring engines): heurísticas locais,
LLMs, APIs externas, consultas a banco etc. Cada candidatura é avaliada por **todos os
motores ativos**; cada vaga escolhe qual motor ordena seu ranking padrão.

### Conceitos
- `EngineKind`: `"heuristic" | "llm" | "external_api" | "database"`
- `Execution`: `"inline"` (rápido, roda dentro do POST /apply) | `"background"` (lento: LLM/API; roda em fila)
- `EvaluationStatus`: `"pending" | "running" | "done" | "failed"`
- Todo motor devolve um `ScoreReport` (mesmo formato do heuristic-v1). Os `breakdown[].key`
  são livres por motor — o frontend deve renderizar qualquer lista de critérios.

### Motores incluídos agora
| id | kind | execução | descrição |
|---|---|---|---|
| `heuristic-v1` | heuristic | inline | motor atual (6 critérios ponderados) — padrão das vagas |

`keywords-v1` ("ATS clássico") existe no código como exemplo, mas não está registrado. Motor fora do
registro fica desligado: suas avaliações não aparecem na API e vagas que o usavam caem para `heuristic-v1`.

### Banco — migration `0002_scoring_engines.sql`
```
scoring_engines(id TEXT PK, name TEXT, kind TEXT, description TEXT, execution TEXT,
                enabled INTEGER DEFAULT 1, config_json TEXT DEFAULT '{}', created_at, updated_at)
evaluations(id INTEGER PK, application_id FK applications ON DELETE CASCADE,
            engine_id FK scoring_engines, status TEXT, score REAL NULL, report_json TEXT NULL,
            error TEXT NULL, attempts INTEGER DEFAULT 0, duration_ms INTEGER NULL,
            created_at, updated_at, finished_at NULL, UNIQUE(application_id, engine_id))
jobs.ranking_engine TEXT NOT NULL DEFAULT 'heuristic-v1'   (nova coluna)
```
`applications.score/report_json/engine_version` ficam como legado (nota do heuristic-v1 no momento do envio).
A fonte da verdade passa a ser `evaluations`.

### Tipos novos
```ts
Engine = { id, name, kind: EngineKind, description, execution: Execution,
           enabled: boolean,      // ligado no banco
           available: boolean,    // configurado (ex.: API key presente)
           unavailable_reason: string | null }

Evaluation = { engine_id, engine_name, engine_kind: EngineKind,
               status: EvaluationStatus,
               score: number | null,          // só quando done
               position: number | null,       // posição no ranking DESTE motor (só done)
               total: number | null,          // nº de candidatos avaliados (done) por este motor
               report: ScoreReport | null,
               error: string | null,          // só quando failed
               updated_at: string }
```

### Mudanças na API (compatíveis — só acréscimos)
| método | rota | mudança |
|---|---|---|
| GET | `/api/engines` | **novo** → `[Engine]` (todos os registrados; ativos = `enabled && available`) |
| GET | `/api/jobs` | `JobSummary` ganha `ranking_engine: string`; `top` usa o motor do ranking da vaga |
| POST | `/api/jobs` | corpo aceita `ranking_engine?` (precisa ser motor ativo; padrão `heuristic-v1`) |
| GET | `/api/jobs/:slug` | `Job` ganha `ranking_engine: string` e `engines: [Engine]` (motores ativos) |
| GET | `/api/jobs/:slug/ranking?engine=ID` | `engine` opcional (padrão = `ranking_engine` da vaga). Só entram avaliações `done` desse motor. `RankingEntry` ganha `engine_id` |
| GET | `/api/jobs/:slug/applications/:codename` | ganha `primary_engine: string` e `evaluations: [Evaluation]` |
| POST | `/api/jobs/:slug/apply` | `ApplyResponse` ganha `primary_engine` e `evaluations` (inline já `done`; background `pending`) |
| GET | `/api/resumes/:codename` | `applications[]` usa o motor de ranking de cada vaga |

Campos de topo `score/position/total/report` continuam existindo = motor de ranking da vaga
(`primary_engine`); se ele ainda não terminou, caem para `heuristic-v1` (sempre inline).

Para motores `background`, o frontend deve fazer polling de
`GET /api/jobs/:slug/applications/:codename` (ex.: a cada 3 s) enquanto houver
avaliação `pending`/`running`.
