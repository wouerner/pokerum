-- Pokerum: schema inicial.
-- NUNCA edite esta migration depois de aplicada; crie 0002_*.sql etc.

CREATE TABLE IF NOT EXISTS jobs (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    slug         TEXT    NOT NULL UNIQUE,
    title        TEXT    NOT NULL,
    company      TEXT    NULL,
    description  TEXT    NOT NULL,
    profile_json TEXT    NOT NULL,
    created_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS resumes (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    codename         TEXT    NOT NULL UNIQUE,
    alias            TEXT    NOT NULL,
    anonymized_text  TEXT    NOT NULL,
    redactions_json  TEXT    NOT NULL,
    owner_token_hash TEXT    NOT NULL,
    created_at       TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS applications (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id         INTEGER NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    resume_id      INTEGER NOT NULL REFERENCES resumes(id) ON DELETE CASCADE,
    score          REAL    NOT NULL,
    report_json    TEXT    NOT NULL,
    engine_version TEXT    NOT NULL,
    created_at     TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE (job_id, resume_id)
);

CREATE INDEX IF NOT EXISTS idx_applications_job_score
    ON applications (job_id, score DESC, created_at ASC);
CREATE INDEX IF NOT EXISTS idx_applications_resume
    ON applications (resume_id);
CREATE INDEX IF NOT EXISTS idx_jobs_created_at
    ON jobs (created_at DESC);
