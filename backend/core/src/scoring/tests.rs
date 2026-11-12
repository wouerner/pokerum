use super::*;

const JOB_TITLE: &str = "Desenvolvedor Backend Sênior";
const JOB_DESC: &str = "Sobre a vaga\n\
Buscamos uma pessoa desenvolvedora backend sênior para evoluir nossa plataforma de pagamentos.\n\
\n\
Responsabilidades:\n\
- Projetar e manter microsserviços em Java com Spring Boot\n\
- Modelar dados em PostgreSQL e trabalhar com mensageria (Kafka)\n\
- Garantir qualidade com testes automatizados e code review\n\
\n\
Requisitos:\n\
- 5+ anos de experiência com desenvolvimento backend\n\
- Java, Spring Boot, PostgreSQL, Kafka\n\
- Docker e Kubernetes\n\
- Inglês intermediário\n\
- Graduação em Ciência da Computação ou áreas afins\n\
\n\
Diferenciais:\n\
- AWS\n\
- Terraform\n";

const STRONG_CV: &str = "Alex Duarte\n\
Desenvolvedor Backend Sênior\n\
\n\
Resumo\n\
Desenvolvedor backend sênior com 8 anos de experiência em plataformas de pagamentos, microsserviços Java e Spring Boot.\n\
\n\
Experiência Profissional\n\
Fintech X — Desenvolvedor Backend Sênior — 03/2020 - atual\n\
- Projetei microsserviços em Java 17 e Spring Boot para a plataforma de pagamentos\n\
- Reduzi a latência das APIs em 40% otimizando consultas PostgreSQL\n\
- Implantei mensageria com Kafka processando 2 milhões de eventos/dia\n\
- Deploy em Kubernetes (EKS) na AWS com Docker e Terraform\n\
Banco Y — Desenvolvedor Backend — 01/2016 - 02/2020\n\
- Desenvolvimento de APIs REST em Java, testes automatizados com JUnit e code review\n\
- Aumentei a cobertura de testes de 30% para 85%\n\
\n\
Formação\n\
Bacharelado em Ciência da Computação — Universidade Federal — 2011 - 2015\n\
\n\
Habilidades\n\
Java, Spring Boot, PostgreSQL, Kafka, Docker, Kubernetes, AWS, Terraform, Git\n\
\n\
Idiomas\n\
Inglês avançado\n";

const WEAK_CV: &str = "Sam Rocha\n\
Atendente de loja\n\
Trabalho com vendas e atendimento ao cliente há 2 anos.\n\
Conhecimentos em Excel e Word.\n";

fn job() -> JobProfile {
    analyze_job(JOB_TITLE, JOB_DESC)
}

fn crit<'a>(r: &'a ScoreReport, key: &str) -> &'a CriterionScore {
    r.breakdown.iter().find(|c| c.key == key).unwrap()
}

#[test]
fn strong_beats_weak() {
    let j = job();
    let strong = score_at(&j, STRONG_CV, 2025, 6);
    let weak = score_at(&j, WEAK_CV, 2025, 6);
    assert!(
        strong.total >= 80.0,
        "forte = {} {:#?}",
        strong.total,
        strong.breakdown
    );
    assert!(
        weak.total <= 30.0,
        "fraco = {} {:#?}",
        weak.total,
        weak.breakdown
    );
    assert!(strong.total - weak.total >= 40.0);
    assert!(weak.missing_keywords.contains(&"Kubernetes".to_string()));
    assert!(strong.matched_keywords.contains(&"Kubernetes".to_string()));
    assert!(weak.suggestions.iter().any(|s| s.contains("'Kubernetes'")));
}

#[test]
fn report_shape_and_weights() {
    let r = score_at(&job(), STRONG_CV, 2025, 6);
    assert_eq!(r.engine_version, "heuristic-v1");
    let keys: Vec<&str> = r.breakdown.iter().map(|c| c.key.as_str()).collect();
    assert_eq!(
        keys,
        [
            "hard_skills",
            "semantic",
            "experience",
            "seniority_title",
            "education_lang",
            "ats_format"
        ]
    );
    let wsum: f64 = r.breakdown.iter().map(|c| c.weight).sum();
    assert!((wsum - 1.0).abs() < 1e-9);
    for c in &r.breakdown {
        assert!((0.0..=100.0).contains(&c.score), "{c:?}");
        assert!(!c.label.is_empty() && !c.detail.is_empty());
    }
    let expected: f64 = r.breakdown.iter().map(|c| c.weight * c.score).sum();
    assert!((r.total - (expected * 10.0).round() / 10.0).abs() < 1e-9);
    assert_eq!(r.total, (r.total * 10.0).round() / 10.0);
    assert!(crit(&r, "hard_skills")
        .detail
        .contains("skills encontradas"));
    // serializa
    let j = serde_json::to_string(&r).unwrap();
    let back: ScoreReport = serde_json::from_str(&j).unwrap();
    assert_eq!(back, r);
}

#[test]
fn deterministic() {
    let j = job();
    let a = score_at(&j, STRONG_CV, 2025, 6);
    let b = score_at(&j, STRONG_CV, 2025, 6);
    assert_eq!(a, b);
    assert_eq!(analyze_job(JOB_TITLE, JOB_DESC), j);
}

#[test]
fn synonyms_count_as_match() {
    let j = analyze_job(
        "Dev Fullstack",
        "Requisitos: JavaScript, PostgreSQL, Kubernetes, Node.js",
    );
    let r = score_at(
        &j,
        "Experiência\nUso JS, Postgres, k8s e NodeJS diariamente.",
        2025,
        1,
    );
    let hs = crit(&r, "hard_skills");
    assert!(hs.detail.starts_with("4 de 4"), "{}", hs.detail);
    assert!(r.missing_keywords.iter().all(|k| ![
        "JavaScript",
        "PostgreSQL",
        "Kubernetes",
        "Node.js"
    ]
    .contains(&k.as_str())));
}

#[test]
fn nice_to_have_weighs_less() {
    let j = analyze_job(
        "Dev",
        "Requisitos:\n- Python\n- Django\nDiferenciais:\n- AWS\n- Docker\n",
    );
    let only_req = score_at(&j, "Python e Django", 2025, 1);
    let only_nice = score_at(&j, "AWS e Docker", 2025, 1);
    assert!(crit(&only_req, "hard_skills").score > crit(&only_nice, "hard_skills").score);
}

#[test]
fn experience_criterion() {
    let j = job();
    let strong = score_at(&j, STRONG_CV, 2025, 6);
    assert_eq!(
        crit(&strong, "experience").score,
        100.0,
        "{:?}",
        crit(&strong, "experience")
    );
    let junior_cv = "Experiência\nEmpresa Z — 01/2023 - 01/2024\n- Java e Spring Boot";
    let r = score_at(&j, junior_cv, 2025, 6);
    let e = crit(&r, "experience");
    assert!(e.score > 0.0 && e.score < 40.0, "{e:?}");
    assert!(r.suggestions.iter().any(|s| s.contains("5 anos")));
}

#[test]
fn english_job_and_resume() {
    let title = "Senior Data Engineer";
    let jd = "We are looking for a Senior Data Engineer.\nRequirements:\n- 5+ years of experience building data pipelines\n- Python, SQL, Apache Spark and Airflow\n- Experience with AWS or GCP\n- Fluent English\nNice to have:\n- dbt\n- Kafka\n";
    let j = analyze_job(title, jd);
    assert_eq!(j.years_required, Some(5.0));
    assert_eq!(j.seniority, Some(Seniority::Senior));
    let good = "Summary\nSenior data engineer with 7 years of experience building data pipelines.\n\nExperience\nAcme Corp — Senior Data Engineer — Jan 2019 - Present\n- Built batch pipelines with Python, PySpark and Airflow on AWS\n- Cut warehouse costs by 35% using dbt and SQL optimizations\n- Streamed events with Kafka\nBeta Inc — Data Engineer — Feb 2017 - Dec 2018\n- ETL in Python and SQL\n\nEducation\nB.Sc. in Computer Science, State University, 2016\n\nSkills\nPython, SQL, Spark, Airflow, dbt, Kafka, AWS\n";
    let bad = "Summary\nGraphic designer focused on branding.\n\nExperience\nStudio — Designer — 2022 - 2023\n- Photoshop and Illustrator\n";
    let rg = score_at(&j, good, 2025, 6);
    let rb = score_at(&j, bad, 2025, 6);
    assert!(rg.total > 80.0, "{} {:#?}", rg.total, rg.breakdown);
    assert!(rb.total < 35.0, "{} {:#?}", rb.total, rb.breakdown);
    // inglês detectado pelo idioma do texto
    assert_eq!(
        crit(&rg, "education_lang").score,
        100.0,
        "{:?}",
        crit(&rg, "education_lang")
    );
}

#[test]
fn missing_sections_generate_suggestions() {
    let r = score_at(&job(), WEAK_CV, 2025, 6);
    assert!(r
        .suggestions
        .iter()
        .any(|s| s.contains("'Experiência Profissional'")));
    assert!(r.suggestions.iter().any(|s| s.contains("Quantifique")));
    assert!(crit(&r, "ats_format").score < 30.0);
}

#[test]
fn empty_inputs_do_not_panic() {
    let j = analyze_job("", "");
    let r = score(&j, "");
    assert!((0.0..=100.0).contains(&r.total));
    let r = score(&job(), "");
    assert!(r.total < 15.0, "{}", r.total);
    assert!(!r.suggestions.is_empty());
}

#[test]
fn keywords_for_display() {
    let k = job().keywords();
    assert!(k.starts_with(&["Java".to_string()]) || k.contains(&"Java".to_string()));
    let pos_java = k.iter().position(|x| x == "Java").unwrap();
    let pos_aws = k.iter().position(|x| x == "AWS").unwrap();
    assert!(
        pos_java < pos_aws,
        "obrigatórias antes dos diferenciais: {k:?}"
    );
}

#[test]
fn today_is_sane() {
    let (y, m) = super::today();
    assert!(y >= 2024 && (1..=12).contains(&m));
}
