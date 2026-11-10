//! Kubernetes credential-edge adapter for ESS.
//!
//! The adapter is the only ESS component that invokes `kubectl`. It sanitizes every Secret before
//! serializing an `infra-observation/3` bundle; all compilation and analysis happens downstream on
//! those credential-free bytes.

use std::io::Write;
use std::path::Path;
use std::process::Command;

use sha2::Digest;

pub mod recovery;
mod topology;
pub use topology::{collect_namespace, scan_namespace};

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

/// Optional kinds from API groups a cluster may not serve, as `(bundle key, kubectl resource)`.
///
/// The resource is fully qualified, so another group's `gateways` (a service mesh's, say) is
/// never read in the Gateway API's place. A kind the server does not serve is left out of the
/// bundle — absent, "not observable here" — never written as an empty list.
pub const GATEWAY_KINDS: &[(&str, &str)] = &[
    ("gateways", "gateways.gateway.networking.k8s.io"),
    ("httproutes", "httproutes.gateway.networking.k8s.io"),
];

/// The API groups whose served kinds a full scan reports when it does not read them: the
/// cluster's network edge and policy, its certificates and secrets operators, its autoscalers,
/// delivery and monitoring. A kind of any other group, or of the core group, is not reported.
pub const WATCHED_GROUPS: &[&str] = &[
    "networking.k8s.io",
    "policy",
    "gateway.networking.k8s.io",
    "networking.istio.io",
    "security.istio.io",
    "elbv2.k8s.aws",
    "cert-manager.io",
    "external-secrets.io",
    "secrets-store.csi.x-k8s.io",
    "autoscaling.k8s.io",
    "keda.sh",
    "karpenter.sh",
    "argoproj.io",
    "monitoring.coreos.com",
];

/// The phrase kubectl prints for a resource type the API server does not serve.
const UNSERVED_RESOURCE: &[u8] = b"the server doesn't have a resource type";

fn kubectl(operation: &'static str, args: &[&str]) -> Result<Vec<u8>, String> {
    let output = Command::new("kubectl")
        .args(args)
        .output()
        .map_err(|error| format!("kubectl not runnable: {error}"))?;
    if !output.status.success() {
        // Stderr and arguments can contain credentials or caller-selected values. Only the
        // adapter's static operation label and the process status cross the diagnostic boundary.
        return Err(format!("kubectl {operation} failed: {}", output.status));
    }
    Ok(output.stdout)
}

/// Runs kubectl; `Ok(None)` when it failed only because the server does not serve the resource.
///
/// Stderr is matched against one fixed phrase and never copied: whether it matched is the only
/// thing that crosses the boundary. Any other failure is refused exactly as [`kubectl`] does.
fn kubectl_unless_unserved(
    operation: &'static str,
    args: &[&str],
) -> Result<Option<Vec<u8>>, String> {
    let output = Command::new("kubectl")
        .args(args)
        .output()
        .map_err(|error| format!("kubectl not runnable: {error}"))?;
    if output.status.success() {
        return Ok(Some(output.stdout));
    }
    if output
        .stderr
        .windows(UNSERVED_RESOURCE.len())
        .any(|window| window == UNSERVED_RESOURCE)
    {
        return Ok(None);
    }
    Err(format!("kubectl {operation} failed: {}", output.status))
}

/// Prints the kubeconfig contexts the adapter can target.
pub fn contexts() -> Result<(), String> {
    let output = kubectl("list contexts", &["config", "get-contexts", "-o", "name"])?;
    print!("{}", String::from_utf8_lossy(&output));
    Ok(())
}

/// Scans one cluster and writes a sanitized `infra-observation/3` bundle.
///
/// Secret values are replaced before the first serialization or filesystem write. This function
/// never writes raw Kubernetes response bytes.
pub fn scan(context: Option<&str>, output_path: &Path) -> Result<(), String> {
    let context = match context {
        Some(context) => context.to_owned(),
        None => String::from_utf8_lossy(&kubectl(
            "read current context",
            &["config", "current-context"],
        )?)
        .trim()
        .to_owned(),
    };

    let mut kinds = serde_json::Map::new();
    let total = KINDS.len() + GATEWAY_KINDS.len();
    let requests = KINDS.iter().map(|kind| (*kind, *kind, true)).chain(
        GATEWAY_KINDS
            .iter()
            .map(|(key, resource)| (*key, *resource, false)),
    );
    for (index, (kind, resource, required)) in requests.enumerate() {
        eprint!("[{:>2}/{total}] {kind:<24}\r", index + 1);
        let args = ["--context", &context, "get", resource, "-A", "-o", "json"];
        let raw = if required {
            kubectl("get resources in all namespaces", &args)?
        } else if let Some(raw) = kubectl_unless_unserved("get resources in all namespaces", &args)?
        {
            raw
        } else {
            eprintln!("[{:>2}/{total}] {kind:<24} not served", index + 1);
            continue;
        };
        let mut value: serde_json::Value = ess_primitives::json::from_slice(&raw)
            .map_err(|_| format!("kubectl get {kind}: response is not JSON"))?;
        if kind == "secrets" {
            sanitize_secret_list(&mut value)?;
        }
        let count = value
            .get("items")
            .and_then(serde_json::Value::as_array)
            .map_or(0, Vec::len);
        eprintln!("[{:>2}/{total}] {kind:<24} {count:>5}", index + 1);
        kinds.insert(kind.to_owned(), value);
    }
    let collected = kinds.len();
    let unobserved = unobserved_kinds(&context);

    let mut bundle = serde_json::json!({
        "format": infra_domain::PRESENCE_OBSERVATION_FORMAT,
        "context": context,
        "scanned_at": utc_timestamp(),
        "scout_version": env!("CARGO_PKG_VERSION"),
        "kinds": kinds,
    });
    // Written only when there is one, so a cluster serving nothing unread keeps its bytes.
    if !unobserved.is_empty() {
        bundle["unobserved_kinds"] = serde_json::json!(unobserved);
    }
    let bytes = serde_json::to_vec_pretty(&bundle).map_err(|error| error.to_string())?;
    let digest = hex(&sha2::Sha256::digest(&bytes));
    std::fs::File::create(output_path)
        .and_then(|mut file| file.write_all(&bytes))
        .map_err(|error| format!("writing {}: {error}", output_path.display()))?;
    eprintln!(
        "wrote {} ({collected} kinds, sha256 {digest})",
        output_path.display()
    );
    Ok(())
}

/// The served kinds of [`WATCHED_GROUPS`] this scan does not read, sorted, from one
/// `kubectl api-resources` call.
///
/// Discovery informs the report and never decides the scan: when it fails, nothing is recorded,
/// a notice says so, and nothing of kubectl's output crosses the boundary. A name that is not
/// `<plural>.<watched group>` is never recorded.
fn unobserved_kinds(context: &str) -> Vec<String> {
    let Ok(names) = kubectl(
        "discover API resources",
        &[
            "--context",
            context,
            "api-resources",
            "--verbs=list",
            "-o",
            "name",
        ],
    ) else {
        eprintln!("API discovery failed: unobserved kinds not recorded");
        return Vec::new();
    };
    let read = |name: &str, plural: &str| {
        KINDS.contains(&plural) || GATEWAY_KINDS.iter().any(|(_, resource)| *resource == name)
    };
    let mut unobserved: Vec<String> = String::from_utf8_lossy(&names)
        .lines()
        .map(str::trim)
        .filter_map(|name| {
            let (plural, group) = name.split_once('.')?;
            let wellformed = !plural.is_empty()
                && plural
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit());
            (wellformed && WATCHED_GROUPS.contains(&group) && !read(name, plural))
                .then(|| name.to_owned())
        })
        .collect();
    unobserved.sort();
    unobserved.dedup();
    unobserved
}

/// Replaces every Secret value with `{"present": true}`: the key survives, nothing about the value.
///
/// Not a digest and not a length. An unsalted digest of a low-entropy value lets anyone holding
/// the file confirm a guess, and a length narrows the guessing; nothing downstream needs either.
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
                    if !value.is_string() {
                        return Err(format!("secret {field} value is not a string"));
                    }
                    *value = serde_json::json!({"present": true});
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
        // Presence, and nothing derived from the value: no digest to test a guess against and no
        // length to narrow one.
        assert_eq!(
            list["items"][0]["data"]["password"],
            serde_json::json!({"present": true})
        );
        assert_eq!(
            list["items"][0]["stringData"]["token"],
            serde_json::json!({"present": true})
        );
    }

    #[test]
    fn malformed_secret_lists_are_refused_before_any_output_can_be_written() {
        let mut not_a_list = serde_json::json!({"kind": "SecretList"});
        let error = sanitize_secret_list(&mut not_a_list).expect_err("missing items is refused");
        assert_eq!(error, "secret list has no items array");
    }
}
