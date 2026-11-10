//! beyond10x/ess#765: a payload source the arrangement evaluates to an absent value is claimed
//! through its leaf (no present value admitted), never as an exact `null`.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{
    scenario::{Holds, LeafShape},
    ConformanceSuite as Suite, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

fn model(agent_presence: &str, creator_agent: &str) -> String {
    format!(
        "format: ess/25
system: example
version: v1
domain: example.token
types:
  - {{name: example.token.TokenId, kind: newtype, of: String}}
  - {{name: example.token.AgentName, kind: newtype, of: String}}
entities:
  - name: example.token.PoolToken
    identity: {{name: token_id, type: example.token.TokenId}}
    fields:
      - {{name: owner, type: String}}
      - {{name: agent, type: 'Optional<example.token.AgentName>'}}
    lifecycle:
      initial: Contributed
      states: [Contributed, Withdrawn]
      terminal: [Withdrawn]
      transitions:
        - {{name: withdraw, from: [Contributed], to: Withdrawn}}
events:
  - name: example.token.TokenContributed
    fields:
      - {{name: token_id, type: example.token.TokenId}}
  - name: example.token.TokenWithdrawn
    fields:
      - {{name: token_id, type: example.token.TokenId}}
      - {{name: owner, type: String}}
      - {{name: agent, type: 'Optional<example.token.AgentName>'{agent_presence}}}
commands:
  - name: example.token.ContributeToken
    input:
      - {{name: owner, type: String}}
    outcomes:
      - name: contributed
        creates: example.token.PoolToken
        instance: token_id
        sets: {{owner: input.owner, agent: {creator_agent}}}
        emits: [example.token.TokenContributed]
        payload:
          example.token.TokenContributed:
            token_id: {{generated: true}}
  - name: example.token.WithdrawToken
    input:
      - {{name: token_id, type: example.token.TokenId}}
    outcomes:
      - name: withdrawn
        moves: example.token.PoolToken.withdraw
        instance: token_id
        sets: {{agent: {{cleared: true}}}}
        emits: [example.token.TokenWithdrawn]
        payload:
          example.token.TokenWithdrawn:
            token_id: input.token_id
            owner: {{subject: owner}}
            agent: {{subject: agent}}
"
    )
}

fn suite(body: &str) -> Suite {
    let raw = RawSpecFile::parse(body).unwrap_or_else(|error| panic!("{error}\n{body}"));
    let spec = Specification::assemble([(Source::new("token.yaml"), raw)])
        .unwrap_or_else(|error| panic!("{error}\n{body}"));
    let ir = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}\n{body}"));
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    synthesis.suite
}

const WITHDRAWN: &str = "example.token.WithdrawToken/outcome/withdrawn";

fn agent_claim(suite: &Suite) -> (Option<Node>, Option<LeafShape>) {
    let (_, scenario) = suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == WITHDRAWN)
        .unwrap_or_else(|| panic!("no scenario {WITHDRAWN}"));
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExpectEvent { payload, shape, .. } => {
                return (
                    payload.get("agent").cloned(),
                    shape.leaves().get("agent").cloned(),
                );
            }
            ScenarioStep::ExpectEventValues { payload, shape, .. } => {
                let value = payload.get("agent").map(|value| match value {
                    ScenarioValue::Literal { value } => value.clone(),
                    other => panic!("not a literal: {other:?}"),
                });
                return (value, shape.leaves().get("agent").cloned());
            }
            _ => {}
        }
    }
    panic!("no event step in {scenario:#?}");
}

fn claims_absent(presence: &str) -> LeafShape {
    let (value, leaf) = agent_claim(&suite(&model(presence, "{cleared: true}")));
    assert_eq!(value, None, "no exact value claim for an absent read");
    let leaf = leaf.expect("the agent leaf");
    assert!(leaf.optional);
    assert_eq!(
        leaf.holds,
        Holds::Enum {
            variants: Vec::new()
        }
    );
    leaf
}

#[test]
fn an_absent_subject_read_admits_no_present_value() {
    claims_absent("");
}

#[test]
fn an_absent_subject_read_keeps_the_declared_presence() {
    let leaf = claims_absent(", presence: omitted_when_absent");
    assert!(leaf.presence.is_some());
}

#[test]
fn a_present_subject_read_keeps_its_value_claim() {
    let (value, _) = agent_claim(&suite(&model("", "lessee-1")));
    assert_eq!(value, Some(Node::Text("lessee-1".to_owned())));
}
