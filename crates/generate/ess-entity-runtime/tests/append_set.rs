//! An append to a stored list, `sets: {tags: {append: input.tag}}` (`ess/31`), at Entity Runtime
//! lowering.
//!
//! Entity-core stores the values a command carries and computes none from a stored value, so a
//! branch appending to its subject's list is refused by name (`ValueExpressionUnsupported`,
//! construct "`{append: …}` (ess/31)") rather than lowered as a value the caller supplies. The
//! same command storing the whole list from its input earns no such refusal.
use std::collections::BTreeMap;
use std::num::NonZeroU32;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_entity_runtime::{lower, LoweringCode, LoweringOptions};
use ess_service_contract::extract;
use ess_synth::SynthesisPlan;

fn model(source: &str) -> String {
    format!(
        "format: ess/31
system: demo
version: v1
domain: demo.jobs
types:
  - {{name: demo.jobs.JobId, kind: newtype, of: Uuid}}
entities:
  - name: demo.jobs.Job
    identity: {{name: job_id, type: demo.jobs.JobId}}
    fields:
      - {{name: tags, type: List<String>}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open], transitions: []}}
events:
  - name: demo.jobs.Changed
    fields: [{{name: job_id, type: demo.jobs.JobId}}]
commands:
  - name: demo.jobs.AddTag
    input: [{{name: job_id, type: demo.jobs.JobId}}, {{name: tag, type: String}}, {{name: tags, type: List<String>}}]
    outcomes:
      - name: tagged
        updates: demo.jobs.Job
        instance: job_id
        sets: {{tags: {source}}}
        emits: [demo.jobs.Changed]
        payload: {{demo.jobs.Changed: {{job_id: input.job_id}}}}
components:
  - component: jobs-service
    owns: {{domains: [demo.jobs]}}
    accepts: {{commands: [demo.jobs.AddTag]}}
    publishes: {{events: [demo.jobs.Changed]}}
    reached_by: network
"
    )
}

fn ir_of(text: &str) -> EssIr {
    let spec =
        Specification::assemble([(Source::new("jobs.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn lowered(ir: &EssIr) -> Vec<(LoweringCode, String, String)> {
    let plan = SynthesisPlan::of(ir);
    let service = extract(ir, &plan, &ComponentName::new("jobs-service").unwrap()).unwrap();
    let options = LoweringOptions {
        definition_versions: [(
            QualifiedName::new("demo.jobs.Job").unwrap(),
            NonZeroU32::new(1).unwrap(),
        )]
        .into_iter()
        .collect(),
        scales: BTreeMap::new(),
    };
    match lower(&service, &options) {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics
            .into_vec()
            .into_iter()
            .map(|d| (d.code, d.construct.to_owned(), d.message.clone()))
            .collect(),
    }
}

#[test]
fn an_append_is_refused_by_name() {
    let refused = lowered(&ir_of(&model("{append: input.tag}")));
    assert!(
        refused.iter().any(|(code, construct, _)| {
            *code == LoweringCode::ValueExpressionUnsupported
                && construct == "`{append: …}` (ess/31)"
        }),
        "{refused:#?}"
    );
}

#[test]
fn the_same_command_storing_its_input_earns_no_such_refusal() {
    let refused = lowered(&ir_of(&model("input.tags")));
    assert!(
        refused
            .iter()
            .all(|(code, _, _)| *code != LoweringCode::ValueExpressionUnsupported),
        "{refused:#?}"
    );
}
