//! Google Analytics opcional (`POKERUM_GA_ID`): a tag entra no `<head>` do index.html servido
//! pelo próprio servidor, então a ID muda sem rebuild da imagem.
//!
//! LGPD (Consent Mode): por padrão `analytics_storage` fica negado, e o GA só manda sinais
//! anônimos, sem cookies. O aviso de cookies do site (`web/src/components/ConsentBar.vue`) grava a
//! escolha em `localStorage["pokerum-ga-consent"]` (`granted` | `denied`) e chama
//! `gtag('consent','update',…)`. `window.pokerumAnalytics` avisa o site de que a tag existe.

/// Chave do localStorage com a escolha do visitante (igual em `ConsentBar.vue`).
pub const CONSENT_KEY: &str = "pokerum-ga-consent";

/// Aceita só IDs do GA4 (`G-XXXXXXXXXX`): letras maiúsculas e dígitos, sem nada que escape do HTML.
pub fn valid_ga_id(id: &str) -> bool {
    let Some(rest) = id.strip_prefix("G-") else {
        return false;
    };
    (4..=20).contains(&rest.len())
        && rest
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
}

/// Lê `POKERUM_GA_ID`; vazio = sem analytics. ID inválida é ignorada com aviso no log.
pub fn ga_id_from_env() -> Option<String> {
    let id = std::env::var("POKERUM_GA_ID").ok()?;
    let id = id.trim();
    if id.is_empty() {
        return None;
    }
    if !valid_ga_id(id) {
        tracing::warn!(id, "POKERUM_GA_ID inválido (esperado G-XXXXXXXXXX); analytics desligado");
        return None;
    }
    Some(id.to_owned())
}

/// O trecho do `<head>`: consentimento padrão negado, config do GA4 e o gtag.js.
pub fn snippet(id: &str) -> String {
    debug_assert!(valid_ga_id(id));
    format!(
        r#"<script>
window.dataLayer=window.dataLayer||[];function gtag(){{dataLayer.push(arguments);}}
(function(){{var c=null;try{{c=localStorage.getItem("{CONSENT_KEY}");}}catch(e){{}}
gtag("consent","default",{{ad_storage:"denied",ad_user_data:"denied",ad_personalization:"denied",analytics_storage:c==="granted"?"granted":"denied"}});
gtag("js",new Date());gtag("config","{id}");window.pokerumAnalytics={{id:"{id}"}};}})();
</script>
<script async src="https://www.googletagmanager.com/gtag/js?id={id}"></script>
"#
    )
}

/// Insere o trecho antes de `</head>` (ou no começo, se não houver `</head>`).
pub fn inject(html: &str, id: &str) -> String {
    let tag = snippet(id);
    match html.find("</head>") {
        Some(i) => format!("{}{}{}", &html[..i], tag, &html[i..]),
        None => format!("{tag}{html}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_ids() {
        assert!(valid_ga_id("G-47NEQJSJN8"));
        assert!(!valid_ga_id("G-"));
        assert!(!valid_ga_id("GTM-ABC123"));
        assert!(!valid_ga_id("g-47neqjsjn8"));
        assert!(!valid_ga_id("G-ABC\"<script>"));
        assert!(!valid_ga_id("UA-12345-1"));
    }

    #[test]
    fn injects_before_head_end() {
        let html = "<html><head><title>x</title></head><body></body></html>";
        let out = inject(html, "G-ABCD1234");
        let tag = out.find("googletagmanager.com/gtag/js?id=G-ABCD1234").unwrap();
        assert!(tag < out.find("</head>").unwrap());
        assert!(out.contains(r#"analytics_storage:c==="granted"?"granted":"denied""#));
        assert!(out.contains(CONSENT_KEY));
    }
}
