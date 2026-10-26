//! Two structs with `fields: []` (`ess/32`, beyond10x/ess#649), one ignoring undeclared fields
//! and one refusing them, selected from a compiled model as the realizers' roots.

use std::collections::BTreeSet;

use schema_contract::realize::Plan;

const SOURCE: &str = "\
format: ess/32
system: demo
version: v1
domains: [demo.ack]
domain: demo.ack
types:
  - name: demo.ack.Open
    kind: struct
    undeclared_fields: ignored
    fields: []
  - name: demo.ack.Closed
    kind: struct
    fields: []
";

/// The realization plan of `demo.ack.Open` and `demo.ack.Closed`.
pub fn plan() -> Plan {
    let specification = ess_domain::spec::Specification::assemble([(
        ess_domain::system::Source::new("ack.yaml"),
        ess_domain::spec::RawSpecFile::parse(SOURCE).unwrap(),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    let ir =
        ess_compiler::resolve::compile(&specification, &ess_compiler::source::SourceMap::new())
            .unwrap_or_else(|errors| panic!("{errors}"));
    let selected = ess_gen::schema::ModelTypes::select(
        &ir,
        &BTreeSet::from(["demo.ack.Open".to_owned(), "demo.ack.Closed".to_owned()]),
    )
    .unwrap();
    Plan::from_model(&selected).unwrap()
}
