//! Stage 1: closed extraction and unaccepted candidate accounting.
mod account;
mod consumer;
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
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
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
    let build: Value = serde_json::from_str(include_str!(concat!(
        env!("OUT_DIR"),
        "/consumer-build.json"
    )))?;
    validate_build(
        &build,
        |key| std::env::var(key).ok(),
        |path| Ok(fs::read(path)?),
    )?;
    let profile = json!({"stage":"source-output-checkpoint","eligibility":"UNACCEPTED","source":files,"compiled_provider_source":compiled,"provider_executable_sha256":hash_bytes(&fs::read(executable)?),"compiled_build":build,"rust_roots":rust::ROOTS});
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
    Ok(format!("Stage 1 source/output checkpoint: {} exact unaccepted cells; zero Supported, Refused or BaselineUnknown. Eligibility remains invalid pending root review, owners and actual case qualification.\n",summary["cells"]))
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
            || !identity["version"]
                .as_str()
                .context("compiled tool version")?
                .contains("1.98.1")
        {
            bail!("compiled tool identity changed or unsupported: {key} {path}");
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
