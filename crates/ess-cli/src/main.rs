//! The `ess` command: a deterministic shell over the ESS libraries and explicit adapters.

mod load;
mod schema;

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
    /// Compile exact component surfaces into composition IR and generated clients.
    Compose {
        /// An `ess-composition/1` JSON or YAML document.
        #[arg(long)]
        path: PathBuf,
        /// A compiled ESS source, written `service-key=path`. Repeat for every import.
        #[arg(long = "service", value_name = "KEY=PATH", required = true)]
        services: Vec<ServiceInput>,
        /// Where to write canonical `ess-composition/1` IR.
        #[arg(long)]
        out: Option<PathBuf>,
        /// Where to write canonical `ess-client-plan/1`.
        #[arg(long)]
        client_plan_out: Option<PathBuf>,
        /// Root for the generated dependency-free Rust composition client.
        #[arg(long)]
        client_rust_out: Option<PathBuf>,
        /// Output rendering.
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
    /// Compile and inspect canonical build graphs.
    Build {
        #[command(subcommand)]
        command: BuildCommand,
    },
    /// Validate, compile, or document a physical realization of one exact ESS.
    Realization {
        #[command(subcommand)]
        command: RealizationCommand,
    },
    /// Compile semantic-component to deployable runtime mappings.
    Runtime {
        #[command(subcommand)]
        command: RuntimeCommand,
    },
    /// Verify immutable executor-produced releases.
    Release {
        #[command(subcommand)]
        command: ReleaseCommand,
    },
    /// Resolve generic product stacks from an offline release catalogue.
    Stack {
        #[command(subcommand)]
        command: StackCommand,
    },
    /// Compile and compare exact environment deployments.
    Deployment {
        #[command(subcommand)]
        command: DeploymentCommand,
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
    ///
    /// `--kind site` opens on the `README.md` beside the specification, where there is one, and
    /// takes any number of `--include` pages beside the generated ones — a plan board another tool
    /// rendered, a runbook. Both are markdown somebody wrote, read into the document and styled
    /// like every other page.
    Generate {
        #[command(flatten)]
        input: SpecLocation,
        #[arg(long, value_enum)]
        kind: Option<Projection>,
        /// A markdown page to publish beside the generated ones, written `<page-id>=<path>`.
        ///
        /// The id is where the page is filed and what links to it — `plan/board`. Repeat for more
        /// than one.
        #[arg(long, value_name = "PAGE=PATH")]
        include: Vec<String>,
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
    /// Validate adopter-owned JSON Schema contracts or project their TypeScript types.
    Schema {
        #[command(subcommand)]
        command: schema::SchemaCommand,
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

#[derive(Debug, Clone)]
struct ServiceInput {
    key: ess_composition::ServiceKey,
    path: PathBuf,
}

impl std::str::FromStr for ServiceInput {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (key, path) = value
            .split_once('=')
            .ok_or_else(|| "expected a service binding written `service-key=path`".to_owned())?;
        if path.is_empty() {
            return Err("a service binding path must not be empty".to_owned());
        }
        Ok(Self {
            key: ess_composition::ServiceKey::new(key).map_err(|error| error.to_string())?,
            path: PathBuf::from(path),
        })
    }
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
    /// The same pages as a browsable site: HTML, navigation, stylesheet and diagrams.
    Site,
    /// The `ess-docs/1` document the other two render, for a presentation layer of your own.
    #[value(name = "docs-ir")]
    DocsIr,
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
            Self::Site => "site",
            Self::DocsIr => "docs-ir",
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
    ///
    /// `ir` writes the canonical suite document to the file `--out` names. `go` writes a Go test
    /// package into the directory `--out` names — the runner, the evaluator and the suite — so an
    /// implementation in Go can be held to the specification by `go test` rather than by nothing,
    /// which is what a synthesized suite no runner can reach amounts to.
    Synthesize {
        #[command(flatten)]
        input: SpecPath,
        /// What to write the suite as.
        #[arg(long, value_enum, default_value_t = SuiteTarget::Ir)]
        target: SuiteTarget,
        /// A file for `--target ir`, a directory for `--target go`.
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

#[derive(Debug, Subcommand)]
enum RealizationCommand {
    /// Validate and resolve an `ess-realization/1` declaration.
    Validate(RealizationInput),
    /// Compile a declaration into canonical `ess-realization-ir/1` JSON.
    Compile {
        #[command(flatten)]
        input: RealizationInput,
        /// Where to write canonical JSON IR.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Generate the deterministic public-facing run-mode guide.
    Generate {
        #[command(flatten)]
        input: RealizationInput,
        /// Markdown file to write or compare.
        #[arg(long)]
        out: PathBuf,
        /// Compare with `--out` and fail on drift instead of writing.
        #[arg(long)]
        check: bool,
    },
}

#[derive(Debug, clap::Args)]
struct RealizationInput {
    /// An `ess-realization/1` JSON or YAML document.
    #[arg(long)]
    path: PathBuf,
    /// One ESS file or a directory containing `system.yaml`.
    #[arg(long = "spec")]
    specification: PathBuf,
    /// Output and diagnostic rendering.
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
}

/// What a synthesized suite is written as.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum SuiteTarget {
    /// The canonical `ess-conformance/1` document.
    Ir,
    /// A Go test package: the runner, the evaluator and the suite.
    Go,
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
    /// Project canonical build IR to `BuildKit` Dockerfile and Bake inputs.
    Buildkit {
        /// Compiled `ess-build-ir/1` JSON.
        #[arg(long)]
        ir: PathBuf,
        /// Output directory.
        #[arg(long)]
        out: PathBuf,
    },
    /// Project runtime IR into one configuration-neutral component Helm chart.
    Helm {
        /// Compiled `ess-runtime-ir/1` JSON.
        #[arg(long)]
        ir: PathBuf,
        /// Stable chart name.
        #[arg(long)]
        chart: ess_deployment::Identifier,
        /// Independent chart version.
        #[arg(long)]
        version: semver::Version,
        /// Output directory.
        #[arg(long)]
        out: PathBuf,
    },
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

#[derive(Debug, Subcommand)]
enum BuildCommand {
    /// Validate and compile `ess-build/1` to canonical `ess-build-ir/1`.
    Compile {
        #[arg(long)]
        path: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
    /// Render the validated build DAG as deterministic Mermaid source.
    Graph {
        #[arg(long)]
        path: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum RuntimeCommand {
    /// Compile `ess-runtime/1` against exact semantic, realization, and build inputs.
    Compile {
        /// Authored runtime JSON or YAML.
        #[arg(long)]
        path: PathBuf,
        /// ESS source file or directory.
        #[arg(long)]
        system: PathBuf,
        /// Authored `ess-realization/1` bound to the same ESS source.
        #[arg(long)]
        realization: PathBuf,
        /// Compiled `ess-build-ir/1` JSON.
        #[arg(long)]
        build_ir: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
}

#[derive(Debug, Subcommand)]
enum ReleaseCommand {
    /// Verify an `ess-release/1` against exact build and runtime IR.
    Verify {
        #[arg(long)]
        path: PathBuf,
        #[arg(long)]
        build_ir: PathBuf,
        #[arg(long)]
        runtime_ir: PathBuf,
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
}

#[derive(Debug, Subcommand)]
enum StackCommand {
    /// Resolve constraints to an exact `ess-stack-lock/1` using only the supplied catalogue.
    Resolve {
        #[arg(long)]
        path: PathBuf,
        #[arg(long)]
        catalog: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
    /// Validate that a generic stack resolves completely.
    Validate {
        #[arg(long)]
        path: PathBuf,
        #[arg(long)]
        catalog: PathBuf,
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
}

#[derive(Debug, Subcommand)]
enum DeploymentCommand {
    /// Bind an exact stack lock to an environment and emit `ess-deployment/1`.
    Compile {
        #[arg(long)]
        path: PathBuf,
        #[arg(long)]
        stack_lock: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
    /// Report which independent releases differ between two deployment IR documents.
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
        Command::Compose {
            path,
            services,
            out,
            client_plan_out,
            client_rust_out,
            format,
        } => compose(
            &path,
            &services,
            out.as_deref(),
            client_plan_out.as_deref(),
            client_rust_out.as_deref(),
            format,
        ),
        Command::Realization { command } => realization(&command),
        Command::Build { command } => build(command),
        Command::Runtime { command } => runtime(command),
        Command::Release { command } => release(command),
        Command::Stack { command } => stack(command),
        Command::Deployment { command } => deployment(command),
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
            include,
            out,
            format,
        } => generate(&input.path, kind, &include, out.as_deref(), format),
        Command::Synthesize {
            input,
            target,
            out,
            format,
        } => synthesize(&input.path, target, out.as_deref(), format),
        Command::Conform { command } => conform(command),
        Command::Import { adapter } => import(adapter),
        Command::Project { adapter } => project(adapter),
        Command::Schema { command } => schema::run(command),
        Command::Infra { command } => infra(command),
    }
}

fn realization(command: &RealizationCommand) -> Result<ExitCode> {
    let (input, out, generate) = match command {
        RealizationCommand::Validate(input) => (input, None, None),
        RealizationCommand::Compile { input, out } => (input, out.as_deref(), None),
        RealizationCommand::Generate { input, out, check } => {
            (input, Some(out.as_path()), Some(*check))
        }
    };
    let text = fs::read_to_string(&input.path)
        .with_context(|| format!("reading {}", input.path.display()))?;
    let specification = if input
        .path
        .extension()
        .is_some_and(|extension| extension == "json")
    {
        ess_realization::RealizationSpec::from_json(&text)
            .with_context(|| format!("reading {} as realization JSON", input.path.display()))?
    } else {
        ess_realization::RealizationSpec::from_yaml(&text)
            .with_context(|| format!("reading {} as realization YAML", input.path.display()))?
    };
    let Ok((ess, _)) = resolved(&input.specification, input.format)? else {
        return Ok(ExitCode::from(1));
    };
    let realization = match ess_realization::compile(&specification, &ess) {
        Ok(realization) => realization,
        Err(diagnostics) => {
            if matches!(input.format, Format::Text) {
                eprintln!("{} was refused:\n{diagnostics}", input.path.display());
            } else {
                render(&diagnostics, input.format)?;
            }
            return Ok(ExitCode::from(1));
        }
    };

    match command {
        RealizationCommand::Validate(_) => match input.format {
            Format::Text => println!(
                "{} — {} entrypoint(s), valid",
                realization.id(),
                realization.entrypoints().len()
            ),
            _ => render(&realization, input.format)?,
        },
        RealizationCommand::Compile { .. } => {
            let json = realization.to_canonical_json();
            if let Some(path) = out {
                fs::write(path, &json).with_context(|| format!("writing {}", path.display()))?;
            }
            match input.format {
                Format::Text => println!(
                    "{} — {} entrypoint(s), compiled{}",
                    realization.id(),
                    realization.entrypoints().len(),
                    out.map_or_else(String::new, |path| format!(" to {}", path.display()))
                ),
                Format::Json => print!("{json}"),
                Format::Yaml => render(&realization, Format::Yaml)?,
            }
        }
        RealizationCommand::Generate { .. } => {
            let path = out.expect("generate always supplies --out");
            let markdown = realization.to_markdown();
            if generate == Some(true) {
                let existing = fs::read_to_string(path)
                    .with_context(|| format!("reading {} for drift check", path.display()))?;
                if existing != markdown {
                    eprintln!(
                        "{} is stale; regenerate it with `ess realization generate`",
                        path.display()
                    );
                    return Ok(ExitCode::from(1));
                }
                println!("{} — current", path.display());
            } else {
                fs::write(path, markdown).with_context(|| format!("writing {}", path.display()))?;
                println!("{} — generated", path.display());
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn render<T: serde::Serialize>(value: &T, format: Format) -> Result<()> {
    match format {
        Format::Text | Format::Yaml => print!("{}", serde_yaml::to_string(value)?),
        Format::Json => println!("{}", serde_json::to_string_pretty(value)?),
    }
    Ok(())
}

fn read_document<T>(path: &Path) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    if path
        .extension()
        .is_some_and(|extension| extension == "json")
    {
        serde_json::from_str(&text).with_context(|| format!("parsing {} as JSON", path.display()))
    } else {
        serde_yaml::from_str(&text).with_context(|| format!("parsing {} as YAML", path.display()))
    }
}

fn write_canonical(path: Option<&Path>, json: &str) -> Result<()> {
    if let Some(path) = path {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, json).with_context(|| format!("writing {}", path.display()))?;
    }
    Ok(())
}

fn deployment_refusal(
    diagnostics: &ess_deployment::Diagnostics,
    format: Format,
) -> Result<ExitCode> {
    match format {
        Format::Text => eprintln!("lowering was refused:\n{diagnostics}"),
        Format::Json | Format::Yaml => render(&diagnostics.as_slice(), format)?,
    }
    Ok(ExitCode::from(1))
}

fn build(command: BuildCommand) -> Result<ExitCode> {
    match command {
        BuildCommand::Compile { path, out, format } => {
            let specification: ess_deployment::BuildSpec = read_document(&path)?;
            let ir = match ess_deployment::compile_build(&specification) {
                Ok(ir) => ir,
                Err(diagnostics) => return deployment_refusal(&diagnostics, format),
            };
            let json = ir.to_canonical_json();
            write_canonical(out.as_deref(), &json)?;
            match format {
                Format::Text => println!(
                    "{} — {} node(s), {} output(s), compiled{}",
                    ir.build(),
                    ir.nodes().len(),
                    ir.outputs().len(),
                    out.as_ref()
                        .map_or_else(String::new, |path| format!(" to {}", path.display()))
                ),
                Format::Json => print!("{json}"),
                Format::Yaml => render(&ir, format)?,
            }
            Ok(ExitCode::SUCCESS)
        }
        BuildCommand::Graph { path, out } => {
            let specification: ess_deployment::BuildSpec = read_document(&path)?;
            let ir = match ess_deployment::compile_build(&specification) {
                Ok(ir) => ir,
                Err(diagnostics) => return deployment_refusal(&diagnostics, Format::Text),
            };
            let graph = ess_deployment::project_build_mermaid(&ir);
            write_canonical(out.as_deref(), &graph)?;
            if out.is_none() {
                print!("{graph}");
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn runtime(command: RuntimeCommand) -> Result<ExitCode> {
    match command {
        RuntimeCommand::Compile {
            path,
            system,
            realization,
            build_ir,
            out,
            format,
        } => {
            let specification: ess_deployment::RuntimeSpec = read_document(&path)?;
            let build: ess_deployment::BuildIr = read_document(&build_ir)?;
            let Ok((semantic, _)) = resolved(&system, format)? else {
                return Ok(ExitCode::from(1));
            };
            let realization_text = fs::read_to_string(&realization)
                .with_context(|| format!("reading {}", realization.display()))?;
            let realization_specification = if realization
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                ess_realization::RealizationSpec::from_json(&realization_text).with_context(
                    || format!("reading {} as realization JSON", realization.display()),
                )?
            } else {
                ess_realization::RealizationSpec::from_yaml(&realization_text).with_context(
                    || format!("reading {} as realization YAML", realization.display()),
                )?
            };
            let physical = match ess_realization::compile(&realization_specification, &semantic) {
                Ok(realization) => realization,
                Err(diagnostics) => {
                    if matches!(format, Format::Text) {
                        eprintln!("{} was refused:\n{diagnostics}", realization.display());
                    } else {
                        render(&diagnostics, format)?;
                    }
                    return Ok(ExitCode::from(1));
                }
            };
            let ir =
                match ess_deployment::compile_runtime(&specification, &semantic, &physical, &build)
                {
                    Ok(ir) => ir,
                    Err(diagnostics) => return deployment_refusal(&diagnostics, format),
                };
            let json = ir.to_canonical_json();
            write_canonical(out.as_deref(), &json)?;
            match format {
                Format::Text => println!(
                    "{} — {} process(es), {} container role(s), {} workload(s), compiled{}",
                    ir.runtime(),
                    ir.processes().len(),
                    ir.containers().len(),
                    ir.workloads().len(),
                    out.as_ref()
                        .map_or_else(String::new, |path| format!(" to {}", path.display()))
                ),
                Format::Json => print!("{json}"),
                Format::Yaml => render(&ir, format)?,
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn release(command: ReleaseCommand) -> Result<ExitCode> {
    match command {
        ReleaseCommand::Verify {
            path,
            build_ir,
            runtime_ir,
            format,
        } => {
            let release: ess_deployment::ReleaseManifest = read_document(&path)?;
            let build: ess_deployment::BuildIr = read_document(&build_ir)?;
            let realization: ess_deployment::RuntimeIr = read_document(&runtime_ir)?;
            if let Err(diagnostics) = ess_deployment::verify_release(&release, &build, &realization)
            {
                return deployment_refusal(&diagnostics, format);
            }
            match format {
                Format::Text => println!(
                    "{} {} — {} immutable artifact(s), verified",
                    release.release_unit,
                    release.version,
                    release.artifacts.len()
                ),
                Format::Json | Format::Yaml => render(&release, format)?,
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn stack(command: StackCommand) -> Result<ExitCode> {
    let (path, catalog_path, out, format) = match command {
        StackCommand::Resolve {
            path,
            catalog,
            out,
            format,
        } => (path, catalog, out, format),
        StackCommand::Validate {
            path,
            catalog,
            format,
        } => (path, catalog, None, format),
    };
    let specification: ess_deployment::StackSpec = read_document(&path)?;
    let catalog: ess_deployment::ReleaseCatalog = read_document(&catalog_path)?;
    let lock = match ess_deployment::resolve_stack(&specification, &catalog) {
        Ok(lock) => lock,
        Err(diagnostics) => return deployment_refusal(&diagnostics, format),
    };
    let json = lock.to_canonical_json();
    write_canonical(out.as_deref(), &json)?;
    match format {
        Format::Text => println!(
            "{} — {} exact release(s), {} external system(s), resolved{}",
            lock.stack,
            lock.systems.len(),
            lock.external_systems.len(),
            out.as_ref()
                .map_or_else(String::new, |path| format!(" to {}", path.display()))
        ),
        Format::Json => print!("{json}"),
        Format::Yaml => render(&lock, format)?,
    }
    Ok(ExitCode::SUCCESS)
}

fn deployment(command: DeploymentCommand) -> Result<ExitCode> {
    match command {
        DeploymentCommand::Compile {
            path,
            stack_lock,
            out,
            format,
        } => {
            let environment: ess_deployment::EnvironmentSpec = read_document(&path)?;
            let lock: ess_deployment::StackLock = read_document(&stack_lock)?;
            let ir = match ess_deployment::compile_deployment(&environment, &lock) {
                Ok(ir) => ir,
                Err(diagnostics) => return deployment_refusal(&diagnostics, format),
            };
            let json = ir.to_canonical_json();
            write_canonical(out.as_deref(), &json)?;
            match format {
                Format::Text => println!(
                    "{} — {} independent release(s), compiled{}",
                    ir.environment,
                    ir.releases.len(),
                    out.as_ref()
                        .map_or_else(String::new, |path| format!(" to {}", path.display()))
                ),
                Format::Json => print!("{json}"),
                Format::Yaml => render(&ir, format)?,
            }
            Ok(ExitCode::SUCCESS)
        }
        DeploymentCommand::Diff { from, to, format } => {
            let from: ess_deployment::DeploymentIr = read_document(&from)?;
            let to: ess_deployment::DeploymentIr = read_document(&to)?;
            let added: Vec<_> = to
                .releases
                .keys()
                .filter(|service| !from.releases.contains_key(*service))
                .cloned()
                .collect();
            let removed: Vec<_> = from
                .releases
                .keys()
                .filter(|service| !to.releases.contains_key(*service))
                .cloned()
                .collect();
            let changed: Vec<_> = to
                .releases
                .iter()
                .filter(|(service, release)| from.releases.get(*service) != Some(*release))
                .filter(|(service, _)| from.releases.contains_key(*service))
                .map(|(service, _)| service.clone())
                .collect();
            let report = serde_json::json!({
                "format": "ess-deployment-diff/1",
                "from": from.digest(),
                "to": to.digest(),
                "added": added,
                "changed": changed,
                "removed": removed,
            });
            match format {
                MachineFormat::Text => {
                    println!("added: {}", report["added"]);
                    println!("changed: {}", report["changed"]);
                    println!("removed: {}", report["removed"]);
                }
                MachineFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
            }
            Ok(ExitCode::SUCCESS)
        }
    }
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

fn compose(
    path: &Path,
    services: &[ServiceInput],
    out: Option<&Path>,
    client_plan_out: Option<&Path>,
    client_rust_out: Option<&Path>,
    format: Format,
) -> Result<ExitCode> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let specification = if path
        .extension()
        .is_some_and(|extension| extension == "json")
    {
        ess_composition::CompositionSpec::from_json(&text)
            .with_context(|| format!("reading {} as composition JSON", path.display()))?
    } else {
        ess_composition::CompositionSpec::from_yaml(&text)
            .with_context(|| format!("reading {} as composition YAML", path.display()))?
    };

    let mut compiled = Vec::with_capacity(services.len());
    for service in services {
        let Ok((ir, _)) = resolved(&service.path, format)? else {
            return Ok(ExitCode::from(1));
        };
        compiled.push((service.key.clone(), ir));
    }
    let inputs = compiled
        .iter()
        .map(|(key, ir)| ess_composition::CompiledService::new(key, ir));
    let composition = match ess_composition::compile(&specification, inputs) {
        Ok(composition) => composition,
        Err(diagnostics) => {
            if matches!(format, Format::Text) {
                eprintln!("{} was refused:\n{diagnostics}", path.display());
            } else {
                render(&diagnostics.as_slice(), format)?;
            }
            return Ok(ExitCode::from(1));
        }
    };

    let composition_json = composition.to_canonical_json();
    if let Some(out) = out {
        fs::write(out, &composition_json).with_context(|| format!("writing {}", out.display()))?;
    }
    let client_plan = composition.client_plan();
    let client_plan_json = client_plan.to_canonical_json();
    if let Some(out) = client_plan_out {
        fs::write(out, &client_plan_json).with_context(|| format!("writing {}", out.display()))?;
    }
    let client_artifacts = client_plan.rust_artifacts();
    if let Some(root) = client_rust_out {
        for artifact in client_artifacts.values() {
            let path = root.join(artifact.path());
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, artifact.contents())
                .with_context(|| format!("writing {}", path.display()))?;
        }
    }

    match format {
        Format::Text => println!(
            "{} — {} exact component surface(s), {} semantic reference(s), compiled{}{}{}",
            composition.composition(),
            composition.services().len(),
            composition.references().len(),
            out.map_or_else(String::new, |path| format!(" to {}", path.display())),
            client_plan_out.map_or_else(String::new, |path| {
                format!("; client plan written to {}", path.display())
            }),
            client_rust_out.map_or_else(String::new, |path| {
                format!(
                    "; {} Rust client artifact(s) written to {}",
                    client_artifacts.len(),
                    path.display()
                )
            })
        ),
        Format::Json => print!("{composition_json}"),
        Format::Yaml => render(&composition, Format::Yaml)?,
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

/// The `README.md` beside a specification, as blocks, or nothing where there is none.
///
/// Absent is the common case and is not an error: most models have no prose beside them, and a
/// documentation build that failed for want of a file nobody promised would be a build nobody
/// runs. `--path` may name one file rather than a directory, in which case its parent is where a
/// README would sit.
fn front_page(path: &Path) -> Vec<ess_gen::document::Block> {
    let directory = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent().unwrap_or(Path::new(".")).to_path_buf()
    };
    fs::read_to_string(directory.join("README.md"))
        .map(|markdown| ess_gen::authored::titled(&markdown, "").1)
        .unwrap_or_default()
}

/// One `--include` argument, read into the page it names.
///
/// The id is the caller's: it is where the page is filed and what a link to it says, and deriving
/// it from a filename would make `board.md` and `plan/board.md` two different sites.
fn included(argument: &str) -> Result<(String, String)> {
    let (id, path) = argument.split_once('=').with_context(|| {
        format!("`--include {argument}` is not `<page-id>=<path>`, such as `plan/board=board.md`")
    })?;
    let markdown =
        fs::read_to_string(path).with_context(|| format!("reading the included page {path}"))?;
    Ok((id.to_owned(), markdown))
}

fn generate(
    path: &Path,
    kind: Option<Projection>,
    include: &[String],
    out: Option<&Path>,
    format: Format,
) -> Result<ExitCode> {
    let Ok((ir, _)) = resolved(path, format)? else {
        return Ok(ExitCode::from(1));
    };
    let artifacts = match kind {
        // Not a `Generator`: it writes the document the generators read, so it has no rendering of
        // its own and nothing to stamp per page beyond what the document already carries.
        Some(Projection::DocsIr) => {
            let mint = ess_gen::provenance::ProvenanceMint::new(&ir);
            let document = ess_gen::docs::document(&ir, &mint);
            let json = serde_json::to_string_pretty(&document)
                .context("the document does not serialise")?;
            [(
                "docs-ir/document.json".to_owned(),
                ess_gen::Artifact::new("docs-ir/document.json", format!("{json}\n")),
            )]
            .into_iter()
            .collect()
        }
        // The site is the one projection an adopter contributes to, so it is built here rather
        // than fetched by name: a `Generator` is handed the model and nothing else.
        Some(Projection::Site) => {
            let mint = ess_gen::provenance::ProvenanceMint::new(&ir);
            let mut document = ess_gen::docs::document(&ir, &mint);
            for argument in include {
                let (id, markdown) = included(argument)?;
                let (title, blocks) = ess_gen::authored::titled(&markdown, &id);
                document.pages.push(ess_gen::document::Page {
                    id: ess_gen::document::PageId(id),
                    title,
                    about: None,
                    provenance: mint.whole(),
                    blocks,
                });
            }
            let site = ess_gen::html::Site::new().with_front_page(front_page(path));
            site.render(&document, &mint.whole().provenance)
                .into_iter()
                .map(|artifact| (format!("site/{}", artifact.path), artifact))
                .collect()
        }
        Some(kind) => {
            let generator = ess_gen::generator(kind.name()).context("projection unavailable")?;
            ess_gen::artifact::run(generator.as_ref(), &ir)?
                .into_iter()
                .collect()
        }
        None => ess_gen::generate_all(&ir)?,
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
        ConformCommand::Synthesize { input, target, out } => {
            let Ok((ir, _)) = resolved(&input.path, input.format)? else {
                return Ok(ExitCode::from(1));
            };
            let synthesis = ess_conformance::synthesize(&ir);
            let json = synthesis.suite.to_canonical_json();

            let written = match (target, &out) {
                (SuiteTarget::Ir, Some(out)) => {
                    fs::write(out, &json).with_context(|| format!("writing {}", out.display()))?;
                    Some(format!("written to {}", out.display()))
                }
                (SuiteTarget::Go, Some(out)) => {
                    let files = ess_conformance::go::emit(&synthesis.suite);
                    for file in &files {
                        let path = out.join(&file.path);
                        if let Some(parent) = path.parent() {
                            fs::create_dir_all(parent)
                                .with_context(|| format!("creating {}", parent.display()))?;
                        }
                        fs::write(&path, &file.contents)
                            .with_context(|| format!("writing {}", path.display()))?;
                    }
                    Some(format!(
                        "{} file(s) written to {}",
                        files.len(),
                        out.display()
                    ))
                }
                // Refusing to write without `--out` is the rule every generating verb here keeps:
                // a verb that scatters a tree over a working directory the first time somebody
                // tries it is a verb nobody tries twice.
                (_, None) => None,
            };

            match input.format {
                Format::Text => {
                    // Printed, not counted. A refusal is a construct the specification declares and
                    // the suite does not check, and a number saying there are thirty-one of them
                    // tells a reader that something is unchecked without telling them what — which
                    // is the same "generated tests are green" failure the refusal list exists to
                    // rule out, one step further back.
                    for refusal in &synthesis.refusals {
                        println!("refused: {refusal}");
                    }
                    println!(
                        "{} scenario(s), {} refusal(s), {}",
                        synthesis.suite.len(),
                        synthesis.refusals.len(),
                        written.unwrap_or_else(|| "nothing written".to_owned())
                    );
                }
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
        ProjectAdapter::Buildkit { ir, out } => {
            let build: ess_deployment::BuildIr = read_document(&ir)?;
            let projection = ess_deployment::project_buildkit(&build);
            write_projection_files(&out, projection.files())?;
            println!(
                "{} BuildKit file(s) projected to {} without executing them",
                projection.files().len(),
                out.display()
            );
            Ok(ExitCode::SUCCESS)
        }
        ProjectAdapter::Helm {
            ir,
            chart,
            version,
            out,
        } => {
            let realization: ess_deployment::RuntimeIr = read_document(&ir)?;
            let projection = ess_deployment::project_helm(&realization, &chart, &version);
            write_projection_files(&out, projection.files())?;
            println!(
                "{} Helm chart file(s) projected to {} without applying them",
                projection.files().len(),
                out.display()
            );
            Ok(ExitCode::SUCCESS)
        }
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
            (Some(path), None) => generate(
                &path,
                Some(Projection::OpenApi),
                &[],
                out.as_deref(),
                format,
            ),
            (None, Some(ir)) => project_openapi_interface(&ir, out.as_deref(), format),
            _ => bail!("exactly one of --path or --ir is required"),
        },
    }
}

fn write_projection_files(
    root: &Path,
    files: &std::collections::BTreeMap<String, String>,
) -> Result<()> {
    for (relative, contents) in files {
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, contents).with_context(|| format!("writing {}", path.display()))?;
    }
    Ok(())
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
