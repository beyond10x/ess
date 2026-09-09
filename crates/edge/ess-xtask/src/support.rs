//! The finite public source-capability block, separate from dated release evidence.
//!
//! CLI metadata and emitted headers establish inventory and output kinds. The linked semantic
//! tests remain the authority for support, obligations and refusal; this check never applies an
//! adapter or establishes publication. Keep this a concrete repository check, not a registry.

use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const BEGIN: &str = "[ess-source-support-begin]: #";
const END: &str = "[ess-source-support-end]: #";
const STATUS: &str = "website/docs/status/where-this-stands.md";

pub(super) fn run(root: &Path, check: bool) -> Result<String> {
    let expected = render(root).context("observing the public source-capability block")?;
    if check {
        compare(&fs::read_to_string(root.join(STATUS))?, &expected).map_err(anyhow::Error::msg)?;
        Ok(format!(
            "{STATUS}: complete source-support block agrees with offline CLI observations\n"
        ))
    } else {
        Ok(format!("{expected}\n"))
    }
}

fn compare(recorded: &str, expected: &str) -> Result<(), String> {
    let lines: Vec<_> = recorded.lines().collect();
    let starts: Vec<_> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| **line == BEGIN)
        .map(|(i, _)| i)
        .collect();
    let ends: Vec<_> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| **line == END)
        .map(|(i, _)| i)
        .collect();
    let ([start], [end]) = (starts.as_slice(), ends.as_slice()) else {
        return Err(format!(
            "{STATUS}: source-support block needs exactly one begin and end row; found {} and {}",
            starts.len(),
            ends.len()
        ));
    };
    if start >= end {
        return Err(format!(
            "{STATUS}: source-support end row precedes its begin row"
        ));
    }
    let actual = &lines[*start..=*end];
    let expected: Vec<_> = expected.lines().collect();
    for row in 0..actual.len().max(expected.len()) {
        if actual.get(row) != expected.get(row) {
            return Err(format!("{STATUS}:{}: source-support row {} differs\nexpected: {}\n   found: {}\nRun `cargo xtask support` to inspect the complete expected block.",
                start + row + 1, row + 1, expected.get(row).unwrap_or(&"<no row>"), actual.get(row).unwrap_or(&"<missing row>")));
        }
    }
    Ok(())
}

fn cli(root: &Path, args: &[&str]) -> Result<String> {
    let args: Vec<OsString> = args.iter().map(Into::into).collect();
    String::from_utf8(crate::cli_output(root, &args)?).context("CLI output is not UTF-8")
}

/// Read one option's actual long-help block, including Clap's multiline value descriptions.
fn option<'a>(help: &'a str, flag: &str) -> Result<Vec<&'a str>> {
    let mut lines = help
        .lines()
        .skip_while(|line| line.split_whitespace().next() != Some(flag));
    let first = lines
        .next()
        .with_context(|| format!("CLI help is missing {flag}"))?;
    let mut block = vec![first];
    block.extend(lines.take_while(|line| {
        let text = line.trim_start();
        !text.starts_with("--") && !text.starts_with("-h,")
    }));
    Ok(block)
}

fn choices(help: &str, flag: &str) -> Result<Vec<String>> {
    let block = option(help, flag)?;
    let joined = block.join("\n");
    let values = if let Some((_, suffix)) = joined.split_once("[possible values: ") {
        let (values, _) = suffix
            .split_once(']')
            .context("unterminated CLI possible-values metadata")?;
        values.split(", ").map(str::to_owned).collect::<Vec<_>>()
    } else {
        let position = block
            .iter()
            .position(|line| line.trim() == "Possible values:")
            .with_context(|| format!("CLI help for {flag} has no possible-values metadata"))?;
        block[position + 1..]
            .iter()
            .filter_map(|line| line.trim().strip_prefix("- "))
            .map(|line| {
                line.split_once(':')
                    .map_or(line, |(name, _)| name)
                    .trim()
                    .to_owned()
            })
            .collect()
    };
    if values.is_empty()
        || values.iter().any(String::is_empty)
        || values.iter().collect::<BTreeSet<_>>().len() != values.len()
    {
        bail!("CLI help for {flag} has empty or duplicate possible values");
    }
    Ok(values)
}

fn default(help: &str, flag: &str) -> Result<String> {
    let block = option(help, flag)?.join("\n");
    let (_, suffix) = block
        .split_once("[default: ")
        .with_context(|| format!("CLI help for {flag} has no default"))?;
    Ok(suffix
        .split_once(']')
        .context("unterminated CLI default metadata")?
        .0
        .to_owned())
}

fn commands(help: &str) -> Result<Vec<String>> {
    let (_, body) = help
        .split_once("Commands:\n")
        .context("CLI help has no command inventory")?;
    let (body, _) = body
        .split_once("\nOptions:")
        .context("CLI command inventory has no boundary")?;
    let names: Vec<_> = body
        .lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("  ")?;
            (!rest.starts_with(' '))
                .then(|| rest.split_whitespace().next())
                .flatten()
        })
        .map(str::to_owned)
        .collect();
    if names.is_empty() || names.iter().collect::<BTreeSet<_>>().len() != names.len() {
        bail!("CLI command inventory is empty or duplicated");
    }
    Ok(names)
}

fn code_list(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("`{value}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn json_marker(text: &str, pointer: &str) -> Result<String> {
    let value: Value = serde_json::from_str(text).context("parsing emitted JSON")?;
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .with_context(|| format!("emitted JSON has no string marker at {pointer}"))
}

/// These generated YAML artifacts have an unquoted, top-level version header. This deliberately
/// reads only that emitted header; it is not a YAML validator or a support proof for the document.
fn yaml_header(text: &str, key: &str) -> Result<String> {
    let prefix = format!("{key}: ");
    let values: Vec<_> = text
        .lines()
        .filter_map(|line| line.strip_prefix(&prefix))
        .collect();
    let [value] = values.as_slice() else {
        bail!("emitted YAML needs one {key} version header");
    };
    if value.is_empty()
        || !value
            .chars()
            .all(|character| character.is_ascii_digit() || character == '.')
    {
        bail!("emitted YAML has an unsupported {key} version header {value:?}");
    }
    Ok((*value).to_owned())
}

fn artifact<'a>(artifacts: &'a BTreeMap<String, String>, path: &str) -> Result<&'a str> {
    artifacts
        .get(path)
        .map(String::as_str)
        .with_context(|| format!("CLI artifact map is missing {path}"))
}

fn site(artifacts: &BTreeMap<String, String>, prefix: &str) -> Result<()> {
    let page = artifact(artifacts, &format!("{prefix}index.html"))?;
    if !page.starts_with("<!DOCTYPE html>")
        || !page.contains("assets/style.css")
        || !page.contains("assets/mermaid.min.js")
    {
        bail!("{prefix}index.html is not HTML with local stylesheet/Mermaid references");
    }
    for path in ["assets/style.css", "assets/mermaid.min.js"] {
        if artifact(artifacts, &format!("{prefix}{path}"))?.is_empty() {
            bail!("empty site asset {prefix}{path}");
        }
    }
    Ok(())
}

fn projection_version(
    artifacts: &BTreeMap<String, String>,
    directory: &str,
    marker: &str,
    json: bool,
) -> Result<String> {
    let prefix = format!("{directory}/");
    let mut versions = BTreeSet::new();
    for (path, contents) in artifacts
        .iter()
        .filter(|(path, _)| path.starts_with(&prefix))
    {
        let version = if json {
            json_marker(contents, marker)
        } else {
            yaml_header(contents, marker)
        };
        versions.insert(version.with_context(|| format!("reading {path}"))?);
    }
    if versions.len() != 1 {
        bail!("{directory} needs nonempty artifacts with one agreed version; got {versions:?}");
    }
    Ok(versions.into_iter().next().expect("one version"))
}

fn source(label: &str, path: &str) -> String {
    format!("[{label}](https://github.com/beyond10x/ess/blob/main/{path})")
}

fn row(output: &mut String, name: &str, capability: &str, boundary: &str) {
    writeln!(output, "| {name} | {capability} | {boundary} |").expect("write to string");
}

struct ProjectionFacts {
    names: Vec<String>,
    docs_format: String,
    schema: String,
    openapi: String,
    asyncapi: String,
}

fn observe_projections(root: &Path) -> Result<ProjectionFacts> {
    let generated = crate::projections(root, Path::new(crate::NORMATIVE_EXAMPLE))?;
    let names: Vec<_> = generated
        .projections
        .iter()
        .map(|projection| projection.name.clone())
        .collect();
    for projection in &generated.projections {
        if !generated
            .artifacts
            .keys()
            .any(|path| path.starts_with(&format!("{}/", projection.directory)))
        {
            bail!(
                "registered projection {} produced no artifacts",
                projection.name
            );
        }
    }
    let docs =
        crate::projection_artifacts(root, Path::new(crate::NORMATIVE_EXAMPLE), Some("docs"))?;
    if !artifact(&docs, "docs/index.md")?.contains("# ")
        || !docs.values().any(|text| text.contains("```mermaid"))
    {
        bail!("explicit docs did not emit Markdown with Mermaid diagrams");
    }
    let explicit_site =
        crate::projection_artifacts(root, Path::new(crate::NORMATIVE_EXAMPLE), Some("site"))?;
    site(&explicit_site, "")?;
    site(&generated.artifacts, "site/")?;
    let docs_ir =
        crate::projection_artifacts(root, Path::new(crate::NORMATIVE_EXAMPLE), Some("docs-ir"))?;
    if docs_ir.len() != 1 {
        bail!("docs-ir must emit exactly its document artifact");
    }
    let docs_format = json_marker(artifact(&docs_ir, "docs-ir/document.json")?, "/format")?;
    let schema = projection_version(&generated.artifacts, "schema", "/$schema", true)?;
    let openapi = projection_version(&generated.artifacts, "openapi", "openapi", false)?;
    let asyncapi = projection_version(&generated.artifacts, "asyncapi", "asyncapi", false)?;
    let kinds = choices(&cli(root, &["generate", "--help"])?, "--kind")?;
    let mut expected_kinds = names.clone();
    expected_kinds.push("docs-ir".to_owned());
    if kinds.iter().collect::<BTreeSet<_>>() != expected_kinds.iter().collect::<BTreeSet<_>>() {
        bail!("CLI projection choices disagree with registered defaults plus explicit docs-ir");
    }
    Ok(ProjectionFacts {
        names,
        docs_format,
        schema,
        openapi,
        asyncapi,
    })
}

struct ConformanceFacts {
    suite_default: String,
    suite_choices: Vec<String>,
    report_default: String,
    report_choices: Vec<String>,
    reference_targets: Vec<String>,
    default_suite: String,
    default_report: String,
    run_format: String,
    report_format: String,
    coverage_suite: String,
}

fn observe_conformance(root: &Path) -> Result<ConformanceFacts> {
    let suite_help = cli(root, &["verify", "conform", "synthesize", "--help"])?;
    let run_help = cli(root, &["verify", "conform", "run", "--help"])?;
    let suite_default = default(&suite_help, "--suite-format")?;
    let suite_choices = choices(&suite_help, "--suite-format")?;
    let report_default = default(&run_help, "--report-format")?;
    let report_choices = choices(&run_help, "--report-format")?;
    let reference_targets = choices(&run_help, "--target")?;

    // Standalone report/1 has no stdout mode. Retain this small actual CLI output under target;
    // unique, exclusive directory creation prevents concurrent checks overwriting each other.
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let parent = root.join("target/ess-support-check");
    fs::create_dir_all(&parent)?;
    let scratch = parent.join(format!("{}-{nonce}", std::process::id()));
    fs::create_dir(&scratch)?;
    let report_path = scratch.join("report.json");
    let mut args: Vec<OsString> = [
        "verify",
        "conform",
        "run",
        "--path",
        crate::NORMATIVE_EXAMPLE,
        "--target",
        "billing",
        "--format",
        "json",
        "--report-out",
    ]
    .into_iter()
    .map(Into::into)
    .collect();
    args.push(report_path.into_os_string());
    let default_run = String::from_utf8(crate::cli_output(root, &args)?)?;
    let default_suite = json_marker(&default_run, "/suite/suite_version")?;
    let default_report = json_marker(&fs::read_to_string(scratch.join("report.json"))?, "/format")?;
    let detailed = cli(
        root,
        &[
            "verify",
            "conform",
            "run",
            "--path",
            crate::NORMATIVE_EXAMPLE,
            "--target",
            "billing",
            "--report-format",
            "2",
            "--format",
            "json",
        ],
    )?;
    let run_format = json_marker(&detailed, "/format")?;
    let report_format = json_marker(&detailed, "/summary/format")?;
    let coverage = cli(
        root,
        &[
            "verify",
            "conform",
            "run",
            "--path",
            crate::NORMATIVE_EXAMPLE,
            "--target",
            "billing",
            "--suite-format",
            "5",
            "--report-format",
            "2",
            "--format",
            "json",
        ],
    )?;
    let coverage_suite = json_marker(&coverage, "/summary/suite/version")?;

    Ok(ConformanceFacts {
        suite_default,
        suite_choices,
        report_default,
        report_choices,
        reference_targets,
        default_suite,
        default_report,
        run_format,
        report_format,
        coverage_suite,
    })
}

/// Observe this separately invoked projection without executing its generated application.
fn observe_cli_binding(root: &Path) -> Result<(String, String, String)> {
    let specify = cli(root, &["specify", "cli", "--help"])?;
    let generate = cli(root, &["generate", "cli", "--help"])?;
    for help in [&specify, &generate] {
        for flag in ["--path", "--binding", "--format"] {
            option(help, flag)?;
        }
    }
    for flag in ["--out", "--check"] {
        option(&generate, flag)?;
    }
    if choices(&specify, "--format")? != choices(&generate, "--format")? {
        bail!("CLI binding routes disagree on their output format choices");
    }
    let inputs = [
        "--path",
        "crates/generate/ess-cli-project/tests/fixtures/model.yaml",
        "--binding",
        "crates/generate/ess-cli-project/tests/fixtures/cli.yaml",
        "--format",
        "json",
    ];
    let mut args = vec!["specify", "cli"];
    args.extend(inputs);
    let compiled = cli(root, &args)?;
    let plan_format = json_marker(&compiled, "/format")?;

    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let parent = root.join("target/ess-support-check");
    fs::create_dir_all(&parent)?;
    let scratch = parent.join(format!("cli-{}-{nonce}", std::process::id()));
    fs::create_dir(&scratch)?;
    let destination = scratch.join("package");
    let out = destination.to_str().context("UTF-8 CLI output path")?;
    let mut args = vec!["generate", "cli"];
    args.extend(inputs);
    args.extend(["--out", out]);
    let report = cli(root, &args)?;
    let report_format = json_marker(&report, "/format")?;
    let report: Value = serde_json::from_str(&report)?;
    let files = report["files"].as_array().context("CLI generation files")?;
    for name in [
        "Cargo.toml",
        "binding.json",
        "manifest.json",
        "src/lib.rs",
        "src/main.rs",
        "src/runtime.rs",
        "src/wire.rs",
        "help.txt",
        "README.md",
        "completions/demo.bash",
    ] {
        if !files.contains(&Value::String(name.into()))
            || fs::read(destination.join(name))?.is_empty()
        {
            bail!("CLI projection did not emit its declared {name} artifact");
        }
    }
    let written_binding: Value =
        serde_json::from_str(&fs::read_to_string(destination.join("binding.json"))?)?;
    if written_binding != serde_json::from_str::<Value>(&compiled)? {
        bail!("CLI projection changed the validated binding document");
    }
    let manifest_format = json_marker(
        &fs::read_to_string(destination.join("manifest.json"))?,
        "/format",
    )?;
    args.push("--check");
    let checked: Value = serde_json::from_str(&cli(root, &args)?)?;
    if report["checked"] != false
        || checked["checked"] != true
        || checked["files"] != report["files"]
    {
        bail!("CLI generation and drift-check inventories disagree");
    }
    Ok((plan_format, manifest_format, report_format))
}

fn render(root: &Path) -> Result<String> {
    let manifest = fs::read_to_string(root.join("Cargo.toml"))?;
    let version = crate::workspace_version(&manifest)
        .context("Cargo.toml has no workspace package version")?;
    let ProjectionFacts {
        names,
        docs_format,
        schema,
        openapi,
        asyncapi,
    } = observe_projections(root)?;
    let imports = commands(&cli(root, &["infra", "import", "--help"])?)?;
    let projects = commands(&cli(root, &["generate", "project", "--help"])?)?;
    let targets = choices(
        &cli(root, &["generate", "synthesize", "--help"])?,
        "--target",
    )?;
    let schema_commands = commands(&cli(root, &["generate", "schema", "--help"])?)?;
    let (cli_plan, cli_artifacts, cli_generation) = observe_cli_binding(root)?;
    let ConformanceFacts {
        suite_default,
        suite_choices,
        report_default,
        report_choices,
        reference_targets,
        default_suite,
        default_report,
        run_format,
        report_format,
        coverage_suite,
    } = observe_conformance(root)?;

    let mut output = format!("{BEGIN}\n\nThe source checkout’s workspace version is `{version}` and includes separately documented unreleased changes.\n\n| Capability | Current source | Limits and evidence |\n|---|---|---|\n");
    row(&mut output, "Default projections", &code_list(&names), &format!("Generator inventory and actual CLI artifacts; {}. `docs-ir` is an additional explicit choice.", source("ess-gen", "crates/generate/ess-gen/src/lib.rs")));
    row(&mut output, "Documentation", "`docs`: Markdown with Mermaid diagrams", &format!("A projection of the document model; {} establishes rendering, not implementation behavior.", source("docs emitter", "crates/generate/ess-gen/src/docs.rs")));
    row(&mut output, "Site", "`site`: HTML and local stylesheet/Mermaid assets; explicit output at `index.html` and `assets/`, combined output under `site/`", &format!("Explicit authored pages and downloads are supported; ESS does not host the site. {}.", source("authored-site tests", "crates/edge/ess-cli/tests/authored_site.rs")));
    row(
        &mut output,
        "Document IR",
        &format!("Explicit `docs-ir`: `docs-ir/document.json`, `{docs_format}`"),
        &format!(
            "A document projection, not HTML or a general persisted EssIr reader. {}.",
            source(
                "document emitter",
                "crates/generate/ess-gen/src/document.rs"
            )
        ),
    );
    row(&mut output, "JSON Schema", &format!("`{schema}`"), &format!("Named types, entities, command inputs, events and errors; structural validation does not establish behavior. {}.", source("schema emitter/tests", "crates/generate/ess-gen/src/schema.rs")));
    row(
        &mut output,
        "Native API projections",
        &format!("OpenAPI `{openapi}`; AsyncAPI `{asyncapi}`"),
        &format!(
            "Projection directions; {} and {} own their structural coverage.",
            source(
                "OpenAPI emitter/tests",
                "crates/generate/ess-gen/src/openapi.rs"
            ),
            source(
                "AsyncAPI emitter/tests",
                "crates/generate/ess-gen/src/asyncapi.rs"
            )
        ),
    );
    row(&mut output, "Adapter directions", &format!("`infra import`: {}; `generate project`: {}", code_list(&imports), code_list(&projects)), "Availability comes from CLI help. No AsyncAPI importer is declared; the following rows qualify each adapter.");
    row(&mut output, "OpenAPI adapter", "Supported 3.1 service/interface import to `ess-openapi-import/1`, retaining source and accounting; checked projection", &format!("External references refuse. Semantic gaps, unresolved references and legacy interface-only inputs block checked projection; annotation normalization alone may be allowed. {} and {}.", source("accounting tests", "crates/generate/ess-openapi/tests/accounting.rs"), source("import/refusal owner", "crates/generate/ess-openapi/src/lib.rs")));
    row(&mut output, "Kubernetes import", "Sanitized observation bundle or explicit live context to infrastructure IR", &format!("The live scanner is the credential edge. A fixed category list and empty `coverage_gaps` do not prove complete observation. {}.", source("import/redaction owner", "crates/infra/ess-kubernetes/src/lib.rs")));
    row(&mut output, "Kubernetes projection", "Intent plus observed IR to patches, new objects and obligations", &format!("No apply operation; unstated decisions remain obligations and unsupported conditions may refuse. {}.", source("projection/refusal tests", "crates/infra/infra-project/tests/projection.rs")));
    row(&mut output, "BuildKit and Helm projection", "Checked build IR to Dockerfile/Bake inputs; runtime IR to a configuration-neutral Helm chart", &format!("These projections neither execute BuildKit nor apply a chart or establish live resource availability. {}.", source("deployment projection tests", "crates/generate/ess-deployment/tests/deployment.rs")));
    row(&mut output, "Structural synthesis", &code_list(&targets), &format!("Generated structure plus obligations/refusals, not business behavior. All four full targets refuse modeled Binary64; separate structural data libraries have their own support boundary. {} and [synthesis guide](../guides/synthesize.md).", source("feasibility tests", "crates/generate/ess-synth/tests/feasibility.rs")));
    row(&mut output, "Clap synthesis", "Command grammar, completion support and handler seams receiving `clap::ArgMatches`; generated `clap` and `clap_complete` 4 dependencies", &format!("No additional type layer or implemented command behavior. {} and {}.", source("Clap emitter", "crates/generate/ess-synth/src/clap/mod.rs"), source("handler tests", "crates/generate/ess-synth/tests/clap.rs")));
    row(&mut output, "Typed CLI presentation", &format!("`specify cli` validates `ess-cli/1` to `{cli_plan}`; `generate cli` emits a Rust/Clap package, help, Bash completion and reference with `{cli_artifacts}` and `{cli_generation}`; `--check` compares generated bytes"), &format!("Typed inputs, results and declared errors remain model-owned; unsupported types and invariants refuse. Process context is separate from payloads. Application behavior requires `Handler`, dynamic native validation requires `DynamicValidator`, and the generated default handler is unavailable. {} and {}.", source("binding admission tests", "crates/specify/ess-cli-contract/tests/binding.rs"), source("projection and process tests", "crates/generate/ess-cli-project/tests/projection.rs")));
    row(&mut output, "Conformance targets", &code_list(&reference_targets), "Built-in reference implementations. A production adapter must establish its own execution boundary; these targets do not prove independent deployment.");
    row(&mut output, "Conformance formats", &format!("Defaults: `{default_suite}`, `{default_report}`. Explicit count surfaces: `{report_format}`, `{run_format}`. CLI suite choices: {} (default `{suite_default}`); report choices: {} (default `{report_default}`).", code_list(&suite_choices), code_list(&report_choices)), &format!("Actual report markers and CLI metadata; all-pass legacy execution can still mean inconclusive conformance. {}.", source("count-report tests", "crates/edge/ess-cli/tests/count_reports.rs")));
    row(&mut output, "Coverage qualification", &format!("Current-source `{coverage_suite}` requires explicit report/2 before execution"), &format!("Only a nonempty all-pass selection with complete inventory and no in-scope refusal can qualify. Suite/5, carrier and paired replay are unreleased relative to the dated 0.20.0 observation. {} and [conformance guide](../guides/verify-conformance.md#opt-into-declared-coverage).", source("coverage CLI tests", "crates/edge/ess-cli/tests/coverage_cli.rs")));
    row(&mut output, "Browser conformance", "Replay presentation with no execution report", &format!("A green replay is not independent execution evidence; digest comparison does not authenticate the publisher. {}.", source("browser admission tests", "crates/edge/ess-cli/tests/coverage_browser.rs")));
    row(
        &mut output,
        "Runtime compilation",
        "Checks supplied identities, component coverage, replica bounds and stateful storage",
        &format!(
            "Does not establish live provisioning or all resource requirements. {}.",
            source(
                "runtime checks/tests",
                "crates/generate/ess-deployment/src/runtime.rs"
            )
        ),
    );
    row(&mut output, "Explicit executors", "`execute`, `publish`, `fetch`, `reconcile` invoke external clients; reconciliation requires `--authority` naming a protected registry entry and refuses without one, compares an admitted baseline desired deployment with the desired one, and attempts at most one admitted mutation per release", &format!("Caller-supplied state, authority and credentials remain material; a supplied baseline is admitted intent rather than proof of application, and a stopped invocation leaves the affected release unknown rather than absent or rolled back. {}. The support check invokes none of these verbs.", source("CLI executor owner", "crates/edge/ess-cli/src/main.rs")));
    row(&mut output, "Schema commands", &code_list(&schema_commands), "Current-source command inventory for import, validation, types and normalization; [CLI reference](../reference/cli.md#adopter-owned-schema-contracts) describes the selected operations. Availability is independent of the dated release record.");
    write!(output, "\n{END}")?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser as _;

    fn expected(version: &str) -> String {
        format!(
            "{BEGIN}\nSource version: {version}\n\n\
             | Capability | Source |\n|---|---|\n\
             | Site | HTML |\n| Synthesis | rust, go, web, clap |\n\
             | OpenAPI | import, project |\n{END}"
        )
    }

    #[test]
    fn help_inventory_reads_both_clap_layouts_without_swallowing_neighbor_options() {
        let inline = "Options:\n      --target <TARGET>  [default: rust] [possible values: rust, go, web, clap]\n      --format <FORMAT>  [possible values: text, json]\n";
        assert_eq!(
            choices(inline, "--target").unwrap(),
            ["rust", "go", "web", "clap"]
        );
        assert_eq!(default(inline, "--target").unwrap(), "rust");
        let multiline = "Options:\n      --kind <KIND>\n          Possible values:\n          - docs\n          - site: HTML and assets\n          - docs-ir: document JSON\n          [default: docs]\n      --other <OTHER>\n          [possible values: unrelated]\n";
        assert_eq!(
            choices(multiline, "--kind").unwrap(),
            ["docs", "site", "docs-ir"]
        );
        assert_eq!(default(multiline, "--kind").unwrap(), "docs");
        assert!(choices(inline, "--absent").is_err());
        assert!(choices("--target <TARGET>\nNo values", "--target").is_err());
        assert!(choices(
            "--target <TARGET> [possible values: rust, rust]",
            "--target"
        )
        .is_err());
        assert!(choices("--target <TARGET> [possible values: ]", "--target").is_err());
        assert!(default("--target <TARGET>", "--target").is_err());
    }

    #[test]
    fn command_inventory_requires_a_complete_nonempty_unique_section() {
        let help = "Usage: ess infra import <COMMAND>\n\nCommands:\n  kubernetes  Sanitized import\n  openapi     Supported subset\n\nOptions:\n  -h, --help  Print help\n";
        assert_eq!(commands(help).unwrap(), ["kubernetes", "openapi"]);
        assert!(commands(&help.replace("  openapi", "  kubernetes")).is_err());
        assert!(commands("Commands:\n\nOptions:\n").is_err());
        assert!(commands("Commands:\n  openapi\n").is_err());
    }

    #[test]
    fn emitted_version_markers_are_parsed_and_must_agree_across_the_projection() {
        assert_eq!(
            json_marker(r#"{"format":"ess-docs/1"}"#, "/format").unwrap(),
            "ess-docs/1"
        );
        for invalid in ["ess-docs/1", r#"{"other":"ess-docs/1"}"#, r#"{"format":1}"#] {
            assert!(json_marker(invalid, "/format").is_err());
        }
        assert_eq!(
            yaml_header(
                "# comment\nopenapi: 3.1.0\ninfo:\n  version: 1\n",
                "openapi"
            )
            .unwrap(),
            "3.1.0"
        );
        for invalid in [
            "  openapi: 3.1.0",
            "openapi: wrong",
            "openapi: 3.1.0\nopenapi: 3.1.0",
        ] {
            assert!(yaml_header(invalid, "openapi").is_err());
        }
        let mut artifacts =
            BTreeMap::from([("openapi/one.yaml".to_owned(), "openapi: 3.1.0".to_owned())]);
        assert_eq!(
            projection_version(&artifacts, "openapi", "openapi", false).unwrap(),
            "3.1.0"
        );
        artifacts.insert("openapi/two.yaml".to_owned(), "openapi: 3.0.0".to_owned());
        assert!(projection_version(&artifacts, "openapi", "openapi", false).is_err());
        assert!(projection_version(&BTreeMap::new(), "openapi", "openapi", false).is_err());
    }

    #[test]
    fn html_requires_the_actual_output_root_and_both_nonempty_local_assets() {
        let artifacts = BTreeMap::from([
            ("index.html".to_owned(), "<!DOCTYPE html><link href=\"assets/style.css\"><script src=\"assets/mermaid.min.js\">".to_owned()),
            ("assets/style.css".to_owned(), "body {}".to_owned()),
            ("assets/mermaid.min.js".to_owned(), "mermaid".to_owned()),
        ]);
        site(&artifacts, "").unwrap();
        assert!(site(&artifacts, "site/").is_err());
        for path in artifacts.keys() {
            let mut missing = artifacts.clone();
            missing.remove(path);
            assert!(site(&missing, "").is_err());
            let mut empty = artifacts.clone();
            empty.insert(path.clone(), String::new());
            assert!(site(&empty, "").is_err());
        }
        let combined = artifacts
            .into_iter()
            .map(|(path, text)| (format!("site/{path}"), text))
            .collect();
        site(&combined, "site/").unwrap();
    }

    #[test]
    fn support_check_is_an_available_maintenance_command() {
        let parsed = crate::Cli::try_parse_from(["ess-xtask", "support", "--check"]);
        assert!(parsed.is_ok(), "{parsed:?}");
    }

    #[test]
    fn a_complete_source_block_is_accepted_without_owning_release_prose() {
        let expected = expected("1.2.3");
        for release in ["Observed release: 0.20.0", "Observed release: 0.19.0"] {
            let page = format!("{release}\n\n{expected}\n\nOrdinary explanatory prose.\n");
            assert_eq!(compare(&page, &expected), Ok(()));
        }
    }

    #[test]
    fn every_material_row_change_is_refused_with_its_location() {
        let expected = expected("1.2.3");
        let changed = [
            expected.replace("| Site | HTML |", "| Site | Markdown |"),
            expected.replace("| Site | HTML |\n", ""),
            expected.replace("| Site | HTML |", "| Site | HTML |\n| Site | HTML |"),
            expected.replace(END, &format!("| Unowned | supported |\n{END}")),
            expected.replace("rust, go, web, clap", "rust, go, web"),
            expected.replace("import, project", "project"),
            expected.replace(
                "| Site | HTML |\n| Synthesis | rust, go, web, clap |",
                "| Synthesis | rust, go, web, clap |\n| Site | HTML |",
            ),
        ];
        for page in changed {
            let error =
                compare(&page, &expected).expect_err("changed public capability must refuse");
            assert!(error.contains(STATUS) && error.contains("row"), "{error}");
        }
    }

    #[test]
    fn cargo_version_changes_invalidate_only_the_source_block() {
        let old_manifest = "[workspace.package]\nversion = \"1.2.3\"\n";
        let new_manifest = "[workspace.package]\nversion = \"1.2.4\"\n";
        let old = expected(crate::workspace_version(old_manifest).unwrap());
        let new = expected(crate::workspace_version(new_manifest).unwrap());
        let page = format!("Release observed on a date: 0.20.0\n{old}");
        let error = compare(&page, &new).expect_err("stale Cargo source version must refuse");
        assert!(error.contains(STATUS) && error.contains("row"), "{error}");
        assert_eq!(compare(&page.replace(&old, &new), &new), Ok(()));
    }

    #[test]
    fn missing_duplicated_or_reordered_block_markers_refuse() {
        let expected = expected("1.2.3");
        for page in [
            expected.replace(BEGIN, ""),
            expected.replace(END, ""),
            format!("{expected}\n{expected}"),
            format!("{END}\n{BEGIN}"),
        ] {
            assert!(compare(&page, &expected).is_err(), "{page}");
        }
    }

    fn adversary_fixture(label: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "ess-support-adversary-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        fs::write(
            path.join("system.yaml"),
            "format: ess/1\nsystem: publication\nversion: v1\ndomains: []\n",
        )
        .unwrap();
        println!("retained fixture: {}", path.display());
        path
    }

    #[test]
    fn adversary_adjacent_readme_is_selected_without_authored_flags() {
        let root = crate::workspace_root().unwrap();
        let fixture = adversary_fixture("readme");
        fs::write(
            fixture.join("README.md"),
            "# Default authored front page\n\nADVERSARY_README_DEFAULT\n",
        )
        .unwrap();
        fs::write(fixture.join("sibling.md"), "UNSELECTED_SIBLING").unwrap();
        fs::write(fixture.join("download.json"), "UNSELECTED_DOWNLOAD").unwrap();
        let args: Vec<OsString> = ["generate", "--kind", "site", "--format", "json", "--path"]
            .into_iter()
            .map(Into::into)
            .chain(std::iter::once(fixture.as_os_str().to_owned()))
            .collect();
        fs::write(fixture.join("argv.txt"), format!("{args:?}\n")).unwrap();
        let output = crate::cli_output(&root, &args).unwrap();
        fs::write(fixture.join("actual-cli-stdout.json"), &output).unwrap();
        let report: serde_json::Value = serde_json::from_slice(&output).unwrap();
        let index = report["site/index.html"]["contents"].as_str().unwrap();
        assert!(index.contains("ADVERSARY_README_DEFAULT"));
        let all = String::from_utf8(output).unwrap();
        assert!(!all.contains("UNSELECTED_SIBLING"));
        assert!(!all.contains("UNSELECTED_DOWNLOAD"));
        println!("actual CLI included adjacent README in index.html without --front-page, --include or --asset; siblings and downloads stayed absent");
    }

    #[test]
    fn adversary_actual_explicit_site_and_combined_maps_keep_distinct_roots() {
        let root = crate::workspace_root().unwrap();
        let fixture = adversary_fixture("roots");
        let explicit = crate::projection_artifacts(&root, &fixture, Some("site")).unwrap();
        let combined = crate::projection_artifacts(&root, &fixture, None).unwrap();
        fs::write(
            fixture.join("observed-maps.json"),
            serde_json::to_vec_pretty(&(&explicit, &combined)).unwrap(),
        )
        .unwrap();
        assert!(explicit.contains_key("index.html"));
        assert!(!explicit.keys().any(|path| path.starts_with("site/")));
        assert!(combined.contains_key("site/index.html"));
        assert!(!combined.contains_key("index.html"));
        for path in explicit.keys() {
            assert!(combined.contains_key(&format!("site/{path}")), "{path}");
        }
        site(&explicit, "").unwrap();
        site(&combined, "site/").unwrap();
        assert!(site(&explicit, "site/").is_err());
        assert!(site(&combined, "").is_err());
        assert!(!combined.keys().any(|path| path.starts_with("docs-ir/")));
    }

    #[test]
    fn adversary_all_real_material_rows_refuse_cell_removal_duplicate_and_order_drift() {
        let root = crate::workspace_root().unwrap();
        let expected = render(&root).unwrap();
        let page = fs::read_to_string(root.join(STATUS)).unwrap();
        assert_eq!(compare(&page, &expected), Ok(()));
        let fixture = adversary_fixture("rows");
        fs::write(fixture.join("actual-expected.md"), &expected).unwrap();
        let lines: Vec<_> = expected.lines().map(str::to_owned).collect();
        let rows: Vec<_> = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.starts_with("| ") && !line.starts_with("| Capability |"))
            .map(|(index, _)| index)
            .collect();
        assert_eq!(rows.len(), 21);
        let mut refused = 0;
        for &index in &rows {
            let cells: Vec<_> = lines[index].split('|').map(str::to_owned).collect();
            for cell in 1..cells.len() - 1 {
                let mut changed = cells.clone();
                changed[cell].push_str(" CHANGED ");
                let mut mutant = lines.clone();
                mutant[index] = changed.join("|");
                let error = compare(&mutant.join("\n"), &expected).unwrap_err();
                assert!(error.contains(STATUS) && error.contains("row"), "{error}");
                refused += 1;
            }
            let mut removed = lines.clone();
            removed.remove(index);
            assert!(compare(&removed.join("\n"), &expected).is_err());
            let mut duplicate = lines.clone();
            duplicate.insert(index, lines[index].clone());
            assert!(compare(&duplicate.join("\n"), &expected).is_err());
            refused += 2;
        }
        for pair in rows.windows(2) {
            let mut reordered = lines.clone();
            reordered.swap(pair[0], pair[1]);
            assert!(compare(&reordered.join("\n"), &expected).is_err());
            refused += 1;
        }
        let extra = expected.replace(END, &format!("| Extra | supported | unowned |\n{END}"));
        assert!(compare(&extra, &expected).is_err());
        println!(
            "all {} actual rows attacked: {} mutation refusals",
            rows.len(),
            refused + 1
        );
    }

    #[test]
    fn adversary_real_source_version_drift_keeps_release_bytes_independent() {
        let root = crate::workspace_root().unwrap();
        let manifest = fs::read_to_string(root.join("Cargo.toml")).unwrap();
        let current = crate::workspace_version(&manifest).unwrap();
        let changed = manifest.replacen(
            &format!("version = \"{current}\""),
            "version = \"99.1.2\"",
            1,
        );
        assert_eq!(crate::workspace_version(&changed), Some("99.1.2"));
        let page = fs::read_to_string(root.join(STATUS)).unwrap();
        let start = page.find(BEGIN).unwrap();
        let end = page.find(END).unwrap() + END.len();
        let old = &page[start..end];
        let new = old.replacen(current, "99.1.2", 1);
        assert_ne!(old, new);
        assert!(compare(&page, &new).is_err());
        let replaced = format!("{}{}{}", &page[..start], new, &page[end..]);
        assert_eq!(compare(&replaced, &new), Ok(()));
        assert!(replaced.starts_with(&page[..start]));
        assert!(replaced.ends_with(&page[end..]));
    }

    #[test]
    fn adversary_actual_help_missing_target_metadata_cannot_borrow_neighbor_values() {
        let root = crate::workspace_root().unwrap();
        let help = cli(&root, &["generate", "synthesize", "--help"]).unwrap();
        assert_eq!(
            choices(&help, "--target").unwrap(),
            ["rust", "go", "web", "clap"]
        );
        let block = option(&help, "--target").unwrap().join("\n");
        let missing = help.replace(
            &block,
            "      --target <TARGET>\n          metadata removed",
        );
        assert!(choices(&missing, "--target").is_err(), "{missing}");
        let fixture = adversary_fixture("metadata");
        fs::write(fixture.join("actual-help.txt"), help).unwrap();
        fs::write(fixture.join("missing-metadata-help.txt"), missing).unwrap();
    }

    #[test]
    fn adversary_actual_cli_refusal_is_not_a_successful_support_observation() {
        let root = crate::workspace_root().unwrap();
        let fixture = adversary_fixture("refusal");
        let failure = crate::projection_artifacts(&root, &fixture, Some("unregistered-kind"))
            .expect_err("a refused CLI cannot supply projection facts");
        let diagnostic = format!("{failure:#}");
        fs::write(fixture.join("actual-refusal.txt"), &diagnostic).unwrap();
        assert!(diagnostic.contains("refused") && diagnostic.contains("unregistered-kind"));
        let failure = crate::projection_artifacts(&root, &fixture.join("missing"), Some("docs"))
            .expect_err("an absent specification cannot supply projection facts");
        fs::write(
            fixture.join("missing-input-refusal.txt"),
            format!("{failure:#}"),
        )
        .unwrap();
    }

    #[test]
    fn adversary_real_docs_ir_marker_is_nested_json_not_visible_marker_text() {
        let root = crate::workspace_root().unwrap();
        let fixture = adversary_fixture("docs-ir");
        let artifacts = crate::projection_artifacts(&root, &fixture, Some("docs-ir")).unwrap();
        assert_eq!(artifacts.len(), 1);
        let contents = artifacts.get("docs-ir/document.json").unwrap();
        fs::write(fixture.join("actual-document.json"), contents).unwrap();
        assert_eq!(json_marker(contents, "/format").unwrap(), "ess-docs/1");
        let mut document: serde_json::Value = serde_json::from_str(contents).unwrap();
        document.as_object_mut().unwrap().remove("format");
        document["description"] = "ess-docs/1".into();
        assert!(json_marker(&document.to_string(), "/format").is_err());
        document["format"] = serde_json::json!({"nested": "ess-docs/1"});
        assert!(json_marker(&document.to_string(), "/format").is_err());
    }

    #[test]
    fn adversary_front_page_override_replaces_adjacent_readme_for_a_specification_file() {
        let root = crate::workspace_root().unwrap();
        let fixture = adversary_fixture("front-page-override");
        fs::write(
            fixture.join("README.md"),
            "# Read first\n\nADJACENT_README_DEFAULT\n",
        )
        .unwrap();
        fs::write(
            fixture.join("front.md"),
            "# Replacement\n\nEXPLICIT_FRONT_OVERRIDE\n",
        )
        .unwrap();
        let mut args: Vec<OsString> = ["generate", "--kind", "site", "--format", "json", "--path"]
            .into_iter()
            .map(Into::into)
            .chain(std::iter::once(
                fixture.join("system.yaml").into_os_string(),
            ))
            .collect();
        fs::write(fixture.join("default-argv.txt"), format!("{args:?}\n")).unwrap();
        let default_output = crate::cli_output(&root, &args).unwrap();
        fs::write(fixture.join("default-stdout.json"), &default_output).unwrap();
        let default_report: serde_json::Value = serde_json::from_slice(&default_output).unwrap();
        let default_index = default_report["site/index.html"]["contents"]
            .as_str()
            .unwrap();
        assert!(default_index.contains("ADJACENT_README_DEFAULT"));
        assert!(!default_index.contains("EXPLICIT_FRONT_OVERRIDE"));

        args.extend([
            "--front-page".into(),
            fixture.join("front.md").into_os_string(),
        ]);
        fs::write(fixture.join("override-argv.txt"), format!("{args:?}\n")).unwrap();
        let override_output = crate::cli_output(&root, &args).unwrap();
        fs::write(fixture.join("override-stdout.json"), &override_output).unwrap();
        let override_report: serde_json::Value = serde_json::from_slice(&override_output).unwrap();
        let override_index = override_report["site/index.html"]["contents"]
            .as_str()
            .unwrap();
        assert!(override_index.contains("EXPLICIT_FRONT_OVERRIDE"));
        assert!(!override_index.contains("ADJACENT_README_DEFAULT"));

        *args.last_mut().unwrap() = fixture.join("missing.md").into_os_string();
        let refused = crate::cli_output(&root, &args)
            .expect_err("an absent explicit front page must not silently fall back to README");
        fs::write(
            fixture.join("missing-override-refusal.txt"),
            format!("{refused:#}"),
        )
        .unwrap();
        println!("specification-file path uses adjacent README by default, explicit front page replaces it, and an absent explicit front page refuses");
    }
}
