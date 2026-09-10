//! The scenario player: a specification and its authored scenarios, rendered as a page somebody can
//! press play on.
//!
//! # Why this is here and not in a synthesis target
//!
//! A synthesis target emits code somebody ships, and gap register D-2 is the rule that the machinery
//! never chooses a realization for it. This emits no implementation. It emits a **reading**: the
//! model as a page, and the scenarios as a queue that page walks. Every transition it shows is one
//! the model already declared, and where the model decides nothing, the scenario says what happened
//! — which is what a scenario is for.
//!
//! It lives beside [`reference`](crate::reference) and [`faulty`](crate::faulty) because it belongs
//! to the same half of the toolchain: things that exist to make a suite legible and checkable rather
//! than to be deployed.
//!
//! # What it emits
//!
//! | path | what it is |
//! | --- | --- |
//! | `index.html` | the page. Every panel is built from `model.json`; nothing about any system is typed into it |
//! | `player.js` | the engine: groups the suite's flat steps back into acts, walks them, applies each outcome's declared effect |
//! | `model.json` | **the generated part** — the projection of the specification the page needs |
//! | `suite.json` | the compiled `ess-conformance/3` suite |
//! | `assets/vue.esm-browser.prod.js` | vendored, unmodified, with its licence beside it |
//! | `README.md` | how to serve it, and what it does not claim |
//!
//! A specification may add `skin.js` beside these by hand: one module rendering its own outer
//! representation from the same reactive state. The player loads it when present and says so when
//! absent. That is the one place a system's own shape may enter the page, and it is never generated.
//!
//! # The page is machinery; the model is the projection
//!
//! `index.html` and `player.js` are static assets, embedded here and written out unchanged. Only
//! `model.json` is derived. Emitting the page as a string would make every future change to it a
//! change to Rust, and the reason to keep them apart is the same one `ess-gen` keeps
//! `assets/default.css` apart from the HTML it styles.

use std::collections::BTreeMap;

use ess_compiler::EssIr;
use ess_gen::Artifact;

use crate::scenario::ConformanceSuite;

const INDEX: &str = include_str!("../assets/index.html");
const PLAYER: &str = include_str!("../assets/player.js");
const VUE: &str = include_str!("../assets/vue.esm-browser.prod.js");
const VUE_LICENCE: &str = include_str!("../assets/vue.LICENSE");

/// Emit a paired model and completely admitted coverage input for browser replay.
pub fn emit_input(
    ir: &EssIr,
    input: &crate::coverage::AdmittedInput,
) -> Result<BTreeMap<String, Artifact>, crate::admission::AdmissionError> {
    let replay = crate::web_replay::AdmittedReplay::new(ir, input)?;
    let mut out = BTreeMap::new();
    for (path, contents) in [
        ("index.html", INDEX.to_owned()),
        ("player.js", include_str!("../assets/coverage-player.js").to_owned()),
        ("admission.js", include_str!("../assets/coverage-admission.js").to_owned()),
        ("assets/vue.esm-browser.prod.js", VUE.to_owned()),
        ("assets/vue.LICENSE", VUE_LICENCE.to_owned()),
        ("replay.json", replay.to_canonical_json()?),
        ("README.md", format!("{}\nCoverage replay uses the closed replay/1 model and complete input/1 parent chain. It displays selection and omissions, emits no execution report, and cannot reconstruct or authenticate the full model digest from this reduced projection. Literal assignment values and full view evaluation remain outside this player.\n", readme(ir, input.selected().suite()))),
    ] { out.insert(path.into(), Artifact::new(path, contents)); }
    Ok(out)
}

/// Emits the player for `ir` and `suite`.
///
/// The map is keyed by path so a caller writes it the way it writes any other artifact set.
pub fn emit(
    ir: &EssIr,
    suite: &ConformanceSuite,
) -> Result<BTreeMap<String, Artifact>, crate::admission::AdmissionError> {
    crate::admission::model(ir)?;
    let json = suite.to_canonical_json()?;
    let mut out = BTreeMap::new();
    let mut add = |path: &str, contents: String| {
        out.insert(path.to_owned(), Artifact::new(path, contents));
    };

    add("index.html", INDEX.to_owned());
    add("player.js", PLAYER.to_owned());
    add("assets/vue.esm-browser.prod.js", VUE.to_owned());
    add("assets/vue.LICENSE", VUE_LICENCE.to_owned());
    add("model.json", model(ir));
    add("suite.json", json);
    add("README.md", readme(ir, suite));
    Ok(out)
}

/// The projection the page reads.
///
/// Deliberately not the whole IR. A page that received everything would invite a reader to believe
/// it renders everything, and what it renders is exactly this: which entities exist and how they
/// move, what each command outcome does, what each view selects, who may ask, and what reacts.
pub(crate) fn model(ir: &EssIr) -> String {
    let value = serde_json::json!({
        "system": ir.system().to_string(),
        "version": ir.version().to_string(),
        "entities": ir.entities().values().map(entity).collect::<Vec<_>>(),
        "commands": ir.commands().values().map(command).collect::<Vec<_>>(),
        "views": ir.views().values().map(view).collect::<Vec<_>>(),
        "actors": ir.actors().values().map(actor).collect::<Vec<_>>(),
        "bindings": ir.bindings().values().map(binding).collect::<Vec<_>>(),
    });
    serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_owned())
}

fn entity(entity: &ess_compiler::ir::ResolvedEntity) -> serde_json::Value {
    serde_json::json!({
        "name": entity.name.to_string(),
        "display": entity.naming.display_or(&entity.name),
        "identity": entity.identity.name.clone(),
        "initial": entity.lifecycle.initial.to_string(),
        "states": names(entity.lifecycle.states.iter()),
        "terminal": names(entity.lifecycle.terminal.iter()),
        "fields": entity.fields.iter().map(|f| f.name.clone()).collect::<Vec<_>>(),
    })
}

fn command(command: &ess_compiler::ir::ResolvedCommand) -> serde_json::Value {
    serde_json::json!({
        "name": command.name.to_string(),
        "display": command.naming.display_or(&command.name),
        "outcomes": command.outcomes.iter().map(outcome).collect::<Vec<_>>(),
    })
}

fn outcome(outcome: &ess_compiler::ir::ResolvedOutcome) -> serde_json::Value {
    serde_json::json!({
        "name": outcome.name.as_str(),
        "refuses": outcome.refuses,
        "subject": outcome.subject.as_ref().map(subject),
        "emits": names(outcome.emits.iter()),
        "sets": outcome
            .sets
            .iter()
            .map(|set| serde_json::json!({ "target": set.target, "from": set_source(set) }))
            .collect::<Vec<_>>(),
    })
}

/// What an outcome does to its subject, in the words the page needs to move an instance.
fn subject(subject: &ess_compiler::ir::ResolvedSubject) -> serde_json::Value {
    let (kind, transition, from, to) = match &subject.effect {
        ess_compiler::ir::ResolvedEffect::Creates => ("creates", None, Vec::new(), None),
        ess_compiler::ir::ResolvedEffect::Updates => ("updates", None, Vec::new(), None),
        ess_compiler::ir::ResolvedEffect::Moves { transition } => (
            "moves",
            Some(transition.name.clone()),
            names(transition.from.iter()),
            Some(transition.to.to_string()),
        ),
    };
    serde_json::json!({
        "entity": subject.entity.to_string(),
        "kind": kind,
        "transition": transition,
        "from": from,
        "to": to,
    })
}

fn view(view: &ess_compiler::ir::ResolvedView) -> serde_json::Value {
    serde_json::json!({
        "name": view.name.to_string(),
        "display": view.naming.display_or(&view.name),
        "entity": view.source.to_string(),
        "consistency": view.consistency.to_string(),
        "filter": view.filter.as_ref().map(ToString::to_string),
        "fields": view.fields.iter().map(|f| f.name.clone()).collect::<Vec<_>>(),
        "params": view.params.iter().map(|p| p.name.clone()).collect::<Vec<_>>(),
    })
}

fn actor(actor: &ess_compiler::ir::ResolvedActor) -> serde_json::Value {
    serde_json::json!({
        "name": actor.name.to_string(),
        "display": actor.naming.display_or(&actor.name),
        "may": names(actor.may.iter()),
    })
}

fn binding(binding: &ess_compiler::ir::ResolvedBinding) -> serde_json::Value {
    serde_json::json!({
        "name": binding.name.as_str(),
        "event": binding.event.to_string(),
        "command": binding.command.to_string(),
        "delivery": delivery(binding.delivery),
        "failure": binding.failure.to_string(),
    })
}

fn names<T: ToString>(items: impl Iterator<Item = T>) -> Vec<String> {
    items.map(|x| x.to_string()).collect()
}

/// Which input a `sets:` entry reads, or `None` where it writes a literal.
///
/// The player needs the *input's* name to find the value in a scenario's step, and it is not always
/// the field's name — `TerminateSession` writes `termination` from an input called `reason`.
fn set_source(set: &ess_compiler::ir::ResolvedPayloadField) -> Option<String> {
    match &set.value {
        ess_compiler::ir::ResolvedPayloadValue::InputField { field, .. } => Some(field.clone()),
        ess_compiler::ir::ResolvedPayloadValue::Literal { .. } => None,
    }
}

/// The word an author wrote. `Delivery` carries no `Display`, and inventing one here would put a
/// spelling in a projection that the model does not own.
fn delivery(delivery: ess_domain::binding::Delivery) -> &'static str {
    match delivery {
        ess_domain::binding::Delivery::AtLeastOnce => "at_least_once",
        ess_domain::binding::Delivery::AtMostOnce => "at_most_once",
    }
}

fn readme(ir: &EssIr, suite: &ConformanceSuite) -> String {
    format!(
        "<!--\n  generated from {system} {version}\n  do not edit: regenerate with `ess verify conform web`\n-->\n\
         # {system} {version} — scenarios\n\n\
         {count} scenario(s), compiled from the documents an author wrote. Serve this directory and \
         open `index.html`; a browser will not instantiate a module from a `file://` URL.\n\n\
         ```console\n$ python3 -m http.server\n$ open http://localhost:8000/index.html\n```\n\n\
         ## What it shows\n\n\
         **Flow.** Commands and every original query, expectation and control remain declarations, \
         in their original order. Replay progress is not assertion success. Binding cards are \
         unexecuted declarations and do not establish instances or observed events.\n\n\
         **State.** A matching creation capture can establish a scenario-local alias and the declared \
         initial lifecycle state. It cannot establish an actual generated identity. Fields remain \
         explicitly unknown: the projection omits assignment literals, types and conversions. Updates \
         and moves also omit the subject identity source. Unknown effects remove affected knowledge; \
         they never stand for null, false, zero or an empty string.\n\n\
         **Views.** Every view remains visible with its filter, declared parameters and reached query \
         arguments. Results and model ordering are unavailable; replay does not compute membership, \
         ordering or consistency. Typed literal values and unresolved references retain their tags.\n\n\
         **UI.** An optional `skin.js` consumes the same reactive state. Skins must handle absent \
         state/field knowledge and separate unknown explanations; unavailable facts are not values.\n\n\
         ## What it does not claim\n\n\
         Replay executes no implementation, fills no obligation and establishes no specification \
         coherence. It emits no execution report or qualifying conformance evidence. Step, Back, Reset, \
         Select and Play navigate the same declared prefix; they do not verify its expectations.\n",
        system = ir.system(),
        version = ir.version(),
        count = suite.scenarios.len(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The page must not name any particular system. It is emitted for every specification, and a
    /// system's own shape enters only through a hand-written `skin.js`.
    #[test]
    fn the_page_is_specification_neutral() {
        for asset in [INDEX, PLAYER] {
            // Device words too. A skin brings its own styles; the page carried `.phone`, `.bezel` and
            // `.tile` for a while, which is a leak this list did not catch because it only looked
            // for system names.
            for word in [
                "softphone",
                "acd",
                "billing",
                "invoice",
                "phone",
                "bezel",
                "dialpad",
                "keypad",
            ] {
                assert!(
                    !asset.to_lowercase().contains(word),
                    "an emitted asset names `{word}`"
                );
            }
        }
    }

    /// Every method the page calls on the player has to exist on it. This is the mistake the web
    /// bridge made in the other direction — emitting a call to a method its own system crate did not
    /// generate — and it costs one test to refuse.
    #[test]
    fn the_page_calls_nothing_the_player_does_not_return() {
        for name in [
            "play",
            "step",
            "back",
            "reset",
            "select",
            "rowState",
            "mark",
            "lifecycle",
        ] {
            assert!(INDEX.contains(name), "the page never calls `{name}`");
            assert!(PLAYER.contains(name), "the player never returns `{name}`");
        }
    }

    /// A `<` in a text-node mustache makes the browser's HTML parser open a tag before Vue compiles
    /// the in-DOM template, and the page dies with no message. It has happened once.
    #[test]
    fn no_comparison_sits_in_a_text_node_mustache() {
        for capture in INDEX.split("{{").skip(1) {
            let Some(expression) = capture.split("}}").next() else {
                continue;
            };
            assert!(
                !expression.contains('<'),
                "`{{{{{expression}}}}}` puts a `<` where the HTML parser will read a tag"
            );
        }
    }
}
