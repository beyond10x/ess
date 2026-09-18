//! Who owns an entity, as the one question an arrangement asks.
//!
//! `relations_carried_by` answers "what do my fields mean" for both relation kinds.
//! [`EssIr::owner_of`] asks something narrower — *this row cannot exist without which other row* —
//! and the narrowing is the whole content: a `references` is a link the far side outlives, so it is
//! not an answer to that question however much it looks like one from the carrying field.
//!
//! No example model in this repository declares a `references` relation, which is exactly why this
//! model is written here rather than measured over `examples/billing/`.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

/// A client that its account owns and that *names* a region, plus a region that stands alone.
///
/// `area` sorts before `parent_account`, deliberately: the carried relations arrive in field order,
/// so a lookup that took the first one rather than the first `owns` one would answer `Region` here
/// and this test would be the thing that says so.
const MODEL: &str = r"
format: ess/4
system: crm
version: v1
domain: crm.core
entities:
  - name: crm.core.Account
    identity: {name: account_id, type: Uuid}
    fields:
      - {name: label, type: String}
    relations:
      - {name: clients, kind: owns, target: crm.core.Client, cardinality: many, via: parent_account}
    lifecycle: {initial: Live, states: [Live], terminal: [Live]}
  - name: crm.core.Client
    identity: {name: client_id, type: Uuid}
    fields:
      - {name: parent_account, type: Uuid}
      - {name: area, type: Uuid}
    relations:
      - {name: region, kind: references, target: crm.core.Region, cardinality: one, via: area}
    lifecycle: {initial: Live, states: [Live], terminal: [Live]}
  - name: crm.core.Region
    identity: {name: region_id, type: Uuid}
    fields:
      - {name: label, type: String}
    lifecycle: {initial: Live, states: [Live], terminal: [Live]}
views:
  - name: crm.core.Accounts
    source: crm.core.Account
    consistency: read_your_writes
    fields:
      - {name: account_id, type: Uuid}
      - {name: state, type: crm.core.Account.State}
  - name: crm.core.Clients
    source: crm.core.Client
    consistency: read_your_writes
    fields:
      - {name: client_id, type: Uuid}
      - {name: state, type: crm.core.Client.State}
  - name: crm.core.Regions
    source: crm.core.Region
    consistency: read_your_writes
    fields:
      - {name: region_id, type: Uuid}
      - {name: state, type: crm.core.Region.State}
";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("the model parses");
    let spec = Specification::assemble([(Source::new("crm.yaml"), raw)]).expect("it assembles");
    compile(&spec, &SourceMap::new()).expect("it compiles")
}

/// The handle for one entity, taken from the IR rather than minted — a projection cannot mint one.
///
/// Through the views, which is why the model declares one per entity: `projections()` is keyed by
/// the entity each view reads, and it is the only place a test can reach a handle for an entity it
/// did not arrive at through a relation.
fn handle<'a>(ir: &'a EssIr, name: &str) -> &'a ess_compiler::ir::EntityHandle {
    let name: QualifiedName = name.parse().expect("a qualified name");
    ir.projections()
        .into_keys()
        .find(|handle| *handle.name() == name)
        .expect("the entity is projected by a view of its own")
}

#[test]
fn an_owned_entity_answers_its_owner_and_the_field_that_carries_it() {
    let ir = ir();
    let owned = ir.owner_of(handle(&ir, "crm.core.Client")).expect("owned");
    assert_eq!(owned.owner.name().to_string(), "crm.core.Account");
    assert_eq!(owned.via, "parent_account");
    assert_eq!(owned.relation.name.as_str(), "clients");
}

#[test]
fn a_reference_is_not_an_owner_however_the_field_is_carried() {
    // `Region` is named by `Client.area` and owns nothing. A client can be deleted and the region
    // is still there; that is what `references` says, and it is why the far side of one is not
    // something an arrangement has to establish first.
    let ir = ir();
    assert!(ir.owner_of(handle(&ir, "crm.core.Region")).is_none());
}

#[test]
fn a_root_answers_none_and_that_is_an_answer() {
    // Refusing an unowned entity would make every aggregate root an error (entity-relations design
    // §3), so the absence here is a fact and not a gap in the model.
    let ir = ir();
    assert!(ir.owner_of(handle(&ir, "crm.core.Account")).is_none());
}
