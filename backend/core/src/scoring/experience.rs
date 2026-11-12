//! Extração de anos de experiência (exigidos pela vaga e presentes no currículo).

use once_cell::sync::Lazy;
use regex::Regex;

use super::sections::{line_sections, SectionKind};
use crate::text::fold;

static RANGE_REQ: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\d{1,2})\s*(?:a|-|–|to|ate|ou)\s*(\d{1,2})\s*\+?\s*(?:anos|ano|years|year|yrs)\b")
        .unwrap()
});
static SINGLE_REQ: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\d{1,2})\s*(\+)?\s*(?:\([a-z]+\)\s*)?(?:anos|ano|years|year|yrs)\b").unwrap()
});

const EXP_CONTEXT: &[&str] = &[
    "experien",
    "atua",
    "vivencia",
    "working",
    "work ",
    "profissional",
    "minimo",
    "at least",
    "pelo menos",
    "desenvolv",
    "trabalh",
    "mercado",
    "carreira",
    "background",
];

/// Anos de experiência exigidos em um texto de vaga ("5+ anos", "mínimo de 3 anos",
/// "3+ years", "3 a 5 anos" -> 3). Retorna o maior valor encontrado em contexto
/// de experiência.
pub fn years_required(text: &str) -> Option<f64> {
    let mut best: Option<f64> = None;
    for line in text.lines() {
        let f = fold(line);
        let has_ctx = EXP_CONTEXT.iter().any(|w| f.contains(w));
        let mut consumed: Vec<(usize, usize)> = Vec::new();
        for c in RANGE_REQ.captures_iter(&f) {
            let m = c.get(0).unwrap();
            consumed.push((m.start(), m.end()));
            if !has_ctx {
                continue;
            }
            if let Ok(lo) = c[1].parse::<f64>() {
                if lo > 0.0 && lo <= 20.0 {
                    best = Some(best.map_or(lo, |b: f64| b.max(lo)));
                }
            }
        }
        for c in SINGLE_REQ.captures_iter(&f) {
            let m = c.get(0).unwrap();
            if consumed.iter().any(|(s, e)| m.start() < *e && m.end() > *s) {
                continue;
            }
            if !(has_ctx || c.get(2).is_some()) {
                continue;
            }
            if let Ok(v) = c[1].parse::<f64>() {
                if v > 0.0 && v <= 20.0 {
                    best = Some(best.map_or(v, |b: f64| b.max(v)));
                }
            }
        }
    }
    best
}

static EXPLICIT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\d{1,2})\s*\+?\s*(?:anos|years)\s+(?:de\s+|of\s+)?(?:experiencia|experience|atuacao|carreira|mercado|trabalho|working|professional)").unwrap()
});

/// Anos de experiência declarados explicitamente no currículo ("8 anos de experiência").
pub fn explicit_years(text: &str) -> Option<f64> {
    let f = fold(text);
    EXPLICIT
        .captures_iter(&f)
        .filter_map(|c| c[1].parse::<f64>().ok())
        .filter(|v| *v > 0.0 && *v <= 50.0)
        .fold(None, |acc: Option<f64>, v| {
            Some(acc.map_or(v, |a| a.max(v)))
        })
}

static DATE_RANGE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?x)
        (?:(\d{1,2})\s*/\s*)?
        (?:([a-z]{3,9})\.?\s*(?:de\s+|/\s*)?)?
        ((?:19|20)\d{2})
        \s*(?:-|–|—|\ba\b|\bate\b|\bto\b|\buntil\b|\bao\b)\s*
        (?:(\d{1,2})\s*/\s*)?
        (?:([a-z]{3,9})\.?\s*(?:de\s+|/\s*)?)?
        ((?:19|20)\d{2}|atualmente|atual|presente|present|current|hoje|now|o\s+momento|today|momento)
        ",
    )
    .unwrap()
});

fn month_from_name(s: &str) -> Option<u32> {
    let m = match s.get(..3)? {
        "jan" => 1,
        "fev" | "feb" => 2,
        "mar" => 3,
        "abr" | "apr" => 4,
        "mai" | "may" => 5,
        "jun" => 6,
        "jul" => 7,
        "ago" | "aug" => 8,
        "set" | "sep" => 9,
        "out" | "oct" => 10,
        "nov" => 11,
        "dez" | "dec" => 12,
        _ => return None,
    };
    Some(m)
}

fn month_of(num: Option<regex::Match>, name: Option<regex::Match>) -> Option<u32> {
    if let Some(n) = num.and_then(|m| m.as_str().parse::<u32>().ok()) {
        if (1..=12).contains(&n) {
            return Some(n);
        }
    }
    name.and_then(|m| month_from_name(m.as_str()))
}

/// Períodos (início, fim) em meses absolutos (ano*12 + mês-1) encontrados no texto.
pub fn date_ranges(text: &str, now_year: i32, now_month: u32) -> Vec<(i32, i32)> {
    let f = fold(text);
    let now_idx = now_year * 12 + now_month as i32 - 1;
    let mut out = Vec::new();
    for c in DATE_RANGE.captures_iter(&f) {
        let Ok(sy) = c[3].parse::<i32>() else {
            continue;
        };
        let sm = month_of(c.get(1), c.get(2));
        let end_raw = &c[6];
        let (ey, em) = if end_raw.chars().all(|ch| ch.is_ascii_digit()) {
            let Ok(ey) = end_raw.parse::<i32>() else {
                continue;
            };
            (ey, month_of(c.get(4), c.get(5)))
        } else {
            (now_year, Some(now_month))
        };
        if sy < 1960 || ey < sy || sy > now_year {
            continue;
        }
        let start = sy * 12 + sm.unwrap_or(1) as i32 - 1;
        let mut end = ey * 12 + em.unwrap_or(1) as i32 - 1;
        if sm.is_none() && em.is_none() && ey == sy {
            end = start + 6;
        }
        let end = end.min(now_idx.max(start));
        if end > start {
            out.push((start, end));
        }
    }
    out
}

/// Soma de meses dos períodos, unindo sobreposições.
pub fn merged_months(mut ranges: Vec<(i32, i32)>) -> i32 {
    ranges.sort();
    let mut total = 0;
    let mut cur: Option<(i32, i32)> = None;
    for (s, e) in ranges {
        match cur {
            Some((cs, ce)) if s <= ce => cur = Some((cs, ce.max(e))),
            Some((cs, ce)) => {
                total += ce - cs;
                cur = Some((s, e));
            }
            None => cur = Some((s, e)),
        }
    }
    if let Some((cs, ce)) = cur {
        total += ce - cs;
    }
    total
}

/// Estimativa de anos de experiência profissional no currículo: o maior valor
/// entre a soma dos períodos (fora das seções de formação/certificações/projetos)
/// e anos declarados explicitamente.
pub fn estimate_years(text: &str, now_year: i32, now_month: u32) -> f64 {
    let secs = line_sections(text);
    let relevant: String = text
        .lines()
        .zip(secs)
        .filter(|(_, s)| {
            !matches!(
                s,
                Some(SectionKind::Education)
                    | Some(SectionKind::Certifications)
                    | Some(SectionKind::Projects)
                    | Some(SectionKind::Languages)
            )
        })
        .map(|(l, _)| l)
        .collect::<Vec<_>>()
        .join("\n");
    let months = merged_months(date_ranges(&relevant, now_year, now_month));
    let by_dates = months as f64 / 12.0;
    let explicit = explicit_years(text).unwrap_or(0.0);
    let years = by_dates.max(explicit);
    (years * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_years_pt_en() {
        assert_eq!(years_required("Experiência de 5+ anos com Java"), Some(5.0));
        assert_eq!(
            years_required("Mínimo de 3 anos de experiência em backend"),
            Some(3.0)
        );
        assert_eq!(
            years_required("3+ years of experience with Python"),
            Some(3.0)
        );
        assert_eq!(
            years_required("At least 4 years working with Go"),
            Some(4.0)
        );
        assert_eq!(years_required("Experiência de 3 a 5 anos"), Some(3.0));
        assert_eq!(years_required("Empresa com 20 anos de história"), None);
        assert_eq!(years_required("Sem requisitos de tempo"), None);
    }

    #[test]
    fn resume_years_from_dates() {
        let cv = "Experiência\nEmpresa A — jan/2015 - dez/2017\nEmpresa B — 03/2018 a 02/2021\n\nFormação\nUSP 2010 - 2014";
        let y = estimate_years(cv, 2025, 6);
        assert!((5.8..=6.0).contains(&y), "{y}");
    }

    #[test]
    fn resume_years_current_and_overlap() {
        let cv = "Experience\nAcme — January 2020 - Present\nSide gig 2021 - 2022";
        let y = estimate_years(cv, 2025, 1);
        assert!((4.9..=5.1).contains(&y), "{y}");
    }

    #[test]
    fn resume_years_explicit() {
        let cv = "Sou desenvolvedora com mais de 10 anos de experiência em dados.";
        assert_eq!(estimate_years(cv, 2025, 1), 10.0);
    }

    #[test]
    fn merges_overlaps() {
        assert_eq!(merged_months(vec![(0, 12), (6, 18), (30, 36)]), 24);
    }
}
