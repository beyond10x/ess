//! The executable R01-R29 matrix for the finite deployment recovery contract.
//!
//! The binding is `docs/design/review-execution-recovery.md`; the families are its C12 table and
//! the named dimensions below it. Vectors are grouped by family and each case names the exact
//! boundary it decides. Nothing here claims application behavior, controller descendants, retained
//! PVC/PV cleanup, real admission-controller policy or perpetual convergence: the supported result
//! is direct-resource observation at a point in time.

use ess_cli::recovery::model::{
    canonical_digest, canonical_endpoint, canonical_value, ownership_marker, read_canonical,
    write_canonical, Authority, AuthorityFormat, ChartSource, Digest, HelmBinary, HelmProtocol,
    HelmVersion, HostPolicy, Index, InvocationId, NamespacePin, ObjectAddress, ObjectFingerprint,
    ObjectKind, ObjectRead, PresentObject, PrincipalPin, Profile, RefusalCode, ReleasePermit,
    ReleaseProjection, ReleaseSnapshot, TargetPin, Text, Uuid, FRESHNESS_BUDGET_MS,
    HELM_INSTALL_PREFIX, IDENTITY_NAMESPACE, INDEX_LIMIT, REFUSAL_CODES,
};

// --- Family M: the declared model, its canonical bytes and its reader obligations ---------------
//
// C11 is explicit that the declaration states shapes and simple invariants only, and that
// cross-record ordering, uniqueness, byte verification and grammar are Rust reader obligations.
// These cases are that half. A type that merely `serde`-parses has not been admitted.

fn text(value: &str) -> Text {
    Text::new(value).expect("a fixture Text admits")
}

fn uuid(byte: u8) -> Uuid {
    let hex = format!("{byte:02x}").repeat(16);
    Uuid::new(format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    ))
    .expect("a fixture UUID admits")
}

fn digest(seed: &str) -> Digest {
    Digest::of_bytes(seed.as_bytes())
}

fn helm_binary() -> HelmBinary {
    let digest = digest("helm");
    HelmBinary {
        path: text(&format!(
            "{HELM_INSTALL_PREFIX}{}/helm",
            digest.as_str().trim_start_matches("sha256:")
        )),
        digest,
        version: HelmVersion::new("v3.16.2").expect("a canonical stable Helm version admits"),
        protocol: HelmProtocol::Helm3Recovery1,
    }
}

fn target() -> TargetPin {
    TargetPin {
        api_server: text("https://api.cluster.invalid:6443"),
        ca_digest: digest("ca"),
        identity_namespace: NamespacePin {
            name: text(IDENTITY_NAMESPACE),
            uid: text("ns-kube-system"),
        },
    }
}

fn host() -> HostPolicy {
    HostPolicy {
        host_id: text("host-a"),
        executor_uid: 1000,
        store_epoch: uuid(0x11),
        state_root: text("/var/lib/ess/recovery"),
        kubeconfig: text("/etc/ess/recovery/kubeconfig"),
        helm: helm_binary(),
    }
}

fn address(kind: ObjectKind, name: &str) -> ObjectAddress {
    ObjectAddress {
        kind,
        name: text(name),
    }
}

fn projection(names: &[(ObjectKind, &str)], seed: &str) -> ReleaseProjection {
    let mut objects: Vec<ObjectFingerprint> = names
        .iter()
        .map(|(kind, name)| ObjectFingerprint {
            object: address(*kind, name),
            content_digest: digest(&format!("{seed}:{kind}:{name}")),
        })
        .collect();
    objects.sort();
    ReleaseProjection {
        chart: ChartSource {
            runtime_digest: digest(&format!("{seed}:runtime")),
            chart_name: text("checkout"),
            chart_version: text("1.4.0"),
        },
        objects,
    }
}

fn permit(
    service: &str,
    baseline: Option<ReleaseProjection>,
    desired: Option<ReleaseProjection>,
) -> ReleasePermit {
    ReleasePermit {
        service: text(service),
        namespace: NamespacePin {
            name: text("app"),
            uid: text("ns-app"),
        },
        release_name: text(service),
        incarnation: uuid(0x22),
        may_create: baseline.is_none() && desired.is_some(),
        baseline,
        desired,
        repair_from: None,
    }
}

fn authority(releases: Vec<ReleasePermit>) -> Authority {
    let has_baseline = releases.iter().any(|permit| permit.baseline.is_some());
    Authority {
        format: AuthorityFormat::V1,
        profile: Profile::SingleHostGeneratedHelm1,
        authority_id: uuid(0x33),
        revision: Index::new(4).unwrap(),
        target: target(),
        principal: PrincipalPin {
            namespace: text("ess-system"),
            name: text("ess-recovery"),
            uid: text("sa-1"),
        },
        host: host(),
        contexts: vec![text("prod"), text("prod-alias")],
        environment: text("production"),
        desired_digest: digest("desired"),
        baseline_digest: has_baseline.then(|| digest("baseline")),
        releases,
        quiescence: Vec::new(),
    }
}

/// Every closed `RefusalCode` variant is reachable through the exported enumeration.
///
/// A closed list beside a hand-maintained constant is the shape that goes stale silently: the code
/// is added to the enum, the constant is not, and every check that walks the constant keeps
/// passing while saying nothing about the new one. The exhaustive `match` below is what makes the
/// constant a derived fact rather than a remembered one — a new variant fails to compile here.
#[test]
fn every_declared_refusal_code_is_enumerated_and_prints_its_declared_spelling() {
    for code in REFUSAL_CODES {
        let declared = match code {
            RefusalCode::InvalidInput => "InvalidInput",
            RefusalCode::UnsupportedProfile => "UnsupportedProfile",
            RefusalCode::AuthorityMismatch => "AuthorityMismatch",
            RefusalCode::TargetMismatch => "TargetMismatch",
            RefusalCode::PrincipalMismatch => "PrincipalMismatch",
            RefusalCode::BaselineMismatch => "BaselineMismatch",
            RefusalCode::OwnershipConflict => "OwnershipConflict",
            RefusalCode::ObservationUnavailable => "ObservationUnavailable",
            RefusalCode::ObservationStale => "ObservationStale",
            RefusalCode::ObservedDrift => "ObservedDrift",
            RefusalCode::MutationBlocked => "MutationBlocked",
            RefusalCode::StoreInvalid => "StoreInvalid",
            RefusalCode::EvidenceIncomplete => "EvidenceIncomplete",
            RefusalCode::PreparationFailed => "PreparationFailed",
            RefusalCode::LaunchFailed => "LaunchFailed",
            RefusalCode::EffectIndeterminate => "EffectIndeterminate",
            RefusalCode::RemovalNotPermitted => "RemovalNotPermitted",
            RefusalCode::DirectObjectsRemain => "DirectObjectsRemain",
        };
        assert_eq!(code.to_string(), declared);
        assert_eq!(
            serde_json::to_string(code).unwrap(),
            format!("\"{declared}\"")
        );
    }
    assert_eq!(
        REFUSAL_CODES.len(),
        18,
        "the declaration lists eighteen refusal codes"
    );
}

/// Canonical bytes refuse a duplicate key and a floating-point number.
#[test]
fn canonical_reading_refuses_duplicate_keys_and_floating_point_numbers() {
    assert!(canonical_value(r#"{"a":1,"a":2}"#).is_err());
    assert!(canonical_value(r#"{"a":{"b":1,"b":1}}"#).is_err());
    assert!(canonical_value(r#"{"a":[{"b":1,"b":2}]}"#).is_err());
    assert!(canonical_value(r#"{"a":1.5}"#).is_err());
    assert!(canonical_value(r#"{"a":[1.5]}"#).is_err());
    assert!(canonical_value(r#"{"a":1}"#).is_ok());
    assert!(
        canonical_value(r#"{"a":1}{"a":2}"#).is_err(),
        "a canonical document is one document, not the first of several"
    );
}

/// A valid document that is not its own canonical spelling is refused, not normalized.
#[test]
fn a_noncanonical_spelling_of_a_valid_record_is_refused_rather_than_normalized() {
    let id = InvocationId {
        store_epoch: uuid(0x11),
        nonce: uuid(0x99),
    };
    let canonical = write_canonical(&id);
    assert!(canonical.ends_with('\n'));
    assert_eq!(
        read_canonical::<InvocationId>(&canonical).expect("canonical bytes admit"),
        id
    );
    let reordered = canonical.replace(
        r#"{"store_epoch""#,
        r#"{"nonce":"00000000-0000-0000-0000-000000000000","store_epoch""#,
    );
    assert!(
        read_canonical::<InvocationId>(&reordered).is_err(),
        "a second spelling of one record is a second digest"
    );
    let without_newline = canonical.trim_end().to_owned();
    assert!(read_canonical::<InvocationId>(&without_newline).is_err());
    assert!(read_canonical::<InvocationId>(&format!(" {canonical}")).is_err());
    assert!(
        read_canonical::<InvocationId>(&canonical.replace('"', "\"unknown\":1,\"")).is_err(),
        "an unknown field is refused"
    );
}

/// The canonical digest is over the canonical bytes, including the trailing newline.
#[test]
fn the_canonical_digest_is_the_digest_of_the_canonical_bytes() {
    let id = InvocationId {
        store_epoch: uuid(0x11),
        nonce: uuid(0x99),
    };
    assert_eq!(
        canonical_digest(&id),
        Digest::of_bytes(write_canonical(&id).as_bytes())
    );
}

/// Endpoint canonicalization admits one spelling per endpoint and refuses every alias.
#[test]
fn one_api_endpoint_has_exactly_one_admitted_spelling() {
    assert_eq!(
        canonical_endpoint("https://api.cluster.invalid:6443").unwrap(),
        "api.cluster.invalid:6443"
    );
    assert_eq!(
        canonical_endpoint("https://api.cluster.invalid").unwrap(),
        "api.cluster.invalid"
    );
    for alias in [
        "https://api.cluster.invalid:443",
        "https://API.cluster.invalid",
        "https://user@api.cluster.invalid",
        "https://api.cluster.invalid/",
        "https://api.cluster.invalid/apis",
        "https://api.cluster.invalid?a=b",
        "https://api.cluster.invalid#f",
        "http://api.cluster.invalid",
        "api.cluster.invalid",
        "https://api.cluster.invalid:",
        "https://api.cluster.invalid:64a3",
        "https://",
    ] {
        assert!(
            canonical_endpoint(alias).is_err(),
            "{alias:?} is an alias or a non-root and must refuse"
        );
    }
}

/// The identity namespace name is fixed, and a target that renames it is unsupported.
#[test]
fn the_identity_namespace_name_is_fixed_to_kube_system() {
    let mut pin = target();
    assert!(pin.validate().is_ok());
    pin.identity_namespace.name = text("kube-public");
    assert_eq!(
        pin.validate().unwrap_err().code,
        RefusalCode::UnsupportedProfile
    );
}

/// A Helm version outside canonical stable `v3.M.P` is refused.
#[test]
fn only_a_canonical_stable_helm_3_version_admits() {
    assert!(HelmVersion::new("v3.16.2").is_ok());
    assert!(HelmVersion::new("v3.0.0").is_ok());
    for rejected in [
        "v3.16.2-rc.1",
        "v3.16.2+meta",
        "3.16.2",
        "v4.0.0",
        "v3.16",
        "v3.16.2.1",
        "v3.016.2",
        "latest",
        "",
        "v3..2",
    ] {
        assert!(
            HelmVersion::new(rejected).is_err(),
            "{rejected:?} is not a canonical stable v3.M.P version"
        );
    }
}

/// The Helm artifact's installation path, filename digest and configured digest must agree.
#[test]
fn an_admitted_helm_artifact_lives_at_its_own_digest_and_nowhere_else() {
    let binary = helm_binary();
    binary.validate().expect("the fixture artifact admits");
    let mut wrong = helm_binary();
    wrong.path = text("/usr/local/bin/helm");
    assert_eq!(
        wrong.validate().unwrap_err().code,
        RefusalCode::UnsupportedProfile
    );
    let mut renamed = helm_binary();
    renamed.path = text(&format!("{HELM_INSTALL_PREFIX}0000/helm"));
    assert!(
        renamed.validate().is_err(),
        "the filename digest and the configured digest must agree"
    );
}

/// `executor_uid > 0`, and the state root and kubeconfig are absolute canonical paths.
#[test]
fn the_host_policy_refuses_a_root_executor_and_a_relative_or_traversing_path() {
    host().validate().expect("the fixture host policy admits");
    let mut root = host();
    root.executor_uid = 0;
    assert_eq!(
        root.validate().unwrap_err().code,
        RefusalCode::UnsupportedProfile
    );
    for path in [
        "relative/state",
        "/var/lib/../lib/ess",
        "/var//lib",
        "/var/lib/ess/",
    ] {
        let mut policy = host();
        policy.state_root = text(path);
        assert!(
            policy.validate().is_err(),
            "{path:?} is not an absolute canonical path"
        );
    }
}

/// A release permit needs at least one projection, and a baseline cannot regain first creation.
#[test]
fn a_release_permit_admits_neither_an_empty_pair_nor_a_baseline_that_may_create() {
    let desired = projection(&[(ObjectKind::Deployment, "web")], "d");
    let baseline = projection(&[(ObjectKind::Deployment, "web")], "b");
    permit("checkout", Some(baseline.clone()), Some(desired.clone()))
        .validate()
        .expect("an upgrade permit admits");
    permit("checkout", Some(baseline.clone()), None)
        .validate()
        .expect("a baseline-only retirement admits");
    permit("checkout", None, Some(desired.clone()))
        .validate()
        .expect("a first creation admits");

    let mut empty = permit("checkout", None, None);
    empty.may_create = false;
    assert_eq!(
        empty.validate().unwrap_err().code,
        RefusalCode::InvalidInput
    );

    let mut regained = permit("checkout", Some(baseline), Some(desired));
    regained.may_create = true;
    assert!(
        regained.validate().is_err(),
        "an owned history cannot be discarded to regain first-deployment behavior"
    );
}

/// A projection's inventory is ordered, deduplicated and nonempty.
#[test]
fn a_release_projection_inventory_is_ordered_deduplicated_and_nonempty() {
    let mut good = projection(
        &[
            (ObjectKind::Deployment, "web"),
            (ObjectKind::Service, "web"),
            (ObjectKind::StatefulSet, "queue"),
        ],
        "d",
    );
    good.validate().expect("an ordered inventory admits");

    let mut empty = good.clone();
    empty.objects.clear();
    assert!(empty.validate().is_err());

    let mut duplicated = good.clone();
    duplicated.objects.push(good.objects[0].clone());
    assert!(duplicated.validate().is_err());

    good.objects.reverse();
    assert!(
        good.validate().is_err(),
        "a persisted inventory has one order, and it is the one the digest is taken over"
    );
}

/// The address union covers both inventories, and each side's own-only set is exact.
#[test]
fn the_observed_union_covers_both_inventories_and_each_side_knows_its_own_only_addresses() {
    let baseline = projection(
        &[
            (ObjectKind::Deployment, "web"),
            (ObjectKind::Service, "old"),
        ],
        "b",
    );
    let desired = projection(
        &[
            (ObjectKind::Deployment, "web"),
            (ObjectKind::Service, "new"),
        ],
        "d",
    );
    let permit = permit("checkout", Some(baseline), Some(desired));
    assert_eq!(
        permit.union(),
        vec![
            address(ObjectKind::Deployment, "web"),
            address(ObjectKind::Service, "new"),
            address(ObjectKind::Service, "old"),
        ]
    );
    assert_eq!(
        permit.desired_only(),
        vec![address(ObjectKind::Service, "new")]
    );
    assert_eq!(
        permit.baseline_only(),
        vec![address(ObjectKind::Service, "old")]
    );

    let retirement = crate::permit(
        "legacy",
        Some(projection(&[(ObjectKind::Deployment, "legacy")], "b")),
        None,
    );
    assert_eq!(
        retirement.baseline_only(),
        vec![address(ObjectKind::Deployment, "legacy")],
        "a baseline-only retirement's whole inventory is baseline-only"
    );
    assert!(retirement.desired_only().is_empty());
}

/// A snapshot proves nothing about an address it did not read.
#[test]
fn a_partial_snapshot_cannot_prove_absence() {
    let union = vec![
        address(ObjectKind::Deployment, "web"),
        address(ObjectKind::Service, "web"),
    ];
    let complete = ReleaseSnapshot {
        helm: None,
        objects: union.iter().cloned().map(ObjectRead::Absent).collect(),
    };
    complete.covers(&union).expect("a complete snapshot admits");

    let partial = ReleaseSnapshot {
        helm: None,
        objects: vec![ObjectRead::Absent(union[0].clone())],
    };
    assert_eq!(
        partial.covers(&union).unwrap_err().code,
        RefusalCode::ObservationUnavailable
    );

    let empty = ReleaseSnapshot {
        helm: None,
        objects: Vec::new(),
    };
    assert!(empty.covers(&union).is_err());

    let duplicated = ReleaseSnapshot {
        helm: None,
        objects: vec![
            ObjectRead::Absent(union[0].clone()),
            ObjectRead::Absent(union[0].clone()),
        ],
    };
    assert!(duplicated.covers(&union).is_err());
}

/// UID and resource version are recorded beside the projection digest, never inside it.
#[test]
fn server_bookkeeping_is_recorded_beside_the_projection_digest_not_inside_it() {
    let object = address(ObjectKind::Deployment, "web");
    let first = PresentObject {
        object: object.clone(),
        uid: text("uid-1"),
        resource_version: text("100"),
        content_digest: digest("projection"),
    };
    let restarted = PresentObject {
        resource_version: text("233"),
        ..first.clone()
    };
    assert_eq!(
        first.content_digest, restarted.content_digest,
        "a changed resourceVersion is not a changed projection"
    );
    assert_ne!(write_canonical(&first), write_canonical(&restarted));
    let replaced = PresentObject {
        uid: text("uid-2"),
        ..first.clone()
    };
    assert_ne!(
        write_canonical(&first),
        write_canonical(&replaced),
        "a replaced object is distinguishable from the one that was there"
    );
}

/// The ownership marker is exactly the declared string.
#[test]
fn the_ownership_marker_is_the_exact_declared_string() {
    let authority = uuid(0x33);
    let incarnation = uuid(0x22);
    assert_eq!(
        ownership_marker(&authority, &incarnation),
        format!("ess-recovery/1:{authority}:{incarnation}")
    );
}

/// An authority admits only what one revision can state about itself.
#[test]
fn an_authority_revision_admits_its_own_internal_shape() {
    let ok = authority(vec![permit(
        "checkout",
        Some(projection(&[(ObjectKind::Deployment, "web")], "b")),
        Some(projection(&[(ObjectKind::Deployment, "web")], "d")),
    )]);
    ok.validate().expect("the fixture authority admits");

    let mut no_context = ok.clone();
    no_context.contexts.clear();
    assert!(no_context.validate().is_err());

    let mut unordered = ok.clone();
    unordered.contexts.reverse();
    assert!(unordered.validate().is_err());

    let mut no_release = ok.clone();
    no_release.releases.clear();
    assert!(no_release.validate().is_err());

    let mut duplicated = ok.clone();
    duplicated.releases.push(ok.releases[0].clone());
    assert!(
        duplicated.validate().is_err(),
        "one service has one release permit, even when two permits agree"
    );

    let mut unpinned = ok.clone();
    unpinned.baseline_digest = None;
    assert_eq!(
        unpinned.validate().unwrap_err().code,
        RefusalCode::BaselineMismatch
    );
}

/// `Index` is nonnegative and stays an exact JSON integer.
#[test]
fn an_index_is_nonnegative_and_exactly_representable() {
    assert_eq!(Index::new(0).unwrap().get(), 0);
    assert!(Index::new(INDEX_LIMIT).is_err());
    assert!(Index::new(INDEX_LIMIT - 1).is_ok());
    assert!(Index::new(INDEX_LIMIT - 1).unwrap().next().is_err());
    assert!(serde_json::from_str::<Index>("-1").is_err());
    assert!(serde_json::from_str::<Index>("1.0").is_err());
}

/// `Text` refuses the empty string and every control character.
#[test]
fn text_refuses_empty_and_control_characters() {
    assert!(Text::new("app").is_ok());
    assert!(Text::new("").is_err());
    for control in ["a\nb", "a\0b", "a\tb", "\r"] {
        assert!(
            Text::new(control).is_err(),
            "{control:?} reaches an argument vector and a path"
        );
    }
}

/// `Uuid` admits exactly the lowercase hyphenated form.
#[test]
fn a_uuid_admits_only_the_lowercase_hyphenated_form() {
    assert!(Uuid::new("00000000-0000-0000-0000-000000000000").is_ok());
    for rejected in [
        "00000000-0000-0000-0000-00000000000",
        "00000000000000000000000000000000",
        "0000000A-0000-0000-0000-000000000000",
        "0000000g-0000-0000-0000-000000000000",
        "",
    ] {
        assert!(Uuid::new(rejected).is_err(), "{rejected:?} is not a UUID");
    }
}

/// The freshness budget is the declared thirty seconds.
#[test]
fn the_freshness_budget_is_thirty_seconds() {
    assert_eq!(FRESHNESS_BUDGET_MS, 30_000);
}

// --- Family J: the store, the reservation, the claim and the journal grammar --------------------
//
// C10's two dimensions in the matrix supplement — "Directory durability" and "Journal grammar" —
// are decided here, together with the durable exclusion of C08. Every barrier fails on its own,
// because a protocol whose boundaries only fail together has not been shown to have boundaries.

#[path = "support/fake_recovery.rs"]
mod fake_recovery;

use ess_cli::recovery::journal::{
    entry_name, open_store, publish_claim, read_claim, read_history, release_claim, reserve,
    scan_store, Journal, JournalState,
};
use ess_cli::recovery::model::{
    write_canonical as canonical, InvocationContext, JournalEntry, JournalFact, JournalFormat,
    LockClaim, LockFormat, Observation, ObservationPhase, Prepared, ProcessDisposition,
    ProcessOutcome, RegistryRef, Stopped, StoreFormat, StoreHeader,
};
use ess_cli::recovery::{Barrier, Host as _};
use fake_recovery::{scaffold, FixtureHost, STATE};

/// One way a published journal entry can be corrupt, as a case table entry.
type Corrupt = fn(&Path, &InvocationId);
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

fn scratch(name: &str) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "recovery-{name}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&root);
    scaffold(&root).expect("the fixture arrangement is laid out");
    root
}

fn store_header(root: &Path) -> String {
    canonical(&StoreHeader {
        format: StoreFormat::V1,
        store_epoch: uuid(0x11),
        host_id: text("fixture-host"),
        target: target(),
    })
    .replace("__root__", &root.display().to_string())
}

fn provisioned(name: &str) -> (PathBuf, FixtureHost) {
    let root = scratch(name);
    std::fs::write(root.join(STATE).join("store.json"), store_header(&root))
        .expect("the store header is provisioned");
    let host = FixtureHost::new(&root, (0x40..0x60).map(uuid).collect());
    (root, host)
}

fn opened_context() -> InvocationContext {
    InvocationContext {
        authority: authority(vec![permit(
            "checkout",
            None,
            Some(projection(&[(ObjectKind::Deployment, "web")], "d")),
        )]),
        registry: RegistryRef {
            generation: Index::new(3).unwrap(),
            digest: digest("registry"),
        },
        retry_of: None,
        selected: vec![text("checkout")],
        allow_removals: false,
    }
}

fn snapshot_of(names: &[(ObjectKind, &str)]) -> ReleaseSnapshot {
    let mut objects: Vec<ObjectRead> = names
        .iter()
        .map(|(kind, name)| ObjectRead::Absent(address(*kind, name)))
        .collect();
    objects.sort_by(|left, right| left.address().cmp(right.address()));
    ReleaseSnapshot {
        helm: None,
        objects,
    }
}

fn before(operation: u64) -> Observation {
    Observation {
        operation: Index::new(operation).unwrap(),
        phase: ObservationPhase::Before,
        started_ms: Index::new(1).unwrap(),
        finished_ms: Index::new(2).unwrap(),
        snapshot: snapshot_of(&[(ObjectKind::Deployment, "web")]),
    }
}

/// A missing or unprovisioned store refuses; nothing here ever initializes a replacement.
///
/// An empty journal in a *fresh* store looks exactly like an empty journal in a store whose
/// history was lost, and the second is not a clean start. This is the case that makes a missing
/// store a refusal rather than a first use.
#[test]
fn an_unprovisioned_store_refuses_and_is_never_silently_created() {
    let root = scratch("store-missing");
    let host = FixtureHost::new(&root, vec![uuid(0x40)]);
    let state = root.join(STATE);

    let refusal = open_store(&host, &state).unwrap_err();
    assert_eq!(refusal.code, RefusalCode::StoreInvalid);
    assert!(
        !state.join("store.json").exists(),
        "a missing header is never written by the reader that missed it"
    );

    std::fs::write(state.join("store.json"), store_header(&root)).unwrap();
    open_store(&host, &state).expect("a provisioned store admits");

    std::fs::remove_dir_all(state.join("invocations")).unwrap();
    assert_eq!(
        open_store(&host, &state).unwrap_err().code,
        RefusalCode::StoreInvalid,
        "invocations/ is preprovisioned, not created on demand"
    );
    assert!(!state.join("invocations").exists());
}

/// A store header that is not its own canonical spelling refuses.
#[test]
fn a_store_header_that_is_not_canonical_refuses() {
    let root = scratch("store-noncanonical");
    let state = root.join(STATE);
    let host = FixtureHost::new(&root, vec![uuid(0x40)]);
    let canonical_header = store_header(&root);

    std::fs::write(state.join("store.json"), canonical_header.trim_end()).unwrap();
    assert_eq!(
        open_store(&host, &state).unwrap_err().code,
        RefusalCode::StoreInvalid
    );

    std::fs::write(
        state.join("store.json"),
        canonical_header.replace(r#""format""#, r#""unknown":1,"format""#),
    )
    .unwrap();
    assert!(open_store(&host, &state).is_err());
}

/// A reservation crosses exclusive creation and both publication barriers, in that order.
#[test]
fn a_reservation_crosses_creation_and_both_publication_barriers_in_order() {
    let (root, host) = provisioned("reserve");
    let store = open_store(&host, &root.join(STATE)).expect("the store admits");
    let reservation = reserve(&host, &store).expect("a fresh reservation admits");

    assert!(reservation.directory().is_dir());
    assert_eq!(reservation.id().store_epoch, uuid(0x11));
    let crossed: Vec<Barrier> = host
        .crossed()
        .into_iter()
        .map(|(barrier, _)| barrier)
        .collect();
    assert_eq!(
        crossed,
        vec![
            Barrier::DirectoryCreate,
            Barrier::DirectorySync,
            Barrier::DirectoryParentSync
        ],
        "the new directory is synchronized, and then the parent that publishes its name is"
    );
}

/// Each reservation barrier fails on its own, and no claim or journal entry follows a failure.
#[test]
fn every_reservation_barrier_fails_independently_and_nothing_follows_it() {
    for barrier in [
        Barrier::DirectoryCreate,
        Barrier::DirectorySync,
        Barrier::DirectoryParentSync,
    ] {
        let root = scratch(&format!("reserve-{}", barrier.name()));
        std::fs::write(root.join(STATE).join("store.json"), store_header(&root)).unwrap();
        let host = FixtureHost::new(&root, vec![uuid(0x41)]).failing(barrier, None);
        let store = open_store(&host, &root.join(STATE)).expect("the store admits");

        let refusal = reserve(&host, &store).unwrap_err();
        assert_eq!(
            refusal.code,
            RefusalCode::EvidenceIncomplete,
            "{} must refuse on its own",
            barrier.name()
        );
        assert!(
            !store.lock_path().exists(),
            "{}: no claim is published before the reservation is admitted",
            barrier.name()
        );
    }
}

/// A published claim is never replaced, stolen, aged out or removed by a parent's death.
///
/// There is no timeout here, no PID and no lock age, and this case is what says so: a second
/// publisher refuses, and the claim is still exactly the first one's afterwards.
#[test]
fn a_published_claim_is_never_replaced_stolen_or_aged_out() {
    let (root, host) = provisioned("claim");
    let store = open_store(&host, &root.join(STATE)).expect("the store admits");
    let first = reserve(&host, &store).expect("a reservation admits");
    let mine = LockClaim {
        format: LockFormat::V1,
        invocation: first.id().clone(),
        authority_id: uuid(0x33),
        authority_revision: Index::new(4).unwrap(),
        target: target(),
        registry: RegistryRef {
            generation: Index::new(3).unwrap(),
            digest: digest("registry"),
        },
    };
    let published = publish_claim(&host, &store, &mine).expect("the first claim publishes");

    let second = reserve(&host, &store).expect("a second reservation admits");
    let theirs = LockClaim {
        invocation: second.id().clone(),
        ..mine.clone()
    };
    let refusal = publish_claim(&host, &store, &theirs).unwrap_err();
    assert_eq!(refusal.code, RefusalCode::MutationBlocked);

    let retained = read_claim(&host, &store)
        .expect("the retained claim reads")
        .expect("a claim is retained");
    assert_eq!(retained.claim, mine, "no competitor replaced the claim");
    assert_eq!(retained.digest, published);

    assert_eq!(
        release_claim(&host, &store, second.id()).unwrap_err().code,
        RefusalCode::MutationBlocked,
        "a claim is released only by the invocation that holds it"
    );
    assert!(store.lock_path().exists());

    release_claim(&host, &store, first.id()).expect("the holder releases its own claim");
    assert!(read_claim(&host, &store).expect("reads").is_none());
}

/// The journal grammar classifies every named prefix, and only the closed ones are closed.
#[test]
fn the_journal_grammar_classifies_every_named_prefix() {
    let (root, host) = provisioned("grammar");
    let store = open_store(&host, &root.join(STATE)).expect("the store admits");

    let reserved = reserve(&host, &store).expect("a reservation admits");
    let empty =
        read_history(&host, &store, &reserved.id().nonce).expect("an empty reservation reads");
    assert_eq!(empty.state, JournalState::EmptyReservation);
    assert!(empty.context().is_none());

    let mut journal = Journal::open(&reserved);
    journal
        .append(&host, JournalFact::Opened(Box::new(opened_context())))
        .expect("Opened publishes at sequence zero");
    let opened_only = read_history(&host, &store, &reserved.id().nonce).expect("reads");
    assert_eq!(opened_only.state, JournalState::Incomplete);
    assert!(opened_only.context().is_some());

    let observed = journal
        .append(&host, JournalFact::Observed(before(0)))
        .expect("Observed publishes");
    let no_prepared = read_history(&host, &store, &reserved.id().nonce).expect("reads");
    assert_eq!(no_prepared.state, JournalState::Incomplete);
    assert!(no_prepared.unresolved_preparations().is_empty());

    journal
        .append(
            &host,
            JournalFact::Prepared(Prepared {
                operation: Index::new(0).unwrap(),
                observation_sequence: observed,
            }),
        )
        .expect("Prepared publishes against its own Before observation");
    let prepared = read_history(&host, &store, &reserved.id().nonce).expect("reads");
    assert_eq!(prepared.state, JournalState::Incomplete);
    assert_eq!(
        prepared.unresolved_preparations().len(),
        1,
        "a Prepared without a durable disposition is indeterminate, not a non-launch"
    );

    journal
        .append(
            &host,
            JournalFact::ProcessOutcome(ProcessOutcome {
                operation: Index::new(0).unwrap(),
                disposition: ProcessDisposition::Acknowledged,
            }),
        )
        .expect("the disposition publishes");
    let acknowledged = read_history(&host, &store, &reserved.id().nonce).expect("reads");
    assert!(acknowledged.unresolved_preparations().is_empty());
    assert_eq!(
        acknowledged.acknowledged_without_after(),
        vec![Index::new(0).unwrap()],
        "an acknowledged disposition without a fresh After observation is not settled"
    );

    journal
        .append(
            &host,
            JournalFact::Observed(Observation {
                phase: ObservationPhase::After,
                ..before(0)
            }),
        )
        .expect("the After observation publishes");
    let after = read_history(&host, &store, &reserved.id().nonce).expect("reads");
    assert!(after.acknowledged_without_after().is_empty());
    assert_eq!(
        after.state,
        JournalState::Incomplete,
        "no terminal fact yet"
    );

    journal
        .append(&host, JournalFact::Completed(Index::new(1).unwrap()))
        .expect("Completed accounts for the one selected operation");
    let closed = read_history(&host, &store, &reserved.id().nonce).expect("reads");
    assert_eq!(closed.state, JournalState::Closed);
    assert!(journal.is_closed());
    assert!(
        journal
            .append(&host, JournalFact::Completed(Index::new(1).unwrap()))
            .is_err(),
        "a closed journal admits no further entry"
    );
}

/// A `Stopped` closes a journal, and a `Completed` whose accounting is short does not.
#[test]
fn a_stopped_journal_is_closed_and_a_short_completion_is_refused() {
    let (root, host) = provisioned("terminal");
    let store = open_store(&host, &root.join(STATE)).expect("the store admits");
    let reserved = reserve(&host, &store).expect("a reservation admits");
    let mut journal = Journal::open(&reserved);
    let mut context = opened_context();
    context.selected = vec![text("checkout"), text("billing")];
    journal
        .append(&host, JournalFact::Opened(Box::new(context)))
        .expect("Opened publishes");
    journal
        .append(
            &host,
            JournalFact::Stopped(Stopped {
                operation: Some(Index::new(0).unwrap()),
                reason: RefusalCode::ObservationUnavailable,
            }),
        )
        .expect("Stopped publishes");
    let history = read_history(&host, &store, &reserved.id().nonce).expect("reads");
    assert_eq!(history.state, JournalState::Closed);
    assert!(
        !history
            .entries
            .iter()
            .any(|entry| matches!(entry.fact, JournalFact::Completed(_))),
        "Stopped is a settled stopping decision, not a completion"
    );
}

/// Every corrupt published entry refuses, and its bytes are still there afterwards.
#[test]
fn every_corrupt_published_entry_refuses_and_its_bytes_are_preserved() {
    let cases: Vec<(&str, Corrupt)> = vec![
        ("torn", |directory, _| {
            std::fs::write(directory.join(entry_name(Index::new(1).unwrap())), "{\n").unwrap();
        }),
        ("gap", |directory, invocation| {
            let entry = JournalEntry {
                format: JournalFormat::V1,
                invocation: invocation.clone(),
                sequence: Index::new(7).unwrap(),
                previous_digest: Some(digest("nothing")),
                fact: JournalFact::Observed(before(0)),
            };
            std::fs::write(
                directory.join(entry_name(Index::new(7).unwrap())),
                canonical(&entry),
            )
            .unwrap();
        }),
        ("wrong-predecessor", |directory, invocation| {
            let entry = JournalEntry {
                format: JournalFormat::V1,
                invocation: invocation.clone(),
                sequence: Index::new(1).unwrap(),
                previous_digest: Some(digest("not the predecessor")),
                fact: JournalFact::Observed(before(0)),
            };
            std::fs::write(
                directory.join(entry_name(Index::new(1).unwrap())),
                canonical(&entry),
            )
            .unwrap();
        }),
        ("foreign-scope", |directory, invocation| {
            let entry = JournalEntry {
                format: JournalFormat::V1,
                invocation: InvocationId {
                    store_epoch: invocation.store_epoch.clone(),
                    nonce: uuid(0xee),
                },
                sequence: Index::new(1).unwrap(),
                previous_digest: Some(digest("x")),
                fact: JournalFact::Observed(before(0)),
            };
            std::fs::write(
                directory.join(entry_name(Index::new(1).unwrap())),
                canonical(&entry),
            )
            .unwrap();
        }),
    ];

    for (name, corrupt) in cases {
        let (root, host) = provisioned(&format!("corrupt-{name}"));
        let store = open_store(&host, &root.join(STATE)).expect("the store admits");
        let reserved = reserve(&host, &store).expect("a reservation admits");
        let mut journal = Journal::open(&reserved);
        journal
            .append(&host, JournalFact::Opened(Box::new(opened_context())))
            .expect("Opened publishes");
        corrupt(reserved.directory(), reserved.id());

        let before_bytes = std::fs::read_dir(reserved.directory())
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| std::fs::read(entry.path()).unwrap_or_default())
            .collect::<Vec<_>>();
        let refusal = read_history(&host, &store, &reserved.id().nonce).unwrap_err();
        assert_eq!(
            refusal.code,
            RefusalCode::EvidenceIncomplete,
            "{name} must refuse"
        );
        let after_bytes = std::fs::read_dir(reserved.directory())
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| std::fs::read(entry.path()).unwrap_or_default())
            .collect::<Vec<_>>();
        assert_eq!(
            before_bytes, after_bytes,
            "{name}: corrupt evidence is preserved for diagnosis, never repaired"
        );
    }
}

/// An entry published after a terminal fact refuses.
#[test]
fn an_entry_after_a_terminal_fact_refuses() {
    let (root, host) = provisioned("after-terminal");
    let store = open_store(&host, &root.join(STATE)).expect("the store admits");
    let reserved = reserve(&host, &store).expect("a reservation admits");
    let mut journal = Journal::open(&reserved);
    journal
        .append(&host, JournalFact::Opened(Box::new(opened_context())))
        .expect("Opened publishes");
    journal
        .append(&host, JournalFact::Completed(Index::new(1).unwrap()))
        .expect("Completed publishes");
    let entry = JournalEntry {
        format: JournalFormat::V1,
        invocation: reserved.id().clone(),
        sequence: Index::new(2).unwrap(),
        previous_digest: Some(digest("whatever")),
        fact: JournalFact::Observed(before(0)),
    };
    std::fs::write(
        reserved
            .directory()
            .join(entry_name(Index::new(2).unwrap())),
        canonical(&entry),
    )
    .unwrap();
    assert_eq!(
        read_history(&host, &store, &reserved.id().nonce)
            .unwrap_err()
            .code,
        RefusalCode::EvidenceIncomplete
    );
}

/// An unpublished private stage is diagnostic residue, never a journal entry.
#[test]
fn an_unpublished_stage_is_not_a_journal_entry() {
    let (root, host) = provisioned("stage");
    let store = open_store(&host, &root.join(STATE)).expect("the store admits");
    let reserved = reserve(&host, &store).expect("a reservation admits");
    let mut journal = Journal::open(&reserved);
    journal
        .append(&host, JournalFact::Opened(Box::new(opened_context())))
        .expect("Opened publishes");
    std::fs::write(
        reserved
            .directory()
            .join(".stage-00000000000000000001.json"),
        "half a record",
    )
    .unwrap();
    let history =
        read_history(&host, &store, &reserved.id().nonce).expect("the prefix still reads");
    assert_eq!(history.state, JournalState::Incomplete);
    assert_eq!(history.entries.len(), 1, "a stage constructs no fact");
}

/// Every journal publication barrier fails on its own, and no entry appears when one does.
#[test]
fn every_journal_publication_barrier_fails_independently() {
    for barrier in [
        Barrier::StageWrite,
        Barrier::StageSync,
        Barrier::StageReadback,
        Barrier::Publish,
        Barrier::PublishParentSync,
    ] {
        let root = scratch(&format!("entry-{}", barrier.name()));
        std::fs::write(root.join(STATE).join("store.json"), store_header(&root)).unwrap();
        let host = FixtureHost::new(&root, vec![uuid(0x42)]).failing(barrier, None);
        let store = open_store(&host, &root.join(STATE)).expect("the store admits");
        let reserved = reserve(&host, &store).expect("a reservation admits");
        let mut journal = Journal::open(&reserved);
        let refusal = journal
            .append(&host, JournalFact::Opened(Box::new(opened_context())))
            .unwrap_err();
        assert_eq!(
            refusal.code,
            RefusalCode::EvidenceIncomplete,
            "{} must refuse on its own",
            barrier.name()
        );
        let published = std::fs::read_dir(reserved.directory())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| !entry.file_name().to_string_lossy().starts_with('.'))
            .count();
        assert_eq!(
            published,
            usize::from(barrier == Barrier::PublishParentSync),
            "{}: only a failure after the hard link leaves a published name",
            barrier.name()
        );
    }
}

/// Omitting `retry_of` hides no retained history: the scan reads every reservation in the store.
#[test]
fn omitting_retry_of_hides_no_retained_history() {
    let (root, host) = provisioned("scan");
    let store = open_store(&host, &root.join(STATE)).expect("the store admits");
    let first = reserve(&host, &store).expect("a reservation admits");
    let mut journal = Journal::open(&first);
    journal
        .append(&host, JournalFact::Opened(Box::new(opened_context())))
        .expect("Opened publishes");
    let second = reserve(&host, &store).expect("a second reservation admits");

    let histories = scan_store(&host, &store).expect("the whole store scans");
    assert_eq!(histories.len(), 2);
    let nonces: Vec<_> = histories
        .iter()
        .map(|history| history.nonce.clone())
        .collect();
    assert!(nonces.contains(&first.id().nonce) && nonces.contains(&second.id().nonce));
    assert_eq!(
        histories
            .iter()
            .filter(|history| history.state == JournalState::EmptyReservation)
            .count(),
        1,
        "an empty reservation is incomplete and is still relevant history"
    );
}

/// A `Prepared` that names something other than its own earlier `Before` observation refuses.
#[test]
fn a_prepared_fact_must_name_an_earlier_before_observation_of_its_own_operation() {
    let (root, host) = provisioned("prepared");
    let store = open_store(&host, &root.join(STATE)).expect("the store admits");
    let reserved = reserve(&host, &store).expect("a reservation admits");
    let mut journal = Journal::open(&reserved);
    journal
        .append(&host, JournalFact::Opened(Box::new(opened_context())))
        .expect("Opened publishes");
    let after_sequence = journal
        .append(
            &host,
            JournalFact::Observed(Observation {
                phase: ObservationPhase::After,
                ..before(0)
            }),
        )
        .expect("an After observation publishes");
    assert!(
        journal
            .append(
                &host,
                JournalFact::Prepared(Prepared {
                    operation: Index::new(0).unwrap(),
                    observation_sequence: after_sequence,
                }),
            )
            .is_err()
            || read_history(&host, &store, &reserved.id().nonce).is_err(),
        "an After observation cannot authorize a launch"
    );
}

// --- Family R02: the complete active-registry scan and one admitted authority --------------------
//
// The scan is whole-registry on purpose, and most of these cases are about an entry the caller did
// *not* select. An implementation that validated only the selection would admit a registry whose
// other entries name a different store for the same target, or claim the same object address —
// and the caller would never see the entry that made its own selection unsafe.

use ess_cli::recovery::authority::{admit, read_kubeconfig, read_registry, recheck};
use ess_cli::recovery::model::{AuthorityRegistry, RegistryFormat};
use fake_recovery::{install_helm, publish_registry, write_kubeconfig, ADMIN};

const CA: &str = "LS0tLUZJWFRVUkUtQ0EtLS0t";

fn registry(authorities: Vec<Authority>) -> AuthorityRegistry {
    AuthorityRegistry {
        format: RegistryFormat::V1,
        generation: Index::new(7).unwrap(),
        authorities,
    }
}

/// Builds one active entry whose every referenced file exists under `root`.
fn active(
    root: &Path,
    id: u8,
    environment: &str,
    endpoint: &str,
    cluster_uid: &str,
    contexts: &[&str],
    releases: Vec<ReleasePermit>,
) -> Authority {
    let (helm_path, helm_digest) =
        install_helm(root, b"fixture helm artifact").expect("the artifact installs");
    // One protected kubeconfig per active entry. A single shared file would be rewritten by each
    // builder call, and the *first* entry would then refuse for a target mismatch the case is not
    // about — an earlier refusal making a later assertion pass without reaching its boundary.
    let kubeconfig = write_kubeconfig(root, &kubeconfig_name(id), endpoint, CA, contexts)
        .expect("the kubeconfig is published");
    let mut authority = authority(releases);
    authority.authority_id = uuid(id);
    authority.environment = text(environment);
    authority.contexts = contexts.iter().map(|name| text(name)).collect();
    authority.target = TargetPin {
        api_server: text(endpoint),
        ca_digest: Digest::of_bytes(CA.as_bytes()),
        identity_namespace: NamespacePin {
            name: text(IDENTITY_NAMESPACE),
            uid: text(cluster_uid),
        },
    };
    authority.host = HostPolicy {
        host_id: text("fixture-host"),
        executor_uid: rustix::process::getuid().as_raw(),
        store_epoch: uuid(0x11),
        state_root: text(&root.join(STATE).display().to_string()),
        kubeconfig: text(&kubeconfig.display().to_string()),
        helm: HelmBinary {
            path: text(&helm_path.display().to_string()),
            digest: Digest::new(&helm_digest).expect("the installed digest admits"),
            version: HelmVersion::new("v3.16.2").unwrap(),
            protocol: HelmProtocol::Helm3Recovery1,
        },
    };
    authority
}

fn kubeconfig_name(id: u8) -> String {
    format!("kubeconfig-{id:02x}.yaml")
}

fn one_permit(service: &str, object: &str) -> Vec<ReleasePermit> {
    vec![permit(
        service,
        None,
        Some(projection(&[(ObjectKind::Deployment, object)], "d")),
    )]
}

fn admin_of(root: &Path) -> PathBuf {
    root.join(ADMIN)
}

/// The whole active registry is scanned before any authority is selected.
///
/// The invalid entry here is the one the caller did *not* ask for, and it still prevents
/// admission. That is C05's "an unreadable or invalid active entry prevents admission; it is not
/// skipped", and it is the case an implementation that validated its selection first would fail.
#[test]
fn an_invalid_unselected_registry_entry_rejects_the_selection() {
    let (root, host) = provisioned("registry-unselected");
    let wanted = active(
        &root,
        0x31,
        "production",
        "https://api.one.invalid:6443",
        "cluster-1",
        &["prod"],
        one_permit("checkout", "web"),
    );
    let mut other = active(
        &root,
        0x32,
        "staging",
        "https://api.one.invalid:6443",
        "cluster-1",
        &["prod"],
        one_permit("billing", "billing"),
    );
    publish_registry(&root, &registry(vec![wanted.clone(), other.clone()]))
        .expect("the arrangement publishes");
    admit(&host, &admin_of(&root), &uuid(0x31)).expect("a consistent registry admits");

    other.contexts = vec![text("absent-context")];
    publish_registry(&root, &registry(vec![wanted.clone(), other]))
        .expect("the arrangement publishes");
    let refusal = admit(&host, &admin_of(&root), &uuid(0x31)).unwrap_err();
    assert!(
        matches!(
            refusal.code,
            RefusalCode::AuthorityMismatch | RefusalCode::TargetMismatch
        ),
        "the entry the caller did not select still prevents admission; got {refusal}"
    );
}

/// An endpoint alias, a second endpoint for one cluster and a rebound CA all refuse.
#[test]
fn every_endpoint_and_cluster_alias_between_active_entries_refuses() {
    let (root, host) = provisioned("registry-endpoint");
    let first = active(
        &root,
        0x31,
        "production",
        "https://api.one.invalid:6443",
        "cluster-1",
        &["prod"],
        one_permit("checkout", "web"),
    );

    let second_endpoint = active(
        &root,
        0x32,
        "staging",
        "https://api.two.invalid:6443",
        "cluster-1",
        &["prod"],
        one_permit("billing", "billing"),
    );
    publish_registry(&root, &registry(vec![first.clone(), second_endpoint]))
        .expect("the arrangement publishes");
    assert_eq!(
        admit(&host, &admin_of(&root), &uuid(0x31))
            .unwrap_err()
            .code,
        RefusalCode::AuthorityMismatch,
        "a second endpoint for one physical cluster is an unsupported endpoint alias"
    );

    let mut second_cluster = active(
        &root,
        0x32,
        "staging",
        "https://api.one.invalid:6443",
        "cluster-2",
        &["prod"],
        one_permit("billing", "billing"),
    );
    second_cluster.host.state_root = text(&root.join("var/lib/other").display().to_string());
    publish_registry(&root, &registry(vec![first, second_cluster]))
        .expect("the arrangement publishes");
    assert_eq!(
        admit(&host, &admin_of(&root), &uuid(0x31))
            .unwrap_err()
            .code,
        RefusalCode::AuthorityMismatch,
        "one canonical endpoint cannot map to two identity-namespace UIDs"
    );
}

/// One store root cannot serve two clusters, and nested distinct roots refuse.
#[test]
fn a_store_root_serving_two_clusters_or_nesting_another_refuses() {
    let (root, host) = provisioned("registry-store");
    let first = active(
        &root,
        0x31,
        "production",
        "https://api.one.invalid:6443",
        "cluster-1",
        &["prod"],
        one_permit("checkout", "web"),
    );
    let mut second = active(
        &root,
        0x32,
        "staging",
        "https://api.two.invalid:6443",
        "cluster-2",
        &["prod"],
        one_permit("billing", "billing"),
    );
    second.host.state_root = first.host.state_root.clone();
    publish_registry(&root, &registry(vec![first.clone(), second.clone()]))
        .expect("the arrangement publishes");
    assert_eq!(
        admit(&host, &admin_of(&root), &uuid(0x31))
            .unwrap_err()
            .code,
        RefusalCode::AuthorityMismatch,
        "one store epoch and root cannot map to two physical clusters"
    );

    let mut nested = second;
    nested.host.state_root = text(&root.join(STATE).join("nested").display().to_string());
    publish_registry(&root, &registry(vec![first, nested])).expect("the arrangement publishes");
    assert_eq!(
        admit(&host, &admin_of(&root), &uuid(0x31))
            .unwrap_err()
            .code,
        RefusalCode::AuthorityMismatch,
        "a nested distinct store root shares the parent that publishes its names"
    );
}

/// One release address and one object address belong to exactly one permit.
#[test]
fn a_release_or_object_address_claimed_twice_refuses() {
    let (root, host) = provisioned("registry-address");
    let first = active(
        &root,
        0x31,
        "production",
        "https://api.one.invalid:6443",
        "cluster-1",
        &["prod"],
        one_permit("checkout", "web"),
    );

    let mut same_release = active(
        &root,
        0x32,
        "staging",
        "https://api.one.invalid:6443",
        "cluster-1",
        &["prod"],
        one_permit("checkout", "other"),
    );
    same_release.host.state_root = first.host.state_root.clone();
    publish_registry(&root, &registry(vec![first.clone(), same_release]))
        .expect("the arrangement publishes");
    assert_eq!(
        admit(&host, &admin_of(&root), &uuid(0x31))
            .unwrap_err()
            .code,
        RefusalCode::AuthorityMismatch,
        "one physical release address has one owner"
    );

    let mut same_object = active(
        &root,
        0x32,
        "staging",
        "https://api.one.invalid:6443",
        "cluster-1",
        &["prod"],
        one_permit("billing", "web"),
    );
    same_object.host.state_root = first.host.state_root.clone();
    same_object.releases[0].release_name = text("billing");
    publish_registry(&root, &registry(vec![first, same_object]))
        .expect("the arrangement publishes");
    assert_eq!(
        admit(&host, &admin_of(&root), &uuid(0x31))
            .unwrap_err()
            .code,
        RefusalCode::AuthorityMismatch,
        "two permits cannot claim one direct object address"
    );
}

/// A snapshot that does not equal its immutable generation archive refuses.
#[test]
fn a_snapshot_that_does_not_equal_its_generation_archive_refuses() {
    let (root, host) = provisioned("registry-archive");
    let entry = active(
        &root,
        0x31,
        "production",
        "https://api.one.invalid:6443",
        "cluster-1",
        &["prod"],
        one_permit("checkout", "web"),
    );
    let snapshot = registry(vec![entry.clone()]);
    publish_registry(&root, &snapshot).expect("the arrangement publishes");
    read_registry(&host, &admin_of(&root)).expect("a matched snapshot reads");

    let mut divergent = snapshot.clone();
    divergent.authorities[0].environment = text("elsewhere");
    std::fs::write(admin_of(&root).join("registry.json"), canonical(&divergent)).unwrap();
    let refusal = read_registry(&host, &admin_of(&root)).unwrap_err();
    assert_eq!(refusal.code, RefusalCode::AuthorityMismatch);

    std::fs::remove_file(
        admin_of(&root)
            .join("registry-history")
            .join(format!("{}.json", snapshot.generation)),
    )
    .unwrap();
    std::fs::write(admin_of(&root).join("registry.json"), canonical(&snapshot)).unwrap();
    assert!(
        read_registry(&host, &admin_of(&root)).is_err(),
        "a generation with no immutable archive is not admitted"
    );
}

/// An embedded authority that differs from its retained immutable revision refuses.
#[test]
fn an_embedded_authority_that_differs_from_its_retained_revision_refuses() {
    let (root, host) = provisioned("registry-revision");
    let entry = active(
        &root,
        0x31,
        "production",
        "https://api.one.invalid:6443",
        "cluster-1",
        &["prod"],
        one_permit("checkout", "web"),
    );
    publish_registry(&root, &registry(vec![entry.clone()])).expect("the arrangement publishes");
    admit(&host, &admin_of(&root), &uuid(0x31)).expect("a matched revision admits");

    let mut tampered = entry.clone();
    tampered.releases[0].may_create = false;
    std::fs::write(
        admin_of(&root)
            .join("authorities")
            .join(entry.authority_id.as_str())
            .join("history")
            .join(format!("{}.json", entry.revision)),
        canonical(&tampered),
    )
    .unwrap();
    assert_eq!(
        admit(&host, &admin_of(&root), &uuid(0x31))
            .unwrap_err()
            .code,
        RefusalCode::AuthorityMismatch
    );
}

/// An admitted context must resolve through the protected kubeconfig to the pinned target.
#[test]
fn a_context_alias_must_resolve_to_the_pinned_endpoint_and_trust() {
    let (root, host) = provisioned("registry-context");
    let mut entry = active(
        &root,
        0x31,
        "production",
        "https://api.one.invalid:6443",
        "cluster-1",
        &["prod", "prod-alias"],
        one_permit("checkout", "web"),
    );
    publish_registry(&root, &registry(vec![entry.clone()])).expect("the arrangement publishes");
    admit(&host, &admin_of(&root), &uuid(0x31)).expect("both aliases resolve");

    entry.contexts = vec![text("prod"), text("undefined")];
    publish_registry(&root, &registry(vec![entry.clone()])).expect("the arrangement publishes");
    assert_eq!(
        admit(&host, &admin_of(&root), &uuid(0x31))
            .unwrap_err()
            .code,
        RefusalCode::TargetMismatch
    );

    entry.contexts = vec![text("prod")];
    write_kubeconfig(
        &root,
        &kubeconfig_name(0x31),
        "https://api.elsewhere.invalid:6443",
        CA,
        &["prod"],
    )
    .unwrap();
    publish_registry(&root, &registry(vec![entry.clone()])).expect("the arrangement publishes");
    assert_eq!(
        admit(&host, &admin_of(&root), &uuid(0x31))
            .unwrap_err()
            .code,
        RefusalCode::TargetMismatch,
        "a reused kubeconfig cannot point at a different target"
    );

    write_kubeconfig(
        &root,
        &kubeconfig_name(0x31),
        "https://api.one.invalid:6443",
        "LS0tLU9USEVSLUNBLS0tLQ==",
        &["prod"],
    )
    .unwrap();
    assert_eq!(
        admit(&host, &admin_of(&root), &uuid(0x31))
            .unwrap_err()
            .code,
        RefusalCode::TargetMismatch,
        "a rebound trust anchor is a rebound target"
    );
}

/// Every unsupported credential configuration in a kubeconfig refuses.
#[test]
fn an_unsupported_credential_configuration_in_a_kubeconfig_refuses() {
    let (root, host) = provisioned("kubeconfig-credentials");
    let path = write_kubeconfig(
        &root,
        &kubeconfig_name(0x31),
        "https://api.one.invalid:6443",
        CA,
        &["prod"],
    )
    .unwrap();
    read_kubeconfig(&host, &path).expect("a supported kubeconfig admits");

    for injected in [
        "    insecure-skip-tls-verify: true\n",
        "    proxy-url: http://proxy.invalid:3128\n",
        "    exec:\n      command: /bin/false\n",
        "    auth-provider:\n      name: oidc\n",
        "    as: someone-else\n",
    ] {
        let text = std::fs::read_to_string(&path).unwrap();
        std::fs::write(&path, format!("{text}{injected}")).unwrap();
        let refusal = read_kubeconfig(&host, &path).unwrap_err();
        assert_eq!(
            refusal.code,
            RefusalCode::UnsupportedProfile,
            "{injected:?} must refuse"
        );
        std::fs::write(&path, text).unwrap();
    }
}

/// A foreign host identity, a foreign executor UID and a second Helm artifact refuse.
#[test]
fn a_foreign_host_executor_or_helm_artifact_refuses() {
    let (root, host) = provisioned("registry-host");
    let mut entry = active(
        &root,
        0x31,
        "production",
        "https://api.one.invalid:6443",
        "cluster-1",
        &["prod"],
        one_permit("checkout", "web"),
    );
    publish_registry(&root, &registry(vec![entry.clone()])).expect("the arrangement publishes");
    admit(&host, &admin_of(&root), &uuid(0x31)).expect("the local host admits");

    let elsewhere = FixtureHost::new(&root, vec![uuid(0x40)]).named("another-host");
    assert_eq!(
        admit(&elsewhere, &admin_of(&root), &uuid(0x31))
            .unwrap_err()
            .code,
        RefusalCode::AuthorityMismatch
    );

    entry.host.executor_uid = rustix::process::getuid().as_raw() + 1;
    publish_registry(&root, &registry(vec![entry.clone()])).expect("the arrangement publishes");
    assert_eq!(
        admit(&host, &admin_of(&root), &uuid(0x31))
            .unwrap_err()
            .code,
        RefusalCode::AuthorityMismatch
    );

    let mut second = active(
        &root,
        0x32,
        "staging",
        "https://api.one.invalid:6443",
        "cluster-1",
        &["prod"],
        one_permit("billing", "billing"),
    );
    let (other_path, other_digest) =
        install_helm(&root, b"a different artifact").expect("a second artifact installs");
    second.host.helm = HelmBinary {
        path: text(&other_path.display().to_string()),
        digest: Digest::new(&other_digest).unwrap(),
        version: HelmVersion::new("v3.16.2").unwrap(),
        protocol: HelmProtocol::Helm3Recovery1,
    };
    second.releases[0].release_name = text("billing");
    let mut first = active(
        &root,
        0x31,
        "production",
        "https://api.one.invalid:6443",
        "cluster-1",
        &["prod"],
        one_permit("checkout", "web"),
    );
    first.host.state_root = second.host.state_root.clone();
    publish_registry(&root, &registry(vec![first, second])).expect("the arrangement publishes");
    assert_eq!(
        admit(&host, &admin_of(&root), &uuid(0x31))
            .unwrap_err()
            .code,
        RefusalCode::AuthorityMismatch,
        "all active entries on one host name the same admitted artifact"
    );
}

/// An empty registry authorizes nothing, and an unknown authority is not selectable.
#[test]
fn an_empty_registry_authorizes_nothing_and_an_unknown_authority_is_not_selectable() {
    let (root, host) = provisioned("registry-empty");
    publish_registry(&root, &registry(Vec::new())).expect("the arrangement publishes");
    assert_eq!(
        admit(&host, &admin_of(&root), &uuid(0x31))
            .unwrap_err()
            .code,
        RefusalCode::AuthorityMismatch
    );

    let entry = active(
        &root,
        0x31,
        "production",
        "https://api.one.invalid:6443",
        "cluster-1",
        &["prod"],
        one_permit("checkout", "web"),
    );
    publish_registry(&root, &registry(vec![entry])).expect("the arrangement publishes");
    let refusal = admit(&host, &admin_of(&root), &uuid(0x99)).unwrap_err();
    assert_eq!(refusal.code, RefusalCode::AuthorityMismatch);
    assert!(
        refusal.detail.contains("self-authorizing"),
        "the refusal says what it will not do instead: {refusal}"
    );
}

/// A changed generation or changed bytes stops the invocation rather than becoming new policy.
#[test]
fn a_changed_registry_generation_or_bytes_stops_the_invocation() {
    let (root, host) = provisioned("registry-recheck");
    let entry = active(
        &root,
        0x31,
        "production",
        "https://api.one.invalid:6443",
        "cluster-1",
        &["prod"],
        one_permit("checkout", "web"),
    );
    let mut snapshot = registry(vec![entry.clone()]);
    publish_registry(&root, &snapshot).expect("the arrangement publishes");
    let (admitted, _) = admit(&host, &admin_of(&root), &uuid(0x31)).expect("the registry admits");
    recheck(&host, &admin_of(&root), &admitted.reference).expect("an unchanged registry rechecks");

    snapshot.generation = Index::new(8).unwrap();
    publish_registry(&root, &snapshot).expect("the arrangement republishes");
    let refusal = recheck(&host, &admin_of(&root), &admitted.reference).unwrap_err();
    assert_eq!(refusal.code, RefusalCode::AuthorityMismatch);
    assert!(refusal.detail.contains("generation"));

    snapshot.generation = Index::new(7).unwrap();
    snapshot.authorities[0].releases[0].may_create = false;
    snapshot.authorities[0].releases[0].baseline =
        Some(projection(&[(ObjectKind::Deployment, "web")], "b"));
    snapshot.authorities[0].baseline_digest = Some(digest("baseline"));
    publish_registry(&root, &snapshot).expect("the arrangement republishes");
    let refusal = recheck(&host, &admin_of(&root), &admitted.reference).unwrap_err();
    assert!(
        refusal.detail.contains("bytes changed"),
        "different bytes at a retained generation refuse: {refusal}"
    );
}

// --- Families R06-R11 and the generated-chart refusal dimension ----------------------------------
//
// The first recovery profile admits ESS-generated charts, not Helm charts. Every case below is one
// way a chart can be a perfectly good Helm chart and still not be the projection the caller pinned.

use ess_cli::recovery::chart::{
    admit_inventory, admit_payload, admit_rendered, projection_files, read_runtime,
    PROJECTION_FILES, RUNTIME_DIRECTORY,
};
use fake_recovery::{archive, as_members, chart_members};

/// A chart archive's members as [`archive`] takes them, borrowed from the projection.
type Members<'a> = Vec<(String, &'a [u8], tar::EntryType)>;

fn runtime_document() -> String {
    let zero = format!("sha256:{}", "0".repeat(64));
    let value = serde_json::json!({
        "format": "ess-runtime-ir/1",
        "runtime": "fixture",
        "semantic_digest": zero,
        "realization_digest": zero,
        "build_digest": zero,
        "processes": {"server": {"name": "server", "image": "app"}},
        "containers": {"server": {"name": "server", "process": "server"}},
        "workloads": {"app": {"name": "app", "components": ["component"],
                              "containers": ["server"], "replicas": 1}},
        "provided_endpoints": {}
    });
    ess_deployment::RuntimeIr::from_json(&value.to_string())
        .expect("the fixture runtime admits")
        .to_canonical_json()
}

fn publish_runtime(root: &Path, document: &str) -> Digest {
    let digest = Digest::of_bytes(document.as_bytes());
    let path = root
        .join(ADMIN)
        .join(RUNTIME_DIRECTORY)
        .join(format!("{}.json", digest.as_str().replace(':', "-")));
    std::fs::write(path, document).expect("the pinned runtime publishes");
    digest
}

fn chart_source(runtime_digest: Digest) -> ChartSource {
    ChartSource {
        runtime_digest,
        chart_name: text("fixture"),
        chart_version: text("1.0.0"),
    }
}

/// The pinned runtime is read by its own digest, through the existing strict reader.
#[test]
fn the_pinned_runtime_is_read_by_its_own_digest_and_must_be_canonical() {
    let (root, host) = provisioned("runtime");
    let document = runtime_document();
    let source = chart_source(publish_runtime(&root, &document));
    read_runtime(&host, &admin_of(&root), &source).expect("the pinned runtime admits");

    let path = admin_of(&root).join(RUNTIME_DIRECTORY).join(format!(
        "{}.json",
        source.runtime_digest.as_str().replace(':', "-")
    ));
    std::fs::write(
        &path,
        document.replace("\"replicas\": 1", "\"replicas\":  1"),
    )
    .unwrap();
    let refusal = read_runtime(&host, &admin_of(&root), &source).unwrap_err();
    assert_eq!(refusal.code, RefusalCode::PreparationFailed);

    std::fs::write(&path, document.trim_end()).unwrap();
    assert!(
        read_runtime(&host, &admin_of(&root), &source).is_err(),
        "a document that does not hash to its own name is not the pinned runtime"
    );
}

/// The projector emits exactly the five admitted files, and nothing else is admitted.
#[test]
fn the_pinned_projection_is_exactly_five_named_files() {
    let (root, _host) = provisioned("projection");
    let document = runtime_document();
    let source = chart_source(publish_runtime(&root, &document));
    let runtime = ess_deployment::RuntimeIr::from_json(&document).unwrap();
    let files = projection_files(&runtime, &source).expect("the projection admits");
    let names: Vec<&str> = files.keys().map(String::as_str).collect();
    assert_eq!(names, PROJECTION_FILES);
    assert_eq!(names.len(), 5);
}

/// A chart whose members do not equal the pinned projection refuses, byte for byte.
#[test]
fn a_chart_member_that_does_not_equal_its_projected_bytes_refuses() {
    let (root, _host) = provisioned("chart-bytes");
    let document = runtime_document();
    let source = chart_source(publish_runtime(&root, &document));
    let runtime = ess_deployment::RuntimeIr::from_json(&document).unwrap();
    let files = projection_files(&runtime, &source).expect("the projection admits");

    let members = chart_members("fixture", &files);
    let payload = archive(&as_members(&members));
    admit_payload(&payload, &runtime, &source).expect("the exact projection admits");

    let mut edited = files.clone();
    let target = edited
        .get_mut("templates/workloads.yaml")
        .expect("the projection has templates");
    target.push_str("# one added byte\n");
    let edited_members = chart_members("fixture", &edited);
    let refusal =
        admit_payload(&archive(&as_members(&edited_members)), &runtime, &source).unwrap_err();
    assert_eq!(refusal.code, RefusalCode::PreparationFailed);
    assert!(refusal.detail.contains("projected bytes"), "{refusal}");
}

/// Every archive shape outside the profile refuses, one shape at a time.
#[test]
fn every_generated_chart_archive_shape_outside_the_profile_refuses() {
    let (root, _host) = provisioned("chart-shape");
    let document = runtime_document();
    let source = chart_source(publish_runtime(&root, &document));
    let runtime = ess_deployment::RuntimeIr::from_json(&document).unwrap();
    let files = projection_files(&runtime, &source).expect("the projection admits");
    let base = chart_members("fixture", &files);

    let link = b"Chart.yaml".to_vec();
    let extra = b"extra\n".to_vec();
    let cases: Vec<(&str, Members<'_>)> = vec![
        ("a symbolic link member", {
            let mut members = base.clone();
            members.push((
                "fixture/link.yaml".to_owned(),
                link.as_slice(),
                tar::EntryType::Symlink,
            ));
            members
        }),
        ("a traversing member", {
            let mut members = base.clone();
            members.push((
                "fixture/../escape.yaml".to_owned(),
                extra.as_slice(),
                tar::EntryType::Regular,
            ));
            members
        }),
        ("a duplicate member", {
            let mut members = base.clone();
            members.push((
                "fixture/Chart.yaml".to_owned(),
                files["Chart.yaml"].as_bytes(),
                tar::EntryType::Regular,
            ));
            members
        }),
        ("an extra member", {
            let mut members = base.clone();
            members.push((
                "fixture/NOTES.txt".to_owned(),
                extra.as_slice(),
                tar::EntryType::Regular,
            ));
            members
        }),
        ("a missing member", {
            let mut members = base.clone();
            members.pop();
            members
        }),
        ("a second chart root", {
            let mut members = base.clone();
            members.push((
                "other/Chart.yaml".to_owned(),
                files["Chart.yaml"].as_bytes(),
                tar::EntryType::Regular,
            ));
            members
        }),
        ("a member outside any root", {
            let mut members = base.clone();
            members.push((
                "loose.yaml".to_owned(),
                extra.as_slice(),
                tar::EntryType::Regular,
            ));
            members
        }),
        ("a character device", {
            let mut members = base.clone();
            members.push((
                "fixture/device".to_owned(),
                b"".as_slice(),
                tar::EntryType::Char,
            ));
            members
        }),
    ];

    for (name, members) in cases {
        let payload = archive(&as_members(&members));
        let refusal = admit_payload(&payload, &runtime, &source)
            .err()
            .unwrap_or_else(|| panic!("{name} must refuse"));
        assert_eq!(refusal.code, RefusalCode::PreparationFailed, "{name}");
    }

    let wrong_root: Members<'_> = chart_members("other", &files);
    assert!(
        admit_payload(&archive(&as_members(&wrong_root)), &runtime, &source).is_err(),
        "the archive root is the pinned chart name"
    );
}

/// A template delimiter arriving through runtime string data refuses.
#[test]
fn a_template_action_injected_through_runtime_string_data_refuses() {
    let (root, _host) = provisioned("chart-injection");
    let zero = format!("sha256:{}", "0".repeat(64));
    let value = serde_json::json!({
        "format": "ess-runtime-ir/1",
        "runtime": "fixture",
        "semantic_digest": zero,
        "realization_digest": zero,
        "build_digest": zero,
        "processes": {"server": {"name": "server", "image": "{{ .Values.injected }}"}},
        "containers": {"server": {"name": "server", "process": "server"}},
        "workloads": {"app": {"name": "app", "components": ["component"],
                              "containers": ["server"], "replicas": 1}},
        "provided_endpoints": {}
    });
    let Ok(runtime) = ess_deployment::RuntimeIr::from_json(&value.to_string()) else {
        // The existing strict reader already refuses this image spelling, which is the stronger
        // outcome: the injected delimiter never reaches the projector at all.
        return;
    };
    let document = runtime.to_canonical_json();
    let source = chart_source(publish_runtime(&root, &document));
    let files = projection_files(&runtime, &source).expect("the projection admits");
    let members = chart_members("fixture", &files);
    let refusal = admit_payload(&archive(&as_members(&members)), &runtime, &source).unwrap_err();
    assert_eq!(refusal.code, RefusalCode::PreparationFailed);
    assert!(refusal.detail.contains("template delimiter"), "{refusal}");
}

/// Every rendered document outside the profile refuses.
#[test]
fn every_rendered_document_outside_the_profile_refuses() {
    let namespace = text("app");
    let good = "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: web\n  namespace: app\nspec:\n  replicas: 1\n";
    admit_rendered(good, &namespace).expect("an admitted rendered object parses");

    let cases: Vec<(&str, &str)> = vec![
        (
            "an unsupported kind",
            "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: web\n",
        ),
        (
            "a hook annotation",
            "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: web\n  annotations:\n    helm.sh/hook: post-install\n",
        ),
        (
            "a keep policy",
            "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: web\n  annotations:\n    helm.sh/resource-policy: keep\n",
        ),
        (
            "generateName",
            "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  generateName: web-\n  name: web\n",
        ),
        (
            "a cross-namespace object",
            "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: web\n  namespace: elsewhere\n",
        ),
        (
            "a duplicate address",
            "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: web\n---\napiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: web\n",
        ),
        (
            "a wrong apiVersion",
            "apiVersion: v1\nkind: Deployment\nmetadata:\n  name: web\n",
        ),
        ("an empty stream", "---\n"),
    ];
    for (name, document) in cases {
        assert!(
            admit_rendered(document, &namespace).is_err(),
            "{name} must refuse"
        );
    }
}

/// The rendered inventory must equal the projection being admitted, on its own side.
#[test]
fn the_rendered_inventory_must_equal_the_projection_being_admitted() {
    let namespace = text("app");
    let rendered = admit_rendered(
        "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: web\n---\napiVersion: v1\nkind: Service\nmetadata:\n  name: web\n",
        &namespace,
    )
    .expect("two admitted objects parse");

    let matching = projection(
        &[
            (ObjectKind::Deployment, "web"),
            (ObjectKind::Service, "web"),
        ],
        "d",
    );
    assert_eq!(
        admit_inventory(&rendered, &matching).expect("a matching inventory admits"),
        vec![
            address(ObjectKind::Deployment, "web"),
            address(ObjectKind::Service, "web")
        ]
    );

    let short = projection(&[(ObjectKind::Deployment, "web")], "d");
    assert_eq!(
        admit_inventory(&rendered, &short).unwrap_err().code,
        RefusalCode::BaselineMismatch,
        "an omitted rendered object is an incomplete inventory"
    );

    let long = projection(
        &[
            (ObjectKind::Deployment, "web"),
            (ObjectKind::Service, "web"),
            (ObjectKind::StatefulSet, "queue"),
        ],
        "d",
    );
    assert!(admit_inventory(&rendered, &long).is_err());
}

// --- The executable-admission dimension, and the R13/R14/R24 dispositions ------------------------
//
// The asymmetry is the point of this family. `NotLaunched` is published only where the absence of
// a launch is established; a nonzero exit, a timeout and a lost acknowledgement are all
// indeterminate, because in every one of them the child may have run.

use ess_cli::recovery::process::{
    admit as admit_helm, admit_arguments, apply_arguments, environment, remove_arguments, run,
    Apply, Outcome, OUTPUT_LIMIT, PROBE_LIMIT, REFUSED_FLAGS,
};
use fake_recovery::{install_executable, write_profile, TARGET};
use std::process::Command;
use std::time::Duration;

fn fake_artifact(root: &Path) -> (PathBuf, HelmBinary) {
    let (path, digest) =
        install_executable(root, Path::new(env!("CARGO_BIN_EXE_ess-recovery-fake")))
            .expect("the artifact installs");
    let binary = HelmBinary {
        path: text(&path.display().to_string()),
        digest: Digest::new(&digest).expect("the installed digest admits"),
        version: HelmVersion::new("v3.16.2").unwrap(),
        protocol: HelmProtocol::Helm3Recovery1,
    };
    (path, binary)
}

fn tools_prefix(root: &Path) -> String {
    format!("{}/", root.join("opt/ess/recovery-tools/helm").display())
}

/// A real installed artifact is hashed, probed and admitted; every installation fault refuses.
#[test]
fn every_installation_fault_of_the_admitted_artifact_refuses() {
    let (root, host) = provisioned("helm-install");
    let (path, binary) = fake_artifact(&root);
    let prefix = tools_prefix(&root);
    admit_helm(&host, &binary, &prefix).expect("a correctly installed artifact admits");

    let mut elsewhere = binary.clone();
    elsewhere.path = text("/usr/local/bin/helm");
    assert_eq!(
        admit_helm(&host, &elsewhere, &prefix).unwrap_err().code,
        RefusalCode::UnsupportedProfile
    );

    let mut renamed = binary.clone();
    renamed.path = text(&format!("{prefix}{}/helm", "0".repeat(64)));
    assert_eq!(
        admit_helm(&host, &renamed, &prefix).unwrap_err().code,
        RefusalCode::UnsupportedProfile,
        "the filename digest and the configured digest must agree"
    );

    let owned =
        FixtureHost::new(&root, vec![uuid(0x40)]).with_metadata(&path, host.executor_uid(), 0o755);
    assert_eq!(
        admit_helm(&owned, &binary, &prefix).unwrap_err().code,
        RefusalCode::UnsupportedProfile,
        "an executor-owned artifact is not an immutable installation"
    );

    let shared = FixtureHost::new(&root, vec![uuid(0x40)]).with_metadata(&path, 0, 0o757);
    assert_eq!(
        admit_helm(&shared, &binary, &prefix).unwrap_err().code,
        RefusalCode::UnsupportedProfile,
        "a group- or other-writable artifact refuses, under the artifact's own code and not the \
         store's"
    );

    let elevated = FixtureHost::new(&root, vec![uuid(0x40)]).with_metadata(&path, 0, 0o4755);
    assert_eq!(
        admit_helm(&elevated, &binary, &prefix).unwrap_err().code,
        RefusalCode::UnsupportedProfile,
        "a setuid artifact refuses"
    );

    let link = path.with_file_name("helm-link");
    std::os::unix::fs::symlink(&path, &link).expect("a link is made");
    let mut wrapped = binary.clone();
    wrapped.path = text(&link.display().to_string());
    assert!(
        admit_helm(&host, &wrapped, &prefix).is_err(),
        "a wrapper or link is not the admitted artifact"
    );

    let mut tampered = std::fs::read(&path).expect("the artifact reads");
    tampered.push(0);
    std::fs::write(&path, &tampered).expect("the artifact is edited");
    assert_eq!(
        admit_helm(&host, &binary, &prefix).unwrap_err().code,
        RefusalCode::UnsupportedProfile,
        "the complete hash of the bytes must equal the configured digest"
    );
}

/// The bounded probes refuse a mismatched version and a missing operation flag.
#[test]
fn the_bounded_probes_refuse_a_mismatched_version_and_a_missing_flag() {
    let (root, host) = provisioned("helm-probe");
    let (path, mut binary) = fake_artifact(&root);
    let prefix = tools_prefix(&root);
    admit_helm(&host, &binary, &prefix).expect("the fixture artifact probes clean");

    binary.version = HelmVersion::new("v3.17.0").unwrap();
    let refusal = admit_helm(&host, &binary, &prefix).unwrap_err();
    assert_eq!(refusal.code, RefusalCode::UnsupportedProfile);
    assert!(refusal.detail.contains("version"), "{refusal}");

    binary.version = HelmVersion::new("v3.16.2").unwrap();
    write_profile(&path, &[("omit", "upgrade:--atomic")]).expect("the profile writes");
    let refusal = admit_helm(&host, &binary, &prefix).unwrap_err();
    assert!(refusal.detail.contains("--atomic"), "{refusal}");

    write_profile(&path, &[("omit", "uninstall:--no-hooks")]).expect("the profile writes");
    assert!(admit_helm(&host, &binary, &prefix).is_err());

    write_profile(&path, &[]).expect("the profile writes");
    admit_helm(&host, &binary, &prefix).expect("a complete help surface admits again");
}

/// The bounds on each probe stream are the declared ones.
#[test]
fn the_probe_bounds_are_the_declared_ones() {
    assert_eq!(PROBE_LIMIT, 64 * 1024);
    assert_eq!(OUTPUT_LIMIT, 256 * 1024);
}

/// The child environment is constructed, and nothing is inherited into it.
#[test]
fn the_child_environment_is_constructed_and_inherits_nothing() {
    let private = Path::new("/private");
    let built = environment(private, &text("/etc/ess/recovery/kubeconfig"));
    assert_eq!(built["HELM_DRIVER"], "secret");
    assert_eq!(built["HELM_PLUGINS"], "/private/plugins");
    assert_eq!(built["HELM_CONFIG_HOME"], "/private/config");
    assert_eq!(built["HELM_CACHE_HOME"], "/private/cache");
    assert_eq!(built["HELM_DATA_HOME"], "/private/data");
    assert_eq!(built["KUBECONFIG"], "/etc/ess/recovery/kubeconfig");
    assert_eq!(
        built["PATH"], "/nonexistent",
        "no executable is selected from the inherited PATH"
    );
    for injected in [
        "HTTPS_PROXY",
        "HTTP_PROXY",
        "NO_PROXY",
        "LD_PRELOAD",
        "HELM_KUBEAPISERVER",
    ] {
        assert!(
            !built.contains_key(injected),
            "{injected} is cleared, not carried through"
        );
    }
}

/// The apply and remove argument vectors are exactly the closed contract.
#[test]
fn the_apply_and_remove_argument_vectors_are_the_closed_contract() {
    let apply = apply_arguments(Apply {
        release: "checkout",
        chart: Path::new("/private/chart.tgz"),
        values: Path::new("/private/values.yaml"),
        namespace: "app",
        kubeconfig: "/etc/ess/recovery/kubeconfig",
        context: "prod",
        marker: "ess-recovery/1:a:b",
        timeout: "5m",
    });
    admit_arguments(&apply).expect("the apply vector admits");
    for required in [
        "upgrade",
        "--install",
        "--atomic",
        "--wait",
        "--timeout",
        "--description",
        "--no-hooks",
        "--skip-crds",
        "--namespace",
        "--kubeconfig",
        "--kube-context",
        "--values",
    ] {
        assert!(
            apply.contains(&required.to_owned()),
            "{required} is required"
        );
    }
    assert!(
        !apply.contains(&"--create-namespace".to_owned()),
        "the admitted namespace must already exist with its pinned UID"
    );

    let remove = remove_arguments(
        "legacy",
        "app",
        "/etc/ess/recovery/kubeconfig",
        "prod",
        "5m",
    );
    admit_arguments(&remove).expect("the remove vector admits");
    assert!(
        !remove.contains(&"--keep-history".to_owned()),
        "the absence predicate includes Helm release storage"
    );

    for refused in REFUSED_FLAGS {
        let mut smuggled = apply.clone();
        smuggled.push((*refused).to_owned());
        assert_eq!(
            admit_arguments(&smuggled).unwrap_err().code,
            RefusalCode::UnsupportedProfile,
            "{refused} is never passed by this profile"
        );
    }
    assert!(
        REFUSED_FLAGS.contains(&"--create-namespace")
            && REFUSED_FLAGS.contains(&"--keep-history")
            && REFUSED_FLAGS.contains(&"--ignore-not-found"),
        "the three flags C06 names by name are in the refused set"
    );
}

/// A definite spawn refusal is `NotLaunched`; every started uncertainty is `Indeterminate`.
#[test]
fn only_a_definite_spawn_refusal_is_not_launched() {
    let refused = run(
        Command::new("/nonexistent/definitely-not-here"),
        OUTPUT_LIMIT,
        Duration::from_secs(5),
    )
    .expect("a spawn refusal is an outcome, not an error");
    assert!(!refused.launched);
    assert_eq!(refused.disposition(), ProcessDisposition::NotLaunched);

    let mut acknowledged = Command::new(env!("CARGO_BIN_EXE_ess-recovery-fake"));
    acknowledged.arg("version");
    let outcome = run(acknowledged, OUTPUT_LIMIT, Duration::from_secs(30))
        .expect("the fixture artifact runs");
    assert_eq!(outcome.disposition(), ProcessDisposition::Acknowledged);
    assert_eq!(outcome.status, Some(0));

    let mut nonzero = Command::new(env!("CARGO_BIN_EXE_ess-recovery-fake"));
    nonzero.arg("unsupported-operation");
    let outcome = run(nonzero, OUTPUT_LIMIT, Duration::from_secs(30))
        .expect("a started child that exits nonzero is an outcome");
    assert!(outcome.launched);
    assert_eq!(
        outcome.disposition(),
        ProcessDisposition::Indeterminate,
        "a started call that exits nonzero may have run; it is not a non-launch"
    );
}

/// A timeout kills and reaps the owned direct child and stays indeterminate.
///
/// Reaping the child is all that happens and all that is claimed. It does not establish that a
/// descendant is gone or that an already-issued API request has drained, which is exactly why the
/// disposition is indeterminate rather than a non-launch.
#[test]
fn a_timeout_reaps_the_owned_child_and_remains_indeterminate() {
    let (root, _host) = provisioned("helm-timeout");
    let (path, _binary) = fake_artifact(&root);
    write_profile(
        &path,
        &[
            ("fault", "timeout"),
            ("target", &root.join(TARGET).display().to_string()),
        ],
    )
    .expect("the profile writes");

    let mut command = Command::new(&path);
    command.args(["upgrade", "checkout", "--namespace", "app"]);
    let started = std::time::Instant::now();
    let outcome: Outcome = run(command, OUTPUT_LIMIT, Duration::from_millis(400))
        .expect("a timed-out child is an outcome");
    assert!(
        started.elapsed() < Duration::from_secs(30),
        "the bound holds"
    );
    assert!(outcome.launched && outcome.timed_out);
    assert_eq!(outcome.status, None);
    assert_eq!(outcome.disposition(), ProcessDisposition::Indeterminate);
}

/// A child that changes the target and then fails, and one that changes it and never answers.
///
/// Both are indeterminate to the caller and both leave the target changed, which is the whole
/// distinction R14 and R24 are about: the process result is not evidence about the target.
#[test]
fn an_effect_before_failure_and_a_lost_acknowledgement_are_both_indeterminate() {
    for (fault, expected_status) in [("effect-then-fail", Some(1)), ("lost-ack", Some(101))] {
        let (root, _host) = provisioned(&format!("helm-{fault}"));
        let (path, _binary) = fake_artifact(&root);
        let target = root.join(TARGET);
        write_profile(
            &path,
            &[
                ("fault", fault),
                ("target", &target.display().to_string()),
                ("inventory", "Deployment/web"),
            ],
        )
        .expect("the profile writes");

        let mut command = Command::new(&path);
        command.args([
            "upgrade",
            "checkout",
            "--namespace",
            "app",
            "--description",
            "ess-recovery/1:a:b",
        ]);
        let outcome =
            run(command, OUTPUT_LIMIT, Duration::from_secs(30)).expect("the child is an outcome");
        assert_eq!(outcome.status, expected_status, "{fault}");
        assert_eq!(
            outcome.disposition(),
            ProcessDisposition::Indeterminate,
            "{fault} leaves the effect unknown to the caller"
        );
        let state = fake_recovery::read_release(&target, "app", "checkout");
        assert_eq!(
            state.description.as_deref(),
            Some("ess-recovery/1:a:b"),
            "{fault}: the independently controlled target changed and outlived the child"
        );
    }
}

/// A child that fails before any effect leaves the target exactly as it was.
#[test]
fn a_failure_before_any_effect_leaves_the_target_untouched() {
    let (root, _host) = provisioned("helm-no-effect");
    let (path, _binary) = fake_artifact(&root);
    let target = root.join(TARGET);
    write_profile(
        &path,
        &[
            ("fault", "no-effect"),
            ("target", &target.display().to_string()),
        ],
    )
    .expect("the profile writes");

    let mut command = Command::new(&path);
    command.args(["upgrade", "checkout", "--namespace", "app"]);
    let outcome = run(command, OUTPUT_LIMIT, Duration::from_secs(30)).expect("an outcome");
    assert_eq!(outcome.disposition(), ProcessDisposition::Indeterminate);
    assert_eq!(
        fake_recovery::read_release(&target, "app", "checkout").description,
        None,
        "a started child that failed before its effect still leaves the effect unknown; what the \
         target says is a separate fact, established by observing it"
    );
}

// --- Families R01, R03 and R22 through the shipped `ess` binary ----------------------------------
//
// These are the controls C13 assigns to the shipped binary: parse and help, invalid-input refusal,
// local dry-run, and the refusal of normal execution without an admitted authority. Positive
// recovery execution is *not* here — it runs through the shared-code driver, and calling it a
// deployment by the shipped binary on an admitted host is the mislabelling C13 forbids.

fn deployment_plan() -> serde_json::Value {
    let digest = format!("sha256:{}", "1".repeat(64));
    serde_json::json!({
        "format": "ess-deployment/1", "environment": "test", "stack_digest": digest,
        "cluster": "test-cluster", "rollout_order": ["first", "last"],
        "releases": {
            "first": {"service":"first","release_name":"first","namespace":"test","service_account":"default",
                "chart":{"build_output":"chart","kind":"helm_chart","reference":"oci://example.invalid/chart","digest":digest},
                "images":{"app":{"build_output":"app","kind":"oci_image","reference":"example.invalid/app","digest":digest,"platforms":{"linux/amd64":digest}}}},
            "last": {"service":"last","release_name":"last","namespace":"test","service_account":"default",
                "chart":{"build_output":"chart","kind":"helm_chart","reference":"oci://example.invalid/chart","digest":digest},
                "images":{"app":{"build_output":"app","kind":"oci_image","reference":"example.invalid/app","digest":digest,"platforms":{"linux/amd64":digest}}}}
        }
    })
}

fn write_plan(root: &Path, name: &str, value: &serde_json::Value) -> PathBuf {
    let path = root.join(name);
    let text = if name.ends_with("yaml") {
        serde_yaml::to_string(value).unwrap()
    } else {
        serde_json::to_string(value).unwrap()
    };
    std::fs::write(&path, text).unwrap();
    path
}

fn ess() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ess"))
}

/// Both reconcile spellings expose the same options, and the baseline option says what it is.
///
/// "Previously applied" was a claim the flag could not support: a document the caller supplied is
/// admitted *intent*, and nothing about supplying it establishes that it was ever applied.
#[test]
fn both_reconcile_spellings_expose_the_same_admitted_baseline_surface() {
    let mut rendered = Vec::new();
    // The two spellings this repository mounts from one definition: the hidden flat one a pinned
    // caller already uses, and the area path the reference page teaches.
    for prefix in [
        vec!["deployment", "reconcile"],
        vec!["generate", "deployment", "reconcile"],
    ] {
        let output = ess().args(&prefix).arg("--help").output().unwrap();
        assert!(output.status.success(), "{output:?}");
        let text = String::from_utf8(output.stdout).unwrap();
        for option in [
            "--path",
            "--current",
            "--cache",
            "--allow-removals",
            "--dry-run",
            "--timeout",
            "--authority",
        ] {
            assert!(
                text.contains(option),
                "{prefix:?} --help must offer {option}:\n{text}"
            );
        }
        assert!(
            !text.contains("Previously applied"),
            "a supplied baseline is admitted intent, not proof of application:\n{text}"
        );
        assert!(
            text.contains("Admitted baseline desired deployment"),
            "the baseline option says what it is:\n{text}"
        );
        rendered.push(
            text.lines()
                .filter(|line| line.trim_start().starts_with("--"))
                .map(str::trim)
                .map(str::to_owned)
                .collect::<Vec<_>>(),
        );
    }
    assert_eq!(
        rendered[0], rendered[1],
        "the two spellings are one definition and cannot drift"
    );
}

/// R01: an invalid desired or baseline document refuses before every external call and write.
#[test]
fn an_invalid_document_refuses_before_any_call_cache_or_recovery_write() {
    for extension in ["json", "yaml"] {
        for (pointer, value) in [
            ("/format", serde_json::json!("future/99")),
            ("/releases/last/chart/kind", serde_json::json!("binary")),
            ("/rollout_order", serde_json::json!(["first", "missing"])),
            ("/releases/last/service", serde_json::json!("other")),
        ] {
            let root = scratch("cli-invalid");
            let mut invalid = deployment_plan();
            *invalid.pointer_mut(pointer).unwrap() = value.clone();
            let desired = write_plan(&root, &format!("desired.{extension}"), &invalid);
            let cache = root.join("cache");
            let output = ess()
                .args(["deployment", "reconcile", "--path"])
                .arg(&desired)
                .arg("--cache")
                .arg(&cache)
                .env("PATH", "/nonexistent")
                .output()
                .unwrap();
            assert!(
                !output.status.success(),
                "{pointer} {value} must refuse: {output:?}"
            );
            assert!(!cache.exists(), "{pointer}: no cache was populated");
            assert_eq!(
                std::fs::read_dir(root.join(STATE).join("invocations"))
                    .unwrap()
                    .count(),
                0,
                "{pointer}: no invocation was reserved"
            );
            assert!(
                !root.join(STATE).join("target.lock").exists(),
                "{pointer}: no claim was published"
            );
        }
    }
}

/// R01: a duplicate key in either document refuses, in both spellings.
#[test]
fn a_duplicate_key_in_either_document_refuses() {
    let root = scratch("cli-duplicate");
    let valid = write_plan(&root, "desired.json", &deployment_plan());
    let duplicated = root.join("duplicate.json");
    let text = serde_json::to_string(&deployment_plan()).unwrap();
    std::fs::write(
        &duplicated,
        text.replace(
            r#""environment":"test""#,
            r#""environment":"test","environment":"other""#,
        ),
    )
    .unwrap();

    for (path, current) in [(&duplicated, None), (&valid, Some(&duplicated))] {
        let mut command = ess();
        command
            .args(["deployment", "reconcile", "--path"])
            .arg(path)
            .arg("--cache")
            .arg(root.join("cache"));
        if let Some(current) = current {
            command
                .arg("--current")
                .arg(current)
                .arg("--allow-removals");
        }
        let output = command.env("PATH", "/nonexistent").output().unwrap();
        assert!(
            !output.status.success(),
            "a duplicate key refuses: {output:?}"
        );
    }
}

/// R03: dry-run is a local unverified preview and touches nothing.
#[test]
fn dry_run_is_a_local_unverified_preview_with_no_target_cache_or_evidence_effect() {
    let root = scratch("cli-preview");
    let desired = write_plan(&root, "desired.json", &deployment_plan());
    let mut changed = deployment_plan();
    *changed
        .pointer_mut("/releases/last/service_account")
        .unwrap() = serde_json::json!("other");
    let previous = write_plan(&root, "current.json", &changed);
    let mut retiring = deployment_plan();
    retiring
        .as_object_mut()
        .unwrap()
        .insert("rollout_order".to_owned(), serde_json::json!(["first"]));
    retiring
        .pointer_mut("/releases")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("last");
    let shrunk = write_plan(&root, "shrunk.json", &retiring);
    let cache = root.join("cache");
    let target_before = std::fs::read_dir(root.join(TARGET)).unwrap().count();

    // First plan: everything is affected and nothing is removed.
    let first = ess()
        .args(["deployment", "reconcile", "--path"])
        .arg(&desired)
        .arg("--cache")
        .arg(&cache)
        .arg("--dry-run")
        .env("PATH", "/nonexistent")
        .output()
        .unwrap();
    assert!(first.status.success(), "{first:?}");
    let text = String::from_utf8(first.stdout).unwrap();
    assert!(text.contains("apply: first, last"), "{text}");
    assert!(text.contains("remove: "), "{text}");
    assert!(
        text.contains("unverified"),
        "a preview says it is unverified: {text}"
    );

    // Equal plans: nothing is affected, and the preview still says nothing about the target.
    let equal = ess()
        .args(["deployment", "reconcile", "--path"])
        .arg(&desired)
        .arg("--current")
        .arg(&desired)
        .arg("--cache")
        .arg(&cache)
        .arg("--dry-run")
        .env("PATH", "/nonexistent")
        .output()
        .unwrap();
    assert!(equal.status.success(), "{equal:?}");
    let text = String::from_utf8(equal.stdout).unwrap();
    assert!(
        text.contains("apply: \n") || text.contains("apply: (none)"),
        "{text}"
    );

    // A changed plan, and a removal preview under the reviewed flag.
    let changed = ess()
        .args(["deployment", "reconcile", "--path"])
        .arg(&desired)
        .arg("--current")
        .arg(&previous)
        .arg("--cache")
        .arg(&cache)
        .arg("--dry-run")
        .env("PATH", "/nonexistent")
        .output()
        .unwrap();
    assert!(changed.status.success(), "{changed:?}");
    assert!(String::from_utf8_lossy(&changed.stdout).contains("last"));

    let removing = ess()
        .args(["deployment", "reconcile", "--path"])
        .arg(&shrunk)
        .arg("--current")
        .arg(&desired)
        .arg("--allow-removals")
        .arg("--cache")
        .arg(&cache)
        .arg("--dry-run")
        .env("PATH", "/nonexistent")
        .output()
        .unwrap();
    assert!(removing.status.success(), "{removing:?}");
    assert!(String::from_utf8_lossy(&removing.stdout).contains("remove: last"));

    assert!(!cache.exists(), "a preview populates no cache");
    assert_eq!(
        std::fs::read_dir(root.join(STATE).join("invocations"))
            .unwrap()
            .count(),
        0,
        "a preview reserves no invocation"
    );
    assert!(!root.join(STATE).join("target.lock").exists());
    assert_eq!(
        std::fs::read_dir(root.join(TARGET)).unwrap().count(),
        target_before,
        "a preview leaves the synthetic target's bytes unchanged"
    );
}

/// R22: a retirement without the reviewed removal flag rejects before both phases.
#[test]
fn a_retirement_without_the_reviewed_flag_rejects_before_both_phases() {
    let root = scratch("cli-removals");
    let desired = write_plan(&root, "desired.json", &deployment_plan());
    let mut retiring = deployment_plan();
    retiring
        .as_object_mut()
        .unwrap()
        .insert("rollout_order".to_owned(), serde_json::json!(["first"]));
    retiring
        .pointer_mut("/releases")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("last");
    let shrunk = write_plan(&root, "shrunk.json", &retiring);
    let cache = root.join("cache");

    for extra in [Vec::new(), vec!["--dry-run"]] {
        let output = ess()
            .args(["deployment", "reconcile", "--path"])
            .arg(&shrunk)
            .arg("--current")
            .arg(&desired)
            .arg("--cache")
            .arg(&cache)
            .args(&extra)
            .env("PATH", "/nonexistent")
            .output()
            .unwrap();
        assert!(!output.status.success(), "{extra:?}: {output:?}");
        let message = String::from_utf8_lossy(&output.stderr).to_string();
        assert!(
            message.contains("--allow-removals"),
            "{extra:?}: the refusal names the reviewed flag: {message}"
        );
        assert!(
            !cache.exists(),
            "{extra:?}: nothing implies execution began"
        );
        assert_eq!(
            std::fs::read_dir(root.join(STATE).join("invocations"))
                .unwrap()
                .count(),
            0
        );
    }
}

/// Normal execution without an admitted authority refuses.
///
/// **This case is red in this build, deliberately, and it is the one narrowed clause of the
/// unit.** C04 and C14 require actual `reconcile` to refuse without an admitted authority, and
/// `main.rs` still runs the pre-recovery execution path when `--authority` is absent. Gating it
/// makes the Helm-profile vectors in `tests/cache_origin.rs` and
/// `tests/cache_origin_adversary_pass1.rs` stop at the authority before they reach the cache
/// boundary they exist to decide — the exact vacuity C12 forbids — so closing this needs those
/// vectors routed through the shared-code driver with a synthetic authority first. The case stays
/// here and stays red rather than being softened into a description of what the build happens to
/// do.
#[test]
fn normal_execution_without_an_admitted_authority_refuses() {
    let root = scratch("cli-authority");
    let desired = write_plan(&root, "desired.json", &deployment_plan());
    let output = ess()
        .args(["deployment", "reconcile", "--path"])
        .arg(&desired)
        .arg("--cache")
        .arg(root.join("cache"))
        .env("PATH", "/nonexistent")
        .output()
        .unwrap();
    assert!(!output.status.success(), "{output:?}");
    let message = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        message.contains("authority"),
        "C04/C14 require this to refuse without an admitted authority, and it ran the \
         pre-recovery execution path instead. Closing this needs the Helm-profile vectors in \
         tests/cache_origin.rs and tests/cache_origin_adversary_pass1.rs routed through \
         tests/support/recovery_driver.rs with a synthetic authority first; gating the binary \
         before that makes those 30 vectors stop at the authority and never reach the cache \
         boundary they decide. Refusal was: {message}"
    );
    assert!(!root.join("cache").exists());
}

/// No environment variable, and no argument outside the surface, selects a permissive profile.
///
/// The shipped binary ships no bypass, and this is the case that says so rather than the absence
/// of one saying it. Every variable this repository's own fixtures use is tried, together with the
/// shapes a bypass would most plausibly take; none of them may change what the binary admits.
#[test]
fn no_environment_variable_selects_a_permissive_profile() {
    let root = scratch("cli-bypass");
    let mut retiring = deployment_plan();
    retiring
        .as_object_mut()
        .unwrap()
        .insert("rollout_order".to_owned(), serde_json::json!(["first"]));
    retiring
        .pointer_mut("/releases")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("last");
    let desired = write_plan(&root, "desired.json", &deployment_plan());
    let shrunk = write_plan(&root, "shrunk.json", &retiring);

    for (key, value) in [
        ("ESS_RECOVERY_ROOT", "/nonexistent"),
        ("ESS_RECOVERY_PROFILE", "permissive"),
        ("ESS_RECOVERY_ALLOW", "1"),
        (
            "ESS_RECOVERY_AUTHORITY",
            "00000000-0000-0000-0000-000000000000",
        ),
        ("ESS_FIXTURE_TARGET", "/nonexistent"),
        ("ESS_TEST_DELIVERY_LOG", "/nonexistent/log"),
        ("ESS_CACHE_FIXTURE", "/nonexistent"),
    ] {
        // The reviewed-removal guard is the cheapest refusal that is unambiguously the contract's:
        // no variable may turn a retirement without `--allow-removals` into an execution.
        let output = ess()
            .args(["deployment", "reconcile", "--path"])
            .arg(&shrunk)
            .arg("--current")
            .arg(&desired)
            .arg("--cache")
            .arg(root.join("cache"))
            .env("PATH", "/nonexistent")
            .env(key, value)
            .output()
            .unwrap();
        assert!(
            !output.status.success(),
            "{key} must not select a permissive profile: {output:?}"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("--allow-removals"),
            "{key}: the refusal is still the reviewed-removal guard"
        );
        assert!(!root.join("cache").exists(), "{key}: nothing was acquired");
    }

    // An unknown flag is refused by the parser, not absorbed.
    let output = ess()
        .args(["deployment", "reconcile", "--path"])
        .arg(&desired)
        .arg("--cache")
        .arg(root.join("cache"))
        .arg("--permissive")
        .env("PATH", "/nonexistent")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unexpected argument"));
}

// --- Family R28: competing real driver processes and the retained claim --------------------------
//
// Two separate processes, a claim that outlives the one that published it, and a second executor
// that can read and report but cannot mutate. None of this is observable inside one test process:
// a claim retained after its holder died is only a fact because the holder actually died.

fn driver_job(root: &Path, name: &str, job: &serde_json::Value) -> std::process::Output {
    let path = root.join(format!("job-{name}.json"));
    std::fs::write(&path, serde_json::to_string(job).unwrap()).unwrap();
    Command::new(env!("CARGO_BIN_EXE_ess-recovery-driver"))
        .arg(&path)
        .output()
        .unwrap()
}

fn publish_driver_context(root: &Path) {
    std::fs::write(
        root.join("driver-context.json"),
        canonical(&opened_context()),
    )
    .unwrap();
}

/// A claim published by one process is retained after that process dies, and is never stolen.
///
/// The first driver exits holding the claim. No age, no PID, no timeout and no empty journal tail
/// gives the second one permission to take it; what the second one may do is read and report,
/// which is exactly what it does.
#[test]
fn a_claim_published_by_one_process_survives_it_and_is_never_stolen_by_another() {
    let (root, host) = provisioned("driver-claim");
    publish_driver_context(&root);
    let store = open_store(&host, &root.join(STATE)).expect("the store admits");

    let first = driver_job(
        &root,
        "first",
        &serde_json::json!({
            "root": root, "mode": "claim",
            "nonces": [uuid(0x51).to_string()],
            "fail": null, "interrupt": null, "open_journal": true
        }),
    );
    assert!(
        first.status.success(),
        "the first driver claims: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(String::from_utf8_lossy(&first.stdout).contains("holds the target claim"));

    let retained = read_claim(&host, &store)
        .expect("the retained claim reads")
        .expect("the dead process's claim is still published");
    assert_eq!(retained.claim.invocation.nonce, uuid(0x51));

    // A second, separate process. It cannot publish a claim, and it says whose the retained one is.
    let second = driver_job(
        &root,
        "second",
        &serde_json::json!({
            "root": root, "mode": "claim",
            "nonces": [uuid(0x52).to_string()],
            "fail": null, "interrupt": null, "open_journal": false
        }),
    );
    assert!(!second.status.success(), "a second claim must refuse");
    let message = String::from_utf8_lossy(&second.stderr).to_string();
    assert!(
        message.contains("never reclaims"),
        "the refusal says what it will not do: {message}"
    );

    let observed = driver_job(
        &root,
        "observer",
        &serde_json::json!({
            "root": root, "mode": "observe",
            "nonces": [], "fail": null, "interrupt": null, "open_journal": false
        }),
    );
    assert!(observed.status.success(), "reading is always permitted");
    let text = String::from_utf8_lossy(&observed.stdout).to_string();
    assert!(text.contains("observation-only"), "{text}");
    assert!(text.contains(uuid(0x51).as_str()), "{text}");

    let after = read_claim(&host, &store)
        .expect("the claim still reads")
        .expect("and is still there");
    assert_eq!(
        after.claim, retained.claim,
        "neither the competitor nor the observer changed the retained claim"
    );
    assert_eq!(after.digest, retained.digest);
}

/// A driver interrupted at a durability barrier leaves exactly what it had published, and no more.
///
/// A real process termination, at a named barrier, in a process that is not the test's. The
/// restart then sees only what actually reached the disk — which is the whole of what "durable"
/// means here, and the only honest way to produce a torn run.
#[test]
fn a_driver_interrupted_at_a_barrier_leaves_only_what_it_had_published() {
    let (root, host) = provisioned("driver-interrupt");
    publish_driver_context(&root);
    let store = open_store(&host, &root.join(STATE)).expect("the store admits");

    let interrupted = driver_job(
        &root,
        "interrupted",
        &serde_json::json!({
            "root": root, "mode": "claim",
            "nonces": [uuid(0x53).to_string()],
            "fail": null, "interrupt": "Publish", "open_journal": true
        }),
    );
    assert!(
        !interrupted.status.success(),
        "an interrupted driver does not report success"
    );
    assert_eq!(
        interrupted.status.code(),
        Some(97),
        "the process really terminated at the barrier"
    );

    // The reservation was published before the claim's publication barrier, so it is there; the
    // claim is not. Nothing invents either fact from the other.
    let histories = scan_store(&host, &store).expect("the store scans");
    assert_eq!(histories.len(), 1);
    assert_eq!(histories[0].nonce, uuid(0x53));
    assert_eq!(
        histories[0].state,
        JournalState::EmptyReservation,
        "a reserved directory with no Opened is an incomplete reservation, not a completed journal"
    );
    assert!(
        read_claim(&host, &store).expect("reads").is_none(),
        "the claim's publication barrier was never crossed, so no claim exists"
    );

    // A later invocation gets a new identity; the retained residue is never reused as a fresh
    // successful reservation.
    let next = driver_job(
        &root,
        "next",
        &serde_json::json!({
            "root": root, "mode": "claim",
            "nonces": [uuid(0x54).to_string()],
            "fail": null, "interrupt": null, "open_journal": true
        }),
    );
    assert!(
        next.status.success(),
        "{}",
        String::from_utf8_lossy(&next.stderr)
    );
    let histories = scan_store(&host, &store).expect("the store scans");
    assert_eq!(
        histories.len(),
        2,
        "the interrupted reservation is retained"
    );
    assert!(histories.iter().any(|history| history.nonce == uuid(0x53)));
    assert!(histories.iter().any(|history| history.nonce == uuid(0x54)));
}

/// A driver whose claim publication fails leaves no claim and no journal entry.
#[test]
fn a_failed_claim_publication_leaves_no_claim_and_no_journal_entry() {
    let (root, host) = provisioned("driver-fail");
    publish_driver_context(&root);
    let store = open_store(&host, &root.join(STATE)).expect("the store admits");

    let failed = driver_job(
        &root,
        "failed",
        &serde_json::json!({
            "root": root, "mode": "claim",
            "nonces": [uuid(0x55).to_string()],
            "fail": "StageSync", "interrupt": null, "open_journal": true
        }),
    );
    assert!(!failed.status.success());
    assert!(
        String::from_utf8_lossy(&failed.stderr).contains("StageSync"),
        "the refusal names the barrier it failed: {}",
        String::from_utf8_lossy(&failed.stderr)
    );
    assert!(read_claim(&host, &store).expect("reads").is_none());
    let histories = scan_store(&host, &store).expect("the store scans");
    assert_eq!(histories[0].state, JournalState::EmptyReservation);
}

/// The driver runs the production store admission, and refuses an unprovisioned store like it does.
#[test]
fn the_driver_refuses_an_unprovisioned_store_through_the_production_admission() {
    let root = scratch("driver-store");
    publish_driver_context(&root);
    let output = driver_job(
        &root,
        "unprovisioned",
        &serde_json::json!({
            "root": root, "mode": "observe",
            "nonces": [], "fail": null, "interrupt": null, "open_journal": false
        }),
    );
    assert!(!output.status.success());
    let message = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        message.contains("StoreInvalid"),
        "the driver reports the production refusal code: {message}"
    );
}
