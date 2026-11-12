//! Detecção de seções do currículo e métricas de formato (legibilidade ATS).

use serde::{Deserialize, Serialize};

use crate::text::fold;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SectionKind {
    Summary,
    Experience,
    Education,
    Skills,
    Certifications,
    Languages,
    Projects,
}

impl SectionKind {
    pub fn label(&self) -> &'static str {
        match self {
            SectionKind::Summary => "Resumo",
            SectionKind::Experience => "Experiência",
            SectionKind::Education => "Formação",
            SectionKind::Skills => "Habilidades",
            SectionKind::Certifications => "Certificações",
            SectionKind::Languages => "Idiomas",
            SectionKind::Projects => "Projetos",
        }
    }
}

const HEADERS: &[(SectionKind, &[&str])] = &[
    (
        SectionKind::Experience,
        &[
            "experiencia",
            "experiencias",
            "experiencia profissional",
            "experiencias profissionais",
            "historico profissional",
            "trajetoria profissional",
            "carreira",
            "experience",
            "work experience",
            "professional experience",
            "employment history",
            "work history",
            "employment",
            "career",
        ],
    ),
    (
        SectionKind::Education,
        &[
            "formacao",
            "formacao academica",
            "educacao",
            "escolaridade",
            "formacao educacional",
            "education",
            "academic background",
            "academic formation",
            "graduacao",
        ],
    ),
    (
        SectionKind::Skills,
        &[
            "habilidades",
            "competencias",
            "conhecimentos",
            "skills",
            "technical skills",
            "tecnologias",
            "stack",
            "competencias tecnicas",
            "hard skills",
            "soft skills",
            "ferramentas",
            "conhecimentos tecnicos",
            "habilidades tecnicas",
            "tech stack",
            "technologies",
            "core competencies",
            "qualificacoes",
            "qualifications",
        ],
    ),
    (
        SectionKind::Summary,
        &[
            "resumo",
            "resumo profissional",
            "sobre",
            "sobre mim",
            "perfil",
            "perfil profissional",
            "objetivo",
            "objetivos",
            "summary",
            "professional summary",
            "about",
            "about me",
            "profile",
            "objective",
            "apresentacao",
        ],
    ),
    (
        SectionKind::Certifications,
        &[
            "certificacoes",
            "certificados",
            "cursos",
            "cursos complementares",
            "certifications",
            "certificates",
            "courses",
            "licenses",
            "formacao complementar",
        ],
    ),
    (SectionKind::Languages, &["idiomas", "linguas", "languages"]),
    (
        SectionKind::Projects,
        &[
            "projetos",
            "projects",
            "portfolio",
            "projetos pessoais",
            "side projects",
        ],
    ),
];

/// Se a linha é um título de seção, retorna o tipo.
pub fn header_kind(line: &str) -> Option<SectionKind> {
    let t = line
        .trim()
        .trim_start_matches(|c: char| {
            c == '#' || c == '*' || c == '=' || c == '_' || c.is_whitespace()
        })
        .trim_end_matches(|c: char| {
            c == ':' || c == '*' || c == '=' || c == '_' || c.is_whitespace()
        });
    if t.is_empty() || t.chars().count() > 45 {
        return None;
    }
    let f = fold(t);
    let words: Vec<&str> = f
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    if words.is_empty() || words.len() > 5 {
        return None;
    }
    let joined = words.join(" ");
    let mut best: Option<(usize, SectionKind)> = None;
    for (kind, keys) in HEADERS {
        for k in keys.iter() {
            let matches = joined == *k
                || (joined.starts_with(k)
                    && joined[k.len()..].starts_with(' ')
                    && words.len() - k.split(' ').count() <= 2);
            if matches && best.is_none_or(|(l, _)| k.len() > l) {
                best = Some((k.len(), *kind));
            }
        }
    }
    best.map(|(_, k)| k)
}

/// Seção a que cada linha pertence (`None` antes do primeiro título).
pub fn line_sections(text: &str) -> Vec<Option<SectionKind>> {
    let mut cur = None;
    text.lines()
        .map(|l| {
            if let Some(k) = header_kind(l) {
                cur = Some(k);
            }
            cur
        })
        .collect()
}

/// Métricas de formato do currículo.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct FormatStats {
    pub sections: Vec<SectionKind>,
    pub word_count: usize,
    pub bullet_lines: usize,
    /// fração de caracteres "estranhos" (não alfanuméricos, não pontuação comum)
    pub symbol_ratio: f64,
    /// linhas com resultados quantificados (%, R$, números com verbo de impacto)
    pub quantified_lines: usize,
}

pub fn analyze(text: &str) -> FormatStats {
    let mut sections = Vec::new();
    let mut bullets = 0;
    let mut quantified = 0;
    for line in text.lines() {
        if let Some(k) = header_kind(line) {
            if !sections.contains(&k) {
                sections.push(k);
            }
            continue;
        }
        let t = line.trim_start();
        if t.starts_with(['-', '•', '*', '▪', '●', '◦', '–', '‣', '■', '►', '✓', '✔'])
            || t.split_once(['.', ')']).is_some_and(|(n, _)| {
                !n.is_empty() && n.len() <= 2 && n.chars().all(|c| c.is_ascii_digit())
            })
        {
            bullets += 1;
        }
        let has_pct = t.contains('%') || t.contains("R$") || t.contains("US$") || t.contains('$');
        let has_num = t.chars().any(|c| c.is_ascii_digit());
        let f = fold(t);
        let impact = [
            "reduz", "aument", "melhor", "economi", "cresc", "otimiz", "reduc", "increas",
            "improv", "sav", "grew", "boost", "cut ", "lider", "led ", "gerenci", "managed",
            "x mais", "vezes",
        ]
        .iter()
        .any(|w| f.contains(w));
        if has_pct || (has_num && impact && !looks_like_dates_only(&f)) {
            quantified += 1;
        }
    }
    let word_count = text.split_whitespace().count();
    let total = text.chars().filter(|c| !c.is_whitespace()).count().max(1);
    let weird = text
        .chars()
        .filter(|c| {
            !c.is_whitespace()
                && !c.is_alphanumeric()
                && !".,;:!?()[]{}'\"-–—/@%$+#&*•_=<>|".contains(*c)
        })
        .count();
    FormatStats {
        sections,
        word_count,
        bullet_lines: bullets,
        symbol_ratio: weird as f64 / total as f64,
        quantified_lines: quantified,
    }
}

fn looks_like_dates_only(f: &str) -> bool {
    // linha com números mas apenas anos (ex.: "2019 - 2023")
    f.split(|c: char| !c.is_ascii_digit())
        .filter(|n| !n.is_empty())
        .all(|n| n.len() == 4 && (n.starts_with("19") || n.starts_with("20")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_headers() {
        assert_eq!(
            header_kind("EXPERIÊNCIA PROFISSIONAL"),
            Some(SectionKind::Experience)
        );
        assert_eq!(
            header_kind("## Formação Acadêmica:"),
            Some(SectionKind::Education)
        );
        assert_eq!(header_kind("Technical Skills"), Some(SectionKind::Skills));
        assert_eq!(header_kind("Sobre mim"), Some(SectionKind::Summary));
        assert_eq!(
            header_kind("Experiência com Java e Spring em grandes bancos"),
            None
        );
        assert_eq!(header_kind("Desenvolvedor Java"), None);
    }

    #[test]
    fn format_stats() {
        let s = analyze("Resumo\nDev\nExperiência\n- Reduzi custos em 30%\n- Fiz X\nFormação\nUSP\nHabilidades\nJava ★★★");
        assert_eq!(s.sections.len(), 4);
        assert_eq!(s.bullet_lines, 2);
        assert_eq!(s.quantified_lines, 1);
        assert!(s.symbol_ratio > 0.0);
    }
}
