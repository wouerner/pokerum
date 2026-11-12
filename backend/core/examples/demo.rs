//! Demonstração: `cargo run -p pokerum_core --example demo`

use std::collections::HashSet;

use pokerum_core::anonymize::anonymize;
use pokerum_core::scoring::{analyze_job, score};

const TITLE: &str = "Desenvolvedor Backend Sênior";
const DESCRIPTION: &str =
    "Buscamos pessoa desenvolvedora backend sênior para nossa plataforma de pagamentos.

Requisitos:
- 5+ anos de experiência com desenvolvimento backend
- Java, Spring Boot, PostgreSQL e Kafka
- Docker e Kubernetes
- Inglês intermediário

Diferenciais:
- AWS
- Terraform
";

const RESUME: &str = "Joana Prado Martins
joana.prado@email.com | (11) 98765-4321 | linkedin.com/in/joanaprado
Rua das Acácias, 45 - São Paulo/SP - CEP 01234-567

Resumo
Desenvolvedora backend com 7 anos de experiência em Java e microsserviços.

Experiência
Fintech ABC — Desenvolvedora Backend — 02/2019 - atual
- Microsserviços em Java/Spring Boot com PostgreSQL
- Reduzi o tempo de resposta das APIs em 35%
- Deploy com Docker
Empresa XYZ — Desenvolvedora — 01/2017 - 01/2019
- APIs REST em Java

Formação
Bacharelado em Sistemas de Informação — 2012 - 2016

Habilidades
Java, Spring Boot, PostgreSQL, Docker, Git
";

fn main() {
    let job = analyze_job(TITLE, DESCRIPTION);
    println!("== Vaga: {}", job.title);
    println!("Palavras-chave: {}", job.keywords().join(", "));
    println!(
        "Anos exigidos: {:?} | Senioridade: {:?} | Idiomas: {:?}\n",
        job.years_required,
        job.seniority,
        job.languages
            .iter()
            .map(|l| &l.language)
            .collect::<Vec<_>>()
    );

    let mut rng = rand::thread_rng();
    let anon = anonymize(
        RESUME,
        Some("Joana Prado Martins"),
        &HashSet::new(),
        &mut rng,
    );
    println!(
        "== Currículo anonimizado: {} ({})",
        anon.codename, anon.alias
    );
    println!("{}", anon.text);
    println!("Remoções: {:?}\n", anon.redactions);

    let report = score(&job, &anon.text);
    println!(
        "== Pontuação: {:.1} / 100 ({})",
        report.total, report.engine_version
    );
    for c in &report.breakdown {
        println!(
            "  {:<24} {:>5.1}  (peso {:.2})  {}",
            c.label, c.score, c.weight, c.detail
        );
    }
    println!("\nEncontradas: {}", report.matched_keywords.join(", "));
    println!("Ausentes:    {}", report.missing_keywords.join(", "));
    println!("\nSugestões:");
    for s in &report.suggestions {
        println!("  - {s}");
    }
}
