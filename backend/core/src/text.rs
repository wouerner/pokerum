//! Normalização de texto compartilhada: acentos, caixa, tokens técnicos,
//! stopwords PT+EN e um stemmer leve.

use std::collections::HashSet;

use once_cell::sync::Lazy;
use regex::Regex;
use unicode_normalization::char::is_combining_mark;
use unicode_normalization::UnicodeNormalization;

/// Minúsculas + remoção de acentos ("Ação" -> "acao").
pub fn fold(s: &str) -> String {
    s.nfd()
        .filter(|c| !is_combining_mark(*c))
        .collect::<String>()
        .to_lowercase()
}

static TECH: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| {
    vec![
        (Regex::new(r"(^|[^a-z0-9])c\+\+").unwrap(), "$1 cplusplus "),
        (Regex::new(r"(^|[^a-z0-9])c#").unwrap(), "$1 csharp "),
        (Regex::new(r"(^|[^a-z0-9])f#").unwrap(), "$1 fsharp "),
        (Regex::new(r"(^|[^a-z0-9.])\.net\b").unwrap(), "$1 dotnet "),
    ]
});

/// Reescreve tokens técnicos com símbolos ("c++", "c#", ".net") em formas
/// alfanuméricas estáveis. Espera texto já em minúsculas.
fn rewrite_tech(s: &str) -> String {
    let mut out = s.to_string();
    for (re, rep) in TECH.iter() {
        if re.is_match(&out) {
            out = re.replace_all(&out, *rep).into_owned();
        }
    }
    out
}

/// Texto normalizado para comparação: sem acentos, minúsculo, tokens técnicos reescritos.
pub fn normalize(s: &str) -> String {
    rewrite_tech(&fold(s))
}

fn split_raw(s: &str) -> impl Iterator<Item = &str> {
    s.split(|c: char| !(c.is_alphanumeric() || c == '.' || c == '+' || c == '#'))
        .map(|t| t.trim_matches(|c| c == '.' || c == '+' || c == '#'))
        .filter(|t| !t.is_empty())
}

/// Tokeniza o texto normalizado. Mantém pontos internos ("node.js", "asp.net").
/// "C++" -> "cplusplus", "C#" -> "csharp", ".NET" -> "dotnet", "CI/CD" -> ["ci", "cd"].
pub fn tokens(s: &str) -> Vec<String> {
    let n = normalize(s);
    split_raw(&n).map(str::to_string).collect()
}

/// Tokens na forma de superfície (minúsculos, COM acentos) — para exibição.
pub fn surface_tokens(s: &str) -> Vec<String> {
    let n = rewrite_tech(&s.nfc().collect::<String>().to_lowercase());
    split_raw(&n).map(str::to_string).collect()
}

static STOPWORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    const PT: &str = "a o as os um uma uns umas de do da dos das no na nos nas em por para pra pelo pela pelos pelas com sem sob sobre entre ate apos ante e ou mas nem que se como quando onde quanto qual quais cujo cuja ja nao sim mais menos muito muita muitos muitas pouco pouca bem mal tambem so apenas ainda sempre nunca ser estar ter haver ir fazer foi foram era eram sou somos sao esta estao estava estavam tem tinha temos tenho ha houve sera serao seja sejam fui fazia faz fez feito este esta estes estas esse essa esses essas aquele aquela aqueles aquelas isto isso aquilo ele ela eles elas eu tu voce voces nos vos me te lhe lhes meu minha meus minhas seu sua seus suas nosso nossa nossos nossas dele dela deles delas ao aos a à às num numa outro outra outros outras mesmo mesma todo toda todos todas cada qualquer alguns algumas algum alguma nenhum nenhuma tal tais assim entao pois porque porem contudo todavia logo etc ex via atraves durante desde conforme segundo afim fim forma tipo vez vezes";
    const EN: &str = "a an the and or but nor of to in on at by for with without from into onto over under about above below as is are was were be been being am do does did doing done have has had having will would shall should can could may might must it its this that these those there here i me my mine we us our ours you your yours he him his she her hers they them their theirs what which who whom whose when where why how all any both each few more most other some such no not only own same so than too very just also etc via per eg ie up down out off again further then once if because while until against through during before after between";
    PT.split_whitespace().chain(EN.split_whitespace()).collect()
});

/// Palavras comuns em anúncios de vaga que não ajudam a diferenciar candidatos.
static JOB_NOISE: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    const W: &str = "vaga vagas empresa empresas time times equipe equipes buscamos procuramos procura busca oportunidade oportunidades requisitos requisito responsabilidades responsabilidade atividades atividade beneficios beneficio diferenciais diferencial desejavel desejaveis obrigatorio obrigatorios obrigatoria conhecimento conhecimentos experiencia experiencias experiencia anos ano trabalho trabalhar trabalhando atuar atuacao atuando area areas profissional profissionais candidato candidata candidatos pessoa pessoas sera voce local remoto remota hibrido hibrida presencial salario contratacao clt pj plano planos saude vale refeicao alimentacao odontologico gympass auxilio bonus plr home office nivel bom boa bons boas forte fortes solido solida solidos dominio desejado desejada necessario necessaria necessarios capacidade habilidade habilidades competencias perfil sobre nos vamos venha junte faca parte crescimento ambiente cultura missao valores contrato tempo integral regime modelo cargo funcao posicao horario segunda sexta ideal principais principal incluindo outras outros novo nova novos novas grande grandes melhor melhores alta alto altos altas diversas diversos varios varias junior pleno senior jr sr estagio estagiario lead lider work working team teams company looking join role position requirements requirement responsibilities responsibility skills skill nice plus strong ability knowledge including will must have should good great job description tasks task qualifications qualification preferred required minimum solid years year experience experienced level opportunity benefits benefit candidate candidates environment culture mission values ideal you we our help new best high etc least related similar field fields using use used based within across well familiarity understanding proficiency proficient hands excellent communication comunicacao boas praticas pratica praticas ingles english espanhol spanish frances french alemao german italiano italian portugues portuguese mandarim japones intermediario intermediaria avancado avancada fluente fluent basico basica advanced intermediate native nativo conversacao idioma idiomas";
    W.split_whitespace().collect()
});

/// `token` (já normalizado) é stopword PT/EN?
pub fn is_stopword(token: &str) -> bool {
    STOPWORDS.contains(token)
}

/// `token` (já normalizado) é stopword ou palavra genérica de anúncio de vaga?
pub fn is_noise(token: &str) -> bool {
    STOPWORDS.contains(token) || JOB_NOISE.contains(token)
}

/// Token com conteúdo (não stopword, não número puro, tamanho >= 2).
pub fn is_content(token: &str) -> bool {
    token.chars().count() >= 2
        && !token.chars().all(|c| c.is_ascii_digit() || c == '.')
        && !is_stopword(token)
}

const SUFFIXES: &[&str] = &[
    "imentos", "amentos", "imento", "amento", "mente", "edores", "edoras", "edora", "edor",
    "acoes", "acao", "icoes", "icao", "coes", "cao", "oes", "ings", "ing", "ments", "ment", "ness",
    "ers", "er", "ies", "ed", "es", "as", "os", "s", "a", "o", "e",
];

/// Stemmer leve PT+EN (remove sufixos comuns). Determinístico e simétrico: o
/// mesmo stem é aplicado à vaga e ao currículo.
pub fn stem(token: &str) -> String {
    if token.len() <= 4 || token.contains('.') || token.chars().any(|c| c.is_ascii_digit()) {
        return token.to_string();
    }
    for suf in SUFFIXES {
        if token.ends_with(suf) && token.len() - suf.len() >= 4 {
            return token[..token.len() - suf.len()].to_string();
        }
    }
    token.to_string()
}

/// Conjunto de n-gramas (1..=5) de tokens normalizados, para busca de frases.
#[derive(Debug, Default, Clone)]
pub struct NgramSet {
    set: HashSet<String>,
}

impl NgramSet {
    pub const MAX_N: usize = 5;

    pub fn from_tokens(toks: &[String]) -> Self {
        let mut set = HashSet::new();
        for i in 0..toks.len() {
            let mut gram = String::new();
            for (n, t) in toks[i..].iter().take(Self::MAX_N).enumerate() {
                if n > 0 {
                    gram.push(' ');
                }
                gram.push_str(t);
                set.insert(gram.clone());
            }
        }
        NgramSet { set }
    }

    pub fn from_text(text: &str) -> Self {
        Self::from_tokens(&tokens(text))
    }

    /// `phrase` já normalizada (tokens unidos por espaço).
    pub fn contains_normalized(&self, phrase: &str) -> bool {
        self.set.contains(phrase)
    }

    /// Normaliza `phrase` e verifica se ocorre no texto.
    pub fn has_phrase(&self, phrase: &str) -> bool {
        let p = tokens(phrase).join(" ");
        !p.is_empty() && self.set.contains(&p)
    }

    pub fn has_any(&self, phrases: &[&str]) -> bool {
        phrases.iter().any(|p| self.has_phrase(p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_accents() {
        assert_eq!(fold("Ação Sênior ÇÃO"), "acao senior cao");
    }

    #[test]
    fn tech_tokens() {
        assert_eq!(
            tokens("C++, C# e .NET Core"),
            vec!["cplusplus", "csharp", "e", "dotnet", "core"]
        );
        assert_eq!(tokens("Node.js; CI/CD."), vec!["node.js", "ci", "cd"]);
        assert_eq!(tokens("ASP.NET e Python."), vec!["asp.net", "e", "python"]);
        assert_eq!(tokens("5+ anos"), vec!["5", "anos"]);
    }

    #[test]
    fn stems_are_symmetric() {
        assert_eq!(stem("aplicacoes"), stem("aplicacao"));
        assert_eq!(stem("developer"), stem("development"));
        assert_eq!(stem("sistemas"), stem("sistema"));
    }

    #[test]
    fn ngrams() {
        let n = NgramSet::from_text("Experiência com Spring Boot e Power BI");
        assert!(n.has_phrase("spring boot"));
        assert!(n.has_phrase("Power BI"));
        assert!(!n.has_phrase("boot power bi java"));
    }
}
