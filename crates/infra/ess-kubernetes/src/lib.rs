//! Kubernetes credential-edge adapter for ESS.
//!
//! The adapter is the only ESS component that invokes `kubectl`. It sanitizes every Secret before
//! serializing an `infra-observation/1` bundle; all compilation and analysis happens downstream on
//! those credential-free bytes.

use std::io::Write;
use std::path::Path;
use std::process::Command;

use sha2::Digest;

/// Kubernetes API collections included in an observation, in deterministic order.
pub const KINDS: &[&str] = &[
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

fn kubectl(args: &[&str]) -> Result<Vec<u8>, String> {
    let output = Command::new("kubectl")
        .args(args)
        .output()
        .map_err(|error| format!("kubectl not runnable: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "kubectl {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(output.stdout)
}

/// Prints the kubeconfig contexts the adapter can target.
pub fn contexts() -> Result<(), String> {
    let output = kubectl(&["config", "get-contexts", "-o", "name"])?;
    print!("{}", String::from_utf8_lossy(&output));
    Ok(())
}

/// Scans one cluster and writes a sanitized `infra-observation/1` bundle.
///
/// Secret values are replaced before the first serialization or filesystem write. This function
/// never writes raw Kubernetes response bytes.
pub fn scan(context: Option<&str>, output_path: &Path) -> Result<(), String> {
    let context = match context {
        Some(context) => context.to_owned(),
        None => String::from_utf8_lossy(&kubectl(&["config", "current-context"])?)
            .trim()
            .to_owned(),
    };

    let mut kinds = serde_json::Map::new();
    for (index, kind) in KINDS.iter().enumerate() {
        eprint!("[{:>2}/{}] {kind:<24}\r", index + 1, KINDS.len());
        let raw = kubectl(&["--context", &context, "get", kind, "-A", "-o", "json"])
            .or_else(|_| kubectl(&["--context", &context, "get", kind, "-o", "json"]))?;
        let mut value: serde_json::Value = serde_json::from_slice(&raw)
            .map_err(|error| format!("kubectl get {kind}: not JSON: {error}"))?;
        if *kind == "secrets" {
            sanitize_secret_list(&mut value)?;
        }
        let count = value
            .get("items")
            .and_then(serde_json::Value::as_array)
            .map_or(0, Vec::len);
        eprintln!("[{:>2}/{}] {kind:<24} {count:>5}", index + 1, KINDS.len());
        kinds.insert((*kind).to_owned(), value);
    }

    let bundle = serde_json::json!({
        "format": "infra-observation/1",
        "context": context,
        "scanned_at": utc_timestamp(),
        "scout_version": env!("CARGO_PKG_VERSION"),
        "kinds": kinds,
    });
    let bytes = serde_json::to_vec_pretty(&bundle).map_err(|error| error.to_string())?;
    let digest = hex(&sha2::Sha256::digest(&bytes));
    std::fs::File::create(output_path)
        .and_then(|mut file| file.write_all(&bytes))
        .map_err(|error| format!("writing {}: {error}", output_path.display()))?;
    eprintln!(
        "wrote {} ({} kinds, sha256 {digest})",
        output_path.display(),
        KINDS.len()
    );
    Ok(())
}

/// Replaces all Secret values with their digest and byte length.
fn sanitize_secret_list(list: &mut serde_json::Value) -> Result<(), String> {
    let items = list
        .get_mut("items")
        .and_then(serde_json::Value::as_array_mut)
        .ok_or("secret list has no items array")?;
    for item in items {
        let item = item
            .as_object_mut()
            .ok_or("secret list item is not an object")?;
        for field in ["data", "stringData"] {
            if let Some(values) = item.get_mut(field) {
                let values = values
                    .as_object_mut()
                    .ok_or_else(|| format!("secret {field} is not an object"))?;
                for value in values.values_mut() {
                    let original = value
                        .as_str()
                        .ok_or_else(|| format!("secret {field} value is not a string"))?
                        .as_bytes();
                    *value = serde_json::json!({
                        "sha256": hex(&sha2::Sha256::digest(original)),
                        "length": original.len(),
                    });
                }
            }
        }
        if let Some(metadata) = item.get_mut("metadata") {
            let metadata = metadata
                .as_object_mut()
                .ok_or("secret metadata is not an object")?;
            if let Some(annotations) = metadata.get_mut("annotations") {
                let annotations = annotations
                    .as_object_mut()
                    .ok_or("secret annotations is not an object")?;
                if !annotations.values().all(serde_json::Value::is_string) {
                    return Err("secret annotation value is not a string".to_owned());
                }
                annotations.remove("kubectl.kubernetes.io/last-applied-configuration");
            }
        }
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    bytes.iter().fold(String::new(), |mut rendered, byte| {
        let _ = write!(rendered, "{byte:02x}");
        rendered
    })
}

fn utc_timestamp() -> String {
    Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_values_and_last_applied_configuration_never_survive_sanitization() {
        let mut list = serde_json::json!({
            "items": [{
                "metadata": {"annotations": {
                    "kept": "visible",
                    "kubectl.kubernetes.io/last-applied-configuration": "RAW-LAST-APPLIED-SECRET"
                }},
                "data": {"password": "RAW-BASE64-SECRET"},
                "stringData": {"token": "RAW-STRING-SECRET"}
            }]
        });

        sanitize_secret_list(&mut list).expect("a SecretList is sanitized");
        let bytes = serde_json::to_string(&list).expect("sanitized list serializes");

        for raw in [
            "RAW-BASE64-SECRET",
            "RAW-STRING-SECRET",
            "RAW-LAST-APPLIED-SECRET",
        ] {
            assert!(
                !bytes.contains(raw),
                "raw secret survived sanitization: {raw}"
            );
        }
        assert_eq!(
            list["items"][0]["metadata"]["annotations"]["kept"],
            "visible"
        );
        assert_eq!(list["items"][0]["data"]["password"]["length"], 17);
        assert_eq!(list["items"][0]["stringData"]["token"]["length"], 17);
    }

    #[test]
    fn malformed_secret_lists_are_refused_before_any_output_can_be_written() {
        let mut not_a_list = serde_json::json!({"kind": "SecretList"});
        let error = sanitize_secret_list(&mut not_a_list).expect_err("missing items is refused");
        assert_eq!(error, "secret list has no items array");
    }
}
