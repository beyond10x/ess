//! The generated Go runtime holds a `list` or `map` payload leaf to being one, as the Rust runner's
//! `Holds::admits` does (beyond10x/ess#188, adversary correction 1).
//!
//! Synthesis writes such leaves for every event field declared `List<…>` or `Map<…>`; where the
//! value is generated, the leaf is the only check on it
//! (`synthesize.rs`, `Holds::List`, `Holds::Map`), at every suite major. Before this the Go runtime
//! checked nothing about them, so a target publishing a text where a list is declared passed there
//! and failed in Rust.

mod support_go;

use std::collections::BTreeMap;

use ess_compiler::refs::OutcomeRef;
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::target::*;
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str = "format: ess/15
system: demo
version: v1
domain: demo.tags
events:
  - name: demo.tags.Tagged
    fields:
      - {name: tags, type: 'List<String>'}
      - {name: counts, type: 'Map<String, Integer>'}
actors:
  - {name: demo.tags.Clerk, may: [demo.tags.Tag]}
commands:
  - name: demo.tags.Tag
    input:
      - {name: tags, type: 'List<String>'}
      - {name: counts, type: 'Map<String, Integer>'}
    outcomes:
      - name: tagged
        emits: [demo.tags.Tagged]
        payload:
          demo.tags.Tagged: {tags: {generated: true}, counts: {generated: true}}
";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("tags.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Correct,
    /// `tags` published as one text.
    ListAsText,
    /// `counts` published as a list.
    MapAsList,
}

struct Tags(Mode);

impl ConformanceTarget for Tags {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("container-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let mut payload = BTreeMap::from([
            (
                "tags".to_owned(),
                Node::Seq(vec![Node::Text("a".to_owned())]),
            ),
            ("counts".to_owned(), Node::Map(BTreeMap::new())),
        ]);
        match self.0 {
            Mode::Correct => {}
            Mode::ListAsText => {
                payload.insert("tags".to_owned(), Node::Text("a,b".to_owned()));
            }
            Mode::MapAsList => {
                payload.insert("counts".to_owned(), Node::Seq(Vec::new()));
            }
        }
        let mut event = ObservedEvent::new("demo.tags.Tagged".parse().unwrap());
        event.payload = payload;
        Ok(SemanticCommandResult::took(OutcomeRef::new(
            request.command.clone(),
            OutcomeName::new("tagged").unwrap(),
        ))
        .emitting(event))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported("views", "the model declares none"))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

#[test]
fn go_holds_list_and_map_leaves_to_their_container_as_the_reference_runner_does() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(MODEL));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let suite = synthesis.suite;
    for mode in [Mode::Correct, Mode::ListAsText, Mode::MapAsList] {
        let verdicts = support_go::assert_parity(
            &format!("containers-{mode:?}").to_lowercase(),
            &suite,
            Tags(mode),
        );
        assert_eq!(
            support_go::not_passed(&verdicts).is_empty(),
            mode == Mode::Correct,
            "{mode:?}: {verdicts:?}"
        );
    }
}
