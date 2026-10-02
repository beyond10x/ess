// generated from gatepass v1
// model digest 7d021b6ebe1c4715096f165d6564389be0f46311f67d791ed748f627314d611c
// contract digest 2668f3034afb388a33d7add462e15a830b6010fbfe83101f1dd2526fa18d52ed
// do not edit: regenerate with `ess synthesize`

//! Ephemeral storage for generated servers. Restarting loses every row.

/// All storage and the bounded demonstration context, shared by the system's components.
#[derive(Clone, Default)]
pub struct InMemoryPorts {
    /// The rows of `gatepass.visit.Visit`.
    pub visit_storage: std::rc::Rc<std::cell::RefCell<VisitStorage>> ,
}

/// In-memory rows of `gatepass.visit.Visit`, ordered by identity.
#[derive(Default)]
pub struct VisitStorage { rows: Vec<crate::visit::VisitSnapshot> }
impl crate::behaviour::VisitStorage for VisitStorage {
    fn get(&self, identity: &crate::visit::VisitId) -> Option<crate::visit::VisitSnapshot> { self.rows.iter().find(|row| row.data.visit_id == *identity).cloned() }
    fn put(&mut self, snapshot: crate::visit::VisitSnapshot) {
        if let Some(row) = self.rows.iter_mut().find(|row| row.data.visit_id == snapshot.data.visit_id) { *row = snapshot; } else { self.rows.push(snapshot); }
    }
    fn delete(&mut self, identity: &crate::visit::VisitId) { self.rows.retain(|row| row.data.visit_id != *identity); }
    fn list(&self) -> Vec<crate::visit::VisitSnapshot> { let mut rows = self.rows.clone(); rows.sort_by(|left, right| left.data.visit_id.0.cmp(&right.data.visit_id.0)); rows }
}
impl crate::behaviour::VisitStorage for InMemoryPorts {
    fn get(&self, identity: &crate::visit::VisitId) -> Option<crate::visit::VisitSnapshot> { crate::behaviour::VisitStorage::get(&*self.visit_storage.borrow(), identity) }
    fn put(&mut self, snapshot: crate::visit::VisitSnapshot) { crate::behaviour::VisitStorage::put(&mut *self.visit_storage.borrow_mut(), snapshot); }
    fn delete(&mut self, identity: &crate::visit::VisitId) { crate::behaviour::VisitStorage::delete(&mut *self.visit_storage.borrow_mut(), identity); }
    fn list(&self) -> Vec<crate::visit::VisitSnapshot> { crate::behaviour::VisitStorage::list(&*self.visit_storage.borrow()) }
}
