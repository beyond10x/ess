//! Paging: two declared parameters that slice a view's declared order (ess/16, beyond10x/ess#174,
//! `docs/design/view-paging.md`).
//!
//! ```yaml
//! params:
//!   - {name: page, type: Integer}
//!   - {name: size, type: Integer}
//! order_by: [job_id asc]
//! paging: {page: page, size: size, total: true}
//! ```
//!
//! A paged read answers the rows the filter admits, in `order_by:` order, `size` of them starting at
//! `(page - first_page) * size`, and with `total: true` the number of rows the filter admits beside
//! them. A read that sends neither parameter answers every row the filter admits, in order: that is
//! what the scenarios' other reads of the view send, and it is part of what `paging:` declares.
//!
//! What makes the two parameters observable is this block, not the filter, so the `unobservable_fact`
//! refusal of a parameter no filter reads (`ViewSpec::validate_params`) does not apply to them; a
//! filter that reads one as well is refused, because the parameter would then both select rows and
//! slice them. A slice of an unordered view names no particular rows, so `paging:` requires
//! `order_by:`.
//!
//! The free-form, caller-supplied filter expression #174 also asks for is not part of this construct:
//! its value would need a declared grammar before any scenario could settle what it selects.

use std::collections::BTreeSet;

use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};

use super::{unwrap_chain, ViewSpec};
use crate::types::{Primitive, TypeRegistry};

/// How a view's rows are paged: which declared parameters carry the page and its size, how pages
/// are numbered, and whether the answer carries the filtered count.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct Paging {
    /// The declared parameter that selects the page.
    pub page: String,
    /// The declared parameter that bounds how many rows a page holds.
    pub size: String,
    /// The number of the first page: `0` (the default) or `1`.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub first_page: u64,
    /// Whether the answer carries the number of rows the filter admits, beside the page.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub total: bool,
}

#[allow(clippy::trivially_copy_pass_by_ref)] // serde's `skip_serializing_if` passes a reference
fn is_zero(value: &u64) -> bool {
    *value == 0
}

impl Paging {
    /// The two parameter names this block reads.
    pub fn params(&self) -> [&str; 2] {
        [self.page.as_str(), self.size.as_str()]
    }

    /// Whether this block reads the declared parameter named `param`.
    pub fn reads(&self, param: &str) -> bool {
        self.page == param || self.size == param
    }
}

impl ViewSpec {
    /// The parameters `paging:` reads, which are therefore observable without a filter reading them.
    pub(super) fn paging_params(&self) -> BTreeSet<&str> {
        self.paging
            .as_ref()
            .map(|paging| paging.params().into_iter().collect())
            .unwrap_or_default()
    }

    /// Everything `paging:` must be to mean anything: declared parameters, two of them, each an
    /// `Integer`, over a declared order, numbered from 0 or 1, and read by no filter.
    pub(super) fn validate_paging(
        &self,
        types: &TypeRegistry,
        filter_reads: &BTreeSet<String>,
    ) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let Some(paging) = &self.paging else {
            return errors;
        };
        let at = |suffix: &str| format!("view.{}.paging{suffix}", self.name);

        if self.order_by.is_empty() {
            errors.push(
                ValidationError::new(
                    ValidationCode::MissingDeclaration,
                    at(""),
                    format!(
                        "`{}` declares `paging:` and no `order_by:`, so a page is a slice of an \
                         order the view never promised and names no particular rows",
                        self.name
                    ),
                )
                .with_hint("declare `order_by:` over fields the view projects"),
            );
        }
        if paging.page == paging.size {
            errors.push(ValidationError::new(
                ValidationCode::ConflictingDeclaration,
                at(""),
                format!(
                    "`{}` names `{}` as both the page and the size; they are two parameters",
                    self.name, paging.page
                ),
            ));
        }
        if paging.first_page > 1 {
            errors.push(
                ValidationError::new(
                    ValidationCode::UnsupportedConstruct,
                    at(".first_page"),
                    format!(
                        "`first_page` is {}; pages are numbered from 0 or from 1",
                        paging.first_page
                    ),
                )
                .with_hint("write `first_page: 0` (the default) or `first_page: 1`"),
            );
        }
        for (key, name) in [("page", &paging.page), ("size", &paging.size)] {
            let Some(declared) = self.params.iter().find(|param| &param.name == name) else {
                errors.push(
                    ValidationError::new(
                        ValidationCode::UndeclaredReference,
                        at(&format!(".{key}")),
                        format!(
                            "`paging.{key}` names `{name}`, which `params:` does not declare, so \
                             no caller can send it"
                        ),
                    )
                    .with_hint(format!(
                        "declare `{{name: {name}, type: Integer}}` in `params:`"
                    )),
                );
                continue;
            };
            if !unwrap_chain(&declared.type_ref, types)
                .leaf
                .is(Primitive::Integer)
            {
                errors.push(
                    ValidationError::new(
                        ValidationCode::TypeMismatch,
                        format!("view.{}.params.{name}.type", self.name),
                        format!(
                            "`paging.{key}` reads `{name}`, and `{name}` is `{}`; a page and its \
                             size are whole numbers",
                            declared.type_ref
                        ),
                    )
                    .with_hint("declare it `Integer`, a newtype of it, or `Optional<Integer>`"),
                );
            }
            if filter_reads.contains(name.as_str()) {
                errors.push(
                    ValidationError::new(
                        ValidationCode::ConflictingDeclaration,
                        format!("view.{}.filter", self.name),
                        format!(
                            "`{}` reads the paging parameter `{name}` in `filter:`, so it would \
                             both select rows and slice them",
                            self.name
                        ),
                    )
                    .with_hint(format!(
                        "declare a separate parameter for the filter, or drop `param.{name}` from it"
                    )),
                );
            }
        }
        errors
    }

    /// The source-format gate: `paging:` is refused below `ess/16` at the key the author wrote.
    ///
    /// Beside [`Self::absent_value_admission`] in `primitive_admission::specification`, the one
    /// place a view meets its document's format: an older reader fails `paging` as an unknown field
    /// with no version hint.
    pub fn paging_admission(&self, format: crate::system::FormatVersion) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        if self.paging.is_some() && format.major() < crate::system::FormatVersion::V16.major() {
            errors.push(
                ValidationError::new(
                    ValidationCode::UnsupportedFormatVersion,
                    format!("view.{}.paging", self.name),
                    "`paging:` requires specification format ess/16",
                )
                .with_hint("write `format: ess/16` on the source that declares the system"),
            );
        }
        errors
    }
}
