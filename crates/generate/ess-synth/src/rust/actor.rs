//! The types crate's `actor` module: every declared actor, and the commands each may invoke, as
//! data (`story:actor-grants-as-data`).
//!
//! The plan refuses *enforcing* a grant — a grant is checked against a caller identity, which
//! types do not carry — and says the grant is generated as data. This is that data: an `Actor`
//! enum with one variant per declared actor and `may(actor)`, the qualified names of the commands
//! it may invoke. Nothing here checks anything; the caller that knows who is calling does.
//!
//! Emitted only for a model that declares an actor, so a model without one keeps its bytes.
//!
//! # Order
//!
//! Actors in the IR's order, and each grant list in the IR's order: both are keyed by qualified
//! name. The IR holds a grant list as a set on purpose — the same grant written twice, or the same
//! grants reordered, mean the same thing, and a revision that only reorders them is no change in
//! the semantic diff. A table in the order the author happened to type would make that reorder a
//! change in generated code while every other projection calls it none.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use ess_compiler::ir::EssIr;
use ess_domain::name::QualifiedName;
use ess_gen::{Artifact, Provenance};

use crate::plan::REGENERATE;

use super::layout::Layout;
use super::name;

/// The module's name in the types crate, reserved against a domain spelled the same way.
pub(crate) const MODULE: &str = "actor";

/// `true` when the model declares an actor, and so the types crate carries this module.
pub(crate) fn used(ir: &EssIr) -> bool {
    !ir.actors().is_empty()
}

/// The `actor` module, or `None` for a model that declares no actor.
pub(crate) fn module(ir: &EssIr, layout: &Layout, provenance: &Provenance) -> Option<Artifact> {
    if !used(ir) {
        return None;
    }
    let variants = variants(ir);
    let mut out = provenance.commented_for("//", REGENERATE);
    out.push_str(
        "\n//! Every actor the specification declares, and the commands each may invoke — as \
         data.\n//!\n//! Generated, not enforced: a grant is checked against a caller identity, \
         which types do not\n//! carry, so the caller that knows who is calling enforces it, \
         against [`may`]. The `PLAN.md`\n//! beside this workspace records the same refusal \
         for every actor.\n\n/// An actor the specification declares.\n#[derive(Debug, \
         Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]\npub enum Actor {\n",
    );
    for (actor, variant) in &variants {
        let _ = writeln!(out, "    /// `{actor}`.\n    {variant},");
    }
    out.push_str(
        "}\n\nimpl Actor {\n    /// Every declared actor, ordered by qualified name.\n    pub const ALL: \
         &'static [Actor] = &[\n",
    );
    for variant in variants.values() {
        let _ = writeln!(out, "        Actor::{variant},");
    }
    out.push_str(
        "    ];\n\n    /// The actor's qualified name, as the specification spells it.\n    pub \
         const fn name(self) -> &'static str {\n        match self {\n",
    );
    for (actor, variant) in &variants {
        let _ = writeln!(out, "            Actor::{variant} => \"{actor}\",");
    }
    out.push_str(
        "        }\n    }\n}\n\n/// The qualified names of the commands `actor` may invoke, ordered by \
         name; empty for an\n/// actor that only observes.\npub fn may(actor: Actor) -> &'static \
         [&'static str] {\n    match actor {\n",
    );
    for (actor, variant) in &variants {
        let grants = &ir
            .actors()
            .get(actor)
            .expect("a variant names a declared actor")
            .may;
        if grants.is_empty() {
            let _ = writeln!(out, "        Actor::{variant} => &[],");
            continue;
        }
        let _ = writeln!(out, "        Actor::{variant} => &[");
        for command in grants {
            let _ = writeln!(out, "            \"{command}\",");
        }
        out.push_str("        ],\n");
    }
    out.push_str("    }\n}\n");
    Some(Artifact::new(
        format!("crates/{}/src/{MODULE}.rs", layout.package()),
        out,
    ))
}

/// One variant name per actor, collision-free by rule: the actor's domain-relative type name, or
/// — when two domains declare the same one — every actor's full name minus the system prefix,
/// pascal-joined. All of them switch, not just the colliding pair, so declaring one actor cannot
/// silently rename a variant another crate matches on (the rule events follow for the same reason).
fn variants(ir: &EssIr) -> BTreeMap<&QualifiedName, String> {
    let local = |actor: &QualifiedName| {
        let domain = &ir.actors()[actor].domain;
        guard(name::type_name(actor, domain.name().segments().len()))
    };
    let mut candidates: BTreeMap<&QualifiedName, String> = ir
        .actors()
        .keys()
        .map(|actor| (actor, local(actor)))
        .collect();
    let distinct: BTreeSet<&String> = candidates.values().collect();
    if distinct.len() != candidates.len() {
        let prefix = ir.system().segments().len();
        candidates = ir
            .actors()
            .keys()
            .map(|actor| {
                let segments = actor.segments();
                let full: String = segments
                    .get(prefix..)
                    .unwrap_or(segments)
                    .iter()
                    .map(|segment| name::pascal(segment))
                    .collect();
                (actor, guard(full))
            })
            .collect();
    }
    candidates
}

/// `Self` is the one pascal-cased word that cannot name a variant.
fn guard(variant: String) -> String {
    if variant == "Self" {
        "Self_".to_owned()
    } else {
        variant
    }
}
