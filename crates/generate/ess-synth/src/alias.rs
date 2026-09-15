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

use ess_compiler::ir::EssIr;
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
