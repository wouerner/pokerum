//! Extração de texto de arquivos enviados (pdf / txt / md).

use crate::error::{AppError, AppResult};
use crate::MAX_UPLOAD_BYTES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    Pdf,
    Text,
}

/// Detecta o tipo pelo conteúdo (magic `%PDF`), nome do arquivo e content-type.
pub fn detect_kind(
    filename: Option<&str>,
    content_type: Option<&str>,
    bytes: &[u8],
) -> AppResult<FileKind> {
    if bytes.starts_with(b"%PDF") {
        return Ok(FileKind::Pdf);
    }
    let ext = filename
        .and_then(|f| f.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase()))
        .unwrap_or_default();
    let ct = content_type.unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "pdf" => return Ok(FileKind::Pdf),
        "txt" | "md" | "markdown" | "text" => return Ok(FileKind::Text),
        "" => {}
        _ => {
            return Err(AppError::UnsupportedMedia(
                "formato não suportado: envie PDF, TXT ou MD".into(),
            ))
        }
    }
    if ct.starts_with("application/pdf") {
        Ok(FileKind::Pdf)
    } else if ct.starts_with("text/plain") || ct.starts_with("text/markdown") {
        Ok(FileKind::Text)
    } else {
        Err(AppError::UnsupportedMedia(
            "formato não suportado: envie PDF, TXT ou MD".into(),
        ))
    }
}

/// Extrai o texto de um arquivo enviado. Roda o parser de PDF numa thread
/// bloqueante (e isola panics do parser).
pub async fn extract_text(
    filename: Option<&str>,
    content_type: Option<&str>,
    bytes: Vec<u8>,
) -> AppResult<String> {
    if bytes.len() > MAX_UPLOAD_BYTES {
        return Err(AppError::PayloadTooLarge(
            "arquivo excede o limite de 5 MB".into(),
        ));
    }
    if bytes.is_empty() {
        return Err(AppError::BadRequest("arquivo vazio".into()));
    }
    match detect_kind(filename, content_type, &bytes)? {
        FileKind::Text => {
            let s = String::from_utf8(bytes)
                .map_err(|_| AppError::BadRequest("arquivo de texto não está em UTF-8".into()))?;
            Ok(s.trim_start_matches('\u{feff}').to_string())
        }
        FileKind::Pdf => {
            let res =
                tokio::task::spawn_blocking(move || pdf_extract::extract_text_from_mem(&bytes))
                    .await;
            match res {
                Ok(Ok(text)) => Ok(text),
                Ok(Err(e)) => {
                    tracing::warn!(error = %e, "falha ao ler PDF");
                    Err(AppError::Unprocessable(
                        "não foi possível extrair texto do PDF".into(),
                    ))
                }
                Err(e) => {
                    tracing::warn!(error = %e, "parser de PDF falhou");
                    Err(AppError::Unprocessable(
                        "não foi possível extrair texto do PDF".into(),
                    ))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_kinds() {
        assert_eq!(
            detect_kind(Some("cv.PDF"), None, b"x").unwrap(),
            FileKind::Pdf
        );
        assert_eq!(detect_kind(None, None, b"%PDF-1.4").unwrap(), FileKind::Pdf);
        assert_eq!(
            detect_kind(Some("cv.md"), None, b"x").unwrap(),
            FileKind::Text
        );
        assert_eq!(
            detect_kind(None, Some("text/plain; charset=utf-8"), b"x").unwrap(),
            FileKind::Text
        );
        assert!(detect_kind(Some("cv.docx"), None, b"PK").is_err());
        assert!(detect_kind(Some("cv.docx"), Some("text/plain"), b"PK").is_err());
    }
}
