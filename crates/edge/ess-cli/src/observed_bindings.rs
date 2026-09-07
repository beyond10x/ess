//! The concrete comparison between admitted semantic realization and infrastructure observation.
//! No library merges the two bounded contexts; collection remains at the Kubernetes edge.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;

use anyhow::{bail, Context, Result};
use ess_deployment::Digest;
use ess_realization::{
    ArtifactIdentity, ArtifactKind, RealizationId, RealizationIr, RealizationSpec,
};
use serde::{Deserialize, Serialize};

const INPUT_FORMAT: &str = "ess-observed-bindings/1";
const REPORT_FORMAT: &str = "ess-observed-bindings-report/1";

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// Exact semantic ESS source directory or file.
    #[arg(long)]
    spec: PathBuf,
    /// Source-owned ess-realization/1 or /2 document.
    #[arg(long)]
    realization: PathBuf,
    /// Environment-owned ess-observed-bindings/1 document.
    #[arg(long)]
    bindings: PathBuf,
    /// Existing native infrastructure observation or IR; never implies a fresh live read.
    #[arg(long, required_unless_present = "live", conflicts_with = "live")]
    infra: Option<PathBuf>,
    /// Collect the document's exact context and namespace before checking.
    #[arg(long, requires = "observation_out")]
    live: bool,
    /// New observation file outside Git checkouts; its parent must already exist.
    #[arg(long, requires = "live")]
    observation_out: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = super::MachineFormat::Text)]
    format: super::MachineFormat,
    /// Optional deterministic generated reference; parent must exist and file must be new.
    #[arg(long)]
    markdown_out: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Scope {
    context: String,
    namespace: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Kind {
    Deployment,
    Statefulset,
    Daemonset,
}
impl Kind {
    const fn key(&self) -> &'static str {
        match self {
            Self::Deployment => "deployment",
            Self::Statefulset => "statefulset",
            Self::Daemonset => "daemonset",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Workload {
    kind: Kind,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceEvidence {
    repository: String,
    revision: String,
    path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    id: RealizationId,
    implementation: RealizationId,
    workload: Workload,
    container: String,
    image: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    evidence: Vec<SourceEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BindingDocument {
    format: String,
    id: RealizationId,
    realization_digest: ArtifactIdentity,
    scope: Scope,
    bindings: Vec<Binding>,
}

impl BindingDocument {
    fn validate(&self, realization: &RealizationIr) -> Result<()> {
        let mut errors = Vec::new();
        if self.format != INPUT_FORMAT {
            errors.push(format!("expected format {INPUT_FORMAT}"));
        }
        if &self.realization_digest != realization.realization_digest() {
            errors.push("realization digest does not match the compiled source".to_owned());
        }
        if self.scope.context.trim().is_empty() || self.scope.context.chars().any(char::is_control)
        {
            errors.push("scope requires an explicit Kubernetes context".to_owned());
        }
        if !dns_label(&self.scope.namespace, 63) {
            errors.push("invalid namespace".to_owned());
        }
        if self.bindings.is_empty() {
            errors.push("bindings must not be empty".to_owned());
        }
        let mut ids = BTreeSet::new();
        let mut targets = BTreeSet::new();
        for binding in &self.bindings {
            if !ids.insert(&binding.id) {
                errors.push(format!("duplicate binding {}", binding.id));
            }
            if !targets.insert((
                binding.workload.kind.key(),
                &binding.workload.name,
                &binding.container,
            )) {
                errors.push(format!("duplicate target for binding {}", binding.id));
            }
            if !realization
                .implementations()
                .contains_key(&binding.implementation)
            {
                errors.push(format!(
                    "{} references unknown implementation {}",
                    binding.id, binding.implementation
                ));
            }
            if binding.workload.name.len() > 253
                || !binding
                    .workload
                    .name
                    .split('.')
                    .all(|part| dns_label(part, 63))
            {
                errors.push(format!("{} has invalid workload name", binding.id));
            }
            if !dns_label(&binding.container, 63) {
                errors.push(format!("{} has invalid container name", binding.id));
            }
            if binding.image.is_empty()
                || binding.image.chars().any(char::is_whitespace)
                || binding.image.chars().any(char::is_control)
            {
                errors.push(format!("{} requires an exact image reference", binding.id));
            }
            for evidence in &binding.evidence {
                if evidence.repository.trim().is_empty()
                    || evidence.repository.chars().any(char::is_control)
                    || evidence.revision.len() != 40
                    || !evidence
                        .revision
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                    || evidence.path.is_empty()
                    || !Path::new(&evidence.path)
                        .components()
                        .all(|p| matches!(p, Component::Normal(_)))
                {
                    errors.push(format!(
                        "{} requires repository, exact Git revision and relative evidence path",
                        binding.id
                    ));
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            bail!("{}", errors.join("; "))
        }
    }
    fn canonical(&self) -> Self {
        let mut ordered = self.clone();
        ordered.bindings.sort_by(|a, b| a.id.cmp(&b.id));
        for binding in &mut ordered.bindings {
            binding.evidence.sort_by(|a, b| {
                (&a.repository, &a.revision, &a.path).cmp(&(&b.repository, &b.revision, &b.path))
            });
        }
        ordered
    }
}

fn dns_label(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && value.as_bytes()[0].is_ascii_alphanumeric()
        && value.as_bytes()[value.len() - 1].is_ascii_alphanumeric()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Status {
    Satisfied,
    Violated,
    Unknown,
}
impl Status {
    fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Self::Violated, _) | (_, Self::Violated) => Self::Violated,
            (Self::Unknown, _) | (_, Self::Unknown) => Self::Unknown,
            _ => Self::Satisfied,
        }
    }
    const fn exit(self) -> u8 {
        match self {
            Self::Satisfied => 0,
            Self::Violated => 1,
            Self::Unknown => 2,
        }
    }
    const fn label(self) -> &'static str {
        match self {
            Self::Satisfied => "satisfied",
            Self::Violated => "violated",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Serialize)]
struct Check {
    code: &'static str,
    status: Status,
    detail: String,
}
#[derive(Debug, Serialize)]
struct BindingResult {
    implementation: String,
    components: Vec<String>,
    artifact_locator: String,
    artifact_identity: String,
    workload: String,
    container: String,
    expected_image: String,
    observed_uid: Option<String>,
    observed_image: Option<String>,
    declared_evidence: Vec<SourceEvidence>,
    status: Status,
    checks: Vec<Check>,
}
impl BindingResult {
    fn check(&mut self, code: &'static str, status: Status, detail: impl Into<String>) {
        self.status = self.status.combine(status);
        self.checks.push(Check {
            code,
            status,
            detail: detail.into(),
        });
    }
}
#[derive(Debug, Serialize)]
struct Observation {
    digest: String,
    provenance: infra_compiler::Provenance,
    coverage: Option<infra_domain::coverage::CollectionCoverage>,
}
#[derive(Debug, Serialize)]
struct Report {
    format: &'static str,
    status: Status,
    acquisition: &'static str,
    binding_digest: Option<String>,
    realization_digest: Option<String>,
    scope: Option<Scope>,
    observation: Option<Observation>,
    checks: Vec<Check>,
    bindings: BTreeMap<String, BindingResult>,
    exclusions: Vec<&'static str>,
}
impl Report {
    fn new(live: bool) -> Self {
        Self {
            format: REPORT_FORMAT, status: Status::Unknown,
            acquisition: if live { "live_namespace_read" } else { "supplied_observation" },
            binding_digest: None, realization_digest: None, scope: None, observation: None,
            checks: Vec::new(), bindings: BTreeMap::new(),
            exclusions: vec![
                "Component membership is an authored implementation claim, not runtime behavioral conformance.",
                "Workload templates do not attest running Pod image IDs, health or operation exposure.",
                "Source-to-image build provenance and mutable tag resolution are not verified by this adapter.",
                "Evidence locations are declared attribution, not verified contents or signatures.",
                "Only selected bindings are checked; unselected components and deployments remain uncovered.",
                "Secret values, literal configuration, probes and other omitted fields remain unobserved.",
                "Sequential observations are not atomic; offline input does not imply current live state.",
            ],
        }
    }
    fn fail(&mut self, code: &'static str, status: Status, detail: impl Into<String>) {
        self.status = status;
        self.checks.push(Check {
            code,
            status,
            detail: detail.into(),
        });
    }
    fn markdown(&self) -> String {
        fn cell(text: &str) -> String {
            text.replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('|', "&#124;")
                .replace('`', "&#96;")
                .replace(['\n', '\r'], " ")
        }
        let mut out = format!("# Observed component bindings\n\nGenerated by ESS. Result: **{}**. Acquisition: `{}`.\n\n", self.status.label(), self.acquisition);
        if let Some(observation) = &self.observation {
            write!(
                out,
                "Observation: `{}` at `{}`; digest `{}`.\n\n",
                cell(&observation.provenance.context),
                cell(&observation.provenance.scanned_at),
                observation.digest
            )
            .expect("String formatting cannot fail");
        }
        out.push_str("| Binding | Components | Workload / container | Observed image | Result |\n|---|---|---|---|---|\n");
        for (id, binding) in &self.bindings {
            writeln!(
                out,
                "| {} | {} | {} / {} | {} | {} |",
                cell(id),
                cell(&binding.components.join(", ")),
                cell(&binding.workload),
                cell(&binding.container),
                cell(binding.observed_image.as_deref().unwrap_or("unobserved")),
                binding.status.label()
            )
            .expect("String formatting cannot fail");
        }
        out.push_str("\n## Findings\n\n");
        for check in &self.checks {
            writeln!(
                out,
                "- {}: {} — {}",
                check.code,
                check.status.label(),
                cell(&check.detail)
            )
            .expect("String formatting cannot fail");
        }
        for (id, binding) in &self.bindings {
            for check in &binding.checks {
                writeln!(
                    out,
                    "- {} / {}: {} — {}",
                    cell(id),
                    check.code,
                    check.status.label(),
                    cell(&check.detail)
                )
                .expect("String formatting cannot fail");
            }
        }
        out.push_str("\n## Coverage\n\n");
        for exclusion in &self.exclusions {
            writeln!(out, "- {exclusion}").expect("String formatting cannot fail");
        }
        out
    }
}

fn evaluate(
    spec: &BindingDocument,
    realization: &RealizationIr,
    ir: &infra_compiler::InfraIr,
    report: &mut Report,
) {
    report.observation = Some(Observation {
        digest: ir.digest(),
        provenance: ir.provenance.clone(),
        coverage: ir.model().coverage.clone(),
    });
    let scoped = ir.provenance.context == spec.scope.context
        && ir
            .model()
            .coverage
            .as_ref()
            .is_some_and(|c| c.namespace() == spec.scope.namespace);
    report.status = Status::Satisfied;
    for binding in &spec.canonical().bindings {
        let implementation = &realization.implementations()[&binding.implementation];
        let artifact = implementation.artifact();
        let workload = format!(
            "{}/{}/{}",
            spec.scope.namespace,
            binding.workload.kind.key(),
            binding.workload.name
        );
        let mut result = BindingResult {
            implementation: binding.implementation.to_string(),
            components: implementation
                .components()
                .iter()
                .map(ToString::to_string)
                .collect(),
            artifact_locator: artifact.locator().to_owned(),
            artifact_identity: artifact.identity().to_string(),
            workload: workload.clone(),
            container: binding.container.clone(),
            expected_image: binding.image.clone(),
            observed_uid: None,
            observed_image: None,
            declared_evidence: binding.evidence.clone(),
            status: Status::Satisfied,
            checks: Vec::new(),
        };
        result.check(
            "OBS-BIND-001",
            Status::Satisfied,
            "implementation resolves against the exact semantic source and realization digest",
        );
        if !scoped {
            result.check("OBS-BIND-002", Status::Unknown, "observation context and declared namespace coverage must both match; target absence cannot be inferred");
        } else if let Some(target) = ir.model().workloads.get(&workload) {
            result.observed_uid = Some(target.identity.uid.clone());
            result.check(
                "OBS-BIND-002",
                Status::Satisfied,
                "workload exists within the admitted scope",
            );
            if let Some(container) = target
                .containers
                .iter()
                .find(|c| c.name == binding.container)
            {
                result.observed_image = Some(container.image.clone());
                result.check(
                    "OBS-BIND-003",
                    Status::Satisfied,
                    "named container exists in the workload template",
                );
                result.check("OBS-BIND-004", if container.image == binding.image { Status::Satisfied } else { Status::Violated }, "observed workload-template image compared with the explicitly declared reference");
                let pinned = container
                    .image
                    .rsplit_once('@')
                    .filter(|(_, digest)| Digest::new(*digest).is_ok());
                match (artifact.kind(), pinned) {
                    (ArtifactKind::Container, Some((_, digest))) => {
                        let agrees = digest == artifact.identity().as_str() && container.image == artifact.locator();
                        result.check("OBS-BIND-005", if agrees { Status::Satisfied } else { Status::Violated }, "digest-pinned template reference compared with the selected immutable container artifact");
                    }
                    _ => result.check("OBS-BIND-005", Status::Unknown, "source/package artifacts or tag-only image references cannot establish immutable image agreement or source-to-image provenance"),
                }
            } else {
                result.check(
                    "OBS-BIND-003",
                    Status::Violated,
                    "named container is absent from the observed workload template",
                );
            }
        } else {
            result.check(
                "OBS-BIND-002",
                Status::Violated,
                "workload is absent within the admitted scope",
            );
        }
        report.status = report.status.combine(result.status);
        report.bindings.insert(binding.id.to_string(), result);
    }
}

fn read_yaml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    if path.extension().is_some_and(|e| e == "json") {
        Ok(serde_json::from_str(&text)?)
    } else {
        Ok(serde_yaml::from_str(&text)?)
    }
}

fn new_output(path: &Path, outside_git: bool) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => bail!("output already exists: {}", path.display()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error).context("cannot establish output absence"),
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent = parent
        .canonicalize()
        .context("output parent must already exist")?;
    if outside_git {
        for ancestor in parent.ancestors() {
            let marker = ancestor.join(".git");
            match fs::symlink_metadata(&marker) {
                // An empty directory is not a repository marker. Refuse files, symlinks,
                // nonempty directories and unreadable markers conservatively.
                Ok(metadata) if metadata.is_dir() && fs::read_dir(&marker)?.next().is_none() => {}
                Ok(_) => bail!("live observation output must be outside Git checkouts"),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error).context("cannot establish observation placement"),
            }
        }
    }
    Ok(())
}

pub(crate) fn run(args: &Args) -> Result<ExitCode> {
    let mut report = Report::new(args.live);
    let contract = (|| -> Result<_> {
        let semantic = match super::load::specification(&args.spec)? {
            super::load::LoadedSpec::Compiled { ir, .. } => ir,
            super::load::LoadedSpec::Refused {
                problems,
                diagnostics,
                ..
            } => bail!("semantic source refused: {problems:?}; {diagnostics}"),
        };
        let realization_spec: RealizationSpec = read_yaml(&args.realization)?;
        let realization =
            ess_realization::compile(&realization_spec, &semantic).map_err(anyhow::Error::msg)?;
        let bindings: BindingDocument = read_yaml(&args.bindings)?;
        bindings.validate(&realization)?;
        if let Some(path) = &args.markdown_out {
            new_output(path, false)?;
        }
        report.binding_digest =
            Some(Digest::of_bytes(&serde_json::to_vec(&bindings.canonical())?).to_string());
        report.realization_digest = Some(realization.realization_digest().to_string());
        report.scope = Some(bindings.scope.clone());
        Ok((bindings, realization))
    })();
    match contract {
        Err(error) => report.fail(
            "OBS-BIND-000",
            Status::Violated,
            format!("contract refused: {error}"),
        ),
        Ok((bindings, realization)) => {
            let acquisition = (|| -> Result<_> {
                let path = if args.live {
                    let output = args
                        .observation_out
                        .as_ref()
                        .context("live read requires observation output")?;
                    new_output(output, true)?;
                    ess_kubernetes::scan_namespace(
                        &bindings.scope.context,
                        &bindings.scope.namespace,
                        output,
                    )
                    .map_err(anyhow::Error::msg)?;
                    output
                } else {
                    args.infra
                        .as_ref()
                        .context("observation input is required")?
                };
                super::resolved_infrastructure(path)
            })();
            match acquisition {
                Err(error) => report.fail(
                    "OBS-BIND-006",
                    Status::Unknown,
                    format!("observation unavailable: {error}"),
                ),
                Ok(ir) => evaluate(&bindings, &realization, &ir, &mut report),
            }
        }
    }
    if let Some(path) = &args.markdown_out {
        // A refused output contract must not clobber that output while reporting its own refusal.
        if report.binding_digest.is_some() {
            use std::io::Write;
            let written = (|| -> std::io::Result<()> {
                let mut file = fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(path)?;
                file.write_all(report.markdown().as_bytes())
            })();
            if let Err(error) = written {
                report.fail(
                    "OBS-BIND-007",
                    report.status.combine(Status::Unknown),
                    format!("generated reference could not be written: {error}"),
                );
            }
        }
    }
    match args.format {
        super::MachineFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
        super::MachineFormat::Text => print!("{}", report.markdown()),
    }
    Ok(ExitCode::from(report.status.exit()))
}
