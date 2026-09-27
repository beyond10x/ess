//! A model that uses `Json` (beyond10x/ess#138) is refused by every code target by name, at the
//! positions it is used, rather than emitted with a representation nobody chose.
//!
//! The emitted Rust workspace builds with zero third-party crates, so `serde_json::Value` is not a
//! representation it can take; a dependency-free one is a follow-up. Until then the refusal is the
//! one `Binary64` gets.

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Target};

const MODEL: &str = "format: ess/15
system: demo
version: v1
domain: demo.msgs
types:
  - {name: demo.msgs.Body, kind: newtype, of: Json}
events:
  - name: demo.msgs.Sent
    fields:
      - {name: body, type: demo.msgs.Body}
      - {name: headers, type: \"Map<String, Json>\"}
commands:
  - name: demo.msgs.Send
    input:
      - {name: body, type: demo.msgs.Body}
    outcomes:
      - name: sent
        emits: [demo.msgs.Sent]
        payload:
          demo.msgs.Sent: {body: input.body, headers: {generated: true}}
components:
  - component: msgs-service
    owns: {domains: [demo.msgs]}
    accepts: {commands: [demo.msgs.Send]}
    publishes: {events: [demo.msgs.Sent]}
    reached_by: network
";

#[test]
fn issue_138_every_code_target_refuses_json_by_name() {
    let spec = Specification::assemble([(
        Source::new("spec.yaml"),
        RawSpecFile::parse(MODEL).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("spec.yaml", MODEL);
    let ir = compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"));
    for target in [Target::Rust, Target::Go, Target::Web, Target::Clap] {
        let Err(failure) = synthesize_for(&ir, target) else {
            panic!("{target:?} emitted a workspace for a Json model");
        };
        let rendered = failure.to_string();
        assert!(rendered.contains("Json"), "{target:?}: {rendered}");
        assert!(
            rendered.contains("types.demo.msgs.Body.of"),
            "{target:?}: {rendered}"
        );
        assert!(
            rendered.contains("event.demo.msgs.Sent.fields.headers.value"),
            "{target:?}: {rendered}"
        );
    }
}
