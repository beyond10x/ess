//! Bounded topology collection, using the domain's declared raw shapes as an allowlist.

use std::collections::BTreeSet;
use std::path::Path;

use infra_domain::coverage::CollectionCoverage;
use infra_domain::raw::{
    RawBundle, RawClaim, RawConfigMap, RawCronJob, RawHorizontalPodAutoscaler, RawIngress, RawJob,
    RawNamespace, RawNode, RawPod, RawPodDisruptionBudget, RawPodSpec, RawReplicaSet, RawSecret,
    RawService, RawServiceAccount, RawWorkload,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Map, Value};

use crate::{kubectl, utc_timestamp, KINDS};

/// Reads one namespace's topology and only its referenced nodes, then writes observation/2.
///
/// Explicit context and namespace are required. There is no fallback and no output on failed
/// acquisition or validation. Values and unmodeled API fields are dropped before serialization.
pub fn scan_namespace(context: &str, namespace: &str, out: &Path) -> Result<(), String> {
    let coverage = CollectionCoverage::NamespaceTopology {
        namespace: namespace.to_owned(),
    };
    if context.trim().is_empty() || !coverage.is_valid() {
        return Err(
            "namespace topology collection requires an explicit context and valid namespace"
                .to_owned(),
        );
    }
    let mut kinds = Map::new();
    for kind in KINDS.iter().copied().filter(|kind| *kind != "nodes") {
        let args = if kind == "namespaces" {
            vec![
                "--context",
                context,
                "get",
                "namespace",
                namespace,
                "-o",
                "json",
            ]
        } else {
            vec![
                "--context",
                context,
                "get",
                kind,
                "--namespace",
                namespace,
                "-o",
                "json",
            ]
        };
        let response = read(&args)?;
        let objects = if kind == "namespaces" {
            vec![response]
        } else {
            list_items(response)?
        };
        let mut selected = Vec::new();
        for object in objects {
            check_identity(
                &object,
                if kind == "namespaces" {
                    None
                } else {
                    Some(namespace)
                },
                if kind == "namespaces" {
                    Some(namespace)
                } else {
                    None
                },
            )?;
            selected.push(select(kind, object)?);
        }
        kinds.insert(kind.to_owned(), json!({"items": selected}));
    }
    let node_names: BTreeSet<String> = kinds["pods"]["items"]
        .as_array()
        .expect("constructed List")
        .iter()
        .filter_map(|pod| pod.pointer("/spec/nodeName").and_then(Value::as_str))
        .map(str::to_owned)
        .collect();
    let mut nodes = Vec::new();
    for name in node_names {
        // Names came from the API, but remain operands: no caller-controlled flag may be added.
        if name.is_empty() || name.starts_with('-') {
            return Err("invalid referenced node name".to_owned());
        }
        let node = read(&["--context", context, "get", "node", &name, "-o", "json"])?;
        check_identity(&node, None, Some(&name))?;
        nodes.push(select("nodes", node)?);
    }
    kinds.insert("nodes".to_owned(), json!({"items": nodes}));
    let bundle = json!({
        "format": "infra-observation/2", "context": context,
        "scanned_at": utc_timestamp(), "scout_version": env!("CARGO_PKG_VERSION"),
        "coverage": coverage, "kinds": kinds,
    });
    let raw: RawBundle =
        serde_json::from_value(bundle.clone()).map_err(|_| "invalid topology bundle")?;
    infra_domain::Observation::try_from(raw).map_err(|_| {
        "topology response violates the supported observation subset; no observation written"
    })?;
    let bytes = serde_json::to_vec_pretty(&bundle).map_err(|_| "cannot serialize topology")?;
    std::fs::write(out, bytes).map_err(|error| format!("writing observation: {error}"))?;
    Ok(())
}

fn read(args: &[&str]) -> Result<Value, String> {
    let bytes = kubectl("get scoped resources", args)?;
    serde_json::from_slice(&bytes).map_err(|_| "scoped resource response is not JSON".to_owned())
}

fn list_items(mut value: Value) -> Result<Vec<Value>, String> {
    // kubectl normally follows API pagination. A residual continuation token is incomplete,
    // never a complete successful collection.
    if value
        .pointer("/metadata/continue")
        .is_some_and(|v| v != "" && !v.is_null())
    {
        return Err(
            "scoped collection is incomplete: response has a continuation token".to_owned(),
        );
    }
    match value.get_mut("items").map(Value::take) {
        Some(Value::Array(items)) => Ok(items),
        _ => Err("scoped response has no items array".to_owned()),
    }
}

fn check_identity(
    object: &Value,
    namespace: Option<&str>,
    name: Option<&str>,
) -> Result<(), String> {
    let actual_name = object.pointer("/metadata/name").and_then(Value::as_str);
    let actual_namespace = object
        .pointer("/metadata/namespace")
        .and_then(Value::as_str);
    if actual_namespace != namespace
        || actual_name.is_none_or(str::is_empty)
        || name.is_some_and(|expected| actual_name != Some(expected))
    {
        return Err("scoped resource response has an unexpected identity".to_owned());
    }
    Ok(())
}

fn selected<T: DeserializeOwned + Serialize>(
    value: Value,
    redact: impl FnOnce(&mut T),
) -> Result<Value, String> {
    let mut raw: T =
        serde_json::from_value(value).map_err(|_| "resource shape or selector is unsupported")?;
    redact(&mut raw);
    serde_json::to_value(raw).map_err(|_| "cannot serialize selected resource".to_owned())
}

fn pod_spec(spec: &mut RawPodSpec) {
    for volume in &mut spec.volumes {
        if volume.empty_dir.is_some() {
            volume.empty_dir = Some(json!({}));
        }
    }
    for container in &mut spec.containers {
        container.env.retain(|env| env.value_from.is_some());
        for env in &mut container.env {
            env.value = None;
            if let Some(source) = &mut env.value_from {
                if source.resource_field_ref.is_some() {
                    source.resource_field_ref = Some(json!({}));
                }
            }
        }
        container.liveness_probe = None;
        container.readiness_probe = None;
        container.startup_probe = None;
    }
}

fn select(kind: &str, value: Value) -> Result<Value, String> {
    match kind {
        "namespaces" => selected::<RawNamespace>(value, |_| {}),
        "nodes" => selected::<RawNode>(value, |_| {}),
        "deployments" | "statefulsets" | "daemonsets" => selected::<RawWorkload>(value, |raw| {
            if let Some(template) = &mut raw.spec.template {
                pod_spec(&mut template.spec);
            }
        }),
        "pods" => selected::<RawPod>(value, |raw| pod_spec(&mut raw.spec)),
        "services" => selected::<RawService>(value, |_| {}),
        "ingresses" => selected::<RawIngress>(value, |_| {}),
        "configmaps" => selected::<RawConfigMap>(value, |raw| {
            raw.data.clear();
            raw.binary_data.clear();
        }),
        "secrets" => selected::<RawSecret>(value, |raw| {
            raw.data.clear();
            raw.string_data.clear();
        }),
        "serviceaccounts" => selected::<RawServiceAccount>(value, |_| {}),
        "persistentvolumeclaims" => selected::<RawClaim>(value, |_| {}),
        "replicasets" => selected::<RawReplicaSet>(value, |_| {}),
        "jobs" => selected::<RawJob>(value, |_| {}),
        "cronjobs" => selected::<RawCronJob>(value, |_| {}),
        "poddisruptionbudgets" => selected::<RawPodDisruptionBudget>(value, |_| {}),
        "horizontalpodautoscalers" => selected::<RawHorizontalPodAutoscaler>(value, |_| {}),
        _ => Err("kind is not in the topology profile".to_owned()),
    }
}
