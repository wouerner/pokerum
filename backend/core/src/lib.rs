//! `pokerum_core`: anonimização de currículos e pontuação heurística contra vagas,
//! simulando como motores ATS (com IA) avaliariam um candidato.
//!
//! - [`anonymize`]: codinome Pokémon, apelido aleatório e remoção de PII.
//! - [`scoring`]: análise da vaga ([`scoring::analyze_job`]) e pontuação
//!   ([`scoring::score`]).
//!
//! Sem I/O e sem banco de dados.

pub mod anonymize;
pub mod scoring;
pub mod text;
