# Migrations (SQLite / sqlx)

As migrations são embutidas no binário com `sqlx::migrate!("./migrations")` e
aplicadas automaticamente no startup da API.

## Regra de ouro

**Nunca edite uma migration que já foi aplicada** (em qualquer ambiente). O sqlx
guarda o checksum de cada arquivo na tabela `_sqlx_migrations` e recusa iniciar
se um arquivo aplicado mudar.

Para qualquer mudança de schema, crie um **novo arquivo numerado**:

```
0001_init.sql
0002_scoring_engines.sql   # motores de avaliação + tabela evaluations
0003_...
```

- Prefixo numérico sequencial com 4 dígitos + `_descricao.sql`.
- Prefira mudanças aditivas (`ALTER TABLE ... ADD COLUMN`, novas tabelas/índices).
- Para mudanças destrutivas no SQLite (renomear/remover coluna com restrições),
  use o padrão "criar tabela nova → copiar dados → dropar antiga → renomear".
- As queries usam `sqlx::query`/`query_as` em runtime, então não é preciso
  `DATABASE_URL` em tempo de compilação.

## Histórico

| migration | o que faz |
|---|---|
| `0001_init.sql` | `jobs`, `resumes`, `applications` |
| `0002_scoring_engines.sql` | `scoring_engines`, `evaluations`, `jobs.ranking_engine`; migra as notas existentes para `evaluations` |

Adicionar um **motor de avaliação** novo não exige migration (ver `src/engines/README.md`).
