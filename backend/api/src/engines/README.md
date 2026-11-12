# Motores de avaliação

Cada candidatura é avaliada por **todos os motores ativos**. Um motor recebe a vaga e o
currículo **já anonimizado** e devolve um `ScoreReport` (nota 0–100, critérios, palavras-chave
encontradas e faltando, sugestões). O resultado fica na tabela `evaluations`, uma linha por
candidatura × motor, e cada motor tem o seu próprio ranking.

```
POST /apply ──► anonimiza ──► Runner.evaluate_application
                                 ├─ motor inline     → roda na hora, volta na resposta
                                 └─ motor background → fila (pending) ─► worker ─► done | failed
```

## Motores atuais

| id | tipo | execução | o que faz |
|---|---|---|---|
| `heuristic-v1` | heuristic | inline | 6 critérios ponderados (padrão e fallback; não pode ser desligado) |

Por enquanto só o `heuristic-v1` está registrado. `keywords.rs` (`keywords-v1`, "ATS clássico", só
palavras-chave) continua no código como **exemplo completo de motor**; para ativá-lo, registre-o em
`EngineRegistry::builtin()`. Os testes o registram explicitamente para exercitar o modo com vários motores.

## Adicionar um motor

1. Crie `src/engines/<nome>.rs` e implemente `ScoringEngine`.
2. Registre-o em `EngineRegistry::builtin()` (`src/engines/mod.rs`).
3. Pronto: no start ele é gravado em `scoring_engines` e passa a avaliar candidaturas
   novas. **Não precisa de migration**: `evaluations` já comporta qualquer motor.

```rust
use async_trait::async_trait;
use pokerum_core::scoring::{CriterionScore, ScoreReport};
use super::*;

pub struct MeuMotor { /* cliente HTTP, pool, chave... */ }

#[async_trait]
impl ScoringEngine for MeuMotor {
    fn info(&self) -> EngineInfo {
        EngineInfo {
            id: "meu-motor-v1",          // estável e versionado
            name: "Meu motor",
            kind: EngineKind::Llm,      // Heuristic | Llm | ExternalApi | Database
            description: "Uma frase para a interface.",
            execution: Execution::Background, // Inline só se for rápido e local
        }
    }

    // opcional: sem configuração o motor fica registrado mas não avalia
    fn availability(&self) -> Result<(), String> {
        if self.api_key.is_none() { return Err("defina MEU_MOTOR_API_KEY".into()) }
        Ok(())
    }

    // opcionais: timeout() (padrão 10s inline / 120s background)
    //            max_attempts() (padrão 1 inline / 3 background)

    async fn evaluate(&self, input: &EvaluationInput) -> Result<ScoreReport, EngineError> {
        // input.job_title, input.job_description, input.job_profile (análise do core),
        // input.resume_text (ANONIMIZADO), input.resume_codename
        todo!()
    }
}
```

### Regras do contrato

- **Nota** `total` em 0–100. Valores fora da escala são corrigidos; `NaN` vira falha.
- **Critérios** `breakdown[].key` são livres (cada motor tem os seus) e o frontend
  renderiza qualquer lista. `weight` é informativo (0 = não mostra o peso).
- **Erros**:
  - `EngineError::Transient`: timeout, HTTP 429/5xx, rede. Há retentativa com backoff exponencial.
  - `EngineError::Permanent`: resposta inválida, 4xx de configuração. Falha na hora, e a
    mensagem aparece para o usuário. Não coloque segredos nela.
- **Id versionado**: se mudar o algoritmo de um jeito que torna as notas incomparáveis
  (outro prompt, outro modelo), crie um id novo (`meu-motor-v2`) em vez de reaproveitar.
- **Privacidade**: o motor só recebe texto anonimizado. Não envie nada além de `EvaluationInput`.
- **Segredos** (API keys, senhas) vêm de variáveis de ambiente, **nunca** de
  `scoring_engines.config_json`. Essa coluna é para parâmetros não sensíveis (modelo,
  temperatura, URL).

### Notas por tipo

- **LLM**: peça JSON estruturado no formato do `ScoreReport` e valide a resposta (JSON inválido =
  `Permanent`; 429/529 = `Transient`). Use `execution: Background`. Prefira temperatura 0
  para notas estáveis. Lembre que cada candidatura gera uma chamada paga.
- **API externa**: mapeie a resposta do provedor para `ScoreReport`. Timeouts e 5xx = `Transient`.
- **Banco de dados**: o motor pode guardar um `SqlitePool` ou um pool de outro banco
  (ex.: comparar com currículos de contratados, buscar pesos por área).
  Se for só leitura local e rápida, pode ser `Inline`.

## Operação (CLI)

```bash
pokerum_api engines list               # estado: ligado / configurado
pokerum_api engines disable meu-motor-v1
pokerum_api engines enable  meu-motor-v1
pokerum_api engines backfill meu-motor-v1   # avalia candidaturas antigas com o motor
pokerum_api engines retry    meu-motor-v1   # recoloca na fila o que falhou
```

No Docker: `docker compose exec app /app/pokerum_api engines list`.

- **Motores inline** fazem backfill sozinhos no start, porque são grátis.
- **Motores background** (com custo) só fazem com `backfill` explícito.
- O servidor varre a fila a cada `POKERUM_QUEUE_POLL_SECS` (padrão 15 s) e roda até
  `POKERUM_ENGINE_CONCURRENCY` (padrão 4) avaliações em paralelo. O que estava rodando
  quando o servidor caiu é retomado no próximo start.

## Motor que sai do código

Ao remover um motor de `builtin()`, no start ele é marcado como desligado (`enabled = 0`):
- suas notas somem da API, mas continuam guardadas em `evaluations`;
- vagas que eram ordenadas por ele passam a usar o `heuristic-v1`.

Se o motor voltar a ser registrado, ele continua desligado até `engines enable <id>`; aí as notas
antigas reaparecem, e um `backfill` avalia o que entrou nesse meio-tempo.

## Ranking por motor

- Cada vaga tem `jobs.ranking_engine`, o motor do ranking padrão, escolhido ao publicar a vaga.
- `GET /api/jobs/:slug/ranking?engine=<id>` mostra o ranking de qualquer motor. Só entram
  avaliações `done`.
- Enquanto o motor da vaga não termina (ex.: LLM na fila), a nota principal da candidatura
  mostra a do `heuristic-v1`.
