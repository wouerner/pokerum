//! Anonimização de currículos: codinome Pokémon, apelido aleatório e remoção de PII.
//!
//! A função principal é [`anonymize`]. Ela:
//! 1. sorteia um codinome (nome de Pokémon) ainda não usado;
//! 2. sorteia um apelido (nome + sobrenome brasileiros);
//! 3. remove e-mails, telefones, links, documentos (CPF/CNPJ/RG), endereços e
//!    datas de nascimento, trocando-os por marcadores como `[email removido]`;
//! 4. substitui o nome real (declarado e/ou detectado no cabeçalho) pelo apelido.

mod names;
mod pokemon;

pub use pokemon::pokemon_names;

use std::collections::{BTreeMap, HashSet};

use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use rand::Rng;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};

use crate::text::fold;

/// Resultado da anonimização de um currículo.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AnonymizedResume {
    /// Nome de Pokémon, ex: "Bulbasaur". Único globalmente.
    pub codename: String,
    /// Nome aleatório de pessoa, ex: "Marina Duarte".
    pub alias: String,
    /// Texto com PII removida/substituída.
    pub text: String,
    /// Quantidade de remoções por tipo (apenas tipos com contagem > 0).
    pub redactions: Vec<Redaction>,
}

/// Quantidade de ocorrências removidas de um tipo de dado pessoal.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Redaction {
    pub kind: RedactionKind,
    pub count: usize,
}

/// Tipos de dado pessoal removidos.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RedactionKind {
    Name,
    Email,
    Phone,
    Url,
    /// CPF / RG / CNPJ / passaporte.
    Document,
    /// CEP, rua, endereço.
    Address,
    BirthDate,
}

impl RedactionKind {
    /// Marcador inserido no lugar do dado removido (para `Name` o apelido é usado).
    pub fn placeholder(&self) -> &'static str {
        match self {
            RedactionKind::Name => "[nome removido]",
            RedactionKind::Email => "[email removido]",
            RedactionKind::Phone => "[telefone removido]",
            RedactionKind::Url => "[link removido]",
            RedactionKind::Document => "[documento removido]",
            RedactionKind::Address => "[endereço removido]",
            RedactionKind::BirthDate => "[data removida]",
        }
    }

    /// Rótulo em PT-BR para exibição.
    pub fn label(&self) -> &'static str {
        match self {
            RedactionKind::Name => "Nome",
            RedactionKind::Email => "E-mail",
            RedactionKind::Phone => "Telefone",
            RedactionKind::Url => "Link",
            RedactionKind::Document => "Documento",
            RedactionKind::Address => "Endereço",
            RedactionKind::BirthDate => "Data de nascimento",
        }
    }
}

/// Anonimiza um currículo.
///
/// `taken_codenames`: codinomes já usados (devem ser únicos globalmente).
/// `declared_name`: nome real informado pelo candidato no formulário (opcional) —
/// também é usado para encontrar e substituir ocorrências no texto.
pub fn anonymize(
    raw_text: &str,
    declared_name: Option<&str>,
    taken_codenames: &HashSet<String>,
    rng: &mut impl Rng,
) -> AnonymizedResume {
    let codename = pick_codename(taken_codenames, rng);

    let (text, mut counts) = redact_pii(raw_text);

    // Nomes a substituir: declarado + detectado no cabeçalho.
    let mut real_names: Vec<String> = Vec::new();
    if let Some(n) = declared_name.map(str::trim).filter(|n| !n.is_empty()) {
        real_names.push(n.to_string());
    }
    if let Some(n) = detect_header_name(&text) {
        if !real_names.iter().any(|r| fold(r) == fold(&n)) {
            real_names.push(n);
        }
    }

    let forbidden: HashSet<String> = real_names
        .iter()
        .flat_map(|n| n.split_whitespace().map(fold).collect::<Vec<_>>())
        .collect();
    let alias = random_alias_avoiding(rng, &forbidden);

    let (text, n_names) = replace_names(&text, &real_names, &alias);
    if n_names > 0 {
        *counts.entry(RedactionKind::Name).or_default() += n_names;
    }

    let redactions = counts
        .into_iter()
        .filter(|(_, c)| *c > 0)
        .map(|(kind, count)| Redaction { kind, count })
        .collect();

    AnonymizedResume {
        codename,
        alias,
        text,
        redactions,
    }
}

/// Sorteia um codinome de Pokémon que não esteja em `taken` (comparação sem
/// diferenciar maiúsculas). Se todos estiverem em uso, acrescenta um sufixo
/// numérico ("Pikachu 2").
pub fn pick_codename(taken: &HashSet<String>, rng: &mut impl Rng) -> String {
    let taken_lc: HashSet<String> = taken.iter().map(|s| s.trim().to_lowercase()).collect();
    let names = pokemon_names();
    let available: Vec<&str> = names
        .iter()
        .copied()
        .filter(|n| !taken_lc.contains(&n.to_lowercase()))
        .collect();
    if let Some(n) = available.choose(rng) {
        return (*n).to_string();
    }
    let base = names.choose(rng).copied().unwrap_or("Pokemon");
    let mut i = 2usize;
    loop {
        let candidate = format!("{base} {i}");
        if !taken_lc.contains(&candidate.to_lowercase()) {
            return candidate;
        }
        i += 1;
    }
}

/// Gera um apelido aleatório "Nome Sobrenome".
pub fn random_alias(rng: &mut impl Rng) -> String {
    random_alias_avoiding(rng, &HashSet::new())
}

fn random_alias_avoiding(rng: &mut impl Rng, forbidden_folded: &HashSet<String>) -> String {
    let mut last = String::new();
    for _ in 0..32 {
        let first = names::FIRST_NAMES.choose(rng).copied().unwrap_or("Alex");
        let sur = names::SURNAMES.choose(rng).copied().unwrap_or("Silva");
        last = format!("{first} {sur}");
        if !forbidden_folded.contains(&fold(first)) && !forbidden_folded.contains(&fold(sur)) {
            return last;
        }
    }
    last
}

// ---------------------------------------------------------------------------
// Remoção de PII por expressões regulares
// ---------------------------------------------------------------------------

fn re(p: &str) -> Regex {
    Regex::new(p).expect("regex inválida")
}

static EMAIL: Lazy<Regex> =
    Lazy::new(|| re(r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9\-]+(?:\.[A-Za-z0-9\-]+)*\.[A-Za-z]{2,}"));

static URLS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        re(r#"(?i)\b(?:https?://|www\.)[^\s<>()\[\]{}"']+"#),
        re(
            r#"(?i)\b(?:[a-z0-9\-]+\.)*(?:linkedin\.com|github\.com|gitlab\.com|bitbucket\.org|behance\.net|dribbble\.com|medium\.com|twitter\.com|instagram\.com|facebook\.com|wa\.me|lattes\.cnpq\.br|stackoverflow\.com|dev\.to|github\.io|vercel\.app|netlify\.app)(?:/[^\s<>()\[\]{}"']*)?"#,
        ),
    ]
});

static DOCUMENTS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // rotulados: "CPF: 12345678909", "RG 12.345.678-9", "Passaporte nº AB123456"
        re(
            r"\b(?i:cpf|cnpj|rg|ssn|passaporte|passport|cnh)\b\s*(?:n[º°o.]*\s*)?[:\-]?\s*[0-9A-Za-z][0-9A-Za-z.\-/]{4,20}",
        ),
        // CNPJ formatado
        re(r"\d{2}\.\d{3}\.\d{3}/\d{4}-\d{2}"),
        // CPF formatado
        re(r"\d{3}\.\d{3}\.\d{3}-\d{2}"),
        // RG formatado (SP)
        re(r"\d{2}\.\d{3}\.\d{3}-[\dxX]"),
    ]
});

static BIRTH: Lazy<Regex> = Lazy::new(|| {
    re(
        r"(?i)\b((?:data\s+de\s+nascimento|data\s+nasc\.?|nascimento|nascid[oa]\s+em|date\s+of\s+birth|birth\s*date|birthday|born(?:\s+on)?|d\.?o\.?b\.?)\s*[:\-]?\s*(?:em\s+|on\s+|in\s+)?)(\d{1,2}[/.\-]\d{1,2}[/.\-]\d{2,4}|\d{4}-\d{2}-\d{2}|\d{1,2}\s+de\s+[a-zç]+\s+de\s+\d{4}|[a-z]+\.?\s+\d{1,2},?\s+\d{4}|\d{1,2}\s+[a-z]+\.?\s+\d{4})",
    )
});

static ADDRESSES: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // "Endereço: ..." / "Address: ..."
        re(r"(?i:endere[cç]o|address)\s*:\s*[^\n|•]+"),
        // "Rua das Flores, 123 - São Paulo"
        re(
            r"\b(?i:rua|r\.|av\.|avenida|alameda|al\.|travessa|tv\.|rodovia|estrada|pra[cç]a|quadra|qd\.)\s+(?:(?:d[aoe]s?|of|the)\s+)?[A-ZÀ-Ý0-9][^\n|•;]{2,80}",
        ),
        // "123 Main Street"
        re(
            r"\b\d{1,5}\s+(?:[A-Z][a-z]+\s+){1,3}(?:Street|St\.|Avenue|Ave\.|Road|Rd\.|Boulevard|Blvd\.|Lane|Ln\.|Drive)[^\n|•;]{0,60}",
        ),
        // CEP rotulado
        re(r"\b(?i:cep|zip(?:\s*code)?)\s*:?\s*\d{5}-?\d{3,4}"),
        // CEP formatado
        re(r"\b\d{5}-\d{3}\b"),
    ]
});

static PHONES: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // internacional: +55 11 98765-4321, +1 (555) 123-4567
        re(r"\+\d[\d\s().\-]{7,18}\d"),
        // BR: (11) 98765-4321, 11 98765 4321, 11987654321, (11) 3456-7890
        re(r"\(?\d{2}\)?[\s.\-]?9?\s?\d{4}[\s.\-]?\d{4}"),
        // EUA: (555) 123-4567, 555-123-4567
        re(r"\(\d{3}\)\s?\d{3}[\-.\s]\d{4}"),
        re(r"\d{3}[\-.]\d{3}[\-.]\d{4}"),
        // celular sem DDD: 98765-4321
        re(r"9\d{4}-\d{4}"),
    ]
});

/// Aplica `f` a cada ocorrência de `re`; `Some(s)` substitui pelo texto `s`,
/// `None` mantém a ocorrência. Retorna o novo texto e o número de substituições.
fn redact_with<F>(text: &str, re: &Regex, mut f: F) -> (String, usize)
where
    F: FnMut(&str, &Captures) -> Option<String>,
{
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    let mut n = 0;
    for caps in re.captures_iter(text) {
        let m = caps.get(0).expect("grupo 0");
        if let Some(rep) = f(text, &caps) {
            out.push_str(&text[last..m.start()]);
            out.push_str(&rep);
            last = m.end();
            n += 1;
        }
    }
    out.push_str(&text[last..]);
    (out, n)
}

/// A ocorrência não está colada a outros dígitos/letras (evita pegar pedaços de números maiores).
fn isolated(text: &str, start: usize, end: usize) -> bool {
    let before = text[..start].chars().next_back();
    let after = text[end..].chars().next();
    !before.is_some_and(|c| c.is_alphanumeric()) && !after.is_some_and(|c| c.is_alphanumeric())
}

/// Separa pontuação final que provavelmente não faz parte de um link.
fn split_trailing_punct(s: &str) -> (&str, &str) {
    let trimmed = s.trim_end_matches(['.', ',', ';', ':', '!', '?', ')']);
    (trimmed, &s[trimmed.len()..])
}

/// Remove e-mails, links, documentos, datas de nascimento, endereços e telefones.
/// Não trata nomes (veja [`anonymize`]).
pub fn redact_pii(text: &str) -> (String, BTreeMap<RedactionKind, usize>) {
    let mut counts: BTreeMap<RedactionKind, usize> = BTreeMap::new();
    let mut add = |k: RedactionKind, n: usize| {
        if n > 0 {
            *counts.entry(k).or_default() += n;
        }
    };

    let (mut t, n) = redact_with(text, &EMAIL, |_, _| {
        Some(RedactionKind::Email.placeholder().to_string())
    });
    add(RedactionKind::Email, n);

    for r in URLS.iter() {
        let (nt, n) = redact_with(&t, r, |_, c| {
            let (_, trail) = split_trailing_punct(c.get(0)?.as_str());
            Some(format!("{}{}", RedactionKind::Url.placeholder(), trail))
        });
        t = nt;
        add(RedactionKind::Url, n);
    }

    for r in DOCUMENTS.iter() {
        let (nt, n) = redact_with(&t, r, |h, c| {
            let m = c.get(0)?;
            // documentos rotulados precisam conter ao menos 5 dígitos
            let digits = m.as_str().chars().filter(char::is_ascii_digit).count();
            (digits >= 5 && isolated(h, m.start(), m.end()))
                .then(|| RedactionKind::Document.placeholder().to_string())
        });
        t = nt;
        add(RedactionKind::Document, n);
    }

    let (nt, n) = redact_with(&t, &BIRTH, |_, c| {
        Some(format!(
            "{}{}",
            &c[1],
            RedactionKind::BirthDate.placeholder()
        ))
    });
    t = nt;
    add(RedactionKind::BirthDate, n);

    for r in ADDRESSES.iter() {
        let (nt, n) = redact_with(&t, r, |h, c| {
            let m = c.get(0)?;
            isolated(h, m.start(), m.end())
                .then(|| RedactionKind::Address.placeholder().to_string())
        });
        t = nt;
        add(RedactionKind::Address, n);
    }

    for (i, r) in PHONES.iter().enumerate() {
        let (nt, n) = redact_with(&t, r, |h, c| {
            let m = c.get(0)?;
            let s = m.as_str();
            let digits = s.chars().filter(char::is_ascii_digit).count();
            if i == 0 && !(8..=15).contains(&digits) {
                return None;
            }
            // não confundir com intervalos de anos ("2019 - 2023")
            if looks_like_year_range(s) {
                return None;
            }
            isolated(h, m.start(), m.end()).then(|| RedactionKind::Phone.placeholder().to_string())
        });
        t = nt;
        add(RedactionKind::Phone, n);
    }

    (t, counts)
}

fn looks_like_year_range(s: &str) -> bool {
    static YR: Lazy<Regex> = Lazy::new(|| re(r"^(?:19|20)\d{2}\s*[\-–]\s*(?:19|20)\d{2}$"));
    YR.is_match(s.trim())
}

// ---------------------------------------------------------------------------
// Nomes
// ---------------------------------------------------------------------------

const CONNECTORS: &[&str] = &[
    "de", "da", "do", "dos", "das", "e", "van", "von", "der", "del", "di", "du", "la", "le",
];

/// Palavras que indicam que uma linha de cabeçalho NÃO é um nome de pessoa.
const NOT_NAME_WORDS: &[&str] = &[
    "curriculo",
    "curriculum",
    "vitae",
    "resume",
    "cv",
    "desenvolvedor",
    "desenvolvedora",
    "developer",
    "engineer",
    "engenheiro",
    "engenheira",
    "analista",
    "analyst",
    "software",
    "senior",
    "junior",
    "pleno",
    "full",
    "stack",
    "fullstack",
    "backend",
    "frontend",
    "back",
    "front",
    "end",
    "dados",
    "data",
    "scientist",
    "cientista",
    "gerente",
    "manager",
    "designer",
    "arquiteto",
    "arquiteta",
    "architect",
    "programador",
    "programadora",
    "tecnico",
    "tecnica",
    "consultor",
    "consultora",
    "especialista",
    "specialist",
    "coordenador",
    "coordenadora",
    "product",
    "owner",
    "experiencia",
    "experiencias",
    "formacao",
    "resumo",
    "perfil",
    "objetivo",
    "profissional",
    "habilidades",
    "contato",
    "contact",
    "summary",
    "profile",
    "skills",
    "education",
    "experience",
    "sao",
    "paulo",
    "rio",
    "janeiro",
    "brasil",
    "brazil",
    "belo",
    "horizonte",
    "porto",
    "alegre",
    "curitiba",
    "recife",
    "salvador",
    "fortaleza",
    "brasilia",
    "florianopolis",
    "campinas",
    "lead",
    "tech",
    "web",
    "mobile",
    "devops",
    "pessoais",
    "informacoes",
    "dados",
    "sobre",
    "about",
    "me",
    "mim",
    "qa",
    "ux",
    "ui",
    "estagiario",
    "estagiaria",
    "intern",
    "trainee",
    "head",
    "diretor",
    "diretora",
    "director",
    "remoto",
    "remote",
    "linkedin",
    "github",
    "portfolio",
    "competencias",
    "idiomas",
    "certificacoes",
    "projetos",
    "projects",
    "professional",
    "technical",
    "tecnologias",
    "cloud",
    "machine",
    "learning",
    "java",
    "python",
    "javascript",
    "react",
    "node",
    "net",
    "sql",
];

/// Detecta o provável nome do candidato nas primeiras linhas não vazias
/// (linha de 2 a 5 palavras capitalizadas, sem dígitos nem '@').
pub fn detect_header_name(text: &str) -> Option<String> {
    for line in text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .take(5)
    {
        let mut seg = line;
        for sep in ["|", "•", "·", "–", "—", " - ", ",", ";", "\t", "/"] {
            if let Some(i) = seg.find(sep) {
                seg = &seg[..i];
            }
        }
        let mut seg = seg
            .trim()
            .trim_start_matches(['#', '*', '>'])
            .trim()
            .trim_matches('*')
            .trim();
        let lower = fold(seg);
        for prefix in ["nome:", "name:", "nome completo:", "full name:"] {
            if lower.starts_with(prefix) {
                // prefixo é ASCII após o fold, mesmo tamanho em bytes no original? Não
                // necessariamente; corta pelo número de caracteres.
                let n = prefix.chars().count();
                let idx = seg
                    .char_indices()
                    .nth(n)
                    .map(|(i, _)| i)
                    .unwrap_or(seg.len());
                seg = seg[idx..].trim();
                break;
            }
        }
        if is_name_like(seg) {
            return Some(seg.to_string());
        }
    }
    None
}

fn is_name_like(s: &str) -> bool {
    if s.is_empty() || s.chars().count() > 60 {
        return false;
    }
    if s.chars()
        .any(|c| c.is_ascii_digit() || "@:/\\()[]{}+=#$%&_".contains(c))
    {
        return false;
    }
    let words: Vec<&str> = s.split_whitespace().collect();
    if words.len() > 7 {
        return false;
    }
    let mut proper = 0;
    for w in &words {
        let f = fold(w);
        if CONNECTORS.contains(&f.as_str()) {
            continue;
        }
        if NOT_NAME_WORDS.contains(&f.trim_matches('.')) {
            return false;
        }
        let first = w.chars().next().unwrap_or(' ');
        let valid_chars = w
            .chars()
            .all(|c| c.is_alphabetic() || c == '-' || c == '\'' || c == '’' || c == '.');
        if !first.is_uppercase() || !valid_chars {
            return false;
        }
        proper += 1;
    }
    (2..=5).contains(&proper)
}

/// Converte um token em padrão regex que ignora acentos.
fn accent_insensitive(token: &str) -> String {
    let mut out = String::new();
    for ch in fold(token).chars() {
        let class = match ch {
            'a' => "[aáàâãäå]",
            'e' => "[eéèêë]",
            'i' => "[iíìîï]",
            'o' => "[oóòôõö]",
            'u' => "[uúùûü]",
            'c' => "[cç]",
            'n' => "[nñ]",
            'y' => "[yýÿ]",
            _ => "",
        };
        if class.is_empty() {
            out.push_str(&regex::escape(&ch.to_string()));
        } else {
            out.push_str(class);
        }
    }
    out
}

fn clean_token(t: &str) -> &str {
    t.trim_matches(|c: char| !c.is_alphanumeric())
}

/// Substitui o(s) nome(s) real(is) pelo apelido. O nome completo vira o apelido
/// completo; o primeiro nome vira o primeiro nome do apelido; demais nomes viram
/// o sobrenome do apelido.
fn replace_names(text: &str, real_names: &[String], alias: &str) -> (String, usize) {
    let mut alias_parts = alias.split_whitespace();
    let alias_first = alias_parts.next().unwrap_or(alias).to_string();
    let alias_last = alias_parts.last().unwrap_or(&alias_first).to_string();

    // (padrão, substituição), deduplicados pelo padrão.
    let mut entries: Vec<(String, String)> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for name in real_names {
        let toks: Vec<&str> = name
            .split_whitespace()
            .map(clean_token)
            .filter(|t| !t.is_empty())
            .collect();
        if toks.is_empty() {
            continue;
        }
        if toks.len() > 1 {
            let pat = toks
                .iter()
                .map(|t| accent_insensitive(t))
                .collect::<Vec<_>>()
                .join(r"\s+");
            if seen.insert(pat.clone()) {
                entries.push((pat, alias.to_string()));
            }
        }
        for (i, t) in toks.iter().enumerate() {
            let f = fold(t);
            if f.chars().count() < 3 || CONNECTORS.contains(&f.as_str()) {
                continue;
            }
            let pat = accent_insensitive(t);
            if seen.insert(pat.clone()) {
                let rep = if i == 0 {
                    alias_first.clone()
                } else {
                    alias_last.clone()
                };
                entries.push((pat, rep));
            }
        }
    }
    if entries.is_empty() {
        return (text.to_string(), 0);
    }
    // mais longos primeiro para a alternância preferir o nome completo
    entries.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then(a.0.cmp(&b.0)));
    let alternation = entries
        .iter()
        .map(|(p, _)| format!("({p})"))
        .collect::<Vec<_>>()
        .join("|");
    let Ok(rx) = Regex::new(&format!(r"(?i)\b(?:{alternation})\b")) else {
        return (text.to_string(), 0);
    };
    redact_with(text, &rx, |_, c| {
        (1..=entries.len())
            .find(|i| c.get(*i).is_some())
            .map(|i| entries[i - 1].1.clone())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn rng() -> StdRng {
        StdRng::seed_from_u64(42)
    }

    fn count(a: &AnonymizedResume, k: RedactionKind) -> usize {
        a.redactions
            .iter()
            .find(|r| r.kind == k)
            .map(|r| r.count)
            .unwrap_or(0)
    }

    const CV: &str = "João da Silva Pereira\n\
        Desenvolvedor Backend Sênior\n\
        joao.silva@gmail.com | (11) 98765-4321 | +55 11 91234-5678\n\
        linkedin.com/in/joaosilva | https://github.com/joaosilva\n\
        CPF: 123.456.789-09 | RG: 12.345.678-9\n\
        Rua das Flores, 123 - Vila Mariana, São Paulo - SP\n\
        CEP 04001-000\n\
        Data de nascimento: 15/03/1990\n\n\
        Resumo\n\
        João é desenvolvedor com 8 anos de experiência em Java e Python.\n\
        Experiência\n\
        Empresa X — 2019 - 2023\n";

    #[test]
    fn pokemon_list_is_large_enough() {
        let names = pokemon_names();
        assert!(names.len() >= 151, "{} nomes", names.len());
        let set: HashSet<_> = names.iter().map(|n| n.to_lowercase()).collect();
        assert_eq!(set.len(), names.len(), "nomes duplicados");
        assert!(names.contains(&"Pikachu"));
        assert!(names.contains(&"Mew"));
    }

    #[test]
    fn removes_contact_data_and_documents() {
        let a = anonymize(CV, None, &HashSet::new(), &mut rng());
        let t = &a.text;
        assert!(!t.contains("joao.silva@gmail.com"), "{t}");
        assert!(!t.contains("98765-4321"), "{t}");
        assert!(!t.contains("91234-5678"), "{t}");
        assert!(!t.contains("123.456.789-09"), "{t}");
        assert!(!t.contains("12.345.678-9"), "{t}");
        assert!(!t.contains("github.com"), "{t}");
        assert!(!t.contains("linkedin"), "{t}");
        assert!(!t.contains("Flores"), "{t}");
        assert!(!t.contains("04001-000"), "{t}");
        assert!(!t.contains("15/03/1990"), "{t}");
        assert!(t.contains("[email removido]"));
        assert!(t.contains("[telefone removido]"));
        assert!(t.contains("[documento removido]"));
        assert!(t.contains("[link removido]"));
        assert!(t.contains("[endereço removido]"));
        assert!(t.contains("[data removida]"));
        // datas de experiência permanecem
        assert!(t.contains("2019 - 2023"), "{t}");
        assert!(t.contains("8 anos"), "{t}");
        assert_eq!(count(&a, RedactionKind::Email), 1);
        assert_eq!(count(&a, RedactionKind::Phone), 2);
        assert_eq!(count(&a, RedactionKind::Url), 2);
        assert_eq!(count(&a, RedactionKind::Document), 2);
        assert!(count(&a, RedactionKind::Address) >= 2);
        assert_eq!(count(&a, RedactionKind::BirthDate), 1);
        assert!(a.redactions.iter().all(|r| r.count > 0));
    }

    #[test]
    fn replaces_header_name_with_alias() {
        let a = anonymize(CV, None, &HashSet::new(), &mut rng());
        assert!(!a.text.contains("João"), "{}", a.text);
        assert!(!a.text.contains("Pereira"), "{}", a.text);
        assert!(a.text.starts_with(&a.alias), "{}", a.text);
        assert!(count(&a, RedactionKind::Name) >= 2);
        // título não é tratado como nome
        assert!(a.text.contains("Desenvolvedor Backend Sênior"));
    }

    #[test]
    fn declared_name_is_accent_and_case_insensitive() {
        let text = "Resumo\nMaria Conceição Souza trabalha com dados. Contato: MARIA CONCEICAO.\nA Souza lidera times.";
        let a = anonymize(
            text,
            Some("Maria Conceição Souza"),
            &HashSet::new(),
            &mut rng(),
        );
        let f = fold(&a.text);
        assert!(!f.contains("maria"), "{}", a.text);
        assert!(!f.contains("conceicao"), "{}", a.text);
        assert!(!f.contains("souza"), "{}", a.text);
        assert!(a.text.contains(&a.alias));
        assert!(count(&a, RedactionKind::Name) >= 3);
    }

    #[test]
    fn english_resume_pii() {
        let text = "Jane Mary Doe\njane.doe@example.org · +1 (555) 123-4567 · www.janedoe.dev\n123 Main Street, Springfield\nBorn on March 3, 1992\n\nExperience\nAcme Inc 2018 - present";
        let a = anonymize(text, None, &HashSet::new(), &mut rng());
        let t = &a.text;
        assert!(!t.contains("Jane"), "{t}");
        assert!(!t.contains("Doe"), "{t}");
        assert!(!t.contains("example.org"), "{t}");
        assert!(!t.contains("555"), "{t}");
        assert!(!t.contains("janedoe.dev"), "{t}");
        assert!(!t.contains("Main Street"), "{t}");
        assert!(!t.contains("1992"), "{t}");
        assert!(t.contains("2018 - present"), "{t}");
    }

    #[test]
    fn codenames_are_unique() {
        let mut r = rng();
        let mut taken = HashSet::new();
        for _ in 0..pokemon_names().len() {
            let c = pick_codename(&taken, &mut r);
            assert!(pokemon_names().contains(&c.as_str()));
            assert!(taken.insert(c));
        }
        // todos usados: sufixo numérico
        let c = pick_codename(&taken, &mut r);
        assert!(c.ends_with(" 2"), "{c}");
        assert!(taken.insert(c.clone()));
        let base = c.trim_end_matches(" 2").to_string();
        // com "X 2" já usado para essa base, não repete
        for _ in 0..50 {
            let c2 = pick_codename(&taken, &mut r);
            assert!(!taken.contains(&c2));
            if c2.starts_with(&base) {
                assert_eq!(c2, format!("{base} 3"));
            }
        }
    }

    #[test]
    fn codename_respects_taken_case_insensitive() {
        let mut taken: HashSet<String> = pokemon_names().iter().map(|n| n.to_uppercase()).collect();
        taken.remove("PIKACHU");
        let c = pick_codename(&taken, &mut rng());
        assert_eq!(c, "Pikachu");
    }

    #[test]
    fn alias_differs_from_real_name() {
        let mut r = rng();
        for _ in 0..50 {
            let a = anonymize("Texto qualquer", Some("Ana Silva"), &HashSet::new(), &mut r);
            assert!(!a.alias.starts_with("Ana "));
            assert!(!a.alias.ends_with(" Silva"));
            assert_eq!(a.alias.split_whitespace().count(), 2);
        }
    }

    #[test]
    fn header_name_detection() {
        assert_eq!(
            detect_header_name("\n\n  Carlos Eduardo Ramos  \nDev"),
            Some("Carlos Eduardo Ramos".into())
        );
        assert_eq!(
            detect_header_name("Nome: Ana Lúcia Prado\n"),
            Some("Ana Lúcia Prado".into())
        );
        assert_eq!(
            detect_header_name("FERNANDA LIMA | Engenheira de Dados"),
            Some("FERNANDA LIMA".into())
        );
        assert_eq!(
            detect_header_name("Currículo Vitae\nDesenvolvedor Python\n"),
            None
        );
        assert_eq!(detect_header_name("Experiência em 5 empresas"), None);
    }

    #[test]
    fn plain_numbers_are_kept() {
        let text =
            "Reduzi custos em 35% e atendi 1200 clientes entre 2015-2020. Salário R$ 10.000.";
        let (t, counts) = redact_pii(text);
        assert_eq!(t, text);
        assert!(counts.is_empty());
    }
}
