//! Exact aggregate provenance over fresh canonical structures and qualified child cells.
use anyhow::{bail, Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

pub(super) const FORMAT: &str = "ess-consumer-aggregate-closure/1";

const ALLOWED: [(&str, &str, &str); 5] = [
    (
        "rust:ess_domain::command::RawOutcome",
        "a2072ea0863f3bfdaa92b6ab4b30eb1857bf23af0538db9747ea3b3efc3505b0",
        "f167bcf3a4fd9ea3f40d41885bf457433f1601dd39353efb7883626a38c4b160",
    ),
    (
        "wire:RawSpecFile#",
        "687cebb230483729bb9de45eeec08e14b3f8961cfc977c1f470001cc1b41d6a5",
        "35e9219da6bb5023ef23090ace368b30f8f52f17b28a8c9d0a9bafaf14f69963",
    ),
    (
        "wire:RawSpecFile#/definitions",
        "3515c009028136bab26f2c1e31a6a5a15f8a44eba3f3ae6444e47a3aa5aab244",
        "3c1ce721245397ae759f2f3c6fb91ce4039d0aee9c211043c996c73657e6f79a",
    ),
    (
        "wire:RawSpecFile#/definitions/RawOutcome",
        "eed5afcaac752aae6182c13a784afdc777ff207dd5404c20b320a619d7683722",
        "09922c5e6ca0034e702f9ff81caa1dcb7a2a4430701ddbef9f4ac6b12e4afc3b",
    ),
    (
        "wire:RawSpecFile#/definitions/RawOutcome/properties",
        "c72e70a9700e2b90445a761f78283670c50659dffa3bee63ff0bb5878c7297ab",
        "c2da7f3d928ba82ca11ccfceaf9be30cf83736b0845abd372784c5774ca544c5",
    ),
];

pub(super) fn structure(rust: &Value, wire: &Value) -> Result<Value> {
    let mut obligations = rust["obligations"]
        .as_object()
        .context("Rust structural obligations")?
        .clone();
    for (id, shape) in wire["obligations"]
        .as_object()
        .context("wire structural obligations")?
    {
        if obligations.insert(id.clone(), shape.clone()).is_some() {
            bail!("duplicate Rust/wire structural identity {id}");
        }
    }
    let mut references = wire["references"]
        .as_array()
        .context("wire structural references")?
        .clone();
    for (source, targets) in rust["references"]
        .as_object()
        .context("Rust structural references")?
    {
        for target in targets.as_array().context("Rust reference targets")? {
            references.push(json!({
                "source":format!("rust:{source}"),
                "target":format!("rust:{}",target.as_str().context("Rust reference target")?),
            }));
        }
    }
    references.sort_by_key(super::hash_json);
    Ok(json!({"obligations":obligations,"references":references}))
}

pub(super) fn historical_witness() -> Result<Value> {
    let witness = structure(
        &super::rust::historical_raw_outcome_witness()?,
        &super::wire::historical_witness()?,
    )?;
    for (id, old, _) in ALLOWED {
        if witness["obligations"][id] != old {
            bail!("historical source witness differs from pinned aggregate shape: {id}");
        }
    }
    Ok(witness)
}

pub(super) struct FreshStructures {
    old: Value,
    current: Value,
}

pub(super) fn capture(current_rust: &Value, current_wire: &Value) -> Result<FreshStructures> {
    Ok(FreshStructures {
        old: historical_witness()?,
        current: structure(current_rust, current_wire)?,
    })
}

pub(super) fn verify_parent_identities(
    authority: &Value,
    structures: &FreshStructures,
) -> Result<()> {
    for (identity, _) in claims(authority)? {
        if structures.current["obligations"][&identity.model] != identity.shape {
            bail!("aggregate current identity differs from the fresh compiled provider");
        }
    }
    Ok(())
}

pub(super) fn verify_reconciliation(
    authority: &Value,
    resolution: &super::reconciliation::Resolution,
    baseline_sha256: &str,
) -> Result<()> {
    let manifest: Manifest = serde_json::from_value(authority.clone())?;
    if manifest.format != FORMAT {
        bail!("unknown aggregate-closure format");
    }
    for row in &manifest.rows {
        if row.baseline_sha256 != baseline_sha256 || row.reconciliation_sha256 != resolution.digest
        {
            bail!("aggregate closure uses stale baseline or reconciliation authority");
        }
        let replacement = resolution
            .replacements
            .get(&row.old)
            .context("aggregate closure is not an exact reconciled old tuple")?;
        if replacement.current != row.current
            || !matches!(
                &replacement.claim,
                super::reconciliation::ReplacementClaim::AggregateClosure { closure }
                    if closure == &row.id
            )
        {
            bail!("aggregate closure differs from its exact replacement binding");
        }
        match row.mode {
            Mode::ShapeDelta => {
                let residual = row
                    .residual
                    .as_ref()
                    .context("ShapeDelta must preserve visible unresolved provenance")?;
                if residual.owner != replacement.unknown.owner
                    || residual.follow_up != replacement.unknown.follow_up
                    || residual.reason != replacement.unknown.reason
                {
                    bail!("ShapeDelta residual differs from frozen unknown provenance");
                }
            }
            Mode::CompleteCurrentSubtree => {
                if row.old.profile == row.current.profile {
                    bail!("complete-current closure requires an exact changed profile");
                }
            }
        }
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format: String,
    rows: Vec<Row>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    id: String,
    mode: Mode,
    old: super::reconciliation::Identity,
    current: super::reconciliation::Identity,
    baseline_sha256: String,
    reconciliation_sha256: String,
    residual: Option<Residual>,
    frontier: Vec<Frontier>,
    source_evidence: String,
    review_reference: String,
}

pub(super) fn claims(authority: &Value) -> Result<Vec<(super::reconciliation::Identity, String)>> {
    let manifest: Manifest = serde_json::from_value(authority.clone())?;
    if manifest.format != FORMAT {
        bail!("unknown aggregate-closure format");
    }
    let allowed = ALLOWED
        .into_iter()
        .map(|(id, old, current)| (id, (old, current)))
        .collect::<BTreeMap<_, _>>();
    let mut ids = BTreeSet::new();
    let mut cells = BTreeSet::new();
    let mut result = Vec::new();
    for row in manifest.rows {
        let Some((old_hash, current_hash)) = allowed.get(row.old.model.as_str()) else {
            bail!("aggregate identity is outside the exact five");
        };
        if row.old.model != row.current.model
            || row.old.consumer != row.current.consumer
            || row.old.shape != *old_hash
            || row.current.shape != *current_hash
            || row.id.trim().is_empty()
            || row.source_evidence.trim().is_empty()
            || row.review_reference.trim().is_empty()
            || row.baseline_sha256.trim().is_empty()
            || row.reconciliation_sha256.trim().is_empty()
            || !ids.insert(row.id.clone())
            || !cells.insert((row.current.model.clone(), row.current.consumer.clone()))
        {
            bail!("duplicate, stale or incomplete aggregate claim");
        }
        result.push((row.current, row.id));
    }
    Ok(result)
}

pub(super) fn required_cases(authority: &Value) -> Result<BTreeSet<String>> {
    let manifest: Manifest = serde_json::from_value(authority.clone())?;
    if manifest.format != FORMAT {
        bail!("unknown aggregate-closure format");
    }
    let mut cases = BTreeSet::new();
    for row in manifest.rows {
        for child in row.frontier {
            match child.disposition {
                ChildDisposition::Supported { cases: linked }
                | ChildDisposition::Refused { cases: linked, .. } => {
                    if linked.is_empty() || linked.iter().any(|case| case.trim().is_empty()) {
                        bail!("aggregate behavior child lacks exact cases");
                    }
                    cases.extend(linked);
                }
                ChildDisposition::AggregateClosure { .. } | ChildDisposition::Removed => {}
            }
        }
    }
    Ok(cases)
}

#[derive(Deserialize, PartialEq)]
enum Mode {
    ShapeDelta,
    CompleteCurrentSubtree,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Residual {
    owner: String,
    follow_up: String,
    reason: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Frontier {
    path: String,
    disposition: ChildDisposition,
}

#[derive(Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum ChildDisposition {
    Supported { cases: Vec<String> },
    Refused { cases: Vec<String>, refusal: String },
    AggregateClosure { closure: String },
    Removed,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Structures {
    obligations: BTreeMap<String, String>,
    references: Vec<Reference>,
}

#[derive(Clone, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(deny_unknown_fields)]
struct Reference {
    source: String,
    target: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct QualifiedCell {
    model: String,
    consumer: String,
    profile: String,
    kind: QualifiedKind,
    #[serde(default)]
    cases: Vec<String>,
    #[serde(default)]
    refusal: Option<String>,
}

#[derive(Deserialize, PartialEq)]
enum QualifiedKind {
    Supported,
    Refused,
    AggregateClosure,
}

fn descendant(parent: &str, child: &str) -> bool {
    child
        .strip_prefix(parent)
        .is_some_and(|suffix| suffix.starts_with('/') && suffix.len() > 1)
}

fn covered(path: &str, frontier: &BTreeSet<String>) -> usize {
    frontier
        .iter()
        .filter(|root| path == root.as_str() || descendant(root, path))
        .count()
}

fn reference_delta_paths(
    parent: &str,
    old: &Structures,
    current: &Structures,
) -> Result<BTreeSet<String>> {
    let old_edges = old.references.iter().cloned().collect::<BTreeSet<_>>();
    let current_edges = current.references.iter().cloned().collect::<BTreeSet<_>>();
    let mut paths = BTreeSet::new();
    for edge in old_edges.symmetric_difference(&current_edges) {
        if !descendant(parent, &edge.source) {
            continue;
        }
        paths.insert(edge.source.clone());
        if descendant(parent, &edge.target) {
            paths.insert(edge.target.clone());
        } else if edge.target != parent
            && old.obligations.get(&edge.target) != current.obligations.get(&edge.target)
        {
            bail!("aggregate frontier crosses a changed reference endpoint");
        }
    }
    Ok(paths)
}

#[expect(
    clippy::too_many_lines,
    reason = "the aggregate validator keeps its finite partition proof in one auditable pass"
)]
fn validate(
    authority: &Value,
    old_value: &Value,
    current_value: &Value,
    cells_value: &Value,
    executed_value: &Value,
    restrict_identities: bool,
) -> Result<Value> {
    let manifest: Manifest = serde_json::from_value(authority.clone())?;
    let old: Structures = serde_json::from_value(old_value.clone())?;
    let current: Structures = serde_json::from_value(current_value.clone())?;
    let cells: Vec<QualifiedCell> = serde_json::from_value(cells_value.clone())?;
    let executed: BTreeSet<String> = serde_json::from_value(executed_value.clone())?;
    if manifest.format != FORMAT {
        bail!("unknown aggregate-closure format");
    }
    let allowed = ALLOWED
        .into_iter()
        .map(|(id, old, current)| (id, (old, current)))
        .collect::<BTreeMap<_, _>>();
    let mut ids = BTreeSet::new();
    let mut dependencies = BTreeMap::<String, BTreeSet<String>>::new();
    let mut required_cases = BTreeSet::new();
    for row in &manifest.rows {
        if !ids.insert(row.id.clone()) {
            bail!("duplicate aggregate closure {}", row.id);
        }
        if row.id.trim().is_empty()
            || row.baseline_sha256.trim().is_empty()
            || row.reconciliation_sha256.trim().is_empty()
            || row.source_evidence.trim().is_empty()
            || row.review_reference.trim().is_empty()
            || row.old.model != row.current.model
            || row.old.consumer != row.current.consumer
            || row.frontier.is_empty()
        {
            bail!("incomplete aggregate closure {}", row.id);
        }
        if old.obligations.get(&row.old.model) != Some(&row.old.shape)
            || current.obligations.get(&row.current.model) != Some(&row.current.shape)
        {
            bail!("aggregate parent differs from canonical old/current structure");
        }
        if restrict_identities {
            let Some((old_hash, current_hash)) = allowed.get(row.old.model.as_str()) else {
                bail!("aggregate identity is outside the exact five");
            };
            if row.old.shape != *old_hash || row.current.shape != *current_hash {
                bail!("aggregate identity uses the wrong frozen/current hash");
            }
        }
        match row.mode {
            Mode::ShapeDelta => {
                let residual = row
                    .residual
                    .as_ref()
                    .context("ShapeDelta must preserve visible unresolved provenance")?;
                if row.old.profile != row.current.profile
                    || [&residual.owner, &residual.follow_up, &residual.reason]
                        .iter()
                        .any(|value| value.trim().is_empty())
                {
                    bail!("ShapeDelta cannot inherit across a changed profile");
                }
            }
            Mode::CompleteCurrentSubtree => {
                if row.residual.is_some() || row.old.profile == row.current.profile {
                    bail!(
                        "complete-current closure requires a changed profile and no unknown residual"
                    );
                }
            }
        }
        let mut frontier = BTreeSet::new();
        for child in &row.frontier {
            if !descendant(&row.current.model, &child.path) || !frontier.insert(child.path.clone())
            {
                bail!("aggregate frontier is duplicate or outside its parent");
            }
        }
        for left in &frontier {
            for right in &frontier {
                if left != right && descendant(left, right) {
                    bail!("aggregate frontier overlaps");
                }
            }
        }
        let mut changed = old
            .obligations
            .keys()
            .chain(current.obligations.keys())
            .filter(|path| descendant(&row.current.model, path))
            .filter(|path| old.obligations.get(*path) != current.obligations.get(*path))
            .cloned()
            .collect::<BTreeSet<_>>();
        changed.extend(reference_delta_paths(&row.current.model, &old, &current)?);
        let required_paths = if row.mode == Mode::ShapeDelta {
            changed
        } else {
            current
                .obligations
                .keys()
                .filter(|path| descendant(&row.current.model, path))
                .cloned()
                .collect()
        };
        if required_paths.is_empty()
            || required_paths
                .iter()
                .any(|path| covered(path, &frontier) != 1)
            || frontier.iter().any(|path| {
                !required_paths
                    .iter()
                    .any(|required| required == path || descendant(path, required))
            })
        {
            bail!("aggregate frontier is incomplete, overlapping or unrelated");
        }
        for reference in old.references.iter().chain(&current.references) {
            if descendant(&row.current.model, &reference.source)
                && (descendant(&row.current.model, &reference.target)
                    || reference.target == row.current.model)
            {
                let source_changed = old.obligations.get(&reference.source)
                    != current.obligations.get(&reference.source);
                let target_changed = old.obligations.get(&reference.target)
                    != current.obligations.get(&reference.target);
                if (source_changed && covered(&reference.source, &frontier) != 1)
                    || (target_changed && covered(&reference.target, &frontier) != 1)
                {
                    bail!("aggregate frontier is blind to a changed reference endpoint");
                }
            }
        }
        let mut deps = BTreeSet::new();
        for child in &row.frontier {
            match &child.disposition {
                ChildDisposition::Removed => {
                    if current.obligations.contains_key(&child.path)
                        || !old.obligations.contains_key(&child.path)
                    {
                        bail!("removed frontier child is not an exact removal");
                    }
                }
                ChildDisposition::Supported { cases } => {
                    if cases.is_empty()
                        || cases.iter().any(|case| case.trim().is_empty())
                        || cases.iter().collect::<BTreeSet<_>>().len() != cases.len()
                    {
                        bail!("aggregate behavior child lacks exact distinct cases");
                    }
                    let cell = cells.iter().find(|cell| {
                        cell.model == child.path
                            && cell.consumer == row.current.consumer
                            && cell.profile == row.current.profile
                    });
                    let cell = cell.context("aggregate child lacks same-consumer qualification")?;
                    if cell.kind != QualifiedKind::Supported
                        || cell.cases != *cases
                        || cell.refusal.is_some()
                    {
                        bail!("aggregate child disposition differs from qualified behavior");
                    }
                    required_cases.extend(cases.iter().cloned());
                }
                ChildDisposition::Refused { cases, refusal } => {
                    if cases.is_empty()
                        || cases.iter().any(|case| case.trim().is_empty())
                        || cases.iter().collect::<BTreeSet<_>>().len() != cases.len()
                        || refusal.trim().is_empty()
                    {
                        bail!("aggregate refused child lacks exact cases and named boundary");
                    }
                    let cell = cells
                        .iter()
                        .find(|cell| {
                            cell.model == child.path
                                && cell.consumer == row.current.consumer
                                && cell.profile == row.current.profile
                        })
                        .context("aggregate child lacks same-consumer qualification")?;
                    if cell.kind != QualifiedKind::Refused
                        || cell.cases != *cases
                        || cell.refusal.as_deref() != Some(refusal.as_str())
                    {
                        bail!("aggregate refused child differs from qualified behavior boundary");
                    }
                    required_cases.extend(cases.iter().cloned());
                }
                ChildDisposition::AggregateClosure { closure } => {
                    if closure == &row.id {
                        bail!("aggregate closure depends on itself");
                    }
                    let nested = manifest
                        .rows
                        .iter()
                        .find(|candidate| candidate.id == *closure)
                        .context("aggregate frontier names an unknown nested closure")?;
                    if nested.current.model != child.path
                        || nested.current.consumer != row.current.consumer
                        || nested.current.profile != row.current.profile
                        || (row.mode == Mode::CompleteCurrentSubtree
                            && nested.mode != Mode::CompleteCurrentSubtree)
                    {
                        bail!("nested aggregate closure differs from its same-consumer frontier");
                    }
                    deps.insert(closure.clone());
                }
            }
        }
        dependencies.insert(row.id.clone(), deps);
    }
    for deps in dependencies.values() {
        if deps.iter().any(|id| !dependencies.contains_key(id)) {
            bail!("aggregate frontier names an unknown nested closure");
        }
    }
    let mut active = BTreeSet::new();
    let mut complete = BTreeSet::new();
    for id in dependencies.keys() {
        visit_dependencies(id, &dependencies, &mut active, &mut complete)?;
    }
    if !required_cases.is_subset(&executed) {
        bail!("aggregate child cases are absent from the exact executed set");
    }
    Ok(json!({
        "format":FORMAT,
        "aggregate_manifest_sha256":super::hash_json(authority),
        "qualified_closures":manifest.rows.len(),
        "required_cases":required_cases,
        "fresh_old_structure_sha256":super::hash_json(old_value),
        "fresh_current_structure_sha256":super::hash_json(current_value),
    }))
}

fn visit_dependencies(
    id: &str,
    graph: &BTreeMap<String, BTreeSet<String>>,
    active: &mut BTreeSet<String>,
    complete: &mut BTreeSet<String>,
) -> Result<()> {
    if complete.contains(id) {
        return Ok(());
    }
    if !active.insert(id.to_owned()) {
        bail!("aggregate closure dependency cycle");
    }
    for child in &graph[id] {
        visit_dependencies(child, graph, active, complete)?;
    }
    active.remove(id);
    complete.insert(id.to_owned());
    Ok(())
}

#[cfg(test)]
pub(super) fn test_validate(
    authority: &Value,
    old: &Value,
    current: &Value,
    cells: &Value,
    executed: &Value,
) -> Result<Value> {
    validate(authority, old, current, cells, executed, false)
}

pub(super) fn verify(
    authority: &Value,
    structures: &FreshStructures,
    cells: &Value,
    executed: &Value,
) -> Result<Value> {
    validate(
        authority,
        &structures.old,
        &structures.current,
        cells,
        executed,
        true,
    )
}
