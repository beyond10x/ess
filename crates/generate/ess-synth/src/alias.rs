//! The identifiers an author chose for a code emitter.
//!
//! A target with no dotted type names flattens a qualified name to one identifier, so
//! `billing.invoice.Invoice.State` and an authored `billing.invoice.InvoiceState` both want to be
//! `InvoiceState`. One of them has to move. `naming: { code: … }` is where an author says which,
//! and this is the one place that is read — so Rust and Go cannot disagree about a name somebody
//! wrote down.
//!
//! Derived declarations never appear here. An entity's lifecycle enum is filed in `ir.types()` like
//! any other type but carries `Naming::default()`, so it has no alias to find, which is right: it is
//! the one declaration with no author to ask.

use std::collections::BTreeMap;

use ess_compiler::ir::{EssIr, ResolvedBody};
use ess_domain::name::QualifiedName;

/// Every declaration that wrote `naming: { code: … }`, and what it wrote.
pub(crate) fn code_aliases(ir: &EssIr) -> BTreeMap<QualifiedName, String> {
    let mut aliases = BTreeMap::new();
    macro_rules! collect {
        ($($accessor:ident),+ $(,)?) => {
            $(for (name, declared) in ir.$accessor() {
                if let Some(code) = declared.naming.code() {
                    aliases.insert(name.clone(), code.to_owned());
                }
            })+
        };
    }
    collect!(types, entities, commands, events, errors, views);
    aliases
}

/// Every enum variant that wrote `code:`, by its enum and then its name, and what it wrote
/// (beyond10x/ess#662).
///
/// A variant's identifier is derived from its name, and a wire spelling used as the name can
/// derive one another variant holds (`a.b` and `a_b` are both `AB`) or none at all (`1.0`). The
/// author names it the way a declaration is named, and both code targets read it here. A union
/// label declares no naming, so it has no entry.
pub(crate) fn variant_code_aliases(
    ir: &EssIr,
) -> BTreeMap<QualifiedName, BTreeMap<String, String>> {
    let mut aliases: BTreeMap<QualifiedName, BTreeMap<String, String>> = BTreeMap::new();
    for (name, declared) in ir.types() {
        if let ResolvedBody::Enum { variants } = &declared.body {
            for variant in variants {
                if let Some(code) = variant.naming.code() {
                    aliases
                        .entry(name.clone())
                        .or_default()
                        .insert(variant.name.clone(), code.to_owned());
                }
            }
        }
    }
    aliases
}
