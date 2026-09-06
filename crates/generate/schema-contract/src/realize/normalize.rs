//! Source-pinned normalization planning and realization.

mod check;
mod eval;
mod execute;
mod go_target;
mod input;
mod numeric;
mod recipe;
mod source;
mod target;

use serde_json::Value;
use std::collections::BTreeMap;

use super::{finding, path, Finding, Plan as Types, Refused};
use crate::bundle::{source_digest, Bundle};
use recipe::unique_map;
pub use recipe::{
    Binary64Inputs, Binary64Range, Binary64Step, Condition, Expr, IntegerOp, ModelIdentity,
    NumberPath, Overflow, Recipe, Root, Scope, Stage, FORMAT, FORMAT_V2, FORMAT_V3,
};
pub use target::{Realization, Report};

impl Root {
    /// Pins the complete bundle identity; root admission is checked by the recipe planner.
    pub fn pin(
        bundle: &Bundle,
        root: impl Into<String>,
    ) -> Result<Self, crate::bundle::ImportError> {
        Ok(Self::Bundle {
            bundle_digest: source_digest(&bundle.to_json()?),
            root: root.into(),
        })
    }
}

impl Recipe {
    fn envelope_findings(&self) -> Vec<Finding> {
        let mut found = Vec::new();
        if ![FORMAT, FORMAT_V2, FORMAT_V3].contains(&self.format.as_str()) {
            found.push(finding(
                "/format",
                "recipe_format",
                "unsupported normalization format",
            ));
        }
        if self.branches.is_empty() {
            found.push(finding(
                "/branches",
                "empty_dispatch",
                "declare at least one dispatch branch",
            ));
        }
        if let Some(paths) = &self.binary64_inputs {
            if self.format == FORMAT {
                found.push(finding(
                    "/binary64_inputs",
                    "operation_version",
                    "numeric input declarations require ess-normalization/2",
                ));
            }
            for branch in paths.keys() {
                if !self.branches.contains_key(branch) {
                    found.push(finding(
                        &path("/binary64_inputs", branch),
                        "unknown_branch",
                        "numeric input declaration names an unknown branch",
                    ));
                }
            }
        }
        found
    }
}

/// A recipe admitted only after every branch, source identity and expression checks.
#[derive(Debug, Clone)]
pub struct Plan {
    recipe: Recipe,
    bundles: BTreeMap<String, Bundle>,
    models: BTreeMap<ModelIdentity, ess_gen::schema::ModelTypes>,
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
        Self::check_with_models(recipe, bundles, &[])
    }

    /// Check qualified bundles and sealed compiler-owned model selections together.
    pub fn check_with_models(
        recipe: Recipe,
        bundles: &[Bundle],
        models: &[ess_gen::schema::ModelTypes],
    ) -> Result<Self, Refused> {
        let mut found = recipe.envelope_findings();
        let mut retained = BTreeMap::new();
        for bundle in bundles {
            let bytes = bundle.to_json().map_err(|error| {
                Refused(vec![finding("/", "bundle_encoding", &error.to_string())])
            })?;
            retained.insert(source_digest(&bytes), bundle.clone());
        }
        let models = models
            .iter()
            .map(|model| (ModelIdentity::pin(model), model.clone()))
            .collect();
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
                for (position, root) in [("input", &stage.input), ("output", &stage.output)] {
                    if matches!(root, Root::Model { .. }) && recipe.format != FORMAT_V3 {
                        found.push(finding(
                            &format!("{at}/{position}"),
                            "root_version",
                            "model roots require ess-normalization/3",
                        ));
                    }
                }
                let input = source::selection(
                    &stage.input,
                    &retained,
                    &models,
                    &format!("{at}/input"),
                    &mut found,
                );
                let output = source::selection(
                    &stage.output,
                    &retained,
                    &models,
                    &format!("{at}/output"),
                    &mut found,
                );
                if let (Some(input), Some(output)) = (input, output) {
                    if index == 0 {
                        check::input_numbers(
                            recipe.binary64_paths(name),
                            &input,
                            stage.input.name(),
                            &path("/binary64_inputs", name),
                            &mut found,
                        );
                    }
                    check::stage(
                        stage,
                        &input,
                        &output,
                        &at,
                        recipe.format != FORMAT,
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
            models,
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
        let Root::Bundle {
            bundle_digest,
            root: name,
        } = root
        else {
            return source::validate_model(self, root, value, at);
        };
        let found = self.bundles[bundle_digest]
            .validate(name, value)
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
