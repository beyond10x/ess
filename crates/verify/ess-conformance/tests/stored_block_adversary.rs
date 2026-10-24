//! Adversary: the stored-reference block moved to `OVERLAP + 4` (25), which is the block
//! `related_list.rs` creates the rows a list of references names in (`ROWS = 27_720 * 25`). The
//! site a stored reference is pointed at during an arranging run and the first site a list names
//! are then both `site-693001`.
//!
//! `fleet-sites.yaml` plus `AuditMachine`, which reads a list of sites through its input and
//! moves a running machine. Its subject is arranged through `StartMachine`, whose arranging run
//! points the machine's stored `site_id` at a site created in the stored block.

use std::collections::BTreeSet;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::scenario::ScenarioStep;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("fixtures/fleet-sites.yaml");

fn model() -> String {
    let text = MODEL
        .replace(
            "      states: [Idle, Running]\n      terminal: [Running]\n      transitions:\n        - {name: start, from: [Idle], to: Running}",
            "      states: [Idle, Running, Audited]\n      terminal: [Audited]\n      transitions:\n        - {name: start, from: [Idle], to: Running}\n        - {name: audit, from: [Running], to: Audited}",
        )
        .replace(
            "  - {name: demo.fleet.MachineStateConflict,",
            "  - {name: demo.fleet.UnknownSite, summary: A listed site does not exist., fields: [{name: site_id, type: demo.fleet.SiteId}]}\n  - {name: demo.fleet.MachineStateConflict,",
        )
        .replace(
            "actors:\n",
            "  - name: demo.fleet.MachineAudited\n    fields: [{name: machine_id, type: demo.fleet.MachineId}]\nactors:\n",
        )
        .replace(
            "      - demo.fleet.StartMachine\n",
            "      - demo.fleet.StartMachine\n      - demo.fleet.AuditMachine\n",
        )
        .replace(
            "views:\n",
            "  - name: demo.fleet.AuditMachine\n    input:\n      - {name: machine_id, type: demo.fleet.MachineId}\n      - {name: site_ids, type: \"List<demo.fleet.SiteId>\"}\n    outcomes:\n      - {name: wrong-state, wrong_state: true, error: demo.fleet.MachineStateConflict}\n      - name: unknown-site\n        when_related: {via: input.site_ids, exists: false}\n        error: demo.fleet.UnknownSite\n        payload: {demo.fleet.UnknownSite: {site_id: {missing: input.site_ids}}}\n      - name: audited\n        moves: demo.fleet.Machine.audit\n        instance: machine_id\n        emits: [demo.fleet.MachineAudited]\n        payload: {demo.fleet.MachineAudited: {machine_id: input.machine_id}}\nviews:\n",
        );
    assert!(
        text.contains("demo.fleet.AuditMachine\n    input"),
        "{text}"
    );
    assert!(text.contains("audit, from: [Running]"), "{text}");
    text
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("fleet-audit.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

#[test]
fn a_stored_row_and_a_listed_row_are_not_one_name() {
    let model = ir(&model());
    let result = ess_conformance::synthesize::synthesize(&model);
    let refused: Vec<String> = result
        .refusals
        .iter()
        .map(|refusal| {
            format!(
                "{:?} {}: {}",
                refusal.scenario.as_ref().map(ToString::to_string),
                refusal.cause.code(),
                refusal.cause
            )
        })
        .collect();
    let audited = "demo.fleet.AuditMachine/outcome/audited";
    assert!(
        result
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == audited),
        "{audited} is synthesized; refusals: {refused:#?}"
    );
    let mut twice = Vec::new();
    for (id, scenario) in &result.suite.scenarios {
        let mut captured = BTreeSet::new();
        for step in &scenario.steps {
            if let ScenarioStep::CaptureInstance { instance, .. } = step {
                if !captured.insert(instance.clone()) {
                    twice.push(format!("{id} captures `{instance}` twice"));
                }
            }
        }
    }
    assert!(twice.is_empty(), "captured twice: {twice:#?}");
}
