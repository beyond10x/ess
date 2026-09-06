//! Source-pinned normalization planning and realization.

mod check;
mod eval;
mod execute;
mod go_target;
mod input;
mod numeric;
mod recipe;
mod target;

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

use super::{finding, path, Finding, Plan as Types, Refused};
use crate::bundle::{source_digest, Bundle};
use recipe::unique_map;
pub use recipe::{
    Binary64Inputs, Binary64Range, Binary64Step, Condition, Expr, IntegerOp, NumberPath, Overflow,
    Recipe, Root, Scope, Stage, FORMAT, FORMAT_V2,
};
pub use target::{Realization, Report};

impl Root {
    /// Pins the complete bundle identity; root admission is checked by the recipe planner.
    pub fn pin(
        bundle: &Bundle,
        root: impl Into<String>,
    ) -> Result<Self, crate::bundle::ImportError> {
        Ok(Self {
            bundle_digest: source_digest(&bundle.to_json()?),
            root: root.into(),
        })
    }
}

/// A recipe admitted only after every branch, source identity and expression checks.
#[derive(Debug, Clone)]
pub struct Plan {
    recipe: Recipe,
    bundles: BTreeMap<String, Bundle>,
}

impl Plan {
    /// Emit a standalone Rust normalization library with pinned schemas and source provenance.
    pub fn rust(&self, package: &str) -> Result<Realization, Refused> {
        target::rust(self, package)
    }

    /// Emit a standalone Go normalization library with explicit module identity.
    pub fn go(&self, package: &str, module: &str) -> Result<Realization, Refused> {
        go_target::generate(self, package, module)
    }

    /// Strict parsing and checking; unknown fields, versions and operations refuse.
    pub fn read(text: &str, bundles: &[Bundle]) -> Result<Self, Refused> {
        let recipe = serde_json::from_str(text)
            .map_err(|error| Refused(vec![finding("/", "recipe_syntax", &error.to_string())]))?;
        Self::check(recipe, bundles)
    }

    /// Checks all branches before returning any executable plan.
    pub fn check(recipe: Recipe, bundles: &[Bundle]) -> Result<Self, Refused> {
        let mut found = Vec::new();
        let mut retained = BTreeMap::new();
        for bundle in bundles {
            let bytes = bundle.to_json().map_err(|error| {
                Refused(vec![finding("/", "bundle_encoding", &error.to_string())])
            })?;
            retained.insert(source_digest(&bytes), bundle.clone());
        }
        if recipe.format != FORMAT && recipe.format != FORMAT_V2 {
            found.push(finding(
                "/format",
                "recipe_format",
                "unsupported normalization format",
            ));
        }
        if recipe.branches.is_empty() {
            found.push(finding(
                "/branches",
                "empty_dispatch",
                "declare at least one dispatch branch",
            ));
        }
        if let Some(paths) = &recipe.binary64_inputs {
            if recipe.format != FORMAT_V2 {
                found.push(finding(
                    "/binary64_inputs",
                    "operation_version",
                    "numeric input declarations require ess-normalization/2",
                ));
            }
            for branch in paths.keys() {
                if !recipe.branches.contains_key(branch) {
                    found.push(finding(
                        &path("/binary64_inputs", branch),
                        "unknown_branch",
                        "numeric input declaration names an unknown branch",
                    ));
                }
            }
        }
        for (name, stages) in &recipe.branches {
            let at = super::path("/branches", name);
            if stages.is_empty() {
                found.push(finding(&at, "empty_pipeline", "declare at least one stage"));
            }
            for (index, stage) in stages.iter().enumerate() {
                let at = format!("{at}/{index}");
                if index > 0 && stages[index - 1].output != stage.input {
                    found.push(finding(
                        &format!("{at}/input"),
                        "stage_identity",
                        "input identity differs from the previous output",
                    ));
                }
                let input = selection(&stage.input, &retained, &format!("{at}/input"), &mut found);
                let output = selection(
                    &stage.output,
                    &retained,
                    &format!("{at}/output"),
                    &mut found,
                );
                if let (Some(input), Some(output)) = (input, output) {
                    if index == 0 {
                        check::input_numbers(
                            recipe.binary64_paths(name),
                            &input,
                            &stage.input.root,
                            &path("/binary64_inputs", name),
                            &mut found,
                        );
                    }
                    check::stage(
                        stage,
                        &input,
                        &output,
                        &at,
                        recipe.format == FORMAT_V2,
                        &mut found,
                    );
                }
            }
        }
        if !found.is_empty() {
            return Err(Refused(found));
        }
        Ok(Self {
            recipe,
            bundles: retained,
        })
    }

    /// Deterministic authored bytes; reload must still check the referenced bundles.
    pub fn to_json(&self) -> String {
        format!(
            "{}\n",
            serde_json::to_string_pretty(&self.recipe).expect("typed recipe serializes")
        )
    }

    /// Parse duplicate-free JSON without silently rounding numeric values, then execute.
    /// Fractional/exponent spellings remain nonintegral for integer operations.
    pub fn run_json(&self, branch: &str, input: &str) -> Result<Value, Refused> {
        self.run_prepared(
            branch,
            &input::parse(input, self.recipe.binary64_paths(branch))?,
        )
    }

    /// Evaluate a branch atomically with respect to caller data and observable result.
    /// The caller owns decoding precision; use [`Self::run_json`] for raw JSON bytes.
    pub fn run(&self, branch: &str, input: &Value) -> Result<Value, Refused> {
        if self.recipe.binary64_paths(branch).is_empty() {
            return self.run_prepared(branch, input);
        }
        self.run_prepared(
            branch,
            &input::prepare(input, self.recipe.binary64_paths(branch))?,
        )
    }

    fn run_prepared(&self, branch: &str, input: &Value) -> Result<Value, Refused> {
        execute::run(&self.recipe, branch, input, |root, value, at| {
            self.validate(root, value, at)
        })
    }

    fn validate(&self, root: &Root, value: &Value, at: &str) -> Result<(), Refused> {
        let found = self.bundles[&root.bundle_digest]
            .validate(&root.root, value)
            .map_err(|error| Refused(vec![finding(at, "schema_validation", &error.to_string())]))?;
        if found.is_empty() {
            return Ok(());
        }
        Err(Refused(
            found
                .iter()
                .map(|error| {
                    finding(
                        &format!("{at}{}", error.pointer),
                        "schema_validation",
                        "value does not satisfy the selected stage contract",
                    )
                })
                .collect(),
        ))
    }
}

fn selection(
    root: &Root,
    bundles: &BTreeMap<String, Bundle>,
    at: &str,
    found: &mut Vec<Finding>,
) -> Option<Types> {
    let Some(bundle) = bundles.get(&root.bundle_digest) else {
        found.push(finding(
            at,
            "unknown_bundle",
            "no supplied checked bundle has this canonical digest",
        ));
        return None;
    };
    match Types::from_bundle(bundle, &BTreeSet::from([root.root.clone()])) {
        Ok(plan) => Some(plan),
        Err(errors) => {
            found.extend(errors.0.into_iter().map(|error| {
                finding(
                    at,
                    "root_selection",
                    &format!("{}: {}", error.pointer, error.detail),
                )
            }));
            None
        }
    }
}
