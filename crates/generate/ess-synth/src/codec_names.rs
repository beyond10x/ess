//! The stems a served surface's codecs and handlers are named by, allocated once per model
//! (beyond10x/ess#415, `docs/design/served-codec-names.md`).
//!
//! A codec is named after its declaration's whole qualified name flattened into one identifier:
//! `renewal.input.AB` is `renewal_input_a_b` in Rust and `RenewalInputAB` in Go. Flattening drops
//! the separators, so it is not injective — `renewal.input.A_B` and `renewal.input_a.B` flatten to
//! the same identifier — and two codecs under one name are one function too many. This module is
//! the one place that turns the flattened candidates into names, for every call site of every
//! target and for the Rust feasibility inventory, so the name a handler calls is the name the codec
//! was declared under.
//!
//! The rule, per family of declarations whose codecs share their prefixes:
//!
//! 1. A candidate no other declaration of the family flattens to is the name, unchanged. A model
//!    with no such collision keeps every name it had.
//! 2. Declarations that share a candidate are ordered by the bytes of their qualified names. The
//!    first keeps the candidate; each later one takes the smallest `_2`, `_3`, … suffix whose
//!    result no declaration of the family flattens to and no earlier allocation took.
//!
//! The allocation depends only on the set of names, never on declaration order. A collision
//! *across* families (a declared type whose flattened name starts with `command_` and a command's
//! input codec, say) is outside this rule and stays the Rust target's `wire-collision` refusal.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::EssIr;
use ess_domain::name::QualifiedName;

/// Every codec stem of one model, for one target's flattening.
pub(crate) struct CodecNames {
    stems: BTreeMap<QualifiedName, String>,
}

impl CodecNames {
    /// Allocates the stem of every declared type, event, error, command and view of `ir`, from the
    /// target's own flattening of a qualified name.
    pub(crate) fn of(ir: &EssIr, flatten: impl Fn(&QualifiedName) -> String) -> Self {
        let mut stems = BTreeMap::new();
        // Commands and views are one family: both are served, and their handlers share the
        // `serve`/`run` prefixes in the module that routes to them.
        let families: [Vec<&QualifiedName>; 4] = [
            ir.types().keys().collect(),
            ir.events().keys().collect(),
            ir.errors().keys().collect(),
            ir.commands().keys().chain(ir.views().keys()).collect(),
        ];
        for family in families {
            stems.extend(allocate(family, &flatten));
        }
        Self { stems }
    }

    /// The stem of a declaration's codecs.
    ///
    /// Total for this IR's types, events, errors, commands and views, like the layouts' owner
    /// lookups: a miss is a name from a different compilation.
    pub(crate) fn stem(&self, declared: &QualifiedName) -> &str {
        self.stems.get(declared).map_or_else(
            || panic!("`{declared}` has no codec stem: it was derived from a different IR"),
            String::as_str,
        )
    }
}

/// The rule of this module over one family.
fn allocate<'a>(
    family: impl IntoIterator<Item = &'a QualifiedName>,
    flatten: impl Fn(&QualifiedName) -> String,
) -> BTreeMap<QualifiedName, String> {
    let mut groups: BTreeMap<String, Vec<&QualifiedName>> = BTreeMap::new();
    for declared in family {
        groups.entry(flatten(declared)).or_default().push(declared);
    }
    // Every candidate is reserved before any suffix is chosen, so a suffix never takes a name a
    // declaration of its own family flattens to.
    let mut taken: BTreeSet<String> = groups.keys().cloned().collect();
    let mut stems = BTreeMap::new();
    for (candidate, mut declared) in groups {
        declared.sort_by_key(ToString::to_string);
        let mut later = declared.into_iter();
        let first = later.next().expect("a group has a member");
        stems.insert(first.clone(), candidate.clone());
        let mut suffix = 2_usize;
        for name in later {
            let stem = loop {
                let stem = format!("{candidate}_{suffix}");
                suffix += 1;
                if !taken.contains(&stem) {
                    break stem;
                }
            };
            taken.insert(stem.clone());
            stems.insert(name.clone(), stem);
        }
    }
    stems
}

#[cfg(test)]
mod tests {
    use ess_domain::name::QualifiedName;

    use super::allocate;

    fn names(spelled: &[&str]) -> Vec<QualifiedName> {
        spelled
            .iter()
            .map(|name| QualifiedName::new(name).expect("a legal name"))
            .collect()
    }

    fn allocated(
        spelled: &[&str],
        flatten: impl Fn(&QualifiedName) -> String,
    ) -> Vec<(String, String)> {
        allocate(names(spelled).iter(), flatten)
            .into_iter()
            .map(|(name, stem)| (name.to_string(), stem))
            .collect()
    }

    /// A deliberately lossy flattening: every name to its letters, lower-cased.
    fn letters(name: &QualifiedName) -> String {
        name.to_string()
            .chars()
            .filter(char::is_ascii_alphanumeric)
            .collect::<String>()
            .to_lowercase()
    }

    #[test]
    fn distinct_candidates_are_kept_unchanged() {
        assert_eq!(
            allocated(&["a.b.C", "a.b.D"], letters),
            [
                ("a.b.C".to_owned(), "abc".to_owned()),
                ("a.b.D".to_owned(), "abd".to_owned())
            ]
        );
    }

    #[test]
    fn the_first_in_byte_order_keeps_the_candidate_whatever_the_declaration_order() {
        let forward = allocated(&["a.b.AB", "a.b.A_B", "a.b_a.B"], letters);
        let backward = allocated(&["a.b_a.B", "a.b.A_B", "a.b.AB"], letters);
        assert_eq!(forward, backward);
        assert_eq!(
            forward,
            [
                ("a.b.AB".to_owned(), "abab".to_owned()),
                ("a.b.A_B".to_owned(), "abab_2".to_owned()),
                ("a.b_a.B".to_owned(), "abab_3".to_owned()),
            ]
        );
    }

    #[test]
    fn a_suffix_skips_a_name_another_declaration_flattens_to() {
        // `a.b.X2` flattens to `abx_2` under this flattening, so the second `abx` takes `_3`.
        let flatten = |name: &QualifiedName| match name.to_string().as_str() {
            "a.b.X2" => "abx_2".to_owned(),
            _ => letters(name),
        };
        assert_eq!(
            allocated(&["a.b.X", "a.b.X2", "a.b.x"], flatten),
            [
                ("a.b.X".to_owned(), "abx".to_owned()),
                ("a.b.X2".to_owned(), "abx_2".to_owned()),
                ("a.b.x".to_owned(), "abx_3".to_owned()),
            ]
        );
    }
}
