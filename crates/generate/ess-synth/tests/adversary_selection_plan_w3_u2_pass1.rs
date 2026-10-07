//! Adversary pass 1 against wave 3 unit U2 of `epic:one-selection-plan`
//! (`story:generated-behaviour-reads-selection-plan`): the Rust and Go emitters writing their `if`
//! blocks in `PrecedencePlan` order.
//!
//! The story's acceptance rests byte identity on the slow per-model probes
//! (`adversary_e_u2_bytes`, `adversary_e_u6_bytes`): every repository YAML model whose first line
//! is `format: ess/<n>`, as written and relabelled `ess/22`, hashed base against unit. The unit
//! reports 309 of 309 lines identical. That comparison is evidence only for a reorder some probe
//! model can see.
//!
//! These cases take that probe's model set and reorder two phases with the wave-2 seam
//! (`with_phase_order`, the one override every "phases exchanged" test uses), which is exactly the
//! reorder a defect in either emitter or in the plan would make. A swap is a real reorder when a
//! model that validates and that the target generates changes its generated behaviour under it:
//! each case first shows that on an inline witness, then asks whether any probe model changes too.
//! Where none does, an identical probe listing says nothing about that reorder.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::command::precedence::{with_phase_order, Phase};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize_for, Target};

/// One witness model (`ess/22`): every command validates and the Rust and Go targets generate it.
///
/// * `RushOrder`: a related row the input names, an input refusal, a present-related refusal
///   ordered by `wrong_state:` (beyond10x/ess#282), an accepting `when:` and a default.
/// * `CompleteTask`: a stored reference (beyond10x/ess#304), an input refusal, its refusals, an
///   accepting `when:` and a default.
/// * `PlaceDirect`: `existing_instance:` on a command that only creates, an input refusal, an
///   accepting creation and a default.
/// * `NoteOrder`: an input refusal, a held-state refusal, an accepting `when:` and a default.
const WITNESS: &str = r#"format: ess/22
system: kept
version: v1
domain: kept.shop
types:
  - {name: kept.shop.ShopId, kind: newtype, of: Uuid}
  - {name: kept.shop.OrderId, kind: newtype, of: Uuid}
entities:
  - name: kept.shop.Shop
    identity: {name: shop_id, type: kept.shop.ShopId}
    fields:
      - {name: region, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
  - name: kept.shop.Order
    identity: {name: order_id, type: kept.shop.OrderId}
    fields:
      - {name: quantity, type: Integer}
      - {name: blocked_by, type: Optional<kept.shop.OrderId>}
    lifecycle:
      initial: Placed
      states: [Placed, Shipped]
      terminal: [Shipped]
      transitions:
        - {name: ship, from: [Placed], to: Shipped}
events:
  - name: kept.shop.ShopOpened
    fields: [{name: shop_id, type: kept.shop.ShopId}]
  - name: kept.shop.OrderPlaced
    fields: [{name: order_id, type: kept.shop.OrderId}]
  - name: kept.shop.OrderShipped
    fields: [{name: order_id, type: kept.shop.OrderId}]
errors:
  - name: kept.shop.NoShop
  - name: kept.shop.Refused
  - name: kept.shop.Conflict
  - name: kept.shop.Blocked
  - name: kept.shop.BlockerMissing
actors:
  - name: kept.shop.Clerk
    may: [kept.shop.OpenShop, kept.shop.RushOrder, kept.shop.CompleteTask, kept.shop.PlaceDirect, kept.shop.NoteOrder]
commands:
  - name: kept.shop.OpenShop
    input: [{name: region, type: String}]
    outcomes:
      - name: opened
        creates: kept.shop.Shop
        instance: shop_id
        sets: {region: input.region}
        emits: [kept.shop.ShopOpened]
        payload:
          kept.shop.ShopOpened: {shop_id: {generated: true}}
  - name: kept.shop.RushOrder
    input:
      - {name: order_id, type: kept.shop.OrderId}
      - {name: shop_id, type: kept.shop.ShopId}
      - {name: quantity, type: Integer}
    outcomes:
      - {name: invalid-quantity, when: 'quantity < 0', error: kept.shop.Refused}
      - name: no-shop
        when_related: {via: input.shop_id, exists: false}
        error: kept.shop.NoShop
      - name: wrong-region
        when_related: {via: input.shop_id, predicate: 'region != "EU"'}
        error: kept.shop.Refused
      - name: rushed
        when: 'quantity > 5'
        moves: kept.shop.Order.ship
        instance: order_id
        emits: [kept.shop.OrderShipped]
        payload:
          kept.shop.OrderShipped: {order_id: input.order_id}
      - name: shipped
        moves: kept.shop.Order.ship
        instance: order_id
        emits: [kept.shop.OrderShipped]
        payload:
          kept.shop.OrderShipped: {order_id: input.order_id}
      - {name: already-shipped, wrong_state: true, error: kept.shop.Conflict}
  - name: kept.shop.CompleteTask
    input:
      - {name: order_id, type: kept.shop.OrderId}
      - {name: quantity, type: Integer}
    outcomes:
      - {name: invalid-quantity, when: 'quantity < 0', error: kept.shop.Refused}
      - name: blocker-missing
        when_related: {via: blocked_by, exists: false}
        error: kept.shop.BlockerMissing
      - name: blocked
        when_related: {via: blocked_by, predicate: state != Shipped}
        error: kept.shop.Blocked
      - name: rushed
        when: 'quantity > 5'
        moves: kept.shop.Order.ship
        instance: order_id
        emits: [kept.shop.OrderShipped]
        payload:
          kept.shop.OrderShipped: {order_id: input.order_id}
      - name: shipped
        moves: kept.shop.Order.ship
        instance: order_id
        emits: [kept.shop.OrderShipped]
        payload:
          kept.shop.OrderShipped: {order_id: input.order_id}
      - {name: already-shipped, wrong_state: true, error: kept.shop.Conflict}
  - name: kept.shop.PlaceDirect
    input:
      - {name: order_id, type: kept.shop.OrderId}
      - {name: quantity, type: Integer}
    outcomes:
      - {name: invalid-quantity, when: 'quantity < 0', error: kept.shop.Refused}
      - name: bulk
        when: 'quantity > 5'
        creates: kept.shop.Order
        instance: order_id
        sets: {quantity: input.quantity}
        emits: [kept.shop.OrderPlaced]
        payload:
          kept.shop.OrderPlaced: {order_id: input.order_id}
      - name: placed
        creates: kept.shop.Order
        instance: order_id
        sets: {quantity: input.quantity}
        emits: [kept.shop.OrderPlaced]
        payload:
          kept.shop.OrderPlaced: {order_id: input.order_id}
      - {name: already-placed, existing_instance: true, error: kept.shop.Conflict}
  - name: kept.shop.NoteOrder
    input:
      - {name: order_id, type: kept.shop.OrderId}
      - {name: quantity, type: Integer}
    outcomes:
      - {name: invalid-quantity, when: 'quantity < 0', error: kept.shop.Refused}
      - name: shipped-already
        when_subject_state: Shipped
        error: kept.shop.Conflict
      - name: bulk
        when: 'quantity > 5'
        updates: kept.shop.Order
        instance: order_id
        sets: {quantity: input.quantity}
        emits: [kept.shop.OrderPlaced]
        payload:
          kept.shop.OrderPlaced: {order_id: input.order_id}
      - name: noted
        updates: kept.shop.Order
        instance: order_id
        sets: {quantity: input.quantity}
        emits: [kept.shop.OrderPlaced]
        payload:
          kept.shop.OrderPlaced: {order_id: input.order_id}
      - {name: no-such-order, unknown_instance: true, error: kept.shop.Conflict}
components:
  - component: shop
    reached_by: network
    owns: {domains: [kept.shop]}
    accepts: {commands: [kept.shop.OpenShop, kept.shop.RushOrder, kept.shop.CompleteTask, kept.shop.PlaceDirect, kept.shop.NoteOrder]}
    publishes: {events: [kept.shop.ShopOpened, kept.shop.OrderPlaced, kept.shop.OrderShipped]}
"#;

/// The witness commands, each of which the Rust and the Go targets must generate.
const WITNESS_COMMANDS: [&str; 4] = [
    "kept.shop.RushOrder",
    "kept.shop.CompleteTask",
    "kept.shop.PlaceDirect",
    "kept.shop.NoteOrder",
];

/// `Phase::PRECEDENCE` with `first` and `second` exchanged.
fn swapped(first: Phase, second: Phase) -> [Phase; 8] {
    let mut order = Phase::PRECEDENCE;
    let at = |phase| order.iter().position(|held| *held == phase).unwrap();
    let (i, j) = (at(first), at(second));
    order.swap(i, j);
    order
}

/// A YAML document, compiled; `None` where it does not parse, validate or compile.
fn compiled(text: &str) -> Option<EssIr> {
    let raw = RawSpecFile::parse(text).ok()?;
    let specification = Specification::assemble([(Source::new("model.yaml"), raw)]).ok()?;
    compile(&specification, &SourceMap::new()).ok()
}

/// Every generated behaviour artifact of `ir` in `target` (`behaviour.rs`, `behaviour.go`),
/// concatenated with its path; `None` where the target refuses the model or panics, or writes none.
fn behaviour(ir: &EssIr, target: Target) -> Option<String> {
    let suffix = match target {
        Target::Rust => "behaviour.rs",
        Target::Go => "behaviour.go",
        _ => unreachable!("Rust and Go only"),
    };
    let synthesis =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| synthesize_for(ir, target)))
            .ok()?
            .ok()?;
    let mut all = String::new();
    for (path, artifact) in &synthesis.artifacts {
        if path.ends_with(suffix) {
            all.push_str(path);
            all.push('\n');
            all.push_str(&artifact.contents);
        }
    }
    (!all.is_empty()).then_some(all)
}

fn walk(directory: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    let mut entries: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    entries.sort();
    for path in entries {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if matches!(name, "node_modules" | "target" | ".git" | ".engineering") {
            continue;
        }
        if path.is_dir() {
            walk(&path, found);
        } else if path
            .extension()
            .is_some_and(|extension| extension == "yaml")
        {
            found.push(path);
        }
    }
}

/// The byte probes' model set (`adversary_e_u2_bytes`, `adversary_e_u6_bytes`): every repository
/// YAML file whose first line is `format: ess/<n>`, as written and relabelled `ess/22`, kept where
/// it compiles and some target writes a generated behaviour for it.
fn probe_models() -> Vec<(String, EssIr)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the workspace root");
    let mut files = Vec::new();
    walk(&root, &mut files);
    let mut models = Vec::new();
    for path in &files {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        if !text.starts_with("format: ess/") {
            continue;
        }
        let name = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .display()
            .to_string();
        let mut variants = vec![(name.clone(), text.clone())];
        let header = text.lines().next().unwrap_or("");
        if header != "format: ess/22" {
            variants.push((
                format!("{name}@ess22"),
                text.replacen(header, "format: ess/22", 1),
            ));
        }
        for (name, text) in variants {
            if let Some(ir) = compiled(&text) {
                models.push((name, ir));
            }
        }
    }
    models
}

/// The generated behaviour of every model, per target, in the current phase order.
fn behaviours(models: &[(String, EssIr)]) -> Vec<[Option<String>; 2]> {
    models
        .iter()
        .map(|(_, ir)| [behaviour(ir, Target::Rust), behaviour(ir, Target::Go)])
        .collect()
}

/// The reorders the emitters write differently for some model that validates, each named.
const SWAPS: [(Phase, Phase); 6] = [
    (Phase::RelatedRow, Phase::InputRefusal),
    (Phase::InputRefusal, Phase::Existence),
    (Phase::InputRefusal, Phase::PresentRelated),
    (Phase::PresentRelated, Phase::Accepting),
    (Phase::Existence, Phase::Accepting),
    (Phase::HeldState, Phase::Accepting),
];

/// Every swap the witness shows to be a real reorder that no probe model's generated Rust or Go
/// behaviour sees.
#[test]
fn adv_w3u2_p1_the_byte_probes_see_the_pinned_set_of_reorders() {
    let witness = compiled(WITNESS).unwrap_or_else(|| {
        let raw = RawSpecFile::parse(WITNESS).expect("the witness parses");
        let errors = Specification::assemble([(Source::new("model.yaml"), raw)])
            .err()
            .map(|errors| errors.to_string())
            .unwrap_or_default();
        panic!("the witness validates and compiles:\n{errors}")
    });
    for target in [Target::Rust, Target::Go] {
        let text =
            behaviour(&witness, target).unwrap_or_else(|| panic!("{target:?} emits the witness"));
        for command in WITNESS_COMMANDS {
            assert!(
                text.contains(&format!(
                    "`{command}`, generated: every outcome is one the specification fully"
                )),
                "{target:?} generates `{command}` rather than owing it:\n{text}"
            );
        }
    }
    let witness_default = [
        behaviour(&witness, Target::Rust),
        behaviour(&witness, Target::Go),
    ];

    let models = probe_models();
    assert!(models.len() > 50, "{} probe models compile", models.len());
    let default = behaviours(&models);
    let generating = default
        .iter()
        .filter(|both| both.iter().any(Option::is_some))
        .count();

    let results: Vec<((Phase, Phase), bool, Vec<String>)> = std::thread::scope(|scope| {
        let handles: Vec<_> = SWAPS
            .iter()
            .map(|&(first, second)| {
                let models = &models;
                let default = &default;
                let witness = &witness;
                let witness_default = &witness_default;
                scope.spawn(move || {
                    with_phase_order(swapped(first, second), || {
                        let witnessed = [
                            behaviour(witness, Target::Rust),
                            behaviour(witness, Target::Go),
                        ] != *witness_default;
                        let seen: Vec<String> = behaviours(models)
                            .iter()
                            .zip(default)
                            .zip(models)
                            .filter(|((swapped, default), _)| swapped != default)
                            .map(|(_, (name, _))| name.clone())
                            .collect();
                        ((first, second), witnessed, seen)
                    })
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    let mut report = String::new();
    let mut blind = Vec::new();
    for ((first, second), witnessed, seen) in &results {
        assert!(
            witnessed,
            "the witness changes under {first} <-> {second}: the swap is a real reorder"
        );
        writeln!(
            report,
            "{first} <-> {second}: {} of {} probe models change ({} generate a behaviour){}",
            seen.len(),
            models.len(),
            generating,
            seen.first()
                .map(|name| format!(", e.g. {name}"))
                .unwrap_or_default()
        )
        .expect("writing to a String does not fail");
        if seen.is_empty() {
            blind.push(format!("{first} <-> {second}"));
        }
    }
    eprintln!("{report}");
    // Today's coverage, pinned (coordinator decision, story:generated-behaviour-reads-selection-plan):
    // the probes see no reorder of these two pairs, so for them the byte identity of this story
    // rests on the adversary's base-against-unit comparison of 9,720 Rust and 9,600 Go generated
    // models, recorded as the story's verification evidence. When a repository model the probes
    // read covers a pair, it leaves this list: shrink the expected list, never widen it.
    assert_eq!(
        blind,
        [
            "input_refusal <-> existence",
            "present_related <-> accepting"
        ],
        "the byte probes (adversary_e_u2_bytes, adversary_e_u6_bytes) see a different set of \
         reorders than pinned; a pair that left the list is now covered by a probe model, a pair \
         that joined it has lost its coverage.\n{report}"
    );
}

// ---- `precheck` reads the phases after step 5 ------------------------------------------------

/// The `precheck` block of the witness's stored-reference command `CompleteTask` in `target`: the
/// addressed row's existence and held state for the branch selected without the present-related
/// refusals, from its comment to the stored row's read.
fn precheck_block(target: Target) -> String {
    let witness = compiled(WITNESS).expect("the witness compiles");
    let text = behaviour(&witness, target).expect("the target emits the witness");
    let method = &text[text
        .find("`kept.shop.CompleteTask`, generated: every outcome")
        .expect("CompleteTask is generated")..];
    let start = method
        .find("// The addressed row's existence and held state, for the branch the request selects")
        .expect("CompleteTask writes the precheck");
    let end = method[start..]
        .find("// `when_related:` reads")
        .expect("the stored row is read after the precheck");
    method[start..start + end].to_owned()
}

/// The unit's decision 2 (`story:generated-behaviour-reads-selection-plan` Scope): `precheck`
/// reads the same phases as `body`, the phases after step 5 in the plan's order. With the
/// accepting phase and the default exchanged, the precheck looks at the default before the
/// accepting branch; a precheck that read the declared branches in declaration order, the default
/// last, writes the same block in both orders.
fn precheck_follows_the_phase_order(target: Target) {
    let declared = precheck_block(target);
    let exchanged = with_phase_order(swapped(Phase::Accepting, Phase::Default), || {
        precheck_block(target)
    });
    assert_ne!(
        declared, exchanged,
        "{target:?}: the precheck reads the phases after step 5 in the plan's order, so \
         exchanging the accepting phase and the default reorders its arms"
    );
}

#[test]
fn adv_w3u2_p1_precheck_follows_the_phase_order_rust() {
    precheck_follows_the_phase_order(Target::Rust);
}

#[test]
fn adv_w3u2_p1_precheck_follows_the_phase_order_go() {
    precheck_follows_the_phase_order(Target::Go);
}
