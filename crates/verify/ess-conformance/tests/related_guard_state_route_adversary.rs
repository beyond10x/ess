//! Adversary cases for story seeded-state-reaches-related-rows: a related row arranged through a
//! chain of moves, each guarded over the row's stored fields.
//!
//! `chain(n, k)` is a plan moved through `n` steps `S0 -> S1 -> … -> Sn`; step `m` is refused
//! while any of `k` stored integers `f_m … f_{m+k-1}` sits below a literal, and `Start` creates a
//! job only on a plan in `Sn`. Every field is set from the creating command's input, so every row
//! the search needs is one choice of that input away.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::Synthesis;
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use std::fmt::Write as _;

fn chain(n: usize, k: usize) -> String {
    let fields = n + k - 1;
    let mut out = String::from(
        "format: ess/20\nsystem: demo\nversion: v1\ndomain: demo.chain\n\
         summary: A job starts only on a plan moved through guarded steps.\ntypes:\n  \
         - {name: demo.chain.PlanId, kind: newtype, of: Uuid}\n  \
         - {name: demo.chain.JobId, kind: newtype, of: Uuid}\nentities:\n  \
         - name: demo.chain.Plan\n    identity: {name: plan_id, type: demo.chain.PlanId}\n    fields:\n",
    );
    for f in 1..=fields {
        writeln!(out, "      - {{name: f{f}, type: Integer}}").unwrap();
    }
    let states: Vec<String> = (0..=n).map(|s| format!("S{s}")).collect();
    writeln!(
        out,
        "    lifecycle:\n      initial: S0\n      states: [{}]\n      terminal: [S{n}]\n      transitions:",
        states.join(", ")
    )
    .unwrap();
    for m in 1..=n {
        writeln!(
            out,
            "        - {{name: m{m}, from: [S{}], to: S{m}}}",
            m - 1
        )
        .unwrap();
    }
    out.push_str(
        "  - name: demo.chain.Job\n    identity: {name: job_id, type: demo.chain.JobId}\n    fields:\n      \
         - {name: plan_id, type: demo.chain.PlanId}\n    \
         lifecycle: {initial: Started, states: [Started], terminal: [Started]}\nerrors:\n  \
         - {name: demo.chain.NoPlan, summary: No plan., fields: []}\n  \
         - {name: demo.chain.Bad, summary: A guard refuses., fields: []}\n  \
         - {name: demo.chain.WrongState, summary: Wrong state., fields: []}\n  \
         - {name: demo.chain.NotDone, summary: Not done., fields: []}\nevents:\n  \
         - name: demo.chain.Created\n    fields:\n      - {name: plan_id, type: demo.chain.PlanId}\n  \
         - name: demo.chain.Moved\n    fields:\n      - {name: plan_id, type: demo.chain.PlanId}\n  \
         - name: demo.chain.JobStarted\n    fields:\n      - {name: job_id, type: demo.chain.JobId}\n\
         commands:\n  - name: demo.chain.Create\n    input:\n      \
         - {name: plan_id, type: demo.chain.PlanId}\n",
    );
    for f in 1..=fields {
        writeln!(out, "      - {{name: f{f}, type: Integer}}").unwrap();
    }
    let sets: Vec<String> = (1..=fields).map(|f| format!("f{f}: input.f{f}")).collect();
    writeln!(
        out,
        "    outcomes:\n      - name: created\n        creates: demo.chain.Plan\n        \
         instance: plan_id\n        sets: {{{}}}\n        emits: [demo.chain.Created]\n        \
         payload:\n          demo.chain.Created: {{plan_id: input.plan_id}}",
        sets.join(", ")
    )
    .unwrap();
    for m in 1..=n {
        writeln!(
            out,
            "  - name: demo.chain.M{m}\n    input:\n      - {{name: plan_id, type: demo.chain.PlanId}}\n    \
             outcomes:\n      - name: no-plan\n        unknown_instance: true\n        \
             error: demo.chain.NoPlan\n        payload: {{demo.chain.NoPlan: {{}}}}\n      \
             - name: bad\n        when_subject:\n          predicate:\n            any:"
        )
        .unwrap();
        for j in 1..=k {
            writeln!(out, "              - f{} < {}", m + j - 1, 10 * m + j).unwrap();
        }
        writeln!(
            out,
            "        error: demo.chain.Bad\n        payload: {{demo.chain.Bad: {{}}}}\n      \
             - name: wrong\n        wrong_state: true\n        error: demo.chain.WrongState\n        \
             payload: {{demo.chain.WrongState: {{}}}}\n      - name: moved\n        \
             moves: demo.chain.Plan.m{m}\n        instance: plan_id\n        \
             emits: [demo.chain.Moved]\n        payload:\n          \
             demo.chain.Moved: {{plan_id: input.plan_id}}"
        )
        .unwrap();
    }
    writeln!(
        out,
        "  - name: demo.chain.Start\n    input:\n      - {{name: plan_id, type: demo.chain.PlanId}}\n    \
         outcomes:\n      - name: no-plan\n        when_related:\n          via: input.plan_id\n          \
         exists: false\n        error: demo.chain.NoPlan\n        payload: {{demo.chain.NoPlan: {{}}}}\n      \
         - name: not-done\n        when_related:\n          via: input.plan_id\n          \
         predicate: state != S{n}\n        error: demo.chain.NotDone\n        \
         payload: {{demo.chain.NotDone: {{}}}}\n      - name: started\n        \
         creates: demo.chain.Job\n        instance: job_id\n        sets:\n          \
         plan_id: input.plan_id\n        emits: [demo.chain.JobStarted]\n        payload:\n          \
         demo.chain.JobStarted: {{job_id: {{generated: true}}}}"
    )
    .unwrap();
    out
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("chain.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(n: usize, k: usize) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(&chain(n, k)))
}

/// The scenario ids the synthesis refused, each with its cause.
fn refused(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .map(|refusal| {
            format!(
                "{}: {}",
                refusal
                    .scenario
                    .as_ref()
                    .map_or_else(String::new, ToString::to_string),
                refusal.cause
            )
        })
        .collect()
}

fn assert_synthesized(synthesis: &Synthesis, id: &str) {
    assert!(
        synthesis
            .suite
            .scenarios
            .keys()
            .any(|key| key.to_string() == id),
        "`{id}` is synthesized; refused: {:#?}",
        refused(synthesis)
            .into_iter()
            .filter(|line| line.starts_with(id))
            .collect::<Vec<_>>()
    );
}

/// Eight guarded moves, each guard over two stored integers. Before this change (base 14e27626c)
/// synthesis arranged the related plan through `Create, M1 … M8` and `Start/outcome/started`
/// passed the interpreted model. Keying the row search on how each row decides all eight move
/// guards splits rows the search used to merge, the 64-node budget runs out, and the scenario is
/// refused "subject fact arrangement exceeds 64 lifecycle/fact combinations".
#[test]
fn eight_guarded_moves_still_arrange_the_related_row() {
    assert_synthesized(&synthesis(8, 2), "demo.chain.Start/outcome/started");
}

/// The same with three stored integers per guard.
#[test]
fn eight_three_field_guards_still_arrange_the_related_row() {
    assert_synthesized(&synthesis(8, 3), "demo.chain.Start/outcome/started");
}
