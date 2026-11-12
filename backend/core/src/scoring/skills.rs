//! Dicionário de hard skills com sinônimos/aliases e nomes canônicos.
//!
//! Aliases são escritos em forma natural ("node.js", "c++", "ci/cd") e
//! normalizados pelo mesmo tokenizador usado nos textos. Aliases prefixados
//! com `=` são comparados de forma sensível a maiúsculas no texto original
//! (para termos ambíguos como "Go", "R", "C").

use std::collections::{BTreeSet, HashMap, HashSet};

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::text::{tokens, NgramSet};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SkillCategory {
    Language,
    Framework,
    Cloud,
    Database,
    DevOps,
    Data,
    Practice,
    Tool,
    Business,
}

/// Definição de uma skill do dicionário.
#[derive(Debug)]
pub struct SkillDef {
    pub canonical: &'static str,
    pub category: SkillCategory,
    /// aliases normalizados (tokens unidos por espaço)
    pub aliases: Vec<String>,
    /// aliases sensíveis a maiúsculas
    exact: Vec<Regex>,
}

use SkillCategory::*;

#[rustfmt::skip]
static RAW: &[(&str, SkillCategory, &[&str])] = &[
    // ---- linguagens
    ("Java", Language, &["java", "java 8", "java 11", "java 17", "java 21", "jdk"]),
    ("Kotlin", Language, &["kotlin"]),
    ("Scala", Language, &["scala"]),
    ("JavaScript", Language, &["javascript", "js", "ecmascript", "es6", "java script"]),
    ("TypeScript", Language, &["typescript", "type script"]),
    ("Python", Language, &["python", "python3", "python 3", "py"]),
    ("Go", Language, &["golang", "=Go", "linguagem go", "go lang"]),
    ("Rust", Language, &["rust", "rustlang"]),
    ("C", Language, &["=C", "linguagem c", "ansi c"]),
    ("C++", Language, &["c++", "cpp", "cplusplus"]),
    ("C#", Language, &["c#", "csharp", "c sharp"]),
    ("Ruby", Language, &["ruby"]),
    ("PHP", Language, &["php", "php8", "php 8"]),
    ("Swift", Language, &["swift"]),
    ("Objective-C", Language, &["objective-c", "objective c", "objc"]),
    ("Dart", Language, &["dart"]),
    ("R", Language, &["=R", "linguagem r", "rstudio", "r studio"]),
    ("Elixir", Language, &["elixir"]),
    ("Erlang", Language, &["erlang"]),
    ("Clojure", Language, &["clojure"]),
    ("Haskell", Language, &["haskell"]),
    ("Perl", Language, &["perl"]),
    ("Lua", Language, &["=Lua", "linguagem lua"]),
    ("MATLAB", Language, &["matlab"]),
    ("COBOL", Language, &["cobol"]),
    ("Delphi", Language, &["delphi", "object pascal"]),
    ("VB.NET", Language, &["vb.net", "visual basic"]),
    ("F#", Language, &["f#", "fsharp"]),
    ("Julia", Language, &["julialang", "linguagem julia", "julia lang"]),
    ("Solidity", Language, &["solidity"]),
    ("Groovy", Language, &["groovy"]),
    ("Bash/Shell", Language, &["bash", "shell script", "shell scripting", "shellscript", "zsh", "powershell"]),
    ("HTML", Language, &["html", "html5"]),
    ("CSS", Language, &["css", "css3", "sass", "scss", "less css"]),
    ("SQL", Database, &["sql", "t-sql", "tsql", "pl/sql", "plsql", "pl sql", "ansi sql"]),
    // ---- frameworks / libs
    ("React", Framework, &["react", "react.js", "reactjs", "react js"]),
    ("React Native", Framework, &["react native", "react-native"]),
    ("Angular", Framework, &["angular", "angularjs", "angular.js", "angular 2"]),
    ("Vue.js", Framework, &["vue", "vue.js", "vuejs", "vue 3", "vue js"]),
    ("Svelte", Framework, &["svelte", "sveltekit"]),
    ("Next.js", Framework, &["next.js", "nextjs", "next js"]),
    ("Nuxt", Framework, &["nuxt", "nuxt.js", "nuxtjs"]),
    ("Node.js", Framework, &["node.js", "nodejs", "node", "node js"]),
    ("Express", Framework, &["express.js", "expressjs", "=Express"]),
    ("NestJS", Framework, &["nestjs", "nest.js", "nest js"]),
    ("Django", Framework, &["django", "django rest framework", "drf"]),
    ("Flask", Framework, &["flask"]),
    ("FastAPI", Framework, &["fastapi", "fast api"]),
    ("Spring", Framework, &["spring", "spring boot", "springboot", "spring framework", "spring cloud"]),
    ("Quarkus", Framework, &["quarkus"]),
    (".NET", Framework, &[".net", "dotnet", "dotnet core", ".net core", "asp.net", "asp.net core", "net core"]),
    ("Laravel", Framework, &["laravel"]),
    ("Symfony", Framework, &["symfony"]),
    ("Ruby on Rails", Framework, &["ruby on rails", "rails", "ror"]),
    ("Phoenix", Framework, &["phoenix framework", "phoenix liveview"]),
    ("Flutter", Framework, &["flutter"]),
    ("jQuery", Framework, &["jquery"]),
    ("Redux", Framework, &["redux", "redux toolkit"]),
    ("GraphQL", Framework, &["graphql", "apollo"]),
    ("REST API", Framework, &["restful", "rest api", "api rest", "apis rest", "rest apis", "apis restful", "api restful"]),
    ("gRPC", Framework, &["grpc", "protobuf", "protocol buffers"]),
    ("Microsserviços", Practice, &["microsservicos", "microservicos", "microservices", "microservice", "micro servicos", "micro-services", "microsservico"]),
    ("Tailwind CSS", Framework, &["tailwind", "tailwindcss", "tailwind css"]),
    ("Bootstrap", Framework, &["bootstrap"]),
    ("Jest", Tool, &["jest"]),
    ("Cypress", Tool, &["cypress"]),
    ("Selenium", Tool, &["selenium", "webdriver"]),
    ("JUnit", Tool, &["junit", "junit5", "mockito"]),
    ("Pytest", Tool, &["pytest"]),
    ("Hibernate", Framework, &["hibernate", "jpa"]),
    ("RabbitMQ", Tool, &["rabbitmq", "rabbit mq", "amqp"]),
    ("Kafka", Data, &["kafka", "apache kafka"]),
    ("Webpack", Tool, &["webpack", "vite", "babel"]),
    ("Android", Framework, &["android", "android sdk", "jetpack compose"]),
    ("iOS", Framework, &["ios", "swiftui", "uikit"]),
    ("Unity", Tool, &["unity", "unity3d"]),
    ("WordPress", Tool, &["wordpress"]),
    ("Electron", Framework, &["electron"]),
    ("Tokio", Framework, &["tokio", "actix", "axum", "actix-web"]),
    // ---- cloud
    ("AWS", Cloud, &["aws", "amazon web services", "ec2", "s3", "aws lambda", "lambda", "cloudformation", "ecs", "eks", "sqs", "sns"]),
    ("Azure", Cloud, &["azure", "microsoft azure", "azure devops"]),
    ("Google Cloud", Cloud, &["gcp", "google cloud", "google cloud platform", "cloud run", "app engine"]),
    ("Serverless", Cloud, &["serverless"]),
    ("Heroku", Cloud, &["heroku"]),
    ("Firebase", Cloud, &["firebase", "firestore"]),
    // ---- devops
    ("Docker", DevOps, &["docker", "dockerfile", "docker compose", "docker-compose", "containers", "conteineres", "containerizacao"]),
    ("Kubernetes", DevOps, &["kubernetes", "k8s", "eks", "aks", "gke", "openshift"]),
    ("Helm", DevOps, &["helm"]),
    ("Terraform", DevOps, &["terraform", "iac", "infrastructure as code", "infraestrutura como codigo"]),
    ("Ansible", DevOps, &["ansible"]),
    ("Jenkins", DevOps, &["jenkins"]),
    ("GitHub Actions", DevOps, &["github actions"]),
    ("GitLab CI", DevOps, &["gitlab ci", "gitlab-ci"]),
    ("CI/CD", DevOps, &["ci/cd", "cicd", "ci cd", "continuous integration", "integracao continua", "entrega continua", "continuous delivery", "continuous deployment", "ci pipelines", "pipelines ci cd"]),
    ("Linux", DevOps, &["linux", "ubuntu", "debian", "centos", "red hat", "unix"]),
    ("Nginx", DevOps, &["nginx"]),
    ("Prometheus", DevOps, &["prometheus"]),
    ("Grafana", DevOps, &["grafana"]),
    ("Datadog", DevOps, &["datadog", "new relic", "newrelic"]),
    ("Observabilidade", Practice, &["observabilidade", "observability", "monitoramento", "monitoring", "opentelemetry"]),
    ("Git", Tool, &["git", "github", "gitlab", "bitbucket", "versionamento"]),
    ("DevOps", Practice, &["devops"]),
    ("SRE", Practice, &["sre", "site reliability"]),
    // ---- bancos
    ("PostgreSQL", Database, &["postgresql", "postgres", "psql", "pgsql"]),
    ("MySQL", Database, &["mysql"]),
    ("MariaDB", Database, &["mariadb"]),
    ("SQL Server", Database, &["sql server", "sqlserver", "mssql", "ms sql"]),
    ("Oracle", Database, &["oracle", "oracle database", "oracle db"]),
    ("SQLite", Database, &["sqlite"]),
    ("MongoDB", Database, &["mongodb", "mongo", "mongo db"]),
    ("Redis", Database, &["redis"]),
    ("Elasticsearch", Database, &["elasticsearch", "elastic search", "elk", "opensearch", "kibana"]),
    ("Cassandra", Database, &["cassandra"]),
    ("DynamoDB", Database, &["dynamodb", "dynamo db"]),
    ("NoSQL", Database, &["nosql", "no-sql"]),
    ("Neo4j", Database, &["neo4j"]),
    ("Snowflake", Data, &["snowflake"]),
    ("BigQuery", Data, &["bigquery", "big query"]),
    ("Redshift", Data, &["redshift"]),
    // ---- dados / ML
    ("Pandas", Data, &["pandas"]),
    ("NumPy", Data, &["numpy"]),
    ("scikit-learn", Data, &["scikit-learn", "scikit learn", "sklearn"]),
    ("TensorFlow", Data, &["tensorflow", "tf.keras"]),
    ("PyTorch", Data, &["pytorch", "torch"]),
    ("Keras", Data, &["keras"]),
    ("Machine Learning", Data, &["machine learning", "aprendizado de maquina", "ml", "aprendizagem de maquina"]),
    ("Deep Learning", Data, &["deep learning", "aprendizado profundo", "redes neurais", "neural networks"]),
    ("NLP", Data, &["nlp", "pln", "processamento de linguagem natural", "natural language processing"]),
    ("LLM", Data, &["llm", "llms", "large language models", "large language model", "modelos de linguagem", "genai", "ia generativa", "generative ai", "gpt", "openai", "rag", "prompt engineering"]),
    ("LangChain", Data, &["langchain", "llamaindex"]),
    ("Inteligência Artificial", Data, &["inteligencia artificial", "artificial intelligence"]),
    ("Visão Computacional", Data, &["visao computacional", "computer vision", "opencv"]),
    ("Spark", Data, &["spark", "apache spark", "pyspark"]),
    ("Hadoop", Data, &["hadoop", "hive", "hdfs"]),
    ("Airflow", Data, &["airflow", "apache airflow"]),
    ("dbt", Data, &["dbt"]),
    ("Databricks", Data, &["databricks"]),
    ("ETL", Data, &["etl", "elt", "pipelines de dados", "data pipelines", "data pipeline", "pipeline de dados"]),
    ("Data Warehouse", Data, &["data warehouse", "datawarehouse", "dw", "data lake", "datalake", "lakehouse"]),
    ("MLOps", Data, &["mlops", "mlflow", "kubeflow", "sagemaker"]),
    ("Estatística", Data, &["estatistica", "statistics", "statistical", "estatisticos", "estatisticas"]),
    ("Power BI", Business, &["power bi", "powerbi", "dax"]),
    ("Tableau", Business, &["tableau"]),
    ("Looker", Business, &["looker", "looker studio", "data studio"]),
    // ---- práticas
    ("Scrum", Practice, &["scrum", "scrum master", "sprints", "sprint"]),
    ("Kanban", Practice, &["kanban"]),
    ("Metodologias Ágeis", Practice, &["agile", "metodologias ageis", "metodologia agil", "metodos ageis", "scrum/kanban"]),
    ("TDD", Practice, &["tdd", "test driven development", "desenvolvimento orientado a testes"]),
    ("BDD", Practice, &["bdd", "behavior driven development", "cucumber"]),
    ("Testes automatizados", Practice, &["testes automatizados", "testes unitarios", "testes de integracao", "unit tests", "unit testing", "automated tests", "automated testing", "integration tests", "testes e2e", "e2e tests"]),
    ("Clean Code", Practice, &["clean code", "codigo limpo"]),
    ("Clean Architecture", Practice, &["clean architecture", "arquitetura limpa", "arquitetura hexagonal", "hexagonal architecture"]),
    ("DDD", Practice, &["ddd", "domain driven design", "domain-driven design"]),
    ("SOLID", Practice, &["=SOLID", "principios solid", "solid principles"]),
    ("Design Patterns", Practice, &["design patterns", "padroes de projeto"]),
    ("Segurança (OWASP)", Practice, &["owasp", "appsec", "seguranca da informacao", "information security", "pentest"]),
    ("OAuth/JWT", Practice, &["oauth", "oauth2", "jwt", "openid connect", "oidc", "keycloak"]),
    ("Code Review", Practice, &["code review", "revisao de codigo", "pull requests"]),
    ("UX/UI", Practice, &["ux", "ui ux", "ux ui", "user experience", "experiencia do usuario", "ui design", "ux design"]),
    // ---- ferramentas / negócio
    ("Jira", Tool, &["jira"]),
    ("Confluence", Tool, &["confluence"]),
    ("Figma", Tool, &["figma"]),
    ("Photoshop", Tool, &["photoshop"]),
    ("Illustrator", Tool, &["illustrator"]),
    ("Excel", Business, &["excel", "microsoft excel", "ms excel", "planilhas", "spreadsheets", "excel avancado", "advanced excel", "procv", "vlookup", "tabelas dinamicas", "pivot tables"]),
    ("VBA", Business, &["vba", "macros"]),
    ("Pacote Office", Business, &["pacote office", "microsoft office", "ms office", "office 365", "microsoft 365"]),
    ("SAP", Business, &["sap", "sap erp", "sap s/4hana", "s/4hana", "sap fi", "sap mm", "sap sd", "abap"]),
    ("Salesforce", Business, &["salesforce", "sfdc"]),
    ("TOTVS", Business, &["totvs", "protheus", "datasul"]),
    ("ERP", Business, &["erp"]),
    ("CRM", Business, &["crm", "hubspot", "pipedrive"]),
    ("Google Analytics", Business, &["google analytics", "ga4"]),
    ("SEO", Business, &["seo"]),
    ("Marketing Digital", Business, &["marketing digital", "digital marketing", "google ads", "meta ads"]),
    ("Gestão de Projetos", Business, &["gestao de projetos", "gerenciamento de projetos", "project management", "pmp", "pmbok"]),
    ("ITIL", Business, &["itil"]),
    ("Lean / Six Sigma", Business, &["six sigma", "=Lean", "lean six sigma", "lean manufacturing", "green belt"]),
    ("OKR", Business, &["okr", "okrs"]),
    ("LGPD", Business, &["lgpd", "gdpr"]),
    ("Power Automate", Business, &["power automate", "power apps", "powerapps"]),
];

/// Todas as skills do dicionário.
pub static SKILLS: Lazy<Vec<SkillDef>> = Lazy::new(|| {
    RAW.iter()
        .map(|(canonical, category, aliases)| {
            let mut norm = Vec::new();
            let mut exact = Vec::new();
            for a in aliases.iter() {
                if let Some(e) = a.strip_prefix('=') {
                    let p = format!(r"(?:^|[^\w&/.+#$\-]){}(?:$|[^\w&+#$\-])", regex::escape(e));
                    exact.push(Regex::new(&p).expect("regex de skill"));
                } else {
                    let t = tokens(a).join(" ");
                    if !t.is_empty() && !norm.contains(&t) {
                        norm.push(t);
                    }
                }
            }
            SkillDef {
                canonical,
                category: *category,
                aliases: norm,
                exact,
            }
        })
        .collect()
});

/// alias normalizado -> índices de skills
static ALIAS_INDEX: Lazy<HashMap<String, Vec<usize>>> = Lazy::new(|| {
    let mut m: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, s) in SKILLS.iter().enumerate() {
        for a in &s.aliases {
            m.entry(a.clone()).or_default().push(i);
        }
    }
    m
});

/// Aliases de uma única palavra (usado para não repetir skills como
/// "palavras-chave" genéricas).
pub static ALIAS_TOKENS: Lazy<HashSet<String>> = Lazy::new(|| {
    SKILLS
        .iter()
        .flat_map(|s| s.aliases.iter().filter(|a| !a.contains(' ')).cloned())
        .collect()
});

/// Busca uma skill pelo nome canônico (sem diferenciar maiúsculas).
pub fn by_name(name: &str) -> Option<&'static SkillDef> {
    SKILLS
        .iter()
        .find(|s| s.canonical.eq_ignore_ascii_case(name))
}

/// Índices das skills presentes no texto (ordenados).
pub fn detect(text: &str) -> BTreeSet<usize> {
    detect_with(&NgramSet::from_text(text), text)
}

/// Igual a [`detect`], reaproveitando um `NgramSet` já calculado de `raw`.
pub fn detect_with(ngrams: &NgramSet, raw: &str) -> BTreeSet<usize> {
    let mut found = BTreeSet::new();
    for (alias, idxs) in ALIAS_INDEX.iter() {
        if ngrams.contains_normalized(alias) {
            found.extend(idxs.iter().copied());
        }
    }
    for (i, s) in SKILLS.iter().enumerate() {
        if !found.contains(&i) && s.exact.iter().any(|r| r.is_match(raw)) {
            found.insert(i);
        }
    }
    found
}

/// Nomes canônicos das skills presentes no texto.
pub fn detect_names(text: &str) -> Vec<&'static str> {
    detect(text)
        .into_iter()
        .map(|i| SKILLS[i].canonical)
        .collect()
}

/// A skill `def` aparece no texto?
pub fn present(def: &SkillDef, ngrams: &NgramSet, raw: &str) -> bool {
    def.aliases.iter().any(|a| ngrams.contains_normalized(a))
        || def.exact.iter().any(|r| r.is_match(raw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dictionary_is_sizeable_and_unique() {
        assert!(SKILLS.len() >= 150, "{}", SKILLS.len());
        let names: HashSet<_> = SKILLS.iter().map(|s| s.canonical.to_lowercase()).collect();
        assert_eq!(names.len(), SKILLS.len());
    }

    #[test]
    fn synonyms_map_to_canonical() {
        let n = detect_names("Trabalhei com JS, Postgres, k8s, NodeJS, Golang e C# no .NET Core; CI/CD com GitHub Actions.");
        for s in [
            "JavaScript",
            "PostgreSQL",
            "Kubernetes",
            "Node.js",
            "Go",
            "C#",
            ".NET",
            "CI/CD",
            "GitHub Actions",
        ] {
            assert!(n.contains(&s), "faltou {s} em {n:?}");
        }
        assert!(!n.contains(&"Java"), "{n:?}");
    }

    #[test]
    fn ambiguous_short_names() {
        let n = detect_names("Let's go to the meeting. Salário R$ 5.000. Fiz P&D.");
        assert!(!n.contains(&"Go") && !n.contains(&"R"), "{n:?}");
        let n = detect_names("Linguagens: Go, R, C e C++");
        for s in ["Go", "R", "C", "C++"] {
            assert!(n.contains(&s), "faltou {s} em {n:?}");
        }
    }

    #[test]
    fn non_tech_skills() {
        let n = detect_names("Excel avançado (PROCV), SAP FI, Power BI e Scrum");
        for s in ["Excel", "SAP", "Power BI", "Scrum"] {
            assert!(n.contains(&s), "faltou {s} em {n:?}");
        }
    }
}
