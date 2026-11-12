//! Análise da descrição da vaga -> [`JobProfile`].

use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};

use super::experience::years_required;
use super::skills::{self, SkillCategory, SKILLS};
use super::tfidf;
use crate::text::{fold, is_content, is_noise, stem, surface_tokens, tokens, NgramSet};

/// Nível de senioridade.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Seniority {
    Intern,
    Junior,
    Pleno,
    Senior,
    Lead,
}

impl Seniority {
    pub fn label(&self) -> &'static str {
        match self {
            Seniority::Intern => "Estágio",
            Seniority::Junior => "Júnior",
            Seniority::Pleno => "Pleno",
            Seniority::Senior => "Sênior",
            Seniority::Lead => "Lead/Especialista",
        }
    }

    pub fn rank(&self) -> i32 {
        *self as i32
    }

    /// Anos de experiência típicos para o nível.
    pub fn typical_years(&self) -> f64 {
        match self {
            Seniority::Intern => 0.0,
            Seniority::Junior => 1.0,
            Seniority::Pleno => 3.0,
            Seniority::Senior => 5.0,
            Seniority::Lead => 7.0,
        }
    }

    /// Nível típico para uma quantidade de anos.
    pub fn from_years(y: f64) -> Seniority {
        if y < 1.0 {
            Seniority::Intern
        } else if y < 3.0 {
            Seniority::Junior
        } else if y < 5.0 {
            Seniority::Pleno
        } else if y < 8.0 {
            Seniority::Senior
        } else {
            Seniority::Lead
        }
    }
}

const SENIORITY_PHRASES: &[(Seniority, &[&str])] = &[
    (
        Seniority::Intern,
        &[
            "estagio",
            "estagiario",
            "estagiaria",
            "intern",
            "internship",
            "trainee",
            "aprendiz",
        ],
    ),
    (
        Seniority::Junior,
        &["junior", "jr", "entry level", "iniciante"],
    ),
    (
        Seniority::Pleno,
        &["pleno", "mid level", "mid-level", "intermediate", "pl"],
    ),
    (
        Seniority::Senior,
        &["senior", "sr", "especialista", "specialist", "experienced"],
    ),
    (
        Seniority::Lead,
        &[
            "tech lead",
            "team lead",
            "lider tecnico",
            "lider tecnica",
            "lead engineer",
            "lead developer",
            "staff engineer",
            "principal engineer",
            "engineering manager",
            "head of",
            "coordenador",
            "coordenadora",
            "gerente",
            "staff",
            "principal",
            "arquiteto",
            "arquiteta",
            "architect",
            "lead",
            "lider",
        ],
    ),
];

/// Detecta senioridade. `broad` permite palavras soltas ambíguas como "lead"
/// (usado no título da vaga); caso contrário só frases inequívocas de liderança.
pub fn detect_seniority(text: &str, broad: bool) -> Option<Seniority> {
    let ng = NgramSet::from_text(text);
    let mut found: Option<Seniority> = None;
    for (lvl, phrases) in SENIORITY_PHRASES {
        let hit = phrases.iter().any(|p| {
            if !broad && *lvl == Seniority::Lead && !p.contains(' ') {
                return false;
            }
            if !broad && matches!(*p, "pl" | "sr" | "jr" | "experienced") {
                return false;
            }
            ng.has_phrase(p)
        });
        if hit {
            found = Some(*lvl);
        }
    }
    found
}

/// Escolaridade / certificação exigida.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum EducationLevel {
    Superior,
    PosGraduacao,
    Doutorado,
    Certificacao,
}

impl EducationLevel {
    pub fn label(&self) -> &'static str {
        match self {
            EducationLevel::Superior => "Ensino superior",
            EducationLevel::PosGraduacao => "Pós-graduação/Mestrado",
            EducationLevel::Doutorado => "Doutorado",
            EducationLevel::Certificacao => "Certificação",
        }
    }

    pub(crate) fn phrases(&self, in_resume: bool) -> &'static [&'static str] {
        match (self, in_resume) {
            (EducationLevel::Superior, false) => &[
                "ensino superior",
                "graduacao",
                "graduado",
                "graduada",
                "bacharel",
                "bacharelado",
                "formacao superior",
                "degree",
                "bachelor",
                "bachelors",
                "licenciatura",
                "tecnologo",
                "formacao em",
                "nivel superior",
                "superior completo",
                "cursando",
            ],
            (EducationLevel::Superior, true) => &[
                "ensino superior",
                "graduacao",
                "graduado",
                "graduada",
                "bacharel",
                "bacharelado",
                "formacao superior",
                "degree",
                "bachelor",
                "bachelors",
                "licenciatura",
                "tecnologo",
                "nivel superior",
                "superior completo",
                "universidade",
                "faculdade",
                "university",
                "college",
                "instituto federal",
                "fatec",
                "engenharia",
                "ciencia da computacao",
                "sistemas de informacao",
                "b.sc",
                "bsc",
                "computer science",
                "analise e desenvolvimento de sistemas",
            ],
            (EducationLevel::PosGraduacao, _) => &[
                "pos graduacao",
                "posgraduacao",
                "mestrado",
                "mestre",
                "master",
                "masters",
                "mba",
                "especializacao",
                "msc",
                "m.sc",
                "lato sensu",
                "stricto sensu",
            ],
            (EducationLevel::Doutorado, _) => {
                &["doutorado", "doutor", "doutora", "phd", "ph.d", "doctorate"]
            }
            (EducationLevel::Certificacao, _) => &[
                "certificacao",
                "certificacoes",
                "certificado",
                "certificados",
                "certification",
                "certifications",
                "certified",
                "certificate",
            ],
        }
    }
}

pub(crate) const ALL_EDUCATION: [EducationLevel; 4] = [
    EducationLevel::Superior,
    EducationLevel::PosGraduacao,
    EducationLevel::Doutorado,
    EducationLevel::Certificacao,
];

pub(crate) const LANGUAGES: &[(&str, &[&str])] = &[
    ("Inglês", &["ingles", "english"]),
    ("Espanhol", &["espanhol", "spanish", "espanol"]),
    ("Francês", &["frances", "french"]),
    ("Alemão", &["alemao", "german", "deutsch"]),
    ("Italiano", &["italiano", "italian"]),
    ("Mandarim", &["mandarim", "mandarin", "chines", "chinese"]),
    ("Japonês", &["japones", "japanese"]),
    ("Português", &["portugues", "portuguese"]),
];

/// Skill exigida pela vaga.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct JobSkill {
    /// nome canônico (ex.: "PostgreSQL")
    pub name: String,
    pub category: SkillCategory,
    /// `false` = desejável / diferencial
    pub required: bool,
    /// peso na pontuação (1.0 obrigatória, 0.5 desejável)
    pub weight: f64,
}

/// Idioma pedido pela vaga.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LanguageReq {
    pub language: String,
    pub required: bool,
}

/// Palavra-chave relevante (não-skill) da vaga.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct KeyTerm {
    /// forma de exibição (com acentos)
    pub term: String,
    /// stem normalizado usado na comparação
    pub stem: String,
    pub weight: f64,
}

/// Perfil derivado da descrição da vaga.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct JobProfile {
    pub title: String,
    /// Lista de exibição (skills canônicas + termos importantes).
    pub keywords: Vec<String>,
    pub skills: Vec<JobSkill>,
    pub important_terms: Vec<KeyTerm>,
    pub years_required: Option<f64>,
    pub seniority: Option<Seniority>,
    pub education: Vec<EducationLevel>,
    pub languages: Vec<LanguageReq>,
    /// stems significativos do título (sem senioridade)
    pub title_terms: Vec<String>,
    /// vetor de termos (stem -> frequência ponderada) da vaga para TF-IDF
    pub term_freqs: BTreeMap<String, f64>,
    /// frequência de documento por linha (stem -> nº de linhas) na vaga
    pub line_doc_freq: BTreeMap<String, u32>,
    pub line_count: u32,
}

impl JobProfile {
    /// Skills canônicas (obrigatórias primeiro) + palavras-chave importantes, para exibição.
    pub fn keywords(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let mut push = |s: &str| {
            if !out.iter().any(|o| o.eq_ignore_ascii_case(s)) {
                out.push(s.to_string());
            }
        };
        for s in self.skills.iter().filter(|s| s.required) {
            push(&s.name);
        }
        for s in self.skills.iter().filter(|s| !s.required) {
            push(&s.name);
        }
        for t in &self.important_terms {
            push(&t.term);
        }
        out
    }

    pub fn required_skills(&self) -> impl Iterator<Item = &JobSkill> {
        self.skills.iter().filter(|s| s.required)
    }
}

const NICE_MARKERS: &[&str] = &[
    "desejavel",
    "desejaveis",
    "diferencial",
    "diferenciais",
    "nice to have",
    "plus",
    "bonus",
    "sera um diferencial",
    "preferred",
    "preferencial",
    "preferencialmente",
    "good to have",
    "nao obrigatorio",
    "nao obrigatorios",
    "would be great",
    "pontos extras",
    "extra points",
    "bonus points",
    "valorizado",
    "valorizados",
];
const HEADER_MARKERS: &[&str] = &[
    "requisito",
    "requisitos",
    "requirements",
    "requirement",
    "obrigatorio",
    "obrigatorios",
    "must have",
    "must haves",
    "qualificacoes",
    "qualifications",
    "responsabilidades",
    "responsibilities",
    "o que voce vai fazer",
    "what you will do",
    "atividades",
    "sobre a vaga",
    "about",
    "beneficios",
    "benefits",
    "stack",
    "tecnologias",
    "conhecimentos",
    "skills",
];

fn is_bullet(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with(['-', '•', '*', '▪', '●', '◦', '–', '‣', '■', '►', '✓', '✔'])
        || t.split_once(['.', ')']).is_some_and(|(n, _)| {
            !n.is_empty() && n.len() <= 2 && n.chars().all(|c| c.is_ascii_digit())
        })
}

/// Separa a descrição em (texto obrigatório, texto desejável).
pub fn split_zones(description: &str) -> (String, String) {
    let mut required = String::new();
    let mut nice = String::new();
    let mut in_nice = false;
    for line in description.lines() {
        let ng = NgramSet::from_text(line);
        let has_nice = ng.has_any(NICE_MARKERS);
        let trimmed = line.trim();
        let words = trimmed.split_whitespace().count();
        let header = !trimmed.is_empty()
            && !is_bullet(trimmed)
            && (trimmed.ends_with(':')
                || trimmed.starts_with('#')
                || (words <= 4
                    && !trimmed.contains(',')
                    && (has_nice || ng.has_any(HEADER_MARKERS))));
        if header {
            in_nice = has_nice;
        }
        let target = if in_nice || has_nice {
            &mut nice
        } else {
            &mut required
        };
        target.push_str(line);
        target.push('\n');
    }
    (required, nice)
}

/// Analisa a vaga.
pub fn analyze_job(title: &str, description: &str) -> JobProfile {
    let (req_text, nice_text) = split_zones(description);

    // ---- skills
    let title_skills = skills::detect(title);
    let req_skills = skills::detect(&req_text);
    let nice_skills = skills::detect(&nice_text);
    let mut job_skills: Vec<JobSkill> = Vec::new();
    for i in title_skills
        .iter()
        .chain(req_skills.iter())
        .chain(nice_skills.iter())
    {
        let def = &SKILLS[*i];
        if job_skills.iter().any(|s| s.name == def.canonical) {
            continue;
        }
        let required = title_skills.contains(i) || req_skills.contains(i);
        job_skills.push(JobSkill {
            name: def.canonical.to_string(),
            category: def.category,
            required,
            weight: if required { 1.0 } else { 0.5 },
        });
    }
    // obrigatórias primeiro, mantendo a ordem do dicionário
    job_skills.sort_by_key(|s| !s.required);

    // ---- termos importantes (não-skill)
    let skill_tokens: HashSet<String> = title_skills
        .iter()
        .chain(req_skills.iter())
        .chain(nice_skills.iter())
        .flat_map(|i| {
            SKILLS[*i]
                .aliases
                .iter()
                .flat_map(|a| a.split(' ').map(str::to_string))
        })
        .collect();
    let important_terms = important_terms(title, &req_text, &nice_text, &skill_tokens);

    // ---- experiência / senioridade
    let years = years_required(&req_text).or_else(|| years_required(description));
    let seniority = detect_seniority(title, true)
        .or_else(|| detect_seniority(description, false))
        .or_else(|| years.map(Seniority::from_years));

    // ---- formação e idiomas
    let full = format!("{title}\n{description}");
    let ng_full = NgramSet::from_text(&full);
    let ng_nice = NgramSet::from_text(&nice_text);
    let mut education = Vec::new();
    for lvl in ALL_EDUCATION {
        if ng_full.has_any(lvl.phrases(false)) {
            education.push(lvl);
        }
    }
    let mut languages = Vec::new();
    for (lang, phrases) in LANGUAGES {
        if ng_full.has_any(phrases) {
            let ng_req = NgramSet::from_text(&req_text);
            let required = ng_req.has_any(phrases) || !ng_nice.has_any(phrases);
            languages.push(LanguageReq {
                language: lang.to_string(),
                required,
            });
        }
    }

    // ---- título
    let title_terms = title_terms(title);

    // ---- TF-IDF
    let mut term_freqs = tfidf::term_counts(description);
    for t in &title_terms {
        *term_freqs.entry(t.clone()).or_default() += 3.0;
    }
    let (line_doc_freq, line_count) = tfidf::line_doc_freq(&full);

    let mut profile = JobProfile {
        title: title.trim().to_string(),
        keywords: Vec::new(),
        skills: job_skills,
        important_terms,
        years_required: years,
        seniority,
        education,
        languages,
        title_terms,
        term_freqs,
        line_doc_freq,
        line_count,
    };
    profile.keywords = profile.keywords();
    profile
}

const SENIORITY_WORDS: &[&str] = &[
    "junior",
    "jr",
    "pleno",
    "pl",
    "senior",
    "sr",
    "lead",
    "lider",
    "estagio",
    "estagiario",
    "estagiaria",
    "intern",
    "trainee",
    "staff",
    "principal",
    "mid",
    "level",
    "i",
    "ii",
    "iii",
    "iv",
    "remoto",
    "remote",
    "hibrido",
    "presencial",
    "vaga",
];

/// Stems significativos do título (sem senioridade/stopwords).
pub fn title_terms(title: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for t in tokens(title) {
        if !is_content(&t) || SENIORITY_WORDS.contains(&t.as_str()) {
            continue;
        }
        let s = stem(&t);
        if !out.contains(&s) {
            out.push(s);
        }
    }
    out
}

/// Termos mais frequentes da vaga que não são skills nem ruído.
fn important_terms(
    title: &str,
    req: &str,
    nice: &str,
    skill_tokens: &HashSet<String>,
) -> Vec<KeyTerm> {
    const MAX: usize = 12;
    // stem -> (peso, contagem por forma de superfície)
    let mut acc: HashMap<String, (f64, BTreeMap<String, u32>)> = HashMap::new();
    let mut add = |text: &str, w: f64| {
        for surf in surface_tokens(text) {
            let norm = fold(&surf);
            if norm.chars().count() < 3
                || !is_content(&norm)
                || is_noise(&norm)
                || skills::ALIAS_TOKENS.contains(&norm)
                || skill_tokens.contains(&norm)
                || SENIORITY_WORDS.contains(&norm.as_str())
                || norm.contains('.')
            {
                continue;
            }
            let e = acc.entry(stem(&norm)).or_default();
            e.0 += w;
            *e.1.entry(surf).or_default() += 1;
        }
    };
    add(title, 3.0);
    add(req, 1.0);
    add(nice, 0.5);
    let mut v: Vec<(String, f64, String)> = acc
        .into_iter()
        .map(|(stem, (w, forms))| {
            let surface = forms
                .iter()
                .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
                .map(|(s, _)| s.clone())
                .unwrap_or_default();
            (stem, w, surface)
        })
        .collect();
    v.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    v.into_iter()
        .take(MAX)
        .map(|(stem, weight, term)| KeyTerm { term, stem, weight })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const JOB_PT: &str = "Sobre a vaga\n\
        Buscamos pessoa desenvolvedora backend para atuar em microsserviços de pagamentos.\n\
        Requisitos:\n\
        - 5+ anos de experiência com Java e Spring Boot\n\
        - PostgreSQL e Kafka\n\
        - Docker e Kubernetes em produção\n\
        - Inglês intermediário\n\
        - Graduação em Ciência da Computação ou áreas afins\n\
        Diferenciais:\n\
        - AWS\n\
        - Terraform\n";

    #[test]
    fn analyzes_pt_job() {
        let p = analyze_job("Desenvolvedor Backend Sênior", JOB_PT);
        let req: Vec<&str> = p.required_skills().map(|s| s.name.as_str()).collect();
        for s in [
            "Java",
            "Spring",
            "PostgreSQL",
            "Kafka",
            "Docker",
            "Kubernetes",
            "Microsserviços",
        ] {
            assert!(req.contains(&s), "faltou {s} em {req:?}");
        }
        let nice: Vec<&str> = p
            .skills
            .iter()
            .filter(|s| !s.required)
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            nice.contains(&"AWS") && nice.contains(&"Terraform"),
            "{nice:?}"
        );
        assert_eq!(p.years_required, Some(5.0));
        assert_eq!(p.seniority, Some(Seniority::Senior));
        assert!(p.education.contains(&EducationLevel::Superior));
        assert_eq!(p.languages.len(), 1);
        assert_eq!(p.languages[0].language, "Inglês");
        assert!(p.languages[0].required);
        let kw = p.keywords();
        assert_eq!(kw, p.keywords);
        assert!(kw.iter().any(|k| k == "Java"));
        assert!(kw.iter().any(|k| k == "pagamentos"), "{kw:?}");
        assert!(
            !kw.iter().any(|k| k == "requisitos" || k == "anos"),
            "{kw:?}"
        );
    }

    #[test]
    fn analyzes_en_job() {
        let jd = "We are looking for a Junior Data Analyst.\nRequirements:\n- 1+ years with SQL and Power BI\n- Advanced Excel\n- Fluent Spanish\nNice to have:\n- Python\n";
        let p = analyze_job("Data Analyst", jd);
        assert_eq!(p.seniority, Some(Seniority::Junior));
        assert_eq!(p.years_required, Some(1.0));
        let req: Vec<&str> = p.required_skills().map(|s| s.name.as_str()).collect();
        assert!(
            req.contains(&"SQL") && req.contains(&"Power BI") && req.contains(&"Excel"),
            "{req:?}"
        );
        assert!(p.skills.iter().any(|s| s.name == "Python" && !s.required));
        assert_eq!(p.languages[0].language, "Espanhol");
    }

    #[test]
    fn inline_nice_to_have() {
        let (req, nice) = split_zones(
            "Experiência com React.\nConhecimento em GraphQL é um diferencial.\nTypeScript",
        );
        assert!(req.contains("React") && req.contains("TypeScript"));
        assert!(nice.contains("GraphQL"));
    }

    #[test]
    fn profile_roundtrips_json() {
        let p = analyze_job("Dev", JOB_PT);
        let j = serde_json::to_string(&p).unwrap();
        let back: JobProfile = serde_json::from_str(&j).unwrap();
        assert_eq!(p, back);
        // compatível com JSON antigo/incompleto
        let partial: JobProfile = serde_json::from_str(r#"{"title":"x"}"#).unwrap();
        assert_eq!(partial.title, "x");
    }
}
