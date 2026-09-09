//! Authenticated observation, the freshness budget, and the finite operation predicates.
//!
//! An observation is a bounded statement about the part that was actually read, at the interval it
//! was read in. It can establish present, absent, matching or differing state for that part and
//! nothing else: it cannot invent a historical execution attribution, it cannot cover an address it
//! did not read, and it says nothing about Pod behavior, readiness over time, controller-created
//! descendants, Secret contents or retained PVCs.
//!
//! Positive absence is the asymmetry worth naming. An object is absent when an authenticated API
//! read said so. A failed read, a forbidden read, a malformed response and a Helm error message
//! are each `ObservationUnavailable`, never absence.

use std::collections::BTreeMap;

use super::chart::RenderedObject;
use super::model::{
    canonical_digest, ownership_marker, Admitted, Digest, HelmIdentity, ObjectAddress, ObjectRead,
    Observation, PresentObject, PrincipalPin, Refusal, RefusalCode, ReleasePermit,
    ReleaseProjection, ReleaseSnapshot, TargetPin, Text, Uuid, FRESHNESS_BUDGET_MS,
};

/// What one authenticated read of one address returned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiRead {
    /// The API authoritatively answered that no object holds this address.
    Absent,
    /// The API returned this object.
    Present {
        /// The server-assigned UID.
        uid: String,
        /// The server-assigned resource version.
        resource_version: String,
        /// The complete live object, before the projection is taken.
        object: serde_json::Value,
    },
}

/// One authenticated namespace or principal identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    /// The name.
    pub name: String,
    /// The namespace, for a principal.
    pub namespace: Option<String>,
    /// The authenticated UID.
    pub uid: String,
}

/// The bounded authenticated reads this contract needs, and nothing else.
///
/// The credential and TLS boundary is `ess-kubernetes`; this trait is the shape the recovery
/// engine consumes. No implementation of it returns a credential, a Secret value or a raw
/// kubeconfig, and the engine never asks for one.
pub trait Api {
    /// The identity namespace this profile fixes to `kube-system`.
    fn identity_namespace(&self) -> Admitted<Identity>;

    /// The self-subject identity the executing credential resolves to.
    fn self_subject(&self) -> Admitted<Identity>;

    /// One release namespace's authenticated identity.
    fn namespace(&self, name: &str) -> Admitted<Identity>;

    /// One direct object address, read authoritatively.
    fn object(&self, namespace: &str, address: &ObjectAddress) -> Admitted<ApiRead>;
}

/// Authenticates the pinned target and principal before anything depends on them.
///
/// This is the check that makes a context string into a physical identity. A context name is an
/// alias; the identity namespace's authenticated UID is the cluster.
pub fn authenticate(api: &dyn Api, target: &TargetPin, principal: &PrincipalPin) -> Admitted<()> {
    let identity = api.identity_namespace()?;
    if identity.name != target.identity_namespace.name.as_str()
        || identity.uid != target.identity_namespace.uid.as_str()
    {
        return Err(Refusal::new(
            RefusalCode::TargetMismatch,
            "the authenticated identity namespace does not equal the pinned physical cluster",
        ));
    }
    let subject = api.self_subject()?;
    if subject.name != principal.name.as_str()
        || subject.namespace.as_deref() != Some(principal.namespace.as_str())
        || subject.uid != principal.uid.as_str()
    {
        return Err(Refusal::new(
            RefusalCode::PrincipalMismatch,
            "the authenticated self-subject does not equal the pinned principal",
        ));
    }
    Ok(())
}

/// Authenticates one release namespace's pinned UID.
pub fn authenticate_namespace(api: &dyn Api, permit: &ReleasePermit) -> Admitted<()> {
    let namespace = api.namespace(permit.namespace.name.as_str())?;
    if namespace.uid != permit.namespace.uid.as_str() {
        return Err(Refusal::new(
            RefusalCode::TargetMismatch,
            format!(
                "namespace {} does not carry the pinned UID",
                permit.namespace.name
            ),
        ));
    }
    Ok(())
}

/// The complete live projection: everything except server bookkeeping and `status`.
///
/// `uid`, `resourceVersion`, `generation`, timestamps and managed fields are excluded because they
/// change without the object changing; `status` is excluded because it is the controller's report
/// rather than the declared content. Server-defaulted spec fields are *included*, which is why the
/// caller has to supply the intended post-defaulting fingerprint — this function never learns a
/// newly acceptable one by blessing what it found.
pub fn live_projection(object: &serde_json::Value) -> serde_json::Value {
    let mut projected = object.clone();
    let Some(map) = projected.as_object_mut() else {
        return projected;
    };
    map.remove("status");
    if let Some(metadata) = map
        .get_mut("metadata")
        .and_then(serde_json::Value::as_object_mut)
    {
        for bookkeeping in [
            "uid",
            "resourceVersion",
            "generation",
            "creationTimestamp",
            "deletionTimestamp",
            "managedFields",
            "selfLink",
        ] {
            metadata.remove(bookkeeping);
        }
        for collection in ["labels", "annotations", "ownerReferences", "finalizers"] {
            if !metadata.contains_key(collection) {
                let empty = if collection == "ownerReferences" || collection == "finalizers" {
                    serde_json::Value::Array(Vec::new())
                } else {
                    serde_json::Value::Object(serde_json::Map::new())
                };
                metadata.insert(collection.to_owned(), empty);
            }
        }
    }
    projected
}

/// The digest of one object's complete live projection.
pub fn projection_digest(object: &serde_json::Value) -> Digest {
    canonical_digest(&live_projection(object))
}

/// Whether every authored field of `authored` appears with the same value in `live`.
///
/// Recursive and one-directional: the live object may carry fields the chart did not author, and
/// the complete-projection fingerprint is what covers those. What is refused here is an authored
/// image, selector, replica count or secret reference that the live object does not agree with.
pub fn authored_fields_match(authored: &serde_json::Value, live: &serde_json::Value) -> bool {
    match (authored, live) {
        (serde_json::Value::Object(authored), serde_json::Value::Object(live)) => {
            authored.iter().all(|(key, value)| {
                live.get(key)
                    .is_some_and(|found| authored_fields_match(value, found))
            })
        }
        (serde_json::Value::Array(authored), serde_json::Value::Array(live)) => {
            authored.len() == live.len()
                && authored
                    .iter()
                    .zip(live)
                    .all(|(left, right)| authored_fields_match(left, right))
        }
        (authored, live) => authored == live,
    }
}

/// Acquires one complete snapshot over the permit's admitted address union.
///
/// The union is the whole union — baseline and desired together — so a changed inventory never
/// silently omits an address on either side.
pub fn snapshot(
    api: &dyn Api,
    permit: &ReleasePermit,
    helm: Option<HelmIdentity>,
) -> Admitted<ReleaseSnapshot> {
    Ok(observe(api, permit, helm)?.0)
}

/// The snapshot, and the complete live objects it was taken over.
///
/// The snapshot keeps digests, because that is what is compared and what is persisted. The live
/// objects are kept beside it for exactly one reason: C07 step 8 requires comparing *every authored
/// rendered field* as well as the caller-approved projection digest, and a digest cannot do that.
/// A caller fingerprint that happened to approve an object with the wrong image would otherwise
/// silently override the chart.
pub fn observe(
    api: &dyn Api,
    permit: &ReleasePermit,
    helm: Option<HelmIdentity>,
) -> Admitted<(ReleaseSnapshot, BTreeMap<ObjectAddress, serde_json::Value>)> {
    let union = permit.union();
    let mut objects = Vec::with_capacity(union.len());
    let mut live = BTreeMap::new();
    for address in &union {
        let read = api.object(permit.namespace.name.as_str(), address)?;
        objects.push(match read {
            ApiRead::Absent => ObjectRead::Absent(address.clone()),
            ApiRead::Present {
                uid,
                resource_version,
                object,
            } => {
                let present = ObjectRead::Present(PresentObject {
                    object: address.clone(),
                    uid: Text::new(uid)?,
                    resource_version: Text::new(resource_version)?,
                    content_digest: projection_digest(&object),
                });
                live.insert(address.clone(), object);
                present
            }
        });
    }
    let snapshot = ReleaseSnapshot { helm, objects };
    snapshot.covers(&union)?;
    Ok((snapshot, live))
}

/// Requires an observation to be within the freshness budget at the final pre-launch check.
///
/// The budget is charged from `started_ms`, so it covers acquisition, `Observed` publication,
/// `Prepared` publication and the final checks together. All three values come from the same
/// invocation's monotonic clock; a persisted timestamp from another invocation is never fresh.
pub fn admit_freshness(observation: &Observation, launch_check_ms: u64) -> Admitted<()> {
    observation.validate()?;
    if launch_check_ms < observation.finished_ms.get() {
        return Err(Refusal::new(
            RefusalCode::ObservationStale,
            "the launch check precedes the observation it rests on",
        ));
    }
    let age = launch_check_ms.saturating_sub(observation.started_ms.get());
    if age > FRESHNESS_BUDGET_MS {
        return Err(Refusal::new(
            RefusalCode::ObservationStale,
            format!("the observation is {age} ms old and the budget is {FRESHNESS_BUDGET_MS} ms"),
        ));
    }
    Ok(())
}

/// Requires the observed Helm identity to be this authority's own active incarnation.
///
/// A foreign or unmarked existing release refuses. There is no automatic adoption, and no removal
/// flag or repair authority upgrades a foreign incarnation into one this invocation may act on.
pub fn admit_ownership(helm: &HelmIdentity, authority: &Uuid, incarnation: &Uuid) -> Admitted<()> {
    if helm.description.as_str() != ownership_marker(authority, incarnation) {
        return Err(Refusal::new(
            RefusalCode::OwnershipConflict,
            "the observed release carries a foreign or absent ownership marker",
        ));
    }
    if helm.hook_count.get() != 0 {
        return Err(Refusal::new(
            RefusalCode::UnsupportedProfile,
            "the observed release declares hooks, which this profile does not admit",
        ));
    }
    Ok(())
}

/// Requires the observed stored manifest to equal the projection being admitted.
pub fn admit_manifest(helm: &HelmIdentity, projection: &ReleaseProjection) -> Admitted<()> {
    let mut stored = helm.manifest.clone();
    stored.sort();
    stored.dedup();
    if stored != projection.addresses() {
        return Err(Refusal::new(
            RefusalCode::BaselineMismatch,
            "the observed stored manifest inventory does not equal the admitted projection",
        ));
    }
    Ok(())
}

/// Whether every object of `projection` is present with its caller-approved fingerprint.
pub fn projection_holds(snapshot: &ReleaseSnapshot, projection: &ReleaseProjection) -> bool {
    projection.objects.iter().all(|approved| {
        matches!(
            snapshot.read(&approved.object),
            Some(ObjectRead::Present(present))
                if present.content_digest == approved.content_digest
        )
    })
}

/// Whether every named address read authoritatively absent.
pub fn addresses_absent(snapshot: &ReleaseSnapshot, addresses: &[ObjectAddress]) -> bool {
    addresses
        .iter()
        .all(|address| matches!(snapshot.read(address), Some(ObjectRead::Absent(_))))
}

/// What one operation's pre-state observation established.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Predicate {
    /// The complete desired predicate holds: record the observation and skip the mutation.
    DesiredMatches,
    /// The admitted baseline pre-state holds and the change may be applied.
    ApplyFromBaseline,
    /// Nothing exists yet and `may_create` authorizes a first creation.
    FirstCreation,
    /// Helm release storage and every baseline direct object are authoritatively absent.
    AlreadyAbsent,
    /// The admitted baseline pre-state holds and the retirement may be removed.
    RemoveBaseline,
}

/// Decides the finite operation predicate for one permit from one complete observation.
///
/// The five predicates are C07's, and the ordering matters: the desired predicate is checked
/// first, so an operation whose work is already done records an observation rather than repeating a
/// mutation, and does not fabricate an earlier application fact to explain the state it found.
pub fn decide(
    permit: &ReleasePermit,
    snapshot: &ReleaseSnapshot,
    authority: &Uuid,
    retirement: bool,
) -> Admitted<Predicate> {
    if let Some(helm) = &snapshot.helm {
        admit_ownership(helm, authority, &permit.incarnation)?;
    }
    if retirement {
        return decide_retirement(permit, snapshot);
    }
    let desired = permit.desired.as_ref().ok_or_else(|| {
        Refusal::new(
            RefusalCode::InvalidInput,
            "an apply operation was selected for a permit with no desired projection",
        )
    })?;

    if let Some(helm) = &snapshot.helm {
        if admit_manifest(helm, desired).is_ok()
            && projection_holds(snapshot, desired)
            && addresses_absent(snapshot, &permit.baseline_only())
        {
            return Ok(Predicate::DesiredMatches);
        }
        let baseline = permit.baseline.as_ref().ok_or_else(|| {
            Refusal::new(
                RefusalCode::OwnershipConflict,
                "a release exists at this address and the authority admits no baseline for it",
            )
        })?;
        // The admitted baseline pre-state, *or* the exact independently reviewed one. C07's second
        // predicate is one sentence with an `unless` in it, and this is the `unless`: a differing
        // observed pre-state is refused unless a new caller authority revision supplies the exact
        // reviewed `repair_from` snapshot, in which case that snapshot *is* the admitted pre-state
        // and the manifest and projection checks are the caller's review rather than this
        // reader's.
        if !repair_admits(permit, snapshot) {
            admit_manifest(helm, baseline)?;
            if !projection_holds(snapshot, baseline) {
                return Err(drift());
            }
        }
        // What a repair never waives. Foreign ownership is decided above, before either branch;
        // a competing occupant at a newly desired address is decided here, after both.
        if !addresses_absent(snapshot, &permit.desired_only()) {
            return Err(Refusal::new(
                RefusalCode::OwnershipConflict,
                "a newly desired object address is already occupied",
            ));
        }
        return Ok(Predicate::ApplyFromBaseline);
    }

    if !permit.may_create {
        return Err(Refusal::new(
            RefusalCode::ObservedDrift,
            "no release exists at this address and the authority does not permit a creation",
        ));
    }
    if permit.baseline.is_some() {
        return Err(Refusal::new(
            RefusalCode::BaselineMismatch,
            "the authority admits a baseline and no release exists to have produced it",
        ));
    }
    if !addresses_absent(snapshot, &permit.union()) {
        return Err(Refusal::new(
            RefusalCode::OwnershipConflict,
            "a first creation requires complete absence and an address is occupied",
        ));
    }
    Ok(Predicate::FirstCreation)
}

fn decide_retirement(permit: &ReleasePermit, snapshot: &ReleaseSnapshot) -> Admitted<Predicate> {
    let baseline = permit.baseline.as_ref().ok_or_else(|| {
        Refusal::new(
            RefusalCode::InvalidInput,
            "a retirement was selected for a permit with no baseline projection",
        )
    })?;
    let addresses = baseline.addresses();
    if snapshot.helm.is_none() {
        if addresses_absent(snapshot, &addresses) {
            return Ok(Predicate::AlreadyAbsent);
        }
        return Err(Refusal::new(
            RefusalCode::DirectObjectsRemain,
            "Helm release storage is absent and a baseline direct object is still present",
        ));
    }
    let helm = snapshot.helm.as_ref().expect("checked just above");
    // C07's fourth predicate carries the same `unless`: the admitted baseline pre-state, or a
    // separately authorized exact `repair_from` pre-state, before one uninstall.
    if !repair_admits(permit, snapshot) {
        admit_manifest(helm, baseline)?;
        if !projection_holds(snapshot, baseline) {
            return Err(drift());
        }
    }
    Ok(Predicate::RemoveBaseline)
}

/// The refusal a differing pre-state produces when no reviewed snapshot authorizes it.
///
/// One arm, not two. It used to have a second arm that fired when the reviewed snapshot *did*
/// match — refusing the one case the contract admits — which is what made `repair_admits` a
/// function with no production caller.
fn drift() -> Refusal {
    Refusal::new(
        RefusalCode::ObservedDrift,
        "the observed pre-state differs from the admitted projection and no exact reviewed \
         repair_from snapshot authorizes it",
    )
}

/// Whether an exact reviewed repair snapshot authorizes acting on the observed pre-state.
///
/// Equality is exact and whole: the reviewed snapshot is the pre-state, address for address and
/// digest for digest. It cannot waive foreign ownership or a competing occupant, both of which are
/// decided before this is consulted.
pub fn repair_admits(permit: &ReleasePermit, snapshot: &ReleaseSnapshot) -> bool {
    permit
        .repair_from
        .as_ref()
        .is_some_and(|repair| repair == snapshot)
}

/// The complete absence predicate after an acknowledged retirement.
pub fn absence_holds(snapshot: &ReleaseSnapshot, baseline: &ReleaseProjection) -> Admitted<()> {
    if snapshot.helm.is_some() {
        return Err(Refusal::new(
            RefusalCode::DirectObjectsRemain,
            "Helm release storage is still present after an acknowledged removal",
        ));
    }
    let remaining: Vec<&ObjectAddress> = baseline
        .addresses()
        .iter()
        .filter(|address| !matches!(snapshot.read(address), Some(ObjectRead::Absent(_))))
        .map(|address| {
            baseline
                .objects
                .iter()
                .map(|object| &object.object)
                .find(|candidate| *candidate == address)
                .expect("the address comes from this inventory")
        })
        .collect();
    if !remaining.is_empty() {
        return Err(Refusal::new(
            RefusalCode::DirectObjectsRemain,
            format!(
                "{} baseline direct object(s) are still present; retained descendants and PVCs are \
                 outside this claim either way",
                remaining.len()
            ),
        ));
    }
    Ok(())
}

/// Requires the authored rendered fields of every object to hold in the observed live objects.
pub fn authored_holds(
    rendered: &[RenderedObject],
    live: &BTreeMap<ObjectAddress, serde_json::Value>,
) -> Admitted<()> {
    for object in rendered {
        let Some(found) = live.get(&object.address) else {
            return Err(Refusal::new(
                RefusalCode::ObservationUnavailable,
                "an authored object was not read",
            ));
        };
        if !authored_fields_match(&object.content, found) {
            return Err(Refusal::new(
                RefusalCode::ObservedDrift,
                "an authored rendered field does not hold in the observed object",
            ));
        }
    }
    Ok(())
}
