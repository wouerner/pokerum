use rand::Rng;

fn fold_char(c: char) -> Option<char> {
    let c = c.to_lowercase().next().unwrap_or(c);
    let mapped = match c {
        'á' | 'à' | 'â' | 'ã' | 'ä' | 'å' => 'a',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'í' | 'ì' | 'î' | 'ï' => 'i',
        'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
        'ú' | 'ù' | 'û' | 'ü' => 'u',
        'ç' => 'c',
        'ñ' => 'n',
        c if c.is_ascii_alphanumeric() => c,
        _ => return None,
    };
    Some(mapped)
}

/// "Desenvolvedor Backend Rust Sênior" -> "desenvolvedor-backend-rust-senior"
pub fn slugify(s: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in s.chars() {
        match fold_char(c) {
            Some(c) => {
                if dash && !out.is_empty() {
                    out.push('-');
                }
                dash = false;
                out.push(c);
            }
            None => dash = true,
        }
    }
    // limita o tamanho sem cortar no meio de forma feia
    if out.len() > 60 {
        out.truncate(60);
        while out.ends_with('-') {
            out.pop();
        }
    }
    if out.is_empty() {
        out.push_str("vaga");
    }
    out
}

/// Slug + sufixo aleatório curto (6 chars [a-z0-9]).
pub fn job_slug(title: &str, rng: &mut impl Rng) -> String {
    const ALPHA: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let suffix: String = (0..6)
        .map(|_| ALPHA[rng.gen_range(0..ALPHA.len())] as char)
        .collect();
    format!("{}-{}", slugify(title), suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_pt() {
        assert_eq!(
            slugify("Desenvolvedor Backend Rust Sênior"),
            "desenvolvedor-backend-rust-senior"
        );
        assert_eq!(
            slugify("Engenheira(o) de Dados Pleno"),
            "engenheira-o-de-dados-pleno"
        );
        assert_eq!(slugify("  !!! "), "vaga");
    }
}
