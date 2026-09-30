//! The parts of `schemas/ui/ess-ui.schema.yaml` the checks are driven by, read from the schema
//! `ess-ui` embeds rather than restated here.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use serde_yaml::Value;

fn schema() -> &'static Value {
    static PARSED: OnceLock<Value> = OnceLock::new();
    PARSED
        .get_or_init(|| serde_yaml::from_str(ess_ui::SCHEMA).expect("the embedded schema is YAML"))
}

fn strings(value: &Value) -> Vec<String> {
    value
        .as_sequence()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.as_str().map(str::to_owned))
        .collect()
}

fn keys(value: &Value) -> BTreeSet<String> {
    value
        .as_mapping()
        .into_iter()
        .flatten()
        .filter_map(|(key, _)| key.as_str().map(str::to_owned))
        .collect()
}

/// One entry of `constructs.Degrades.capabilities`.
pub(crate) struct Capability {
    /// Fallbacks in preference order; the first applies when a construct declares none.
    pub(crate) fallbacks: Vec<String>,
    /// The constructs that can use the capability.
    pub(crate) applies_to: Vec<String>,
}

/// `constructs.Degrades.capabilities`, by capability.
pub(crate) fn capabilities() -> &'static BTreeMap<String, Capability> {
    static TABLE: OnceLock<BTreeMap<String, Capability>> = OnceLock::new();
    TABLE.get_or_init(|| {
        schema()["constructs"]["Degrades"]["capabilities"]
            .as_mapping()
            .expect("the schema lists the degrade capabilities")
            .iter()
            .filter_map(|(key, entry)| {
                Some((
                    key.as_str()?.to_owned(),
                    Capability {
                        fallbacks: strings(&entry["fallbacks"]),
                        applies_to: strings(&entry["applies_to"]),
                    },
                ))
            })
            .collect()
    })
}

/// `type_rule.primitives`: the lowercase type names.
pub(crate) fn primitive_types() -> &'static BTreeSet<String> {
    static NAMES: OnceLock<BTreeSet<String>> = OnceLock::new();
    NAMES.get_or_init(|| keys(&schema()["type_rule"]["primitives"]))
}

/// `type_rule.constructors.ref.kinds`: what a `{ref: kind}` may name.
pub(crate) fn ref_kinds() -> &'static BTreeSet<String> {
    static KINDS: OnceLock<BTreeSet<String>> = OnceLock::new();
    KINDS.get_or_init(|| {
        strings(&schema()["type_rule"]["constructors"]["ref"]["kinds"])
            .into_iter()
            .collect()
    })
}

/// `constructs.PlacementProfile.profiles.<profile>.defaults`: the store per state class.
pub(crate) fn profile_defaults(profile: &str) -> BTreeMap<String, String> {
    schema()["constructs"]["PlacementProfile"]["profiles"][profile]["defaults"]
        .as_mapping()
        .into_iter()
        .flatten()
        .filter_map(|(class, store)| Some((class.as_str()?.to_owned(), store.as_str()?.to_owned())))
        .collect()
}

/// The props of each primitive kind: the `fields` of the construct each value of
/// `constructs.Primitive.fields.primitive.type.enum` names.
pub(crate) fn primitive_props() -> &'static Vec<BTreeSet<String>> {
    static PROPS: OnceLock<Vec<BTreeSet<String>>> = OnceLock::new();
    PROPS.get_or_init(|| {
        let constructs = &schema()["constructs"];
        strings(&constructs["Primitive"]["fields"]["primitive"]["type"]["enum"])
            .iter()
            .map(|kind| keys(&constructs[kind.as_str()]["fields"]))
            .collect()
    })
}

/// The sources an unnamed action's name is derived from: the `first_present` list of the
/// `shorthands.index` entry for `Action` at `name`.
pub(crate) fn action_name_sources() -> &'static Vec<String> {
    static SOURCES: OnceLock<Vec<String>> = OnceLock::new();
    SOURCES.get_or_init(|| {
        schema()["shorthands"]["index"]
            .as_sequence()
            .into_iter()
            .flatten()
            .find(|entry| {
                entry["construct"].as_str() == Some("Action")
                    && entry["at"].as_str() == Some("name")
            })
            .map(|entry| strings(&entry["expands_to"]["first_present"]))
            .expect("the schema derives an unnamed action's name")
    })
}

/// The keys whose map entries are named nodes: every construct field named so is typed
/// `{map: {key: name, value: <construct>}}` (either side possibly `optional`) — `pages`,
/// `widgets`, `state`, `overlays`, `regions`, … A key some construct types as a map of anything
/// else (`params` holds expressions on an overlay, `args` json) names no node.
pub(crate) fn node_maps() -> &'static BTreeSet<String> {
    static KEYS: OnceLock<BTreeSet<String>> = OnceLock::new();
    KEYS.get_or_init(|| {
        let constructs = &schema()["constructs"];
        let construct_names = keys(constructs);
        let bare = |ty: &'static Value| ty.get("optional").unwrap_or(ty);
        let mut named = BTreeSet::new();
        let mut other = BTreeSet::new();
        for construct in constructs
            .as_mapping()
            .into_iter()
            .flatten()
            .map(|(_, c)| c)
        {
            for (key, spec) in construct["fields"].as_mapping().into_iter().flatten() {
                let Some(key) = key.as_str().filter(|key| !key.starts_with('(')) else {
                    continue;
                };
                let Some(map) = bare(&spec["type"]).get("map") else {
                    continue;
                };
                let of_nodes = map["key"].as_str() == Some("name")
                    && bare(&map["value"])
                        .as_str()
                        .is_some_and(|v| construct_names.contains(v));
                if of_nodes {
                    named.insert(key.to_owned());
                } else {
                    other.insert(key.to_owned());
                }
            }
        }
        named.difference(&other).cloned().collect()
    })
}

/// `expressions.forms[].form`: `state.<name>`, `same_as(<section>)`, …, `operators`.
pub(crate) fn expression_forms() -> &'static Vec<String> {
    static FORMS: OnceLock<Vec<String>> = OnceLock::new();
    FORMS.get_or_init(|| {
        schema()["expressions"]["forms"]
            .as_sequence()
            .expect("the schema lists the expression forms")
            .iter()
            .filter_map(|entry| entry["form"].as_str().map(str::to_owned))
            .collect()
    })
}

/// `unmapped_marker.accepted_by.types`: the primitive type names (`string`, `expr`, …) and the
/// constructors (`enum` for `{enum: …}`, `ref` for `{ref: …}`) that accept the marker.
pub(crate) struct MarkerAccepted {
    pub(crate) names: BTreeSet<String>,
    pub(crate) constructors: BTreeSet<String>,
}

pub(crate) fn marker_accepted_by() -> &'static MarkerAccepted {
    static ACCEPTED: OnceLock<MarkerAccepted> = OnceLock::new();
    ACCEPTED.get_or_init(|| {
        let mut accepted = MarkerAccepted {
            names: BTreeSet::new(),
            constructors: BTreeSet::new(),
        };
        for ty in strings(&schema()["unmapped_marker"]["accepted_by"]["types"]) {
            match ty.strip_prefix('{').and_then(|rest| rest.split_once(':')) {
                Some((constructor, _)) => {
                    accepted.constructors.insert(constructor.trim().to_owned());
                }
                None => {
                    accepted.names.insert(ty);
                }
            }
        }
        assert!(
            !accepted.names.is_empty(),
            "the schema says which types accept the UNMAPPED marker"
        );
        accepted
    })
}

/// `unmapped_marker.pattern`, `^UNMAPPED: .+$`, as a test.
pub(crate) fn is_unmapped_marker(text: &str) -> bool {
    text.strip_prefix("UNMAPPED: ")
        .is_some_and(|reason| !reason.is_empty() && !reason.contains('\n'))
}
