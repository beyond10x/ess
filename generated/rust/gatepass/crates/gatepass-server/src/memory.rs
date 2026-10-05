// generated from gatepass v1
// model digest 7d021b6ebe1c4715096f165d6564389be0f46311f67d791ed748f627314d611c
// contract digest 2668f3034afb388a33d7add462e15a830b6010fbfe83101f1dd2526fa18d52ed
// do not edit: regenerate with `ess synthesize`

//! Ephemeral stores for the generated network entry points; no durability.

/// Ephemeral storage of `gatepass.visit.Visit`, shared by clones.
#[derive(Clone, Default)]
pub struct MemoryVisitStorage(std::rc::Rc<std::cell::RefCell<Vec<gatepass_types::visit::VisitSnapshot>>>);
impl gatepass_types::behaviour::VisitStorage for MemoryVisitStorage {
fn get(&self, identity: &gatepass_types::visit::VisitId) -> Option<gatepass_types::visit::VisitSnapshot> { let key = memory_key_7(identity); self.0.borrow().iter().find(|row| memory_key_7(&row.data.visit_id) == key).cloned() }
fn put(&mut self, snapshot: gatepass_types::visit::VisitSnapshot) { self.delete(&snapshot.data.visit_id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_7(&row.data.visit_id).cmp(&memory_key_7(&other.data.visit_id))); }
fn delete(&mut self, identity: &gatepass_types::visit::VisitId) { let key = memory_key_7(identity); self.0.borrow_mut().retain(|row| memory_key_7(&row.data.visit_id) != key); }
fn list(&self) -> Vec<gatepass_types::visit::VisitSnapshot> { self.0.borrow().clone() }
}
impl gatepass_types::behaviour::VisitStorage for MemoryPorts {
fn get(&self, identity: &gatepass_types::visit::VisitId) -> Option<gatepass_types::visit::VisitSnapshot> { gatepass_types::behaviour::VisitStorage::get(&self.visit_storage, identity) }
fn put(&mut self, snapshot: gatepass_types::visit::VisitSnapshot) { gatepass_types::behaviour::VisitStorage::put(&mut self.visit_storage, snapshot); }
fn delete(&mut self, identity: &gatepass_types::visit::VisitId) { gatepass_types::behaviour::VisitStorage::delete(&mut self.visit_storage, identity); }
fn list(&self) -> Vec<gatepass_types::visit::VisitSnapshot> { gatepass_types::behaviour::VisitStorage::list(&self.visit_storage) }
}

/// Ports shared by generated components. Values disappear when the process exits.
#[derive(Clone, Default)]
pub struct MemoryPorts {
/// Storage of `gatepass.visit.Visit`.
pub visit_storage: MemoryVisitStorage,
}

fn memory_key_7(value: &gatepass_types::visit::VisitId) -> MemoryKey { let _ = value; MemoryKey::Text((&value.0).0.clone()) }

// A common structural order; individual specifications use only a subset of the variants.
#[allow(dead_code)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum MemoryKey {
    Null,
    Boolean(bool),
    Integer(i64),
    Number(String),
    Text(String),
    Bytes(Vec<u8>),
    List(Vec<MemoryKey>),
    Map(Vec<(MemoryKey, MemoryKey)>),
    Optional(Option<Box<MemoryKey>>),
}

impl gatepass_types::visit::obligations::RegisterVisitBehavior for MemoryPorts {
fn register_visit(&mut self, input: gatepass_types::visit::RegisterVisit) -> Result<gatepass_types::visit::RegisterVisitOutcome, gatepass_types::obligation::UnmetObligation> { gatepass_types::visit::obligations::RegisterVisitBehavior::register_visit(&mut gatepass_types::visit::obligations::Unimplemented, input) }
}
