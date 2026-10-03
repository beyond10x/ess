//! The compiled recursive payload retains the full target ancestry of a nested increment.

use ess_compiler::{ir::ResolvedPayloadValue, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str = r"format: ess/20
system: demo
version: v1
domain: demo.counter
types:
  - name: demo.counter.Packet
    kind: struct
    fields: [{name: amount, type: Integer}]
entities:
  - name: demo.counter.Counter
    identity: {name: counter_id, type: Uuid}
    fields: [{name: packet, type: demo.counter.Packet}]
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
events:
  - {name: demo.counter.Advanced, fields: []}
commands:
  - name: demo.counter.Advance
    input: [{name: counter_id, type: Uuid}]
    outcomes:
      - name: advanced
        updates: demo.counter.Counter
        instance: counter_id
        sets: {packet: {amount: {increment: 1}}}
        emits: [demo.counter.Advanced]
";

#[test]
fn nested_increment_uses_its_declared_location() {
    let raw = RawSpecFile::parse(MODEL).expect("the model parses");
    let specification = Specification::assemble([(Source::new("counter.yaml"), raw)])
        .expect("the nested target is admitted without a same-named top-level field");
    let ir = compile(&specification, &SourceMap::new()).expect("the model compiles");
    let outcome = &ir.commands()[&"demo.counter.Advance".parse().unwrap()].outcomes[0];
    let packet = &outcome.sets[0];
    assert_eq!(packet.target, "packet");
    let ResolvedPayloadValue::Struct { fields } = &packet.value else {
        panic!("packet retains its struct node: {packet:?}");
    };
    assert_eq!(fields[0].target, "amount");
    assert_eq!(
        fields[0].value,
        ResolvedPayloadValue::Increment { by: "1".into() }
    );
}
