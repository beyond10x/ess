//! Stage 1: closed extraction and unaccepted candidate accounting.
mod account;
mod consumer;
mod enforce;
mod executor;
mod metadata;
mod native;
mod proposal;
mod rust;
#[cfg(test)]
mod tests;
mod wire;

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path, process::Command};

fn hash_bytes(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .flat_map(|b| {
            [
                char::from(b"0123456789abcdef"[usize::from(b >> 4)]),
                char::from(b"0123456789abcdef"[usize::from(b & 15)]),
            ]
        })
        .collect()
}
fn hash_json(value: &Value) -> String {
    hash_bytes(&serde_json::to_vec(value).expect("JSON values serialize"))
}
#[cfg(test)]
fn model(source: &str) -> Result<Value> {
    rust::fixture(source)
}
#[cfg(test)]
fn wire(schema: &Value) -> Result<Value> {
    wire::extract(schema)
}
#[cfg(test)]
fn eligibility(models: &Value, consumers: &Value, cells: &Value) -> Result<()> {
    account::checkpoint(models, consumers, cells).map(|_| ())
}

pub(super) fn run(root: &Path, output: &Path) -> Result<String> {
    let files = source_files(root)?;
    let compiled: BTreeMap<String, String> = serde_json::from_str(include_str!(concat!(
        env!("OUT_DIR"),
        "/consumer-source.json"
    )))?;
    if files != compiled {
        bail!("compiled RawSpecFile provider/source authority mismatch; rebuild the same complete source before extraction");
    }
    let sources = files
        .keys()
        .map(|p| Ok((p.clone(), fs::read(root.join(p))?)))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .filter_map(|(p, b)| String::from_utf8(b).ok().map(|s| (p, s)))
        .collect::<BTreeMap<_, _>>();
    let build: Value = serde_json::from_str(include_str!(concat!(
        env!("OUT_DIR"),
        "/consumer-build.json"
    )))?;
    validate_build(
        &build,
        |key| std::env::var(key).ok(),
        |path| Ok(fs::read(path)?),
    )?;
    let current_invocation = invocation(root, &build)?;
    let cargo = build["tools"]["CARGO"]["path"]
        .as_str()
        .context("compiled Cargo path")?;
    let metadata = Command::new(cargo)
        .args([
            "metadata",
            "--locked",
            "--offline",
            "--no-deps",
            "--format-version",
            "1",
        ])
        .current_dir(root)
        .output()?;
    if !metadata.status.success() {
        bail!(
            "Cargo consumer metadata failed: {}",
            String::from_utf8_lossy(&metadata.stderr)
        );
    }
    let metadata: Value = serde_json::from_slice(&metadata.stdout)?;
    let mut consumers = consumer::extract(root, &sources, &metadata)?;
    let model = rust::extract(&sources)?;
    let schema = serde_json::to_value(schemars::schema_for!(ess_domain::spec::RawSpecFile))?;
    let wire = wire::extract(&schema)?;
    consumers["generated_diagnostic_symbols"] = model["diagnostic_macros"].clone();
    let executable = std::env::current_exe()?;
    let profile = json!({"stage":"source-output-checkpoint","eligibility":"UNACCEPTED","source":files,"compiled_provider_source":compiled,"provider_executable_sha256":hash_bytes(&fs::read(executable)?),"compiled_build":build,"current_invocation":current_invocation,"rust_roots":rust::ROOTS});
    fs::create_dir(output)
        .with_context(|| format!("create fresh extraction output {}", output.display()))?;
    for (name, v) in [
        ("source-profile.json", &profile),
        ("rust-inventory.json", &model),
        ("wire-inventory.json", &wire),
        ("provider-schema.json", &schema),
        ("consumer-inventory-unclassified.json", &consumers),
        ("cargo-metadata.json", &metadata),
    ] {
        write_json(&output.join(name), v)?;
    }
    let mut obligations = model["obligations"]
        .as_object()
        .context("Rust obligations")?
        .clone();
    for (id, shape) in wire["obligations"]
        .as_object()
        .context("wire obligations")?
    {
        if obligations.insert(id.clone(), shape.clone()).is_some() {
            bail!("duplicate model/wire obligation {id}");
        }
    }
    let proposal = proposal::build(&json!(obligations), &mut consumers, &files, &profile)?;
    let summary = proposal["checkpoint-summary.json"].clone();
    for (name, value) in proposal {
        write_json(&output.join(name), &value)?;
    }
    write_json(&output.join("consumer-inventory.json"), &consumers)?;
    if files != source_files(root)? {
        bail!("complete source changed during extraction; outputs are invalid");
    }
    Ok(format!("Stage 1 source/output checkpoint: {} exact unaccepted cells; zero Supported, Refused, BaselineUnknown or SchemaDocumentMetadata. Eligibility remains invalid pending root review, owners and actual case/guard qualification.\n",summary["cells"]))
}
fn write_json(path: &Path, value: &Value) -> Result<()> {
    use std::io::Write;
    let mut file = std::io::BufWriter::new(fs::File::create(path)?);
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    file.flush()?;
    Ok(())
}
fn validate_build(
    profile: &Value,
    wrapper: impl Fn(&str) -> Option<String>,
    read: impl Fn(&str) -> Result<Vec<u8>>,
) -> Result<()> {
    let environment = profile["environment"]
        .as_object()
        .context("measured compiled environment")?;
    for (key, expected) in [
        ("TARGET", "x86_64-unknown-linux-gnu"),
        ("HOST", "x86_64-unknown-linux-gnu"),
        ("CARGO_CFG_TARGET_ARCH", "x86_64"),
        ("CARGO_CFG_TARGET_OS", "linux"),
        ("CARGO_CFG_TARGET_FAMILY", "unix"),
        ("CARGO_CFG_FEATURE", ""),
        ("CARGO_ENCODED_RUSTFLAGS", "-C\u{1f}link-arg=-fuse-ld=lld"),
        ("PROFILE", "debug"),
        ("OPT_LEVEL", "0"),
        ("DEBUG", "false"),
        ("NUM_JOBS", "2"),
    ] {
        if environment.get(key) != Some(&json!(expected)) {
            bail!(
                "unsupported measured compiled profile {key}: {:?}",
                environment.get(key)
            );
        }
    }
    if environment.keys().any(|k| k.starts_with("CARGO_FEATURE_")) {
        bail!("unreviewed compiled package features");
    }
    for key in [
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
        "CARGO_BUILD_RUSTC_WRAPPER",
        "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER",
    ] {
        if environment.get(key) != Some(&json!("")) || wrapper(key).as_deref() != Some("") {
            bail!("compiled/current wrapper profile mismatch {key}");
        }
    }
    for key in ["CARGO", "RUSTC"] {
        let identity = &profile["tools"][key];
        let path = identity["path"].as_str().context("compiled tool path")?;
        if identity["sha256"] != hash_bytes(&read(path)?)
            || identity["version"]
                .as_str()
                .context("compiled tool version")?
                .split_whitespace()
                .nth(1)
                != Some("1.98.1")
        {
            bail!("compiled tool identity changed or unsupported: {key} {path}");
        }
    }
    for key in ["RUST_LLD", "LD_LLD"] {
        let identity = &profile["tools"][key];
        let path = identity["path"]
            .as_str()
            .context("missing bundled linker path")?;
        if identity["sha256"] != hash_bytes(&read(path)?) {
            bail!("bundled linker identity changed: {key} {path}");
        }
    }
    for (path, expected) in profile["cargo_configuration"]
        .as_object()
        .context("compiled Cargo configuration")?
    {
        if expected != &json!(hash_bytes(&read(path)?)) {
            bail!("Cargo configuration changed since provider build: {path}");
        }
    }
    Ok(())
}
fn source_files(root: &Path) -> Result<BTreeMap<String, String>> {
    let listed = Command::new("git")
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ])
        .current_dir(root)
        .output()?;
    if !listed.status.success() {
        bail!(
            "Git source authority refused: {}",
            String::from_utf8_lossy(&listed.stderr)
        );
    }
    listed
        .stdout
        .split(|b| *b == 0)
        .filter(|p| !p.is_empty())
        .map(|p| {
            let p = std::str::from_utf8(p)?.to_owned();
            Ok((p.clone(), hash_bytes(&fs::read(root.join(p))?)))
        })
        .collect()
}

fn validate_invocation(build: &Value, invocation: &Value) -> Result<()> {
    let env = invocation["environment"]
        .as_object()
        .context("current invocation environment")?;
    for (key, value) in env {
        let flags = matches!(
            key.as_str(),
            "RUSTFLAGS" | "CARGO_ENCODED_RUSTFLAGS" | "CARGO_BUILD_RUSTFLAGS"
        ) || (key.starts_with("CARGO_TARGET_") && key.ends_with("_RUSTFLAGS"));
        if flags {
            let value = value.as_str().context("current flags must be text")?;
            let args = if key == "CARGO_ENCODED_RUSTFLAGS" {
                value.split('\u{1f}').collect::<Vec<_>>()
            } else {
                value.split_whitespace().collect::<Vec<_>>()
            };
            if args != ["-C", "link-arg=-fuse-ld=lld"] {
                bail!("unreviewed current flags {key}");
            }
        }
        if key == "CARGO_TARGET_DIR"
            || (key == "CARGO_BUILD_TARGET" && value != &build["environment"]["TARGET"])
        {
            bail!("unreviewed current target/output selection {key}");
        }
    }
    for key in ["CARGO", "RUSTC"] {
        if invocation["tool_sha256"][key] != build["tools"][key]["sha256"] {
            bail!("current selected tool differs from compiled provider: {key}");
        }
    }
    if invocation["cargo_configuration"] != build["cargo_configuration"] {
        bail!("complete current Cargo configuration set differs from provider build");
    }
    Ok(())
}
fn invocation(root: &Path, build: &Value) -> Result<Value> {
    let environment = std::env::vars()
        .filter(|(key, _)| {
            key.starts_with("CARGO_")
                || key.starts_with("RUST")
                || matches!(key.as_str(), "CARGO" | "PATH" | "HOME" | "TMPDIR")
                || key.starts_with("XDG_")
        })
        .filter(|(key, _)| {
            !key.contains("TOKEN") && !key.contains("PASSWORD") && !key.contains("SECRET")
        })
        .collect::<BTreeMap<_, _>>();
    let mut tool_paths = BTreeMap::new();
    let mut tool_sha256 = BTreeMap::new();
    for key in ["CARGO", "RUSTC"] {
        // The provider's exact tools are the default. An explicit caller override must
        // select those same bytes; owner builds never guess another tool from PATH.
        let path = std::env::var(key).ok().unwrap_or_else(|| {
            build["tools"][key]["path"]
                .as_str()
                .unwrap_or_default()
                .into()
        });
        let path = std::path::PathBuf::from(path);
        let path = if path.components().count() == 1 {
            std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
                .map(|p| p.join(&path))
                .find(|p| p.is_file())
                .context("selected current tool not found")?
        } else {
            path
        };
        tool_sha256.insert(key, hash_bytes(&fs::read(&path)?));
        tool_paths.insert(key, path);
    }
    let cargo_home = std::env::var_os("CARGO_HOME").map_or_else(
        || std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".cargo"),
        std::path::PathBuf::from,
    );
    let mut configuration = BTreeMap::new();
    for dir in root
        .ancestors()
        .map(|p| p.join(".cargo"))
        .chain(std::iter::once(cargo_home))
    {
        for name in ["config", "config.toml"] {
            let path = dir.join(name);
            if path.is_file() {
                configuration.insert(path, hash_bytes(&fs::read(dir.join(name))?));
            }
        }
    }
    let current = json!({"environment":environment,"tool_paths":tool_paths,"tool_sha256":tool_sha256,"cargo_configuration":configuration});
    validate_invocation(build, &current)?;
    Ok(current)
}

pub(super) fn check(root: &Path, selected_output: Option<&Path>) -> Result<String> {
    let output = if let Some(path) = selected_output {
        path.to_owned()
    } else {
        let parent = root.join("target/consumer-coverage");
        fs::create_dir_all(&parent)?;
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        parent.join(format!("run-{nanos}-{}", std::process::id()))
    };
    fs::create_dir(&output).with_context(|| {
        format!(
            "consumer checker requires a fresh output directory: {}",
            output.display()
        )
    })?;
    let result = check_at(root, &output);
    if let Err(error) = &result {
        let pending_metadata = read_extraction(&output, "execution-plan.json")
            .ok()
            .and_then(|plan| plan["pending_metadata_candidates"].as_u64())
            .unwrap_or(0);
        write_json(
            &output.join("refusal.json"),
            &json!({"format":account::FORMAT,"status":"CHECK_REFUSED","error":format!("{error:#}"),"Supported":0,"Refused":0,"SchemaDocumentMetadata":0,"pending_metadata_candidates":pending_metadata,"note":"No completed qualification receipt was produced; partial case/guard evidence remains diagnostic only."}),
        )?;
    }
    result.with_context(|| format!("consumer evidence directory {}", output.display()))
}
fn reconcile_extraction(extraction: Result<String>, accounting: Result<Value>) -> Result<Value> {
    match (extraction, accounting) {
        (Ok(_), plan) => plan,
        (Err(error), accounting) => {
            let diagnostic = match accounting {
                Ok(plan) => {
                    json!({"format":account::FORMAT,"status":"PROVISIONAL_ONLY_CLASSIFICATION_REFUSED","discovered_models":plan["discovered_models"],"bound_profiles":plan["bound_profiles"],"candidate_initial_unknowns":plan["BaselineUnknown"],"pending_metadata_candidates":plan["pending_metadata_candidates"],"Supported":0,"Refused":0,"BaselineUnknown":0,"SchemaDocumentMetadata":0})
                }
                Err(error) => {
                    json!({"format":account::FORMAT,"status":"PROVISIONAL_ACCOUNTING_REFUSAL","details":error.to_string(),"SchemaDocumentMetadata":0,"no_cells_admitted":true})
                }
            };
            bail!("extraction/classification refused: {error:#}; accounting diagnostics: {diagnostic}");
        }
    }
}
fn read_extraction(extraction: &Path, name: &str) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(extraction.join(name))?)?)
}
fn extracted_accounting_inputs(extraction: &Path) -> Result<(Value, Value, Value)> {
    let rust = read_extraction(extraction, "rust-inventory.json")?;
    let wire = read_extraction(extraction, "wire-inventory.json")?;
    let mut models = rust["obligations"]
        .as_object()
        .context("Rust obligations")?
        .clone();
    for (id, shape) in wire["obligations"]
        .as_object()
        .context("wire obligations")?
    {
        if models.insert(id.clone(), shape.clone()).is_some() {
            bail!("duplicate Rust/wire obligation {id}");
        }
    }
    let (profiles, claims) = proposal::accounting_inputs(
        &read_extraction(extraction, "consumer-inventory-unclassified.json")?,
        &read_extraction(extraction, "source-profile.json")?,
    )?;
    Ok((json!(models), profiles, claims))
}
fn plan_extraction(extraction: &Path) -> Result<Value> {
    let (models, profiles, claims) = extracted_accounting_inputs(extraction)?;
    if extraction.join("unaccepted-cells.json").is_file() {
        let candidates =
            account::read_candidates(&read_extraction(extraction, "unaccepted-cells.json")?)?;
        account::check(&models, &profiles, &candidates.cells)?;
    }
    let baseline_bytes = include_bytes!("initial-baseline.json");
    if hash_bytes(baseline_bytes)
        != "3dd8dff59335c8a77c93c2734118566fd1b2d5165c0590d0aa9be397374a47de"
    {
        bail!("root-owned initial eligibility differs from accepted exact bytes");
    }
    let baseline: Value = serde_json::from_slice(baseline_bytes)?;
    let metadata = metadata::candidates(&models, &profiles)?;
    enforce::plan_with_metadata(&models, &profiles, &baseline, &claims, Some(&metadata))
}
fn check_at(root: &Path, output: &Path) -> Result<String> {
    let extraction = output.join("extraction");
    let extracted = run(root, &extraction);
    // Diagnostics use the same model, canonical fingerprint and accounting code even
    // when finite API classification refuses. Only both successful paths reach execution.
    let plan = reconcile_extraction(extracted, plan_extraction(&extraction))?;
    let read = |name: &str| read_extraction(&extraction, name);
    write_json(&output.join("execution-plan.json"), &plan)?;
    let required = serde_json::from_value(plan["required_cases"].clone())?;
    let source_profile = read("source-profile.json")?;
    let authority = metadata::Authority::capture(root, &source_profile)?;
    let (models, profiles, _) = extracted_accounting_inputs(&extraction)?;
    let metadata = metadata::execute(
        &authority,
        &read("provider-schema.json")?,
        &read("wire-inventory.json")?,
        &models,
        &profiles,
    )?;
    write_json(&output.join("metadata-guard.json"), metadata.receipt())?;
    let verified = native::execute(
        root,
        &output.join("cases"),
        &source_profile,
        &read("reviewed-case-candidates.json")?,
        &required,
        &read("cargo-metadata.json")?,
    )?;
    let result = enforce::qualify(&plan, &verified, &authority, &metadata)?;
    if json!(source_files(root)?) != source_profile["source"] {
        bail!("source changed before final consumer admission");
    }
    write_json(&output.join("qualified-cells.json"), &result)?;
    let summary = json!({"format":account::FORMAT,"stage":"qualified","discovered_models":plan["discovered_models"],"bound_profiles":plan["bound_profiles"],"cells":result["cells"].as_array().context("qualified cells")?.len(),"counts":result["counts"],"executed_cases":verified.ids().len(),"executed_metadata_guards":1,"source_sha256":hash_json(&source_profile["source"]),"provider_sha256":source_profile["provider_executable_sha256"],"output":output,"unknown_limit":"Accepted initial gaps remain unproven; schema metadata is bookkeeping, not behavioral coverage; this is not complete consumer support."});
    write_json(&output.join("summary.json"), &summary)?;
    Ok(format!("Consumer coverage: {summary}\n"))
}
