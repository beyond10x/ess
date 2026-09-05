//! Shared native-target decisions over the checked structural plan.

use super::{Node, Plan, Shape};
use serde_json::Value;
use std::collections::BTreeSet;

pub(super) fn nullable(variants: &[Node]) -> Option<&Node> {
    if variants.len() != 2 {
        return None;
    }
    match (&variants[0].shape, &variants[1].shape) {
        (Shape::Null, _) => Some(&variants[1]),
        (_, Shape::Null) => Some(&variants[0]),
        _ => None,
    }
}

pub(super) fn string_values(
    node: &Node,
    plan: &Plan,
    seen: &mut BTreeSet<String>,
) -> Option<BTreeSet<String>> {
    match &node.shape {
        Shape::Literal(Value::String(value)) => Some(BTreeSet::from([value.clone()])),
        Shape::Ref(name) => {
            if !seen.insert(name.clone()) {
                return None;
            }
            let result = string_values(&plan.definitions[name], plan, seen);
            seen.remove(name);
            result
        }
        Shape::Union { variants, .. } => {
            let mut values = BTreeSet::new();
            for variant in variants {
                values.extend(string_values(variant, plan, seen)?);
            }
            Some(values)
        }
        Shape::Intersection(terms) => {
            let mut values = terms
                .iter()
                .find_map(|term| string_values(term, plan, seen))?;
            values.retain(|value| {
                terms
                    .iter()
                    .all(|term| accepts_string(term, value, plan, &mut BTreeSet::new()))
            });
            Some(values)
        }
        _ => None,
    }
}

fn accepts_string(node: &Node, value: &str, plan: &Plan, seen: &mut BTreeSet<String>) -> bool {
    match &node.shape {
        Shape::Json | Shape::String => true,
        Shape::Literal(Value::String(expected)) => value == expected,
        Shape::Ref(name) => {
            if !seen.insert(name.clone()) {
                return false;
            }
            let result = accepts_string(&plan.definitions[name], value, plan, seen);
            seen.remove(name);
            result
        }
        Shape::Union { variants, .. } => variants
            .iter()
            .any(|variant| accepts_string(variant, value, plan, seen)),
        Shape::Intersection(terms) => terms
            .iter()
            .all(|term| accepts_string(term, value, plan, seen)),
        _ => false,
    }
}
