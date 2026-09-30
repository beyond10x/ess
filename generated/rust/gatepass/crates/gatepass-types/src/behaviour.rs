// generated from gatepass v1
// model digest f8ccea748a49e127ca2e18f725481394cc0eab1787fafd77d16c52485bf2abba
// contract digest a6fdd92f3a88ac0abbe59789406f3001df466e87f222e4aad1a8348c17f91d7c
// do not edit: regenerate with `ess synthesize`

//! What the specification fully determines, generated: the behaviour of every command the plan
//! lists as generated, written against ports the implementor supplies.
//!
//! Storage is a port: one trait per entity, get, put and delete of a snapshot by identity. ess
//! generates the trait and never a store. `Context` is the other port: the caller's attributes,
//! every identity and value the model says the implementation assigns, and the answer to each
//! `external:` branch. [`Generated`] implements every generated `…Behavior` trait over those ports
//! and forwards every behaviour and query the plan still owes to them, so it is a complete bundle
//! for every component port. To replace one generated behaviour, write a bundle of your own that
//! implements that trait and delegates the rest to a `Generated`.
//!
//! An `Err` from a generated behaviour is the typed refusal naming the command: the model declares
//! no outcome for the request (a guard is undecidable over it, or no declared branch answers it),
//! or — as `entity invariant` — the declared outcome would leave an entity breaking an invariant.

use crate::obligation::UnmetObligation;

/// Where `gatepass.visit.Visit` is stored — a port the implementor provides.
///
/// Keyed by the identity `visit_id`. ess generates this trait and never an implementation of it.
pub trait VisitStorage {
    /// The instance with this identity, or `None` where none is stored.
    fn get(&self, identity: &crate::visit::VisitId) -> Option<crate::visit::VisitSnapshot>;

    /// Stores this instance under its identity, replacing what was held.
    fn put(&mut self, snapshot: crate::visit::VisitSnapshot);

    /// Removes the instance with this identity.
    fn delete(&mut self, identity: &crate::visit::VisitId);

    /// Every stored instance, in the order the store keeps them: the order a generated query
    /// answers an unordered view in.
    fn list(&self) -> Vec<crate::visit::VisitSnapshot>;
}

/// Every generated behaviour of this workspace, over the ports `P` supplies.
///
/// `P` implements the storage trait of each entity a generated behaviour reads or writes,
/// `Context` where one asks it anything, and every `…Behavior` and `…Query` trait the plan still
/// owes; `Generated<P>` forwards those to it.
pub struct Generated<P> {
    /// The storage and context ports, and every behaviour or query still owed.
    pub ports: P,
}

impl<P> Generated<P> {
    /// The generated behaviours, over `ports`.
    pub fn new(ports: P) -> Self {
        Self { ports }
    }
}

/// `gatepass.visit.AdmitVisitor`, generated: every outcome is one the specification fully determines.
impl<P> crate::visit::obligations::AdmitVisitorBehavior for Generated<P>
where
    P: VisitStorage,
{
    fn admit_visitor(&mut self, input: crate::visit::AdmitVisitor) -> Result<crate::visit::AdmitVisitorOutcome, UnmetObligation> {
        let _ = &input;
        // `admitted`: the default.
        let Some(held) = VisitStorage::get(&self.ports, &input.visit_id) else {
            return Ok(crate::visit::AdmitVisitorOutcome::WrongStateUnknownInstance);
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::visit::AnyVisit::Expected(instance) => crate::visit::AnyVisit::OnSite(instance.arrive()),
            _ => return Ok(crate::visit::AdmitVisitorOutcome::WrongState { error: crate::visit::VisitStateConflict { state: held_state } }),
        };
        let next = moved.snapshot();
        if let Some(broken) = next.data.broken_invariant() {
            let capability = "entity invariant";
            return Err(UnmetObligation { capability, source: broken });
        }
        VisitStorage::put(&mut self.ports, next);
        return Ok(crate::visit::AdmitVisitorOutcome::Admitted { visitor_admitted: crate::visit::VisitorAdmitted { visit_id: input.visit_id.clone(), badge: input.badge.clone() } });
    }
}

impl<P: crate::visit::obligations::RegisterVisitBehavior> crate::visit::obligations::RegisterVisitBehavior for Generated<P> {
    fn register_visit(&mut self, input: crate::visit::RegisterVisit) -> Result<crate::visit::RegisterVisitOutcome, UnmetObligation> {
        crate::visit::obligations::RegisterVisitBehavior::register_visit(&mut self.ports, input)
    }
}

/// `gatepass.visit.SignOutVisitor`, generated: every outcome is one the specification fully determines.
impl<P> crate::visit::obligations::SignOutVisitorBehavior for Generated<P>
where
    P: VisitStorage,
{
    fn sign_out_visitor(&mut self, input: crate::visit::SignOutVisitor) -> Result<crate::visit::SignOutVisitorOutcome, UnmetObligation> {
        let _ = &input;
        // `signed-out`: the default.
        let Some(held) = VisitStorage::get(&self.ports, &input.visit_id) else {
            return Ok(crate::visit::SignOutVisitorOutcome::WrongStateUnknownInstance);
        };
        let _ = &held;
        let held_state = held.state;
        let moved = match held.refine() {
            crate::visit::AnyVisit::OnSite(instance) => crate::visit::AnyVisit::Departed(instance.depart()),
            _ => return Ok(crate::visit::SignOutVisitorOutcome::WrongState { error: crate::visit::VisitStateConflict { state: held_state } }),
        };
        let next = moved.snapshot();
        if let Some(broken) = next.data.broken_invariant() {
            let capability = "entity invariant";
            return Err(UnmetObligation { capability, source: broken });
        }
        VisitStorage::put(&mut self.ports, next);
        return Ok(crate::visit::SignOutVisitorOutcome::SignedOut { visitor_departed: crate::visit::VisitorDeparted { visit_id: input.visit_id.clone() } });
    }
}

/// `gatepass.visit.ExpectedVisits`, generated: every row is one the specification fully determines from the stored `gatepass.visit.Visit`s.
impl<P> crate::visit::obligations::ExpectedVisitsQuery for Generated<P>
where
    P: VisitStorage,
{
    fn expected_visits(&self) -> Result<Vec<crate::visit::ExpectedVisits>, UnmetObligation> {
        let mut admitted = VisitStorage::list(&self.ports);
        // `filter:` shows a row where it holds; false or unknown hides it.
        admitted.retain(|held| equal(Some(&held.state).map(|value| match value { crate::visit::VisitState::Departed => "Departed", crate::visit::VisitState::Expected => "Expected", crate::visit::VisitState::OnSite => "OnSite" }.to_owned()), Some("Expected".to_owned())) == Some(true));
        Ok(admitted
            .into_iter()
            .map(|held| crate::visit::ExpectedVisits {
                visit_id: held.data.visit_id,
                visitor: held.data.visitor,
                building: held.data.building,
                deposit: held.data.deposit,
            })
            .collect())
    }
}

/// `gatepass.visit.VisitById`, generated: every row is one the specification fully determines from the stored `gatepass.visit.Visit`s.
impl<P> crate::visit::obligations::VisitByIdQuery for Generated<P>
where
    P: VisitStorage,
{
    fn visit_by_id(&self) -> Result<Vec<crate::visit::VisitById>, UnmetObligation> {
        let admitted = VisitStorage::list(&self.ports);
        Ok(admitted
            .into_iter()
            .map(|held| crate::visit::VisitById {
                visit_id: held.data.visit_id,
                visitor: held.data.visitor,
                host: held.data.host,
                escorts: held.data.escorts,
                notes: held.data.notes,
                badge: held.data.badge,
            })
            .collect())
    }
}

/// Equality of two read values; an unread one is Unknown.
fn equal<T: PartialEq>(left: Option<T>, right: Option<T>) -> Option<bool> {
    Some(left? == right?)
}
