//! Local report qualification. Original bytes and admitted values stay owned through publication.
//! Neither equality to a supplied model nor an external tool's digest authenticates execution.
use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use ess_conformance::{
    coverage::{AdmittedInput, Scope},
    AdmittedSuite, CountReport, CountStatus, SuiteProvenance,
};
use ess_deployment::{BuildIr, BuildOutputKind, ComponentIr, Digest, ReleaseBundle, RuntimeIr};

#[derive(Debug, clap::Args)]
pub(super) struct Options {
    /// Explicit authored ESS model root, loaded once.
    #[arg(long, requires_all = ["report", "expected_selection"])]
    spec: Option<PathBuf>,
    /// Original standalone ess-conformance-report/2 JSON; never a generic check log.
    #[arg(long, requires_all = ["spec", "expected_selection"])]
    report: Option<PathBuf>,
    /// Independently supplied original unfiltered suite/5 JSON.
    #[arg(long, group = "expected_selection", conflicts_with = "expected_suite_input", requires_all = ["spec", "report"])]
    expected_suite: Option<PathBuf>,
    /// Independently supplied original input/1 carrier, including every parent suite.
    #[arg(long, group = "expected_selection", conflicts_with = "expected_suite", requires_all = ["spec", "report"])]
    expected_suite_input: Option<PathBuf>,
    /// Optional SHA-256 pin of the raw report bytes, distinct from the OCI manifest digest.
    #[arg(long, requires_all = ["spec", "report", "expected_selection"])]
    report_sha256: Option<Digest>,
    /// Optional SHA-256 pin of the raw selected suite file or complete input carrier file.
    #[arg(long, requires_all = ["spec", "report", "expected_selection"])]
    expected_input_sha256: Option<Digest>,
}

/// Unchanged buffers, completely admitted selection, and the single loaded model.
pub(super) struct Inputs {
    report: String,
    _expected: String,
    input: AdmittedInput,
    model: Box<ess_compiler::EssIr>,
}

impl Options {
    pub(super) fn read(&self) -> Result<Option<Inputs>> {
        let Some(spec) = &self.spec else {
            return Ok(None);
        };
        let report = read(self.report.as_deref().context("missing report")?)?;
        let selected = self
            .expected_suite
            .as_deref()
            .or(self.expected_suite_input.as_deref())
            .context("missing expected selection")?;
        let expected = read(selected)?;
        check_pin(&report, self.report_sha256.as_ref(), "raw report")?;
        check_pin(
            &expected,
            self.expected_input_sha256.as_ref(),
            "expected input",
        )?;
        let input = if self.expected_suite.is_some() {
            AdmittedInput::from_suite(
                AdmittedSuite::from_json(&expected)
                    .context("admitting expected selection suite")?,
            )
        } else {
            AdmittedInput::from_json(&expected)
        }
        .context("admitting expected selection")?;
        let model = match crate::load::specification(spec)? {
            crate::load::LoadedSpec::Compiled { ir, .. } => ir,
            crate::load::LoadedSpec::Refused {
                problems,
                diagnostics,
                ..
            } => {
                bail!("explicit conformance model was refused: {problems:?}; {diagnostics:?}");
            }
        };
        let provenance = SuiteProvenance::of(&model);
        let actual = &input.selected().suite().provenance;
        // Exact equality to compiler-produced full digests excludes admitted legacy short digests.
        if actual.system != provenance.system
            || actual.specification_version != provenance.specification_version
            || actual.spec_digest != provenance.spec_digest
            || actual.contract_digest != provenance.contract_digest
        {
            bail!(
                "conformance model identity differs from the independently supplied expected suite"
            );
        }
        let coverage = input
            .selected()
            .coverage()
            .context("expected selection requires suite/5")?;
        match (&coverage.selection.scope, &actual.component) {
            (Scope::System, None) => {}
            (Scope::Component { component }, Some(name)) if component.as_str() == name => {
                if !model.components().contains_key(component) {
                    bail!("expected selection names unknown model component {name}");
                }
            }
            _ => bail!("expected selection scope and model component disagree"),
        }
        Ok(Some(Inputs {
            report,
            _expected: expected,
            input,
            model,
        }))
    }
}

fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .with_context(|| format!("reading {} as original UTF-8", path.display()))
}
fn check_pin(raw: &str, pin: Option<&Digest>, label: &str) -> Result<()> {
    if pin.is_some_and(|pin| *pin != Digest::of_bytes(raw.as_bytes())) {
        bail!("{label} byte pin mismatch");
    }
    Ok(())
}

pub(super) struct Qualified {
    inputs: Inputs,
    _report: CountReport,
}
impl Inputs {
    pub(super) fn qualify(
        self,
        component: &ComponentIr,
        build: &BuildIr,
        runtime: &RuntimeIr,
    ) -> Result<Qualified> {
        component
            .validate()
            .context("validating component context")?;
        build.validate().context("validating build context")?;
        runtime
            .validate_against_build(build)
            .context("validating runtime against build context")?;
        // Each declared unit must be able to supply its required release artifact.
        // Additional outputs and other output kinds remain valid within the build.
        for (role, unit, kind) in [
            (
                "runtime",
                &component.release_units().runtime,
                BuildOutputKind::OciImage,
            ),
            (
                "chart",
                &component.release_units().chart,
                BuildOutputKind::HelmChart,
            ),
        ] {
            if !build
                .outputs()
                .values()
                .any(|output| output.release_unit == *unit && output.kind == kind)
            {
                bail!("component {role} release unit {unit} has no {kind:?} build output");
            }
        }
        if component.system().as_str() != self.model.system().to_string()
            || component.semantic_version() != self.model.version().to_string()
            || *runtime.semantic_digest()
                != Digest::new(format!("sha256:{}", self.model.source_digest()))?
        {
            bail!("compiled deployment context differs from the explicit model identity");
        }
        let report = CountReport::from_json(&self.report, self.input.selected())
            .context("admitting conformance report against exact expected selection")?;
        if report.conformance_status() != CountStatus::Passed {
            bail!("conformance report must be passed for its nonempty complete declared selection");
        }
        Ok(Qualified {
            inputs: self,
            _report: report,
        })
    }
}

impl Qualified {
    pub(super) fn report_bytes(&self) -> &[u8] {
        self.inputs.report.as_bytes()
    }
    pub(super) fn describe(
        &self,
        component: &ComponentIr,
        build: &BuildIr,
        runtime: &RuntimeIr,
    ) -> Result<()> {
        let selected = self.inputs.input.selected();
        let provenance = &selected.suite().provenance;
        eprintln!("conformance: passed for the supplied exact declared selection");
        eprintln!("selected suite: {}", selected.digest());
        eprintln!(
            "selection: {}",
            serde_json::to_string(&selected.coverage().expect("admitted suite/5").selection)?
        );
        eprintln!(
            "model: {}/{}; spec_digest: {}; contract_digest: {}",
            provenance.system,
            provenance.specification_version,
            provenance.spec_digest,
            provenance.contract_digest
        );
        eprintln!(
            "raw report sha256: {}",
            Digest::of_bytes(self.report_bytes())
        );
        eprintln!(
            "component: {}; system: {}; semantic_version: {}; digest: {}",
            component.component(),
            component.system(),
            component.semantic_version(),
            component.digest()
        );
        eprintln!(
            "build digest: {}; runtime digest: {}; runtime semantic digest: {}",
            build.digest(),
            runtime.digest(),
            runtime.semantic_digest()
        );
        Ok(())
    }
}

pub(super) fn qualifiers() {
    eprintln!("attachment binding: unverified");
    eprintln!("producer origin: unverified");
    eprintln!("artifact execution: unverified");
    eprintln!("signature verification: unsupported");
}
pub(super) fn consistency(content_identity: bool) {
    eprintln!("release consistency: checked");
    if content_identity {
        eprintln!("bundle content identity: checked");
    }
    eprintln!("conformance: not assessed");
    qualifiers();
}
pub(super) fn bundle_context(bundle: &ReleaseBundle) -> Result<()> {
    for release in bundle.releases.values() {
        eprintln!("release unit: {}; version: {}; source_commit: {}; semantic_digest: {}; build_digest: {}; runtime_digest: {}",release.release_unit,release.version,release.source_commit,release.semantic_digest,release.build_digest,release.runtime_digest);
        for (name, artifact) in &release.artifacts {
            eprintln!("artifact: {name}; build_output: {}; kind: {}; reference: {}; digest: {}; platforms: {}",artifact.build_output,serde_json::to_string(&artifact.kind)?,artifact.reference,artifact.digest,serde_json::to_string(&artifact.platforms)?);
        }
    }
    Ok(())
}

/// Strict new profile: each owner admits once and the supplied JSON must be canonical.
pub(super) fn deployment(
    component: &Path,
    build: &Path,
    runtime: &Path,
) -> Result<(ComponentIr, BuildIr, RuntimeIr)> {
    let c = read(component)?;
    let b = read(build)?;
    let r = read(runtime)?;
    let component = ComponentIr::from_json(&c).context("admitting component IR JSON")?;
    let build = BuildIr::from_json(&b).context("admitting build IR JSON")?;
    let runtime = RuntimeIr::from_json(&r).context("admitting runtime IR JSON")?;
    if c != component.to_canonical_json()
        || b != build.to_canonical_json()
        || r != runtime.to_canonical_json()
    {
        bail!("qualification requires canonical component/build/runtime JSON bytes");
    }
    Ok((component, build, runtime))
}

pub(super) fn tagged_destination(to: &str) -> Result<()> {
    if to.contains('@')
        || to.chars().any(char::is_whitespace)
        || !to.rsplit('/').next().is_some_and(|tail| {
            tail.split_once(':')
                .is_some_and(|(name, tag)| !name.is_empty() && !tag.is_empty())
        })
    {
        bail!("conformance OCI publication destination must be tagged");
    }
    Ok(())
}
