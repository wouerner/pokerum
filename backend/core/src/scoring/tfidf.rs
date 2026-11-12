//! Similaridade TF-IDF / cosseno entre vaga e currículo.
//!
//! Como só há dois documentos, o IDF é estimado sobre as *linhas* de ambos os
//! textos: termos presentes em quase todas as linhas pesam menos.

use std::collections::{BTreeMap, BTreeSet};

use crate::text::{is_content, stem, tokens};

/// Stems de conteúdo de um texto.
pub fn content_stems(text: &str) -> Vec<String> {
    tokens(text)
        .into_iter()
        .filter(|t| is_content(t))
        .map(|t| stem(&t))
        .collect()
}

/// Frequência de termos (stem -> contagem).
pub fn term_counts(text: &str) -> BTreeMap<String, f64> {
    let mut m = BTreeMap::new();
    for s in content_stems(text) {
        *m.entry(s).or_insert(0.0) += 1.0;
    }
    m
}

/// Frequência de documento por linha não vazia (stem -> nº de linhas) e nº de linhas.
pub fn line_doc_freq(text: &str) -> (BTreeMap<String, u32>, u32) {
    let mut df = BTreeMap::new();
    let mut n = 0;
    for line in text.lines() {
        let set: BTreeSet<String> = content_stems(line).into_iter().collect();
        if set.is_empty() {
            continue;
        }
        n += 1;
        for s in set {
            *df.entry(s).or_insert(0) += 1;
        }
    }
    (df, n)
}

/// Cosseno entre os vetores TF-IDF (tf sublinear) da vaga e do currículo.
pub fn cosine(
    job_tf: &BTreeMap<String, f64>,
    job_df: &BTreeMap<String, u32>,
    job_lines: u32,
    resume_text: &str,
) -> f64 {
    let res_tf = term_counts(resume_text);
    let (res_df, res_lines) = line_doc_freq(resume_text);
    let n = (job_lines + res_lines) as f64;
    let idf = |t: &str| {
        let df = job_df.get(t).copied().unwrap_or(0) + res_df.get(t).copied().unwrap_or(0);
        ((n + 1.0) / (df as f64 + 1.0)).ln() + 1.0
    };
    let w = |tf: f64| if tf > 0.0 { 1.0 + tf.ln() } else { 0.0 };

    let mut dot = 0.0;
    let mut na = 0.0;
    let mut nb = 0.0;
    for (t, tf) in job_tf {
        let a = w(*tf) * idf(t);
        na += a * a;
        if let Some(rtf) = res_tf.get(t) {
            dot += a * w(*rtf) * idf(t);
        }
    }
    for (t, tf) in &res_tf {
        let b = w(*tf) * idf(t);
        nb += b * b;
    }
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na.sqrt() * nb.sqrt())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn similar_texts_score_higher() {
        let job = "Desenvolvedor Python com Django, APIs REST e PostgreSQL. Testes automatizados.";
        let tf = term_counts(job);
        let (df, n) = line_doc_freq(job);
        let good = cosine(
            &tf,
            &df,
            n,
            "Desenvolvedora Python: APIs REST com Django e PostgreSQL, testes automatizados.",
        );
        let bad = cosine(
            &tf,
            &df,
            n,
            "Vendedora de varejo com foco em atendimento ao cliente e metas comerciais.",
        );
        assert!(good > 0.5, "{good}");
        assert!(bad < 0.1, "{bad}");
        assert!(cosine(&tf, &df, n, "") == 0.0);
    }
}
