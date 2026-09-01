//! The `ess` command: a deterministic shell over the ESS libraries and explicit adapters.

mod load;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use ess_compiler::EssIr;
use ess_gen::graph::SystemGraph;

/// Executable System Specification tooling.
#[derive(Debug, Parser)]
#[command(name = "ess", version, about, disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validate and resolve an ESS specification.
    Validate(SpecPath),
    /// Compile a specification into canonical typed IR.
    Compile {
        #[command(flatten)]
        input: SpecPath,
        /// Where to write canonical JSON IR.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Inspect one declaration in resolved IR.
    Inspect {
        #[command(flatten)]
        input: SpecPath,
        /// Fully qualified declaration name or binding/component identifier.
        name: String,
    },
    /// Render the interaction graph.
    Graph {
        #[command(flatten)]
        input: SpecLocation,
        #[arg(long, value_enum, default_value_t = GraphFormat::Mermaid)]
        format: GraphFormat,
    },
    /// Compare two revisions semantically.
    Diff {
        #[arg(long)]
        from: PathBuf,
        #[arg(long)]
        to: PathBuf,
        #[arg(long, value_enum, default_value_t = MachineFormat::Text)]
        format: MachineFormat,
    },
    /// Report conformance and generated artifacts invalidated by a semantic change.
    Impact {
        #[arg(long)]
        from: PathBuf,
        #[arg(long)]
        to: PathBuf,
        #[arg(long)]
        suite: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = MachineFormat::Text)]
        format: MachineFormat,
    },
    /// Generate deterministic documentation, schemas, and interface contracts.
    Generate {
        #[command(flatten)]
        input: SpecLocation,
        #[arg(long, value_enum)]
        kind: Option<Projection>,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
    /// Synthesize implementation artifacts and explicit obligations.
    Synthesize {
        #[command(flatten)]
        input: SpecLocation,
        #[arg(long, value_enum, default_value_t = SynthesisTarget::Rust)]
        target: SynthesisTarget,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
    /// Generate or execute a semantic conformance suite.
    Conform {
        #[command(subcommand)]
        command: ConformCommand,
    },
    /// Import a concrete source through a declared adapter.
    Import {
        #[command(subcommand)]
        adapter: ImportAdapter,
    },
    /// Project typed IR into concrete artifacts; never apply them.
    Project {
        #[command(subcommand)]
        adapter: ProjectAdapter,
    },
    /// Inspect and compare sanitized infrastructure IR.
    Infra {
        #[command(subcommand)]
        command: InfraCommand,
    },
}

#[derive(Debug, clap::Args)]
struct SpecPath {
    /// One ESS file or a directory containing `system.yaml`.
    #[arg(long, default_value = ".")]
    path: PathBuf,
    /// Output rendering.
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
}

#[derive(Debug, clap::Args)]
struct SpecLocation {
    /// One ESS file or a directory containing `system.yaml`.
    #[arg(long, default_value = ".")]
    path: PathBuf,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Format {
    Text,
    Yaml,
    Json,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum MachineFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum GraphFormat {
    Dot,
    Mermaid,
    Json,
    Yaml,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Projection {
    Docs,
    Schema,
    #[value(name = "openapi")]
    OpenApi,
    #[value(name = "asyncapi")]
    AsyncApi,
}

impl Projection {
    fn name(self) -> &'static str {
        match self {
            Self::Docs => "docs",
            Self::Schema => "schema",
            Self::OpenApi => "openapi",
            Self::AsyncApi => "asyncapi",
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SynthesisTarget {
    Rust,
    Go,
    Web,
}

impl SynthesisTarget {
    fn target(self) -> ess_synth::Target {
        match self {
            Self::Rust => ess_synth::Target::Rust,
            Self::Go => ess_synth::Target::Go,
            Self::Web => ess_synth::Target::Web,
        }
    }
}

#[derive(Debug, Subcommand)]
enum ConformCommand {
    /// Generate the suite the specification obliges.
    Synthesize {
        #[command(flatten)]
        input: SpecPath,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Run a generated or committed suite against a built-in reference implementation.
    Run {
        #[arg(long, default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        suite: Option<PathBuf>,
        #[arg(long, value_enum)]
        target: ReferenceTarget,
        /// Where to write `ess-conformance-report/1`.
        #[arg(long)]
        report_out: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ReferenceTarget {
    Billing,
    #[value(name = "oracle-fixture")]
    OracleFixture,
}

#[derive(Debug, Subcommand)]
enum ImportAdapter {
    /// Import a sanitized observation bundle, or scan one live cluster at the credential edge.
    Kubernetes {
        /// Existing `infra-observation/1` bundle.
        #[arg(long, conflicts_with = "context")]
        path: Option<PathBuf>,
        /// Live kubeconfig context. Requires `--observation-out`.
        #[arg(long, conflicts_with = "path")]
        context: Option<String>,
        /// Where a live scan writes its sanitized source bundle.
        #[arg(long, requires = "context")]
        observation_out: Option<PathBuf>,
        /// Where to write `infra-ir/1`.
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
    /// Import supported `OpenAPI` service, operation, and interface-type semantics.
    Openapi {
        #[arg(long)]
        path: PathBuf,
        /// Where to write `ess-service-interface/1`.
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
}

#[derive(Debug, Subcommand)]
enum ProjectAdapter {
    /// Project infrastructure intent and observed IR into Kubernetes manifests and obligations.
    Kubernetes {
        #[arg(long)]
        spec: PathBuf,
        #[arg(long)]
        ir: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
    /// Project supported ESS service/interface structures into `OpenAPI`.
    Openapi {
        /// ESS specification to compile and project.
        #[arg(long, conflicts_with = "ir", required_unless_present = "ir")]
        path: Option<PathBuf>,
        /// Imported `ess-service-interface/1` to project.
        #[arg(long, conflicts_with = "path", required_unless_present = "path")]
        ir: Option<PathBuf>,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
}

#[derive(Debug, Subcommand)]
enum InfraCommand {
    /// Diagnose an observation or IR.
    Diagnose {
        #[arg(long)]
        path: PathBuf,
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
    /// Render the typed infrastructure dependency graph.
    Graph {
        #[arg(long)]
        path: PathBuf,
        #[arg(long)]
        namespace: Option<String>,
        #[arg(long, value_enum, default_value_t = GraphFormat::Mermaid)]
        format: GraphFormat,
    },
    /// Compare two infrastructure snapshots.
    Diff {
        #[arg(long)]
        from: PathBuf,
        #[arg(long)]
        to: PathBuf,
        #[arg(long, value_enum, default_value_t = MachineFormat::Text)]
        format: MachineFormat,
    },
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::from(1)
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode> {
    match cli.command {
        Command::Validate(input) => validate(&input.path, input.format),
        Command::Compile { input, out } => compile(&input.path, out.as_deref(), input.format),
        Command::Inspect { input, name } => inspect(&input.path, &name, input.format),
        Command::Graph { input, format } => graph(&input.path, format),
        Command::Diff { from, to, format } => diff(&from, &to, format),
        Command::Impact {
            from,
            to,
            suite,
            format,
        } => impact(&from, &to, suite.as_deref(), format),
        Command::Generate {
            input,
            kind,
            out,
            format,
        } => generate(&input.path, kind, out.as_deref(), format),
        Command::Synthesize {
            input,
            target,
            out,
            format,
        } => synthesize(&input.path, target, out.as_deref(), format),
        Command::Conform { command } => conform(command),
        Command::Import { adapter } => import(adapter),
        Command::Project { adapter } => project(adapter),
        Command::Infra { command } => infra(command),
    }
}

fn render<T: serde::Serialize>(value: &T, format: Format) -> Result<()> {
    match format {
        Format::Text | Format::Yaml => print!("{}", serde_yaml::to_string(value)?),
        Format::Json => println!("{}", serde_json::to_string_pretty(value)?),
    }
    Ok(())
}

fn resolved_infrastructure(path: &Path) -> Result<Box<infra_compiler::InfraIr>> {
    match load::infrastructure(path)? {
        load::LoadedInfra::Ir(ir) => Ok(ir),
        load::LoadedInfra::Refused(errors) => {
            let reasons = errors
                .as_slice()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; ");
            bail!("{} was refused: {reasons}", path.display())
        }
    }
}

fn refused(path: &Path, format: Format, refusal: &load::LoadedSpec) -> Result<ExitCode> {
    let load::LoadedSpec::Refused {
        files_read,
        problems,
        diagnostics,
    } = refusal
    else {
        unreachable!("only refusals are reported")
    };
    if matches!(format, Format::Text) {
        eprintln!("{} was refused:", path.display());
        for problem in problems {
            eprintln!("  - {problem}");
        }
        for diagnostic in diagnostics.as_slice() {
            eprintln!("  - {diagnostic}");
        }
    } else {
        render(
            &RefusalReport {
                compiled: false,
                files_read: *files_read,
                problems,
                diagnostics,
            },
            format,
        )?;
    }
    Ok(ExitCode::from(1))
}

fn resolved(path: &Path, format: Format) -> Result<Result<(Box<EssIr>, usize), ExitCode>> {
    let loaded = load::specification(path)?;
    match loaded {
        load::LoadedSpec::Compiled { ir, files_read } => Ok(Ok((ir, files_read))),
        refusal @ load::LoadedSpec::Refused { .. } => Ok(Err(refused(path, format, &refusal)?)),
    }
}

fn validate(path: &Path, format: Format) -> Result<ExitCode> {
    let Ok((ir, files_read)) = resolved(path, format)? else {
        return Ok(ExitCode::from(1));
    };
    let report = ValidationSummary {
        valid: true,
        system: ir.system().to_string(),
        version: ir.version().to_string(),
        files_read,
        domains: ir.domains().len(),
        commands: ir.commands().len(),
        events: ir.events().len(),
        components: ir.components().len(),
        unresolved_references: &[],
    };
    if matches!(format, Format::Text) {
        println!(
            "{} {} — {files_read} file(s), valid",
            ir.system(),
            ir.version()
        );
    } else {
        render(&report, format)?;
    }
    Ok(ExitCode::SUCCESS)
}

fn compile(path: &Path, out: Option<&Path>, format: Format) -> Result<ExitCode> {
    let Ok((ir, files_read)) = resolved(path, format)? else {
        return Ok(ExitCode::from(1));
    };
    let json = ir.to_canonical_json();
    if let Some(out) = out {
        fs::write(out, &json).with_context(|| format!("writing {}", out.display()))?;
    }
    match format {
        Format::Text => println!(
            "{} {} — {files_read} file(s), {} declaration(s), compiled{}",
            ir.system(),
            ir.version(),
            ir.domains().len() + ir.types().len() + ir.commands().len() + ir.events().len(),
            out.map_or_else(String::new, |path| format!(" to {}", path.display()))
        ),
        Format::Json => print!("{json}"),
        Format::Yaml => render(&*ir, Format::Yaml)?,
    }
    Ok(ExitCode::SUCCESS)
}

fn inspect(path: &Path, name: &str, format: Format) -> Result<ExitCode> {
    let Ok((ir, _)) = resolved(path, format)? else {
        return Ok(ExitCode::from(1));
    };
    let whole = serde_json::to_value(&*ir)?;
    let families = [
        "domains",
        "types",
        "entities",
        "commands",
        "events",
        "errors",
        "views",
        "actors",
        "bindings",
        "components",
        "workloads",
    ];
    let mut matches = serde_json::Map::new();
    for family in families {
        if let Some(value) = whole.get(family).and_then(|value| value.get(name)) {
            matches.insert(family.to_owned(), value.clone());
        }
    }
    if matches.is_empty() {
        eprintln!("refused: `{name}` is not a resolved declaration");
        return Ok(ExitCode::from(1));
    }
    let value = serde_json::Value::Object(matches);
    match format {
        Format::Text | Format::Yaml => print!("{}", serde_yaml::to_string(&value)?),
        Format::Json => println!("{}", serde_json::to_string_pretty(&value)?),
    }
    Ok(ExitCode::SUCCESS)
}

fn graph(path: &Path, format: GraphFormat) -> Result<ExitCode> {
    let diagnostic_format = if matches!(format, GraphFormat::Json) {
        Format::Json
    } else {
        Format::Text
    };
    let Ok((ir, _)) = resolved(path, diagnostic_format)? else {
        return Ok(ExitCode::from(1));
    };
    let graph = SystemGraph::of(&ir);
    match format {
        GraphFormat::Dot => print!("{}", graph.dot()),
        GraphFormat::Mermaid => print!("{}", graph.mermaid()),
        GraphFormat::Json => render(&graph, Format::Json)?,
        GraphFormat::Yaml => render(&graph, Format::Yaml)?,
    }
    Ok(ExitCode::SUCCESS)
}

fn diff(from: &Path, to: &Path, format: MachineFormat) -> Result<ExitCode> {
    let diagnostic = if matches!(format, MachineFormat::Json) {
        Format::Json
    } else {
        Format::Text
    };
    let Ok((before, _)) = resolved(from, diagnostic)? else {
        return Ok(ExitCode::from(1));
    };
    let Ok((after, _)) = resolved(to, diagnostic)? else {
        return Ok(ExitCode::from(1));
    };
    match ess_diff::diff(&before, &after) {
        Ok(delta) => match format {
            MachineFormat::Text => print!("{}", ess_diff::render::text(&delta)),
            MachineFormat::Json => print!("{}", delta.to_canonical_json()),
        },
        Err(error) => {
            eprintln!("refused: {error}");
            return Ok(ExitCode::from(1));
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn impact(from: &Path, to: &Path, suite: Option<&Path>, format: MachineFormat) -> Result<ExitCode> {
    let diagnostic = if matches!(format, MachineFormat::Json) {
        Format::Json
    } else {
        Format::Text
    };
    let Ok((before, _)) = resolved(from, diagnostic)? else {
        return Ok(ExitCode::from(1));
    };
    let Ok((after, _)) = resolved(to, diagnostic)? else {
        return Ok(ExitCode::from(1));
    };
    let suite = suite
        .map(|path| -> Result<_> {
            let text =
                fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
            Ok(ess_conformance::ConformanceSuite::from_json(&text)?)
        })
        .transpose()?;
    match ess_diff::impact(&before, &after, suite.as_ref(), None) {
        Ok(report) => match format {
            MachineFormat::Text => print!("{}", ess_diff::render::impact(&report)),
            MachineFormat::Json => print!("{}", report.to_canonical_json()),
        },
        Err(error) => {
            eprintln!("refused: {error}");
            return Ok(ExitCode::from(1));
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn write_artifacts(
    out: Option<&Path>,
    artifacts: &std::collections::BTreeMap<String, ess_gen::Artifact>,
) -> Result<()> {
    if let Some(root) = out {
        for artifact in artifacts.values() {
            let path = root.join(&artifact.path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, &artifact.contents)
                .with_context(|| format!("writing {}", path.display()))?;
        }
    }
    Ok(())
}

fn generate(
    path: &Path,
    kind: Option<Projection>,
    out: Option<&Path>,
    format: Format,
) -> Result<ExitCode> {
    let Ok((ir, _)) = resolved(path, format)? else {
        return Ok(ExitCode::from(1));
    };
    let artifacts = if let Some(kind) = kind {
        let generator = ess_gen::generator(kind.name()).context("projection unavailable")?;
        ess_gen::artifact::run(generator.as_ref(), &ir)?
            .into_iter()
            .collect()
    } else {
        ess_gen::generate_all(&ir)?
    };
    write_artifacts(out, &artifacts)?;
    if matches!(format, Format::Text) {
        for artifact in artifacts.values() {
            println!("{} — {} byte(s)", artifact.path, artifact.contents.len());
        }
        println!(
            "{} artifact(s){}",
            artifacts.len(),
            out.map_or_else(
                || ", nothing written".to_owned(),
                |path| format!(", written to {}", path.display())
            )
        );
    } else {
        render(&artifacts, format)?;
    }
    Ok(ExitCode::SUCCESS)
}

fn synthesize(
    path: &Path,
    target: SynthesisTarget,
    out: Option<&Path>,
    format: Format,
) -> Result<ExitCode> {
    let Ok((ir, _)) = resolved(path, format)? else {
        return Ok(ExitCode::from(1));
    };
    let synthesis = ess_synth::synthesize_for(&ir, target.target());
    write_artifacts(out, &synthesis.artifacts)?;
    if matches!(format, Format::Text) {
        let counts = synthesis.plan.counts();
        println!(
            "{} capabilities: {} generated, {} obligation(s), {} refused",
            synthesis.plan.capabilities.len(),
            counts.generated,
            counts.obligations,
            counts.refused
        );
        println!(
            "{} artifact(s){}",
            synthesis.artifacts.len(),
            out.map_or_else(
                || ", nothing written".to_owned(),
                |path| format!(", written to {}", path.display())
            )
        );
    } else {
        render(&synthesis.artifacts, format)?;
    }
    Ok(ExitCode::SUCCESS)
}

fn conform(command: ConformCommand) -> Result<ExitCode> {
    match command {
        ConformCommand::Synthesize { input, out } => {
            let Ok((ir, _)) = resolved(&input.path, input.format)? else {
                return Ok(ExitCode::from(1));
            };
            let synthesis = ess_conformance::synthesize(&ir);
            let json = synthesis.suite.to_canonical_json();
            if let Some(out) = &out {
                fs::write(out, &json).with_context(|| format!("writing {}", out.display()))?;
            }
            match input.format {
                Format::Text => println!(
                    "{} scenario(s), {} refusal(s){}",
                    synthesis.suite.len(),
                    synthesis.refusals.len(),
                    out.map_or_else(
                        || ", nothing written".to_owned(),
                        |path| format!(", written to {}", path.display())
                    )
                ),
                Format::Json => print!("{json}"),
                Format::Yaml => render(&synthesis.suite, Format::Yaml)?,
            }
            Ok(ExitCode::SUCCESS)
        }
        ConformCommand::Run {
            path,
            suite,
            target,
            report_out,
            format,
        } => {
            let suite = if let Some(file) = suite {
                ess_conformance::ConformanceSuite::from_json(&fs::read_to_string(&file)?)?
            } else {
                let Ok((ir, _)) = resolved(&path, format)? else {
                    return Ok(ExitCode::from(1));
                };
                ess_conformance::synthesize(&ir).suite
            };
            let report = match target {
                ReferenceTarget::Billing => ess_conformance::Runner::for_suite(&suite)
                    .run(&suite, &ess_conformance::reference::Billing::new()),
                ReferenceTarget::OracleFixture => ess_conformance::Runner::for_suite(&suite)
                    .run(&suite, &ess_conformance::reference::Oracle::new()),
            };
            if let Some(path) = report_out {
                fs::write(&path, report.standalone().to_canonical_json())
                    .with_context(|| format!("writing {}", path.display()))?;
            }
            match format {
                Format::Text => print!("{report}"),
                _ => render(&report, format)?,
            }
            Ok(match report.status {
                ess_conformance::ConformanceStatus::Passed => ExitCode::SUCCESS,
                ess_conformance::ConformanceStatus::Failed => ExitCode::from(1),
                ess_conformance::ConformanceStatus::Error => ExitCode::from(3),
            })
        }
    }
}

#[derive(serde::Serialize)]
struct AdapterReport {
    adapter: &'static str,
    direction: &'static str,
    supported: Vec<&'static str>,
    coverage_gaps: Vec<String>,
    obligations: Vec<String>,
    refusals: Vec<String>,
    unresolved_references: usize,
    output: Option<String>,
}

#[derive(serde::Serialize)]
struct RefusalReport<'a> {
    compiled: bool,
    files_read: usize,
    problems: &'a [String],
    diagnostics: &'a ess_compiler::Diagnostics,
}

#[derive(serde::Serialize)]
struct ValidationSummary<'a> {
    valid: bool,
    system: String,
    version: String,
    files_read: usize,
    domains: usize,
    commands: usize,
    events: usize,
    components: usize,
    unresolved_references: &'a [&'a str],
}

fn import(adapter: ImportAdapter) -> Result<ExitCode> {
    match adapter {
        ImportAdapter::Kubernetes {
            path,
            context,
            observation_out,
            out,
            format,
        } => {
            let source = if let Some(path) = path {
                path
            } else {
                let observation =
                    observation_out.context("live Kubernetes import requires --observation-out")?;
                ess_kubernetes::scan(context.as_deref(), &observation)
                    .map_err(anyhow::Error::msg)?;
                observation
            };
            let ir = resolved_infrastructure(&source)?;
            let document = ir.document();
            if let Some(path) = &out {
                let mut json = serde_json::to_string_pretty(&document)?;
                json.push('\n');
                fs::write(path, json).with_context(|| format!("writing {}", path.display()))?;
            }
            let report = AdapterReport {
                adapter: "kubernetes",
                direction: "import",
                supported: vec![
                    "cluster",
                    "namespace",
                    "workload",
                    "service",
                    "ingress",
                    "configuration",
                    "secret-shape",
                    "runtime",
                ],
                coverage_gaps: Vec::new(),
                obligations: Vec::new(),
                refusals: Vec::new(),
                unresolved_references: ir.model.unresolved.len(),
                output: out.as_ref().map(|path| path.display().to_string()),
            };
            if matches!(format, Format::Text) {
                println!(
                    "imported Kubernetes observation as infra-ir/1 digest {}",
                    document.digest
                );
                println!("{} unresolved reference(s)", report.unresolved_references);
            } else {
                render(&report, format)?;
            }
            Ok(ExitCode::SUCCESS)
        }
        ImportAdapter::Openapi { path, out, format } => {
            import_openapi(&path, out.as_deref(), format)
        }
    }
}

fn import_openapi(path: &Path, out: Option<&Path>, format: Format) -> Result<ExitCode> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let imported = match ess_openapi::import(&text) {
        Ok(imported) => imported,
        Err(refusals) => {
            let report = AdapterReport {
                adapter: "openapi",
                direction: "import",
                supported: vec![
                    "service",
                    "operation",
                    "interface-type",
                    "local-schema-reference",
                    "json-message",
                ],
                coverage_gaps: Vec::new(),
                obligations: Vec::new(),
                refusals: refusals
                    .iter()
                    .map(|refusal| format!("{}: {}", refusal.pointer, refusal.message))
                    .collect(),
                unresolved_references: 0,
                output: None,
            };
            if matches!(format, Format::Text) {
                for refusal in &report.refusals {
                    println!("refused: {refusal}");
                }
            } else {
                render(&report, format)?;
            }
            return Ok(ExitCode::from(1));
        }
    };
    if let Some(path) = out {
        fs::write(path, imported.interface.to_canonical_json())
            .with_context(|| format!("writing {}", path.display()))?;
    }
    let report = AdapterReport {
        adapter: "openapi",
        direction: "import",
        supported: vec![
            "service",
            "operation",
            "interface-type",
            "local-schema-reference",
            "json-message",
        ],
        coverage_gaps: imported.coverage_gaps,
        obligations: imported
            .unresolved_references
            .iter()
            .map(|reference| format!("declare local interface type `{reference}`"))
            .collect(),
        refusals: Vec::new(),
        unresolved_references: imported.unresolved_references.len(),
        output: out.map(|path| path.display().to_string()),
    };
    if matches!(format, Format::Text) {
        println!(
            "imported OpenAPI {} service `{}` as {}",
            imported.interface.source_openapi,
            imported.interface.service.name,
            ess_openapi::INTERFACE_FORMAT
        );
        println!(
            "{} operation(s), {} interface type(s), {} coverage gap(s), {} unresolved reference(s)",
            imported.interface.operations.len(),
            imported.interface.types.len(),
            report.coverage_gaps.len(),
            report.unresolved_references
        );
    } else {
        render(&report, format)?;
    }
    Ok(ExitCode::SUCCESS)
}

fn project(adapter: ProjectAdapter) -> Result<ExitCode> {
    match adapter {
        ProjectAdapter::Kubernetes {
            spec,
            ir,
            out,
            format,
        } => project_kubernetes(&spec, &ir, out.as_deref(), format),
        ProjectAdapter::Openapi {
            path,
            ir,
            out,
            format,
        } => match (path, ir) {
            (Some(path), None) => {
                generate(&path, Some(Projection::OpenApi), out.as_deref(), format)
            }
            (None, Some(ir)) => project_openapi_interface(&ir, out.as_deref(), format),
            _ => bail!("exactly one of --path or --ir is required"),
        },
    }
}

fn project_openapi_interface(
    ir_path: &Path,
    out: Option<&Path>,
    format: Format,
) -> Result<ExitCode> {
    let text =
        fs::read_to_string(ir_path).with_context(|| format!("reading {}", ir_path.display()))?;
    let interface = ess_openapi::read_interface(&text).map_err(|refusals| {
        anyhow::anyhow!(
            "service-interface IR refused: {}",
            refusals
                .iter()
                .map(|refusal| format!("{}: {}", refusal.pointer, refusal.message))
                .collect::<Vec<_>>()
                .join("; ")
        )
    })?;
    let yaml = ess_openapi::project(&interface).map_err(|refusals| {
        anyhow::anyhow!(
            "OpenAPI projection refused: {}",
            refusals
                .iter()
                .map(|refusal| format!("{}: {}", refusal.pointer, refusal.message))
                .collect::<Vec<_>>()
                .join("; ")
        )
    })?;
    if let Some(path) = out {
        fs::write(path, &yaml).with_context(|| format!("writing {}", path.display()))?;
    }
    match (out, format) {
        (Some(path), Format::Text) => println!(
            "projected `{}` to OpenAPI 3.1 at {} without applying it",
            interface.service.name,
            path.display()
        ),
        (Some(_), _) => {}
        (None, Format::Text | Format::Yaml) => print!("{yaml}"),
        (None, Format::Json) => {
            let value: serde_json::Value = serde_yaml::from_str(&yaml)?;
            let mut json = serde_json::to_string_pretty(&value)?;
            json.push('\n');
            print!("{json}");
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn project_kubernetes(
    spec_path: &Path,
    ir_path: &Path,
    out: Option<&Path>,
    format: Format,
) -> Result<ExitCode> {
    let spec_text = fs::read_to_string(spec_path)?;
    let spec = infra_spec::read_spec(&spec_text)
        .map_err(|errors| anyhow::anyhow!("infrastructure intent refused: {errors}"))?;
    let ir = resolved_infrastructure(ir_path)?;
    let projection = infra_project::project(&spec, &ir);
    let artifacts = projection.artifacts();
    if let Some(root) = out {
        for (relative, contents) in &artifacts {
            let path = root.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, contents)?;
        }
    }
    match format {
        Format::Text => print!("{}", infra_project::projection_to_text(&projection)),
        Format::Json => print!("{}", projection.to_json()),
        Format::Yaml => render(&projection, Format::Yaml)?,
    }
    Ok(ExitCode::SUCCESS)
}

fn infra(command: InfraCommand) -> Result<ExitCode> {
    match command {
        InfraCommand::Diagnose { path, format } => {
            let ir = resolved_infrastructure(&path)?;
            let diagnosis = infra_analyze::diagnose(&ir);
            match format {
                Format::Text => {
                    for finding in &diagnosis.findings {
                        println!(
                            "{} {} {}: {}",
                            finding.code, finding.severity, finding.subject, finding.message
                        );
                    }
                    let (errors, warnings, infos) = diagnosis.counts();
                    println!("{errors} error(s), {warnings} warning(s), {infos} info(s)");
                }
                _ => render(&diagnosis, format)?,
            }
            Ok(ExitCode::SUCCESS)
        }
        InfraCommand::Graph {
            path,
            namespace,
            format,
        } => {
            let ir = resolved_infrastructure(&path)?;
            let mut graph = infra_analyze::InfraGraph::of(&ir);
            if let Some(namespace) = namespace.as_deref() {
                graph = graph.restricted_to(namespace);
            }
            match format {
                GraphFormat::Mermaid => print!("{}", graph.mermaid()),
                GraphFormat::Json => print!(
                    "{}",
                    infra_analyze::GraphDocument::of(&graph, &ir, namespace.as_deref()).to_json()
                ),
                GraphFormat::Yaml => render(
                    &infra_analyze::GraphDocument::of(&graph, &ir, namespace.as_deref()),
                    Format::Yaml,
                )?,
                GraphFormat::Dot => {
                    bail!("the infrastructure graph supports mermaid, json, and yaml")
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        InfraCommand::Diff { from, to, format } => {
            let before = resolved_infrastructure(&from)?;
            let after = resolved_infrastructure(&to)?;
            let report = infra_spec::drift(&before, &after).map_err(anyhow::Error::msg)?;
            match format {
                MachineFormat::Text => print!("{}", infra_spec::drift_to_text(&report)),
                MachineFormat::Json => print!("{}", report.to_json()),
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn every_command_and_argument_name_is_unambiguous() {
        Cli::command().debug_assert();
    }

    #[test]
    fn no_manifest_or_lockfile_depends_on_aep() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("the CLI is under the workspace root");
        let mut pending = vec![root.to_path_buf()];
        let mut violations = Vec::new();
        while let Some(directory) = pending.pop() {
            for entry in fs::read_dir(&directory).expect("workspace directory is readable") {
                let entry = entry.expect("workspace entry is readable");
                let path = entry.path();
                if entry.file_type().expect("entry type is readable").is_dir() {
                    if !matches!(entry.file_name().to_str(), Some(".git" | "target")) {
                        pending.push(path);
                    }
                    continue;
                }
                if !matches!(
                    entry.file_name().to_str(),
                    Some("Cargo.toml" | "Cargo.lock")
                ) {
                    continue;
                }
                let text = fs::read_to_string(&path).expect("manifest is UTF-8");
                for banned in ["aep-", "aep_"] {
                    if text.to_ascii_lowercase().contains(banned) {
                        violations.push(format!("{} contains `{banned}`", path.display()));
                    }
                }
            }
        }
        assert!(
            violations.is_empty(),
            "ESS dependency boundary was crossed:\n{}",
            violations.join("\n")
        );
    }
}
