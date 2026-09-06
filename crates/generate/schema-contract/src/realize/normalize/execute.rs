//! Ordered reference execution, shared verbatim with generated Rust adapters.

use super::{eval, finding, path, Recipe, Refused, Root};
use serde_json::Value;

pub(super) fn run(
    recipe: &Recipe,
    position_arities: &std::collections::BTreeMap<String, u64>,
    branch: &str,
    input: &Value,
    validate: impl Fn(&Root, &Value, &str) -> Result<(), Refused>,
) -> Result<Value, Refused> {
    let stages = recipe.branches.get(branch).ok_or_else(|| {
        Refused(vec![finding(
            "/branches",
            "unknown_dispatch",
            "external discriminator has no declared branch",
        )])
    })?;
    let mut current = input.clone();
    for (index, stage) in stages.iter().enumerate() {
        let at = format!("{}/{index}", path("/branches", branch));
        validate(&stage.input, &current, &format!("{at}/input"))?;
        let context = eval::Context {
            input: &current,
            item: None,
            index: None,
            binary64: [super::recipe::FORMAT_V5, super::recipe::FORMAT_V6]
                .contains(&recipe.format.as_str()),
            position_arities,
        };
        for (index, condition) in stage.requires.iter().enumerate() {
            let at = format!("{at}/requires/{index}");
            if !context.condition(condition, &at)? {
                return Err(Refused(vec![finding(
                    &at,
                    "requirement_failed",
                    "stage requirement is false",
                )]));
            }
        }
        let value = context
            .value(&stage.value, &format!("{at}/value"))?
            .ok_or_else(|| {
                Refused(vec![finding(
                    &at,
                    "missing_output",
                    "stage output is absent",
                )])
            })?;
        validate(&stage.output, &value, &format!("{at}/output"))?;
        current = value;
    }
    Ok(current)
}
