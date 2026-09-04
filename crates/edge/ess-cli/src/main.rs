//! The `ess` command: a deterministic shell over the ESS libraries and explicit adapters.

mod load;
mod schema;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, ExitCode};

use anyhow::{bail, Context, Result};
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum};
use ess_compiler::EssIr;
use ess_gen::graph::SystemGraph;

/// Executable System Specification tooling.
#[derive(Debug, Parser)]
#[command(name = "ess", version, about, disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// The four areas the first level of `ess` is made of.
///
/// One per `crates/<area>/` directory: the command surface says what the crate tree says.
const AREAS: &[&str] = &["specify", "generate", "verify", "infra"];

#[derive(Debug, Subcommand)]
enum Command {
    /// Author a system, resolve what it says, and inspect the result.
    Specify {
        #[command(subcommand)]
        command: SpecifyCommand,
    },
    /// Turn a resolved system into artifacts and cross explicit delivery executor boundaries.
    ///
    /// The options below belong to the `generate` verb, which is spelled either way:
    /// `ess generate --path …` is `ess generate generate --path …`. They cannot be written beside
    /// one of the other verbs, which would say two things at once.
    // `ess generate --path X synthesize` names a specification for a verb that takes one and then
    // runs it against `.`. The flat surface refused that before the areas existed — `--path` was
    // no argument of `ess synthesize`'s parent — and this keeps the refusal, exit 2 and all.
    #[command(args_conflicts_with_subcommands = true)]
    Generate {
        #[command(subcommand)]
        command: Option<GenerateAreaCommand>,
        #[command(flatten)]
        direct: GenerateArgs,
    },
    /// Hold an implementation, or a later revision, to what a system says.
    Verify {
        #[command(subcommand)]
        command: VerifyCommand,
    },
    /// Read an observed cluster, and import a concrete source through a declared adapter.
    Infra {
        #[command(subcommand)]
        command: InfraAreaCommand,
    },
    /// The flat spellings of the `specify` verbs. Hidden by [`command`], never deprecated.
    #[command(flatten)]
    FlatSpecify(SpecifyCommand),
    /// The flat spellings of the `generate` verbs other than `generate` itself, hidden likewise.
    #[command(flatten)]
    FlatGenerate(GenerateCommand),
    /// The flat spellings of the `verify` verbs, hidden likewise.
    #[command(flatten)]
    FlatVerify(VerifyCommand),
    /// The flat spelling of `ess infra import`, hidden likewise.
    #[command(flatten)]
    FlatImport(ImportCommand),
}

/// `ess specify`: an authored system becomes a validated, resolved IR — `crates/specify/`.
#[derive(Debug, Subcommand)]
enum SpecifyCommand {
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
}

/// `ess generate`: what the area offers, which is the `generate` verb and its siblings.
///
/// The verb and the area share a name, so the verb sits below itself — `ess generate generate` —
/// and its flat spelling is the area with no subcommand at all, `ess generate --path …`. Both
/// reach [`generate_projections`], which is what makes them the same bytes.
#[derive(Debug, Subcommand)]
enum GenerateAreaCommand {
    /// Generate deterministic documentation, schemas, and interface contracts.
    ///
    /// `--kind site` opens on the `README.md` beside the specification, where there is one, and
    /// takes any number of `--include` pages beside the generated ones — a plan board another tool
    /// rendered, a runbook. Both are markdown somebody wrote, read into the document and styled
    /// like every other page.
    Generate(GenerateArgs),
    /// Everything else the area offers, which is also spelled flat at the top level.
    #[command(flatten)]
    Other(GenerateCommand),
}

/// What `ess generate` generates, from what, and where to put it.
#[derive(Debug, clap::Args)]
struct GenerateArgs {
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
}

/// `ess generate`: IR becomes artifacts; explicit executor verbs deliver them — `crates/generate/`.
#[derive(Debug, Subcommand)]
enum GenerateCommand {
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
    /// Compile and inspect canonical build graphs.
    Build {
        #[command(subcommand)]
        command: BuildCommand,
    },
    /// Compile a repository-owned independently releasable component descriptor.
    Component {
        #[command(subcommand)]
        command: ComponentCommand,
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
}

/// `ess verify`: an implementation held to the specification — `crates/verify/`.
#[derive(Debug, Subcommand)]
enum VerifyCommand {
    /// Generate or execute a semantic conformance suite.
    Conform {
        #[command(subcommand)]
        command: ConformCommand,
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
}

/// `ess infra`: the observed cluster, and the adapters that read a concrete source.
///
/// The verb and the area share a name here too, so `ess infra infra diagnose` is the area path and
/// `ess infra diagnose` is the flat spelling — the same [`InfraCommand`], mounted twice, with the
/// flat one hidden by [`command`].
#[derive(Debug, Subcommand)]
enum InfraAreaCommand {
    /// Inspect and compare sanitized infrastructure IR.
    Infra {
        #[command(subcommand)]
        command: InfraCommand,
    },
    /// Import a concrete source through a declared adapter.
    #[command(flatten)]
    Import(ImportCommand),
    /// The flat spellings of the `infra` verb's own operations.
    #[command(flatten)]
    Flat(InfraCommand),
}

/// The `import` verb, in an enum of its own so it can be mounted under `infra` and at the top.
#[derive(Debug, Subcommand)]
enum ImportCommand {
    /// Import a concrete source through a declared adapter.
    Import {
        #[command(subcommand)]
        adapter: ImportAdapter,
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
    Clap,
}

impl SynthesisTarget {
    fn target(self) -> ess_synth::Target {
        match self {
            Self::Rust => ess_synth::Target::Rust,
            Self::Go => ess_synth::Target::Go,
            Self::Web => ess_synth::Target::Web,
            Self::Clap => ess_synth::Target::Clap,
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
        /// Hold one component to the specification rather than the whole system.
        ///
        /// Keeps only the scenarios whose every command, event and view this component accepts,
        /// publishes or owns, and lists the rest with what they need — they belong in the suite of
        /// the component that realises it. An implementation of one component answers
        /// `ErrUnsupported` to the other's scenarios, and a run with skips in it cannot say it
        /// passed.
        #[arg(long)]
        component: Option<String>,
        /// The `ess-scenario/1` documents to compile beside the generated scenarios.
        ///
        /// A directory or one file. Defaults to `scenarios/` beside the specification, so a suite
        /// holds everything the specification's own directory says — the obligations it derives and
        /// the checks somebody wrote for what it cannot.
        #[arg(long)]
        scenarios: Option<PathBuf>,
    },
    /// Compile the scenarios an author wrote, and nothing the specification obliges.
    ///
    /// The authoring surface on its own: every command, actor, outcome, event, error, view, entity,
    /// field, enum variant and lifecycle state a scenario names is resolved against the model, and
    /// a name it does not declare is refused here rather than at the first run that reaches it.
    Author {
        #[command(flatten)]
        input: SpecPath,
        /// A directory of `ess-scenario/1` documents, or one file.
        #[arg(long)]
        scenarios: Option<PathBuf>,
        /// Where to write the compiled `ess-conformance/2` suite.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Run a generated or committed suite against a built-in reference implementation.
    Run {
        #[arg(long, default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        suite: Option<PathBuf>,
        /// The `ess-scenario/1` documents to run beside the generated scenarios.
        ///
        /// Read only where the suite is synthesized rather than named by `--suite`: a committed
        /// suite already holds whatever was compiled into it.
        #[arg(long)]
        scenarios: Option<PathBuf>,
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
    /// Compile, project, and execute a build through Docker Buildx Bake.
    Execute {
        /// Authored `ess-build/1` JSON or YAML.
        #[arg(long)]
        path: PathBuf,
        /// Repository root used as the `BuildKit` context.
        #[arg(long, default_value = ".")]
        workdir: PathBuf,
        /// Directory receiving the reviewable `BuildKit` projection.
        #[arg(long)]
        projection_out: PathBuf,
        /// Optional Bake target. Repeat to build a subset; omitted builds the default group.
        #[arg(long = "target")]
        targets: Vec<String>,
        /// Bake override such as `app.tags=registry.example/app:version`. Repeat as needed.
        #[arg(long = "set")]
        settings: Vec<String>,
        /// Push OCI outputs to their configured registries.
        #[arg(long, conflicts_with = "load")]
        push: bool,
        /// Load a single-platform OCI output into the local image store.
        #[arg(long, conflicts_with = "push")]
        load: bool,
    },
}

#[derive(Debug, Subcommand)]
enum ComponentCommand {
    /// Validate and compile `ess-component/1` to canonical `ess-component-ir/1`.
    Compile {
        #[arg(long)]
        path: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
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
    /// Combine a component and its verified runtime/chart releases into an OCI payload.
    Bundle {
        #[arg(long)]
        component_ir: PathBuf,
        #[arg(long)]
        build_ir: PathBuf,
        #[arg(long)]
        runtime_ir: PathBuf,
        /// Executor-produced `ess-release/1` manifest. Supply once per release unit.
        #[arg(long = "release", required = true)]
        releases: Vec<PathBuf>,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
    /// Revalidate an `ess-release-bundle/1` received from an untrusted boundary.
    VerifyBundle {
        #[arg(long)]
        path: PathBuf,
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
    /// Publish a verified release bundle as an OCI artifact through ORAS.
    Publish {
        #[arg(long)]
        path: PathBuf,
        /// Tagged OCI destination. Consumers must use the digest printed by this command.
        #[arg(long)]
        to: String,
    },
    /// Fetch, verify, and cache a release bundle from a digest-pinned OCI reference.
    Fetch {
        /// OCI source ending in `@sha256:<64 lowercase hex characters>`.
        #[arg(long)]
        from: String,
        /// Content-addressed cache root.
        #[arg(long)]
        cache: PathBuf,
        /// Optional copy of the verified canonical bundle.
        #[arg(long)]
        out: Option<PathBuf>,
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
    /// Reconcile only changed independent Helm releases through ORAS and Helm.
    Reconcile {
        /// Desired canonical `ess-deployment/1` document.
        #[arg(long)]
        path: PathBuf,
        /// Previously applied deployment IR. Omit for a first deployment.
        #[arg(long)]
        current: Option<PathBuf>,
        /// Cache root for digest-pinned chart artifacts.
        #[arg(long)]
        cache: PathBuf,
        /// Permit uninstalling releases absent from the desired deployment.
        #[arg(long)]
        allow_removals: bool,
        /// Render the affected set without contacting OCI, Helm, or Kubernetes.
        #[arg(long)]
        dry_run: bool,
        /// Helm wait timeout.
        #[arg(long, default_value = "5m")]
        timeout: String,
    },
}

/// The `ess` command tree as it is parsed and as `--help` renders it.
///
/// The derive mounts every verb twice from one definition — once under its area and once as the
/// flat spelling a pinned caller already uses — so the two cannot drift apart in arguments or in
/// what they run. What tells them apart is this function, and only this function: a command
/// `--help` offers is one of the four areas or something under one, and every other spelling that
/// still parses is a flat one, hidden rather than deprecated. `main` and the tests both ask here,
/// so what is checked is what an adopter runs.
///
/// Only whole commands are hidden. An **argument** is never hidden: `ess generate` carries the
/// `generate` verb's five options so its flat spelling parses, and a help or a usage line that
/// left them out would describe a command that does not exist — which is how a mistyped `--kind`
/// came to be answered with a usage line saying `ess generate` takes no options at all.
fn command() -> clap::Command {
    let command = <Cli as CommandFactory>::command();
    let flat: Vec<String> = command
        .get_subcommands()
        .map(|sub| sub.get_name().to_owned())
        .filter(|name| !AREAS.contains(&name.as_str()))
        .collect();
    let command = flat.into_iter().fold(command, |command, name| {
        command.mut_subcommand(name, |sub| sub.hide(true))
    });
    // `mut_subcommand` moves what it touches to the end of the list, so the areas are ordered
    // here rather than left to the order they were declared in.
    let command = AREAS
        .iter()
        .enumerate()
        .fold(command, |command, (position, area)| {
            command.mut_subcommand(area, |sub| sub.display_order(position))
        });
    let command = command.mut_subcommand("infra", hide_infra_flat_spellings);
    command.mut_subcommand("generate", state_generate_usage)
}

/// `ess generate` is a verb with options and an area with subcommands, and its usage says both.
///
/// clap renders that pair correctly for `--help` and not for an error: a mistyped `--kidn` is
/// answered with `Usage: ess generate --kind <KIND>`, which reads as the only thing the command
/// takes. The string here is the one clap writes for `--help`, said once so every refusal of this
/// command uses it too.
fn state_generate_usage(area: clap::Command) -> clap::Command {
    area.override_usage("ess generate [OPTIONS]\n       ess generate <COMMAND>")
}

/// `ess infra diagnose` is the flat spelling of `ess infra infra diagnose`: one level up, hidden.
///
/// Which names those are is asked of the `infra` verb itself rather than written down, so a
/// fourth operation under it is hidden the day it is added.
fn hide_infra_flat_spellings(area: clap::Command) -> clap::Command {
    let flat: Vec<String> = area
        .find_subcommand("infra")
        .expect("the infra area carries the infra verb")
        .get_subcommands()
        .map(|sub| sub.get_name().to_owned())
        .collect();
    flat.into_iter().fold(area, |area, name| {
        area.mut_subcommand(name, |sub| sub.hide(true))
    })
}

fn main() -> ExitCode {
    let mut parser = command();
    let matches = parser.clone().get_matches();
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(cli) => cli,
        Err(error) => error.format(&mut parser).exit(),
    };
    match run(cli) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::from(1)
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode> {
    match cli.command {
        Command::Specify { command } | Command::FlatSpecify(command) => specify_area(command),
        Command::Generate { command, direct } => match command {
            None => generate_projections(&direct),
            Some(GenerateAreaCommand::Generate(spelled)) => generate_projections(&spelled),
            Some(GenerateAreaCommand::Other(command)) => generate_area(command),
        },
        Command::FlatGenerate(command) => generate_area(command),
        Command::Verify { command } | Command::FlatVerify(command) => verify_area(command),
        Command::Infra { command } => infra_area(command),
        Command::FlatImport(command) => import_area(command),
    }
}

/// `ess specify …`, and the same verbs spelled flat.
fn specify_area(command: SpecifyCommand) -> Result<ExitCode> {
    match command {
        SpecifyCommand::Validate(input) => validate(&input.path, input.format),
        SpecifyCommand::Compile { input, out } => {
            compile(&input.path, out.as_deref(), input.format)
        }
        SpecifyCommand::Compose {
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
        SpecifyCommand::Inspect { input, name } => inspect(&input.path, &name, input.format),
        SpecifyCommand::Graph { input, format } => graph(&input.path, format),
        SpecifyCommand::Realization { command } => realization(&command),
        SpecifyCommand::Runtime { command } => runtime(command),
    }
}

/// `ess generate …` other than the `generate` verb, and the same verbs spelled flat.
fn generate_area(command: GenerateCommand) -> Result<ExitCode> {
    match command {
        GenerateCommand::Synthesize {
            input,
            target,
            out,
            format,
        } => synthesize(&input.path, target, out.as_deref(), format),
        GenerateCommand::Project { adapter } => project(adapter),
        GenerateCommand::Schema { command } => schema::run(command),
        GenerateCommand::Build { command } => build(command),
        GenerateCommand::Component { command } => component(command),
        GenerateCommand::Release { command } => release(command),
        GenerateCommand::Stack { command } => stack(command),
        GenerateCommand::Deployment { command } => deployment(command),
    }
}

/// `ess generate generate …` and `ess generate …`: one verb, two spellings, one call.
fn generate_projections(arguments: &GenerateArgs) -> Result<ExitCode> {
    generate(
        &arguments.input.path,
        arguments.kind,
        &arguments.include,
        arguments.out.as_deref(),
        arguments.format,
    )
}

/// `ess verify …`, and the same verbs spelled flat.
fn verify_area(command: VerifyCommand) -> Result<ExitCode> {
    match command {
        VerifyCommand::Conform { command } => conform(command),
        VerifyCommand::Diff { from, to, format } => diff(&from, &to, format),
        VerifyCommand::Impact {
            from,
            to,
            suite,
            format,
        } => impact(&from, &to, suite.as_deref(), format),
    }
}

/// `ess infra …`, including `ess infra infra …` and the flat spelling of both verbs.
fn infra_area(command: InfraAreaCommand) -> Result<ExitCode> {
    match command {
        InfraAreaCommand::Infra { command } | InfraAreaCommand::Flat(command) => infra(command),
        InfraAreaCommand::Import(command) => import_area(command),
    }
}

/// `ess infra import …`, and `ess import …`.
fn import_area(command: ImportCommand) -> Result<ExitCode> {
    match command {
        ImportCommand::Import { adapter } => import(adapter),
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
        BuildCommand::Execute {
            path,
            workdir,
            projection_out,
            targets,
            settings,
            push,
            load,
        } => {
            let specification: ess_deployment::BuildSpec = read_document(&path)?;
            let ir = match ess_deployment::compile_build(&specification) {
                Ok(ir) => ir,
                Err(diagnostics) => return deployment_refusal(&diagnostics, Format::Text),
            };
            let projection = ess_deployment::project_buildkit(&ir);
            write_projection_files(&projection_out, projection.files())?;

            let workdir = fs::canonicalize(&workdir)
                .with_context(|| format!("resolving build context {}", workdir.display()))?;
            let projection_out = fs::canonicalize(&projection_out).with_context(|| {
                format!("resolving BuildKit projection {}", projection_out.display())
            })?;
            let dockerfile = projection_out.join("Dockerfile.ess");
            let bake_file = projection_out.join("docker-bake.hcl");
            let mut process = ProcessCommand::new("docker");
            process
                .current_dir(&workdir)
                .args(["buildx", "bake", "--file"])
                .arg(&bake_file)
                .arg("--set")
                .arg(format!("*.context={}", workdir.display()))
                .arg("--set")
                .arg(format!("*.dockerfile={}", dockerfile.display()));
            for setting in settings {
                process.arg("--set").arg(setting);
            }
            if push {
                process.arg("--push");
            } else if load {
                process.arg("--load");
            }
            process.args(targets);
            run_external(&mut process, "Docker Buildx Bake")?;
            println!(
                "{} — executed from {}",
                ir.build(),
                projection_out.display()
            );
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn component(command: ComponentCommand) -> Result<ExitCode> {
    match command {
        ComponentCommand::Compile { path, out, format } => {
            let specification: ess_deployment::ComponentSpec = read_document(&path)?;
            let ir = match ess_deployment::compile_component(&specification) {
                Ok(ir) => ir,
                Err(diagnostics) => return deployment_refusal(&diagnostics, format),
            };
            let json = ir.to_canonical_json();
            write_canonical(out.as_deref(), &json)?;
            match format {
                Format::Text => println!(
                    "{} — runtime {} and chart {}, compiled{}",
                    ir.component(),
                    ir.release_units().runtime,
                    ir.release_units().chart,
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

#[allow(clippy::too_many_lines)]
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
        ReleaseCommand::Bundle {
            component_ir,
            build_ir,
            runtime_ir,
            releases,
            out,
            format,
        } => {
            let component: ess_deployment::ComponentIr = read_document(&component_ir)?;
            let build: ess_deployment::BuildIr = read_document(&build_ir)?;
            let runtime: ess_deployment::RuntimeIr = read_document(&runtime_ir)?;
            let releases = releases
                .iter()
                .map(|path| read_document(path))
                .collect::<Result<Vec<ess_deployment::ReleaseManifest>>>()?;
            let bundle = match ess_deployment::bundle_release(component, build, runtime, releases) {
                Ok(bundle) => bundle,
                Err(diagnostics) => return deployment_refusal(&diagnostics, format),
            };
            let json = bundle.to_canonical_json();
            write_canonical(out.as_deref(), &json)?;
            match format {
                Format::Text => println!(
                    "{} — {} release unit(s), bundled as {}{}",
                    bundle.component.component(),
                    bundle.releases.len(),
                    bundle.digest(),
                    out.as_ref()
                        .map_or_else(String::new, |path| format!(" at {}", path.display()))
                ),
                Format::Json => print!("{json}"),
                Format::Yaml => render(&bundle, format)?,
            }
            Ok(ExitCode::SUCCESS)
        }
        ReleaseCommand::VerifyBundle { path, format } => {
            let bundle = verified_bundle(&path, true)?;
            match format {
                Format::Text => println!(
                    "{} — {} release unit(s), verified as {}",
                    bundle.component.component(),
                    bundle.releases.len(),
                    bundle.digest()
                ),
                Format::Json | Format::Yaml => render(&bundle, format)?,
            }
            Ok(ExitCode::SUCCESS)
        }
        ReleaseCommand::Publish { path, to } => {
            let bundle = verified_bundle(&path, true)?;
            if to.contains('@') {
                bail!("OCI publication destination must be a tag, not a digest reference");
            }
            let staging = TemporaryDirectory::create("ess-release-publish")?;
            let payload = staging.path().join("ess-release-bundle.json");
            fs::write(&payload, bundle.to_canonical_json())
                .with_context(|| format!("staging {}", payload.display()))?;
            let output = ProcessCommand::new("oras")
                .current_dir(staging.path())
                .args([
                    "push",
                    "--no-tty",
                    "--artifact-type",
                    "application/vnd.beyond10x.ess.release-bundle.v1",
                    "--format",
                    "go-template={{.digest}}",
                ])
                .arg(&to)
                .arg("ess-release-bundle.json:application/vnd.beyond10x.ess.release-bundle.v1+json")
                .output()
                .context("starting ORAS")?;
            if !output.status.success() {
                bail!(
                    "ORAS publication failed: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                );
            }
            let digest = String::from_utf8(output.stdout)
                .context("ORAS returned a non-UTF-8 manifest digest")?;
            let digest = digest.trim();
            ess_deployment::Digest::new(digest)
                .with_context(|| format!("ORAS returned invalid manifest digest {digest:?}"))?;
            println!("{to} — published at {digest}");
            Ok(ExitCode::SUCCESS)
        }
        ReleaseCommand::Fetch { from, cache, out } => {
            let digest = pinned_oci_digest(&from)?;
            let cache_path = cache
                .join("sha256")
                .join(digest.as_str().trim_start_matches("sha256:"))
                .join("ess-release-bundle.json");
            let bundle = if cache_path.is_file() {
                verified_bundle(&cache_path, true).with_context(|| {
                    format!(
                        "cached OCI release bundle {} is invalid",
                        cache_path.display()
                    )
                })?
            } else {
                let staging = TemporaryDirectory::create("ess-release-fetch")?;
                let mut process = ProcessCommand::new("oras");
                process
                    .args(["pull", "--no-tty", "--output"])
                    .arg(staging.path())
                    .arg(&from);
                run_external(&mut process, "ORAS pull")?;
                let payload = only_regular_file(staging.path())?;
                let bundle = verified_bundle(&payload, true)
                    .with_context(|| format!("verifying OCI payload from {from}"))?;
                write_canonical(Some(&cache_path), &bundle.to_canonical_json())?;
                bundle
            };
            if let Some(out) = out.as_deref() {
                write_canonical(Some(out), &bundle.to_canonical_json())?;
            }
            println!(
                "{} — verified {}{}",
                from,
                bundle.digest(),
                out.as_ref()
                    .map_or_else(String::new, |path| format!(" at {}", path.display()))
            );
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

#[allow(clippy::too_many_lines)]
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
        DeploymentCommand::Reconcile {
            path,
            current,
            cache,
            allow_removals,
            dry_run,
            timeout,
        } => {
            let desired: ess_deployment::DeploymentIr = read_document(&path)?;
            let current = current
                .as_deref()
                .map(read_document::<ess_deployment::DeploymentIr>)
                .transpose()?;
            if current
                .as_ref()
                .is_some_and(|current| current.cluster != desired.cluster)
            {
                bail!(
                    "refusing to reconcile across cluster changes; retire the old deployment explicitly"
                );
            }

            let affected = desired
                .rollout_order
                .iter()
                .filter(|service| {
                    current
                        .as_ref()
                        .and_then(|state| state.releases.get(*service))
                        != desired.releases.get(*service)
                })
                .cloned()
                .collect::<Vec<_>>();
            let removed = current
                .as_ref()
                .map(|state| {
                    state
                        .rollout_order
                        .iter()
                        .rev()
                        .filter(|service| !desired.releases.contains_key(*service))
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if !removed.is_empty() && !allow_removals {
                bail!(
                    "deployment removes {}; rerun with --allow-removals after reviewing the retirement set",
                    removed
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }

            if dry_run {
                println!(
                    "apply: {}",
                    affected
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                println!(
                    "remove: {}",
                    removed
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                return Ok(ExitCode::SUCCESS);
            }

            for service in &affected {
                let release = desired
                    .releases
                    .get(service)
                    .expect("rollout order refers to a release");
                reconcile_release(&desired.cluster, release, &cache, &timeout)?;
            }
            if let Some(current) = &current {
                for service in &removed {
                    let release = current
                        .releases
                        .get(service)
                        .expect("current rollout order refers to a release");
                    let mut process = ProcessCommand::new("helm");
                    process
                        .args(["uninstall", &release.release_name, "--namespace"])
                        .arg(&release.namespace)
                        .args(["--kube-context", &current.cluster]);
                    run_external(&mut process, "Helm uninstall")?;
                }
            }
            println!(
                "{} — {} release(s) reconciled, {} removed",
                desired.environment,
                affected.len(),
                removed.len()
            );
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
        ConformCommand::Synthesize {
            input,
            target,
            out,
            component,
            scenarios,
        } => synthesize_suite(
            &input,
            target,
            out.as_deref(),
            component.as_deref(),
            scenarios.as_deref(),
        ),
        ConformCommand::Author {
            input,
            scenarios,
            out,
        } => author_suite(&input, scenarios.as_deref(), out.as_deref()),
        ConformCommand::Run {
            path,
            suite,
            scenarios,
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
                let mut suite = ess_conformance::synthesize(&ir).suite;
                let authoring = ess_conformance::authored::compile(
                    &ir,
                    &authored_sources(scenarios.as_deref())?,
                );
                for refusal in &authoring.refusals {
                    eprintln!("{refusal}");
                }
                if !authoring.is_complete() {
                    return Ok(ExitCode::from(1));
                }
                for (id, scenario) in authoring.scenarios {
                    if let Err(id) = suite.insert(id, scenario) {
                        bail!("`{id}` is already in the suite");
                    }
                }
                suite
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

/// `ess conform synthesize`: the suite the specification obliges, for the system or for one
/// component of it, written where `--out` says and reported as `--format` says.
fn synthesize_suite(
    input: &SpecPath,
    target: SuiteTarget,
    out: Option<&Path>,
    component: Option<&str>,
    scenarios: Option<&Path>,
) -> Result<ExitCode> {
    let Ok((ir, _)) = resolved(&input.path, input.format)? else {
        return Ok(ExitCode::from(1));
    };
    let mut synthesis = match component {
        None => ess_conformance::synthesize(&ir),
        Some(name) => match ess_conformance::synthesize::synthesize_for(&ir, name) {
            Ok(synthesis) => synthesis,
            Err(unknown) => {
                eprintln!("{unknown}");
                return Ok(ExitCode::from(1));
            }
        },
    };
    // The authored half, compiled against the same model and filed in the same suite. Refused
    // rather than merged where a scenario names something the specification does not declare: a
    // suite carrying a check nobody can resolve is the artifact this whole verb exists to avoid.
    let authoring = ess_conformance::authored::compile(&ir, &authored_sources(scenarios)?);
    let authored = authoring.scenarios.len();
    let complete = authoring.is_complete();
    for (id, scenario) in authoring.scenarios {
        if let Err(id) = synthesis.suite.insert(id, scenario) {
            bail!("`{id}` is already in the suite");
        }
    }
    if !complete {
        for refusal in &authoring.refusals {
            eprintln!("{refusal}");
        }
        return Ok(ExitCode::from(1));
    }
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
            // Listed for the same reason: a scenario the specification obliges and this
            // suite does not hold has to be visible somewhere other than by its absence.
            for outside in &synthesis.outside {
                println!("outside: {outside}");
            }
            let written = written.unwrap_or_else(|| "nothing written".to_owned());
            // Counted apart, because they are not the same claim. A generated scenario is an
            // obligation the specification derived; an authored one is what a person asserted about
            // an algorithm the model cannot derive, and it is only as good as the person. One
            // number covering both would report coverage a specification never promised.
            match component {
                None => println!(
                    "{} scenario(s) ({authored} authored), {} refusal(s), {written}",
                    synthesis.suite.len(),
                    synthesis.refusals.len(),
                ),
                Some(name) => println!(
                    "{} scenario(s) for `{name}` ({authored} authored), {} outside it, {} \
                     refusal(s), {written}",
                    synthesis.suite.len(),
                    synthesis.outside.len(),
                    synthesis.refusals.len(),
                ),
            }
        }
        Format::Json => print!("{json}"),
        Format::Yaml => render(&synthesis.suite, Format::Yaml)?,
    }
    Ok(ExitCode::SUCCESS)
}

/// `ess conform author`: the authored scenarios alone, compiled against the model.
///
/// Separate from `synthesize` because the two answer different questions. `synthesize` asks what the
/// specification requires; this asks whether what somebody wrote still typechecks against it, which
/// is the question an author asks after every model change and does not want a hundred generated
/// scenarios printed at.
fn author_suite(
    input: &SpecPath,
    scenarios: Option<&Path>,
    out: Option<&Path>,
) -> Result<ExitCode> {
    let Ok((ir, _)) = resolved(&input.path, input.format)? else {
        return Ok(ExitCode::from(1));
    };
    let sources = authored_sources(scenarios)?;
    let authoring = ess_conformance::authored::compile(&ir, &sources);
    let mut suite =
        ess_conformance::ConformanceSuite::new(ess_conformance::SuiteProvenance::of(&ir));
    let complete = authoring.is_complete();
    for (id, scenario) in authoring.scenarios {
        if let Err(id) = suite.insert(id, scenario) {
            bail!("`{id}` is already in the suite");
        }
    }
    let json = suite.to_canonical_json();
    let written = match out {
        Some(out) => {
            fs::write(out, &json).with_context(|| format!("writing {}", out.display()))?;
            format!("written to {}", out.display())
        }
        None => "nothing written".to_owned(),
    };

    match input.format {
        // Printed, not counted, for the reason a synthesis refusal is: a number saying a scenario
        // did not compile tells an author that something is unchecked without telling them what.
        Format::Text => {
            for refusal in &authoring.refusals {
                println!("{refusal}");
            }
            println!(
                "{} authored scenario(s) from {} file(s), {} refusal(s), {written}",
                suite.len(),
                sources.len(),
                authoring.refusals.len(),
            );
        }
        Format::Json => print!("{json}"),
        Format::Yaml => render(&suite, Format::Yaml)?,
    }
    Ok(if complete {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

/// Every `ess-scenario/1` document a run should compile, in a deterministic order.
///
/// `--scenarios` names a directory or one file, and there is no default. That is deliberate rather
/// than unfinished: `--path` names a **specification**, every walk of one in this workspace collects
/// `*.yaml` from it, and a document that is not `ess/1` sitting inside would refuse the model
/// everywhere it is read. So authored scenarios live outside the specification directory and are
/// named, which also lets one set of them be compiled against two revisions of one model.
///
/// Sorted by path, because `read_dir` is not: two runs over one directory have to compile the same
/// files in the same order, or a duplicate would be refused in one run and not the other.
fn authored_sources(scenarios: Option<&Path>) -> Result<Vec<ess_conformance::authored::Source>> {
    let read = |path: &Path| -> Result<ess_conformance::authored::Source> {
        let text =
            fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        Ok(ess_conformance::authored::Source::new(
            path.display().to_string(),
            text,
        ))
    };
    let directory = match scenarios {
        None => return Ok(Vec::new()),
        Some(path) if path.is_file() => return Ok(vec![read(path)?]),
        Some(path) => path.to_path_buf(),
    };
    if !directory.is_dir() {
        bail!("{} is not a directory of scenarios", directory.display());
    }
    let mut files: Vec<PathBuf> = fs::read_dir(&directory)
        .with_context(|| format!("reading {}", directory.display()))?
        .collect::<std::io::Result<Vec<_>>>()?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "yaml" || extension == "yml")
        })
        .collect();
    files.sort();
    files.iter().map(|path| read(path)).collect()
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

fn run_external(process: &mut ProcessCommand, operation: &str) -> Result<()> {
    let status = process
        .status()
        .with_context(|| format!("starting {operation}"))?;
    if status.success() {
        Ok(())
    } else {
        bail!("{operation} failed with {status}")
    }
}

fn verified_bundle(path: &Path, require_canonical: bool) -> Result<ess_deployment::ReleaseBundle> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let parsed = if path
        .extension()
        .is_some_and(|extension| extension == "json")
    {
        ess_deployment::ReleaseBundle::from_json(&text)
            .with_context(|| format!("parsing {} as release-bundle JSON", path.display()))?
    } else {
        ess_deployment::ReleaseBundle::from_yaml(&text)
            .with_context(|| format!("parsing {} as release-bundle YAML", path.display()))?
    };
    let verified = ess_deployment::verify_release_bundle(parsed)
        .with_context(|| format!("validating {}", path.display()))?;
    if require_canonical && text != verified.to_canonical_json() {
        bail!(
            "{} is not canonical release-bundle JSON; regenerate it with `ess release bundle`",
            path.display()
        );
    }
    Ok(verified)
}

fn pinned_oci_digest(reference: &str) -> Result<ess_deployment::Digest> {
    let (repository, digest) = reference
        .rsplit_once('@')
        .context("OCI release source must be pinned as <repository>@sha256:<digest>")?;
    if repository.is_empty() {
        bail!("OCI release source repository must not be empty");
    }
    ess_deployment::Digest::new(digest)
        .with_context(|| format!("OCI release source has invalid digest {digest:?}"))
}

fn only_regular_file(directory: &Path) -> Result<PathBuf> {
    let entries = fs::read_dir(directory)
        .with_context(|| format!("reading ORAS output directory {}", directory.display()))?
        .collect::<std::io::Result<Vec<_>>>()?;
    if entries.len() != 1 || !entries[0].file_type()?.is_file() {
        bail!(
            "OCI release bundle must contain exactly one regular payload file; found {} entry(s)",
            entries.len()
        );
    }
    Ok(entries[0].path())
}

fn reconcile_release(
    cluster: &str,
    release: &ess_deployment::DeploymentRelease,
    cache: &Path,
    timeout: &str,
) -> Result<()> {
    if release.chart.kind != ess_deployment::ArtifactKind::HelmChart {
        bail!("{} does not select a Helm chart artifact", release.service);
    }
    let chart = fetch_helm_chart(&release.chart, cache)?;
    let staging = TemporaryDirectory::create("ess-helm-values")?;
    let values_path = staging.path().join("values.yaml");
    let values = serde_yaml::to_string(&serde_json::json!({
        "serviceAccount": {"name": &release.service_account},
        "images": release.images.iter().map(|(name, artifact)| {
            (name.as_str(), serde_json::json!({
                "repository": &artifact.reference,
                "digest": &artifact.digest,
            }))
        }).collect::<std::collections::BTreeMap<_, _>>(),
        "config": &release.config,
        "secrets": &release.secrets,
        "endpoints": &release.endpoints,
    }))?;
    fs::write(&values_path, values)
        .with_context(|| format!("writing transient Helm values for {}", release.service))?;

    let mut process = ProcessCommand::new("helm");
    process
        .args(["upgrade", "--install", &release.release_name])
        .arg(&chart)
        .args(["--namespace", &release.namespace, "--create-namespace"])
        .args(["--kube-context", cluster, "--values"])
        .arg(&values_path)
        .args(["--atomic", "--wait", "--timeout", timeout]);
    run_external(&mut process, "Helm reconciliation")
}

fn fetch_helm_chart(artifact: &ess_deployment::Artifact, cache: &Path) -> Result<PathBuf> {
    let digest_hex = artifact.digest.as_str().trim_start_matches("sha256:");
    let cache_directory = cache.join("helm").join("sha256").join(digest_hex);
    let archive = cache_directory.join("chart.tgz");
    let checksum = cache_directory.join("payload.digest");
    if archive.is_file() && checksum.is_file() {
        let expected = fs::read_to_string(&checksum)
            .with_context(|| format!("reading {}", checksum.display()))?;
        let actual = ess_deployment::Digest::of_bytes(
            &fs::read(&archive).with_context(|| format!("reading {}", archive.display()))?,
        );
        if expected.trim() != actual.as_str() {
            bail!(
                "cached Helm payload {} failed its local checksum",
                archive.display()
            );
        }
        return Ok(archive);
    }

    let staging = TemporaryDirectory::create("ess-helm-fetch")?;
    let reference = format!(
        "{}@{}",
        artifact.reference.trim_start_matches("oci://"),
        artifact.digest
    );
    let mut process = ProcessCommand::new("oras");
    process
        .args(["pull", "--no-tty", "--output"])
        .arg(staging.path())
        .arg(&reference);
    run_external(&mut process, "ORAS chart pull")?;
    let source = only_chart_archive(staging.path())?;
    let bytes = fs::read(&source).with_context(|| format!("reading {}", source.display()))?;
    let payload_digest = ess_deployment::Digest::of_bytes(&bytes);
    fs::create_dir_all(&cache_directory)
        .with_context(|| format!("creating {}", cache_directory.display()))?;
    fs::write(&archive, bytes).with_context(|| format!("writing {}", archive.display()))?;
    fs::write(&checksum, format!("{payload_digest}\n"))
        .with_context(|| format!("writing {}", checksum.display()))?;
    Ok(archive)
}

fn only_chart_archive(directory: &Path) -> Result<PathBuf> {
    let mut pending = vec![directory.to_path_buf()];
    let mut archives = Vec::new();
    while let Some(next) = pending.pop() {
        for entry in fs::read_dir(&next)
            .with_context(|| format!("reading ORAS chart output {}", next.display()))?
        {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_dir() {
                pending.push(entry.path());
            } else if kind.is_file()
                && entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "tgz")
            {
                archives.push(entry.path());
            }
        }
    }
    if archives.len() != 1 {
        bail!(
            "OCI Helm artifact must contain exactly one .tgz archive; found {}",
            archives.len()
        );
    }
    Ok(archives.remove(0))
}

struct TemporaryDirectory(PathBuf);

impl TemporaryDirectory {
    fn create(prefix: &str) -> Result<Self> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("system clock precedes the Unix epoch")?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}-{}-{nonce}", std::process::id()));
        fs::create_dir(&path)
            .with_context(|| format!("creating temporary directory {}", path.display()))?;
        Ok(Self(path))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
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
    use std::collections::BTreeSet;

    #[test]
    fn every_command_and_argument_name_is_unambiguous() {
        command().debug_assert();
    }

    /// Every leaf of the tree, as the path of names that reaches it.
    ///
    /// A leaf is a command with nothing below it — the thing that actually does the work. The
    /// tree carries each one twice, once under its area and once as a flat spelling, and the
    /// cases below are about that pairing, so the enumeration has to come from the tree rather
    /// than from a list somebody keeps up to date.
    fn leaf_paths(command: &clap::Command) -> Vec<Vec<String>> {
        fn walk(command: &clap::Command, prefix: &[String], leaves: &mut Vec<Vec<String>>) {
            let mut children = command.get_subcommands().peekable();
            if children.peek().is_none() {
                if !prefix.is_empty() {
                    leaves.push(prefix.to_vec());
                }
                return;
            }
            for child in children {
                let mut path = prefix.to_vec();
                path.push(child.get_name().to_owned());
                walk(child, &path, leaves);
            }
        }

        let mut leaves = Vec::new();
        walk(command, &[], &mut leaves);
        leaves
    }

    /// The command one path names, or `None` when nothing answers to that spelling.
    fn node<'a>(root: &'a clap::Command, path: &[String]) -> Option<&'a clap::Command> {
        let mut current = root;
        for name in path {
            current = current.find_subcommand(name)?;
        }
        Some(current)
    }

    /// Whether every command along `path` is listed in its parent's `--help`.
    fn listed(root: &clap::Command, path: &[String]) -> bool {
        let mut current = root;
        for name in path {
            let Some(child) = current.find_subcommand(name) else {
                return false;
            };
            if child.is_hide_set() {
                return false;
            }
            current = child;
        }
        true
    }

    /// What an invocation of this command can carry: its arguments, and everything below it.
    ///
    /// Deliberately not whether it is hidden. A grouped path is listed and its flat alias is not,
    /// and that difference is the point rather than a drift.
    fn shape(command: &clap::Command) -> String {
        use std::fmt::Write as _;

        let mut rendered = String::new();
        for argument in command.get_arguments() {
            let id = argument.get_id().as_str();
            if id == "help" {
                continue;
            }
            let long = argument.get_long();
            let short = argument.get_short();
            let required = argument.is_required_set();
            let values = argument.get_num_args();
            let defaults = argument.get_default_values();
            let _ = write!(
                rendered,
                "<{id} long={long:?} short={short:?} required={required} values={values:?} \
                 defaults={defaults:?}>"
            );
        }
        for child in command.get_subcommands() {
            let name = child.get_name();
            let below = shape(child);
            let _ = write!(rendered, "[{name} {below}]");
        }
        rendered
    }

    /// How many leaves the four areas carry between them.
    ///
    /// Written down on purpose. A verb added to the tree and to no area would otherwise be
    /// counted by the enumeration it is missing from and pass every case below.
    const AREA_LEAVES: usize = 41;

    /// The order they are offered in is checked where it is rendered, in
    /// `tests/command_surface.rs`: `mut_subcommand` moves what it touches to the end of the list,
    /// so this order is not the one `--help` prints and asserting on it would say nothing.
    #[test]
    fn the_first_level_is_exactly_the_four_areas() {
        let command = command();
        let listed: BTreeSet<&str> = command
            .get_subcommands()
            .filter(|sub| !sub.is_hide_set())
            .map(clap::Command::get_name)
            .collect();
        assert_eq!(listed, AREAS.iter().copied().collect::<BTreeSet<_>>());
    }

    #[test]
    fn every_leaf_is_reachable_by_its_area_path_and_by_its_flat_spelling() {
        let command = command();
        let leaves = leaf_paths(&command);
        let (grouped, flat): (Vec<Vec<String>>, Vec<Vec<String>>) = leaves
            .iter()
            .cloned()
            .partition(|path| listed(&command, path));

        assert_eq!(grouped.len(), AREA_LEAVES, "grouped leaves: {grouped:?}");
        assert_eq!(
            flat.len(),
            AREA_LEAVES - 1,
            "`ess generate` is the one flat spelling that is not a leaf of the tree, because the \
             area of that name carries the verb's arguments itself: {flat:?}"
        );

        for path in &grouped {
            assert!(
                AREAS.contains(&path[0].as_str()),
                "`ess {}` is listed and is under no area",
                path.join(" ")
            );
            let spelled = path.join(" ");
            let alias = &path[1..];
            let flat_spelling = alias.join(" ");
            let leaf = node(&command, path).expect("the path came from the tree");

            if alias == ["generate"] {
                let area = node(&command, alias).expect("the generate area is the first level");
                assert!(
                    !area.is_subcommand_required_set(),
                    "`ess generate --path …` must parse without a subcommand"
                );
                for argument in leaf.get_arguments() {
                    let id = argument.get_id().as_str();
                    if id == "help" {
                        continue;
                    }
                    let carried = area
                        .get_arguments()
                        .find(|candidate| candidate.get_id() == id)
                        .unwrap_or_else(|| panic!("`ess generate` does not carry `--{id}`"));
                    assert!(
                        !carried.is_hide_set(),
                        "`ess generate` accepts `--{id}` and must offer it: a command is not \
                         allowed to hide an argument it takes, however it is spelled"
                    );
                }
                let combined = command.clone().try_get_matches_from([
                    "ess",
                    "generate",
                    "--path",
                    "examples/billing",
                    "synthesize",
                ]);
                assert!(
                    combined.is_err(),
                    "`ess generate --path … synthesize` says two things at once and must be \
                     refused, as it was before `{spelled}` had an area to sit in"
                );
                continue;
            }

            let aliased = node(&command, alias).unwrap_or_else(|| {
                panic!("`ess {spelled}` has no flat spelling `ess {flat_spelling}`")
            });
            assert_eq!(
                shape(leaf),
                shape(aliased),
                "`ess {spelled}` and `ess {flat_spelling}` are not the same command"
            );
            assert!(
                !listed(&command, alias),
                "the flat spelling `ess {flat_spelling}` is listed in `--help`"
            );
        }

        let mut spellings: BTreeSet<Vec<String>> =
            grouped.iter().map(|path| path[1..].to_vec()).collect();
        assert!(spellings.remove(&vec!["generate".to_owned()]));
        assert_eq!(
            spellings,
            flat.iter().cloned().collect::<BTreeSet<_>>(),
            "the hidden leaves are not exactly the area leaves with the area dropped"
        );
    }

    #[test]
    fn the_generate_area_answers_to_the_flat_spelling_and_to_its_own() {
        for arguments in [
            ["ess", "generate", "--path", "examples/billing"].as_slice(),
            ["ess", "generate", "generate", "--path", "examples/billing"].as_slice(),
        ] {
            let matches = command()
                .try_get_matches_from(arguments)
                .unwrap_or_else(|error| panic!("{arguments:?} is refused: {error}"));
            Cli::from_arg_matches(&matches)
                .unwrap_or_else(|error| panic!("{arguments:?} does not build a Cli: {error}"));
        }
    }

    #[test]
    fn no_manifest_or_lockfile_depends_on_aep() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
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
