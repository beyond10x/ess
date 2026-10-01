//! The types crate's `actor` module: every declared actor, and the commands each may invoke, as
//! data (`story:actor-grants-as-data`).
//!
//! This is the grant as data: an `Actor` enum with one variant per declared actor, `may(actor)`,
//! the qualified names of the commands it may invoke, and `Caller`, who a request was
//! authenticated as. Nothing here checks anything. A served surface checks a `Caller` it is handed
//! before the command runs (beyond10x/ess#265); without one, the caller that knows who is calling
//! does, and the plan says so.
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
         data.\n//!\n//! A grant is checked against a caller identity, which these types do not \
         read from anywhere:\n//! whatever authenticates a request builds a [`Caller`], and a \
         served surface checks it against\n//! [`may`] before the command runs. The `PLAN.md` \
         beside this workspace says, per actor,\n//! whether a generated surface enforces the \
         grant or the caller does.\n\n/// An actor the specification declares.\n#[derive(Debug, \
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
    out.push_str(CALLER);
    Some(Artifact::new(
        format!("crates/{}/src/{MODULE}.rs", layout.package()),
        out,
    ))
}

/// The authenticated caller, and the one question a surface asks of it (beyond10x/ess#265).
///
/// A value the shell that authenticated a request hands the served surface, never one the surface
/// reads out of the request: a client can write anything into a request, so an actor derived from
/// it is an actor any client can claim.
const CALLER: &str = "\n/// Who a request was authenticated as.\n///\n/// Built by whatever \
                      authenticates the request — a session, a token, a certificate — and\n/// \
                      handed to the served surface's `dispatch` and `handle`, which check its \
                      grant\n/// before the command runs. Never derived from the request itself: \
                      a client can write\n/// anything into a request.\n#[derive(Debug, Clone, \
                      Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]\npub struct Caller {\n    /// \
                      The declared actor.\n    pub actor: Actor,\n}\n\nimpl Caller {\n    /// \
                      `true` when this caller may invoke `command`, named by its qualified \
                      name.\n    pub fn may(&self, command: &str) -> bool {\n        \
                      may(self.actor).contains(&command)\n    }\n}\n";

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
