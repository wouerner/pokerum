-- Pokerum: suporte a múltiplos motores de avaliação (heurísticas, LLMs, APIs externas, banco...).
-- NUNCA edite esta migration depois de aplicada; crie 0003_*.sql etc.
--
-- Fonte da verdade das notas passa a ser `evaluations` (uma linha por candidatura × motor).
-- `applications.score/report_json/engine_version` ficam como LEGADO: guardam a nota do
-- heuristic-v1 no momento do envio e não são mais lidas pela API.

-- Catálogo de motores. As linhas são sincronizadas no start a partir do registro em código
-- (nome/tipo/descrição/execução); `enabled` e `config_json` são controlados aqui e
-- sobrevivem a deploys. Segredos (API keys) NÃO vão em config_json: use variáveis de ambiente.
CREATE TABLE IF NOT EXISTS scoring_engines (
    id          TEXT    PRIMARY KEY,                       -- ex.: 'heuristic-v1', 'claude-v1'
    name        TEXT    NOT NULL,
    kind        TEXT    NOT NULL
                CHECK (kind IN ('heuristic', 'llm', 'external_api', 'database')),
    description TEXT    NOT NULL DEFAULT '',
    execution   TEXT    NOT NULL DEFAULT 'inline'
                CHECK (execution IN ('inline', 'background')),
    enabled     INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    config_json TEXT    NOT NULL DEFAULT '{}',
    created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Resultado de cada motor para cada candidatura.
CREATE TABLE IF NOT EXISTS evaluations (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    application_id INTEGER NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    engine_id      TEXT    NOT NULL REFERENCES scoring_engines(id),
    status         TEXT    NOT NULL DEFAULT 'pending'
                   CHECK (status IN ('pending', 'running', 'done', 'failed')),
    score          REAL    NULL CHECK (score IS NULL OR (score >= 0 AND score <= 100)),
    report_json    TEXT    NULL,
    error          TEXT    NULL,
    attempts       INTEGER NOT NULL DEFAULT 0,
    duration_ms    INTEGER NULL,
    created_at     TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at     TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    finished_at    TEXT    NULL,
    UNIQUE (application_id, engine_id)
);

-- ranking por motor: WHERE engine_id = ? AND status = 'done' ORDER BY score DESC
CREATE INDEX IF NOT EXISTS idx_evaluations_engine_status_score
    ON evaluations (engine_id, status, score DESC);
-- fila do executor: WHERE status IN ('pending','running')
CREATE INDEX IF NOT EXISTS idx_evaluations_status
    ON evaluations (status);
CREATE INDEX IF NOT EXISTS idx_evaluations_application
    ON evaluations (application_id);

-- Motor que ordena o ranking padrão de cada vaga.
ALTER TABLE jobs ADD COLUMN ranking_engine TEXT NOT NULL DEFAULT 'heuristic-v1';

-- Motor atual + migração das notas já existentes.
INSERT OR IGNORE INTO scoring_engines (id, name, kind, description, execution)
VALUES ('heuristic-v1', 'Heurístico v1', 'heuristic',
        'Seis critérios ponderados: skills, similaridade, experiência, senioridade, formação/idiomas e formato ATS.',
        'inline');

INSERT OR IGNORE INTO evaluations
    (application_id, engine_id, status, score, report_json, attempts, created_at, updated_at, finished_at)
SELECT id, 'heuristic-v1', 'done', score, report_json, 1, created_at, created_at, created_at
FROM applications;
