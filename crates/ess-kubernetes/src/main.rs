//! `ess-kubernetes` — the actor beside `aep`.
//!
//! This binary is deliberately thin: it enumerates kubeconfig contexts and scans one cluster
//! into an `infra-observation/1` bundle — raw Kubernetes API objects, keyed by kind, exactly as
//! the API returned them, with one exception that is the whole point of doing this in a program
//! rather than a shell script: **secret values never touch disk.** Every `data`/`stringData`
//! value in a Secret is replaced by its SHA-256 and length before the bundle is written.
//!
//! Everything downstream — validation, the IR, the digest, the dependency graph, diagnosis —
//! lives in the pure `aep` toolchain, which reads this bundle as a file and
//! never holds a credential. v0 shells out to `kubectl` rather than linking a Kubernetes client:
//! the credential handling, exec plugins and API version skew all stay kubectl's problem.

use clap::{Parser, Subcommand};
use sha2::Digest;
use std::io::Write;
use std::process::Command;

/// The API kinds a scan collects, in the order they are collected.
///
/// This is the observation surface of `infra-observation/1`: enough to map workloads, their
/// wiring (services, ingress, config) and their identity (service accounts), plus the runtime
/// facts (pods, nodes) diagnosis reads. Extending this list is a format change and belongs in a
/// commit that says so.
const KINDS: &[&str] = &[
    "namespaces",
    "nodes",
    "deployments",
    "statefulsets",
    "daemonsets",
    "replicasets",
    "jobs",
    "cronjobs",
    "pods",
    "services",
    "ingresses",
    "configmaps",
    "secrets",
    "serviceaccounts",
    "persistentvolumeclaims",
    "poddisruptionbudgets",
    "horizontalpodautoscalers",
];

#[derive(Parser)]
#[command(name = "ess-kubernetes", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// List the kubeconfig contexts a scan could target.
    Contexts,
    /// Scan one cluster into an infra-observation/1 bundle.
    Scan {
        /// Kubeconfig context to scan. Defaults to the current context.
        #[arg(long)]
        context: Option<String>,
        /// Where to write the bundle.
        #[arg(long)]
        out: std::path::PathBuf,
    },
}

fn main() -> std::process::ExitCode {
    match run(Cli::parse()) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Cmd::Contexts => contexts(),
        Cmd::Scan { context, out } => scan(context.as_deref(), &out),
    }
}

/// `kubectl` with arguments, returning stdout or a message that names the failing invocation.
fn kubectl(args: &[&str]) -> Result<Vec<u8>, String> {
    let output = Command::new("kubectl")
        .args(args)
        .output()
        .map_err(|e| format!("kubectl not runnable: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "kubectl {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(output.stdout)
}

fn contexts() -> Result<(), String> {
    let out = kubectl(&["config", "get-contexts", "-o", "name"])?;
    print!("{}", String::from_utf8_lossy(&out));
    Ok(())
}

fn scan(context: Option<&str>, out_path: &std::path::Path) -> Result<(), String> {
    let context = match context {
        Some(c) => c.to_owned(),
        None => String::from_utf8_lossy(&kubectl(&["config", "current-context"])?)
            .trim()
            .to_owned(),
    };

    let mut kinds = serde_json::Map::new();
    for (i, kind) in KINDS.iter().enumerate() {
        // Progress goes to stderr, one line per kind, because a scan is the one slow step in the
        // pipeline (a kubectl round-trip per kind) and silence reads as a hang.
        eprint!("[{:>2}/{}] {kind:<24}\r", i + 1, KINDS.len());
        let raw = kubectl(&["--context", &context, "get", kind, "-A", "-o", "json"])
            // Cluster-scoped kinds reject -A; retry without it rather than special-casing a list
            // that would drift from the server's own opinion.
            .or_else(|_| kubectl(&["--context", &context, "get", kind, "-o", "json"]))?;
        let mut value: serde_json::Value = serde_json::from_slice(&raw)
            .map_err(|e| format!("kubectl get {kind}: not JSON: {e}"))?;
        if *kind == "secrets" {
            sanitize_secret_list(&mut value)?;
        }
        let count = value.get("items").and_then(|i| i.as_array()).map_or(0, Vec::len);
        eprintln!("[{:>2}/{}] {kind:<24} {count:>5}", i + 1, KINDS.len());
        kinds.insert((*kind).to_owned(), value);
    }

    let bundle = serde_json::json!({
        "format": "infra-observation/1",
        "context": context,
        "scanned_at": chrono_free_timestamp(),
        "scout_version": env!("CARGO_PKG_VERSION"),
        "kinds": kinds,
    });

    let bytes = serde_json::to_vec_pretty(&bundle).map_err(|e| e.to_string())?;
    let digest = hex(&sha2::Sha256::digest(&bytes));
    std::fs::File::create(out_path)
        .and_then(|mut f| f.write_all(&bytes))
        .map_err(|e| format!("writing {}: {e}", out_path.display()))?;
    eprintln!(
        "wrote {} ({} kinds, sha256 {digest})",
        out_path.display(),
        KINDS.len()
    );
    Ok(())
}

/// Replace every secret value in a `SecretList` with `{sha256, length}`.
///
/// This runs before anything is written to disk, and it is the reason this tool exists as a
/// program rather than a `kubectl | jq` pipeline someone edits under pressure. The *shape*
/// (which secrets exist, which keys they carry) survives — that is what the mapping needs —
/// while the values are unrecoverable.
fn sanitize_secret_list(list: &mut serde_json::Value) -> Result<(), String> {
    let items = list
        .get_mut("items")
        .and_then(|i| i.as_array_mut())
        .ok_or("secret list has no items array")?;
    for item in items {
        for field in ["data", "stringData"] {
            if let Some(map) = item.get_mut(field).and_then(|d| d.as_object_mut()) {
                for (_key, value) in map.iter_mut() {
                    let original = value.as_str().unwrap_or_default().as_bytes().to_vec();
                    *value = serde_json::json!({
                        "sha256": hex(&sha2::Sha256::digest(&original)),
                        "length": original.len(),
                    });
                }
            }
        }
        // A secret's annotations can carry the last-applied configuration, values included.
        if let Some(annotations) = item
            .pointer_mut("/metadata/annotations")
            .and_then(|a| a.as_object_mut())
        {
            annotations.remove("kubectl.kubernetes.io/last-applied-configuration");
        }
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// RFC 3339 UTC from the system clock, without a chrono dependency.
fn chrono_free_timestamp() -> String {
    // `date -u` is as available as `kubectl`; this tool is already a shell-out adapter.
    Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .unwrap_or_default()
}
