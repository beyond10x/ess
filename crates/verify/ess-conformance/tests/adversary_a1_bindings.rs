//! Adversarial cases for the 0.69.0 binding-scenario fixes (defects #1-#3): shapes beside the ones
//! the unit's own tests build.
//!
//! | case | shape |
//! |---|---|
//! | [`TRIGGER_INPUT_GUARDED`] | #1 on a `TriggerInput` destination: the trigger itself guards the destination's stored fields |
//! | [`UNBOUNDED_RETRY`] with a per-refusal policy | #3 where the unbounded retry is selected per refusal |
//! | [`SUBJECT_FIELD_RETRY`] | #3 where the event field the binding maps is a stored field of the trigger's row |
//! | [`SECOND_PUBLISHER`] | #2 where another command publishes the event without refusing the missing row |

use std::collections::BTreeMap;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

/// `RequestPairing` refuses a managed device; the binding pairs the device the request names.
const TRIGGER_INPUT_GUARDED: &str = r"
format: ess/27
system: demo
version: v1
domain: demo.device
types:
  - {name: demo.device.DeviceId, kind: newtype, of: Uuid}
  - {name: demo.device.Label, kind: newtype, of: String}
  - {name: demo.device.Mode, kind: enum, variants: [managed, open]}
entities:
  - name: demo.device.Device
    identity: {name: device_id, type: demo.device.DeviceId}
    fields:
      - {name: label, type: demo.device.Label}
      - {name: mode, type: demo.device.Mode}
      - {name: paired, type: Boolean}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
events:
  - name: demo.device.DeviceRegistered
    fields:
      - {name: device_id, type: demo.device.DeviceId}
  - name: demo.device.DeviceOpened
    fields:
      - {name: device_id, type: demo.device.DeviceId}
  - name: demo.device.PairingRequested
    fields:
      - {name: device_id, type: demo.device.DeviceId}
  - name: demo.device.DevicePaired
    fields:
      - {name: device_id, type: demo.device.DeviceId}
errors:
  - {name: demo.device.NoDevice, fields: []}
  - {name: demo.device.Managed, fields: []}
actors:
  - name: demo.device.Clerk
    may: [demo.device.RegisterDevice, demo.device.OpenDevice, demo.device.RequestPairing, demo.device.PairDevice]
commands:
  - name: demo.device.RegisterDevice
    input:
      - {name: label, type: demo.device.Label}
    outcomes:
      - name: registered
        creates: demo.device.Device
        instance: device_id
        emits: [demo.device.DeviceRegistered]
        payload:
          demo.device.DeviceRegistered: {device_id: {generated: true}}
        sets: {label: input.label, mode: managed, paired: false}
  - name: demo.device.OpenDevice
    input:
      - {name: device_id, type: demo.device.DeviceId}
    outcomes:
      - name: opened
        updates: demo.device.Device
        instance: device_id
        emits: [demo.device.DeviceOpened]
        payload:
          demo.device.DeviceOpened: {device_id: input.device_id}
        sets: {mode: open}
      - {name: no-device, unknown_instance: true, error: demo.device.NoDevice}
  - name: demo.device.RequestPairing
    input:
      - {name: device_id, type: demo.device.DeviceId}
    outcomes:
      - {name: unknown-device, when_related: {via: input.device_id, exists: false}, error: demo.device.NoDevice}
      - {name: managed, when_related: {via: input.device_id, predicate: mode == managed}, error: demo.device.Managed}
      - name: requested
        emits: [demo.device.PairingRequested]
        payload:
          demo.device.PairingRequested: {device_id: input.device_id}
  - name: demo.device.PairDevice
    input:
      - {name: device_id, type: demo.device.DeviceId}
    outcomes:
      - name: paired
        updates: demo.device.Device
        instance: device_id
        emits: [demo.device.DevicePaired]
        payload:
          demo.device.DevicePaired: {device_id: input.device_id}
        sets: {paired: true}
      - {name: no-device, unknown_instance: true, error: demo.device.NoDevice}
views:
  - name: demo.device.Devices
    source: demo.device.Device
    consistency: read_your_writes
    fields:
      - {name: device_id, type: demo.device.DeviceId}
      - {name: mode, type: demo.device.Mode}
      - {name: paired, type: Boolean}
bindings:
  - id: pair-on-requested
    when: {event: demo.device.PairingRequested}
    invoke: {command: demo.device.PairDevice}
    mapping: {device_id: event.device_id}
    delivery: at_least_once
    on_failure: retry
";

/// The unit's own `UNBOUNDED_RETRY` fixture, verbatim.
const UNBOUNDED_RETRY: &str = r"
format: ess/27
system: demo
version: v1
domain: demo.catalog
types:
  - {name: demo.catalog.SectionId, kind: newtype, of: Uuid}
  - {name: demo.catalog.ListingId, kind: newtype, of: Uuid}
  - {name: demo.catalog.Title, kind: newtype, of: String}
entities:
  - name: demo.catalog.Section
    identity: {name: section_id, type: demo.catalog.SectionId}
    fields:
      - {name: title, type: demo.catalog.Title}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.catalog.Listing
    identity: {name: listing_id, type: demo.catalog.ListingId}
    fields:
      - {name: section_id, type: demo.catalog.SectionId}
    lifecycle: {initial: Held, states: [Held], terminal: [Held]}
events:
  - name: demo.catalog.SectionDefined
    fields:
      - {name: section_id, type: demo.catalog.SectionId}
  - name: demo.catalog.SectionAnnounced
    fields:
      - {name: section_id, type: demo.catalog.SectionId}
  - name: demo.catalog.Listed
    fields:
      - {name: listing_id, type: demo.catalog.ListingId}
errors:
  - {name: demo.catalog.NoSection, fields: []}
actors:
  - name: demo.catalog.Clerk
    may: [demo.catalog.DefineSection, demo.catalog.AnnounceSection, demo.catalog.ListForSection]
commands:
  - name: demo.catalog.DefineSection
    input:
      - {name: title, type: demo.catalog.Title}
    outcomes:
      - name: defined
        creates: demo.catalog.Section
        instance: section_id
        emits: [demo.catalog.SectionDefined]
        payload:
          demo.catalog.SectionDefined: {section_id: {generated: true}}
        sets: {title: input.title}
  - name: demo.catalog.AnnounceSection
    input:
      - {name: section_id, type: demo.catalog.SectionId}
    outcomes:
      - name: announced
        emits: [demo.catalog.SectionAnnounced]
        payload:
          demo.catalog.SectionAnnounced: {section_id: input.section_id}
  - name: demo.catalog.ListForSection
    input:
      - {name: section_id, type: demo.catalog.SectionId}
    outcomes:
      - {name: section-missing, when_related: {via: input.section_id, exists: false}, error: demo.catalog.NoSection}
      - name: listed
        creates: demo.catalog.Listing
        instance: listing_id
        emits: [demo.catalog.Listed]
        payload:
          demo.catalog.Listed: {listing_id: {generated: true}}
        sets: {section_id: input.section_id}
views:
  - name: demo.catalog.Listings
    source: demo.catalog.Listing
    consistency: read_your_writes
    fields:
      - {name: listing_id, type: demo.catalog.ListingId}
      - {name: section_id, type: demo.catalog.SectionId}
bindings:
  - id: list-on-announced
    when: {event: demo.catalog.SectionAnnounced}
    invoke: {command: demo.catalog.ListForSection}
    mapping: {section_id: event.section_id}
    delivery: at_least_once
    on_failure: retry
";

/// The section the binding lists for is a stored field of the notice the trigger updates.
const SUBJECT_FIELD_RETRY: &str = r"
format: ess/27
system: demo
version: v1
domain: demo.catalog
types:
  - {name: demo.catalog.SectionId, kind: newtype, of: Uuid}
  - {name: demo.catalog.ListingId, kind: newtype, of: Uuid}
  - {name: demo.catalog.NoticeId, kind: newtype, of: Uuid}
  - {name: demo.catalog.Title, kind: newtype, of: String}
entities:
  - name: demo.catalog.Section
    identity: {name: section_id, type: demo.catalog.SectionId}
    fields:
      - {name: title, type: demo.catalog.Title}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.catalog.Notice
    identity: {name: notice_id, type: demo.catalog.NoticeId}
    fields:
      - {name: section_id, type: demo.catalog.SectionId}
      - {name: sent, type: Boolean}
    lifecycle: {initial: Draft, states: [Draft], terminal: [Draft]}
  - name: demo.catalog.Listing
    identity: {name: listing_id, type: demo.catalog.ListingId}
    fields:
      - {name: section_id, type: demo.catalog.SectionId}
    lifecycle: {initial: Held, states: [Held], terminal: [Held]}
events:
  - name: demo.catalog.SectionDefined
    fields:
      - {name: section_id, type: demo.catalog.SectionId}
  - name: demo.catalog.NoticeDrafted
    fields:
      - {name: notice_id, type: demo.catalog.NoticeId}
  - name: demo.catalog.SectionAnnounced
    fields:
      - {name: notice_id, type: demo.catalog.NoticeId}
      - {name: section_id, type: demo.catalog.SectionId}
  - name: demo.catalog.Listed
    fields:
      - {name: listing_id, type: demo.catalog.ListingId}
errors:
  - {name: demo.catalog.NoSection, fields: []}
  - {name: demo.catalog.NoNotice, fields: []}
actors:
  - name: demo.catalog.Clerk
    may: [demo.catalog.DefineSection, demo.catalog.DraftNotice, demo.catalog.AnnounceSection, demo.catalog.ListForSection]
commands:
  - name: demo.catalog.DefineSection
    input:
      - {name: title, type: demo.catalog.Title}
    outcomes:
      - name: defined
        creates: demo.catalog.Section
        instance: section_id
        emits: [demo.catalog.SectionDefined]
        payload:
          demo.catalog.SectionDefined: {section_id: {generated: true}}
        sets: {title: input.title}
  - name: demo.catalog.DraftNotice
    input:
      - {name: section_id, type: demo.catalog.SectionId}
    outcomes:
      - name: drafted
        creates: demo.catalog.Notice
        instance: notice_id
        emits: [demo.catalog.NoticeDrafted]
        payload:
          demo.catalog.NoticeDrafted: {notice_id: {generated: true}}
        sets: {section_id: input.section_id, sent: false}
  - name: demo.catalog.AnnounceSection
    input:
      - {name: notice_id, type: demo.catalog.NoticeId}
    outcomes:
      - name: announced
        updates: demo.catalog.Notice
        instance: notice_id
        emits: [demo.catalog.SectionAnnounced]
        payload:
          demo.catalog.SectionAnnounced: {notice_id: input.notice_id, section_id: {subject: section_id}}
        sets: {sent: true}
      - {name: no-notice, unknown_instance: true, error: demo.catalog.NoNotice}
  - name: demo.catalog.ListForSection
    input:
      - {name: section_id, type: demo.catalog.SectionId}
    outcomes:
      - {name: section-missing, when_related: {via: input.section_id, exists: false}, error: demo.catalog.NoSection}
      - name: listed
        creates: demo.catalog.Listing
        instance: listing_id
        emits: [demo.catalog.Listed]
        payload:
          demo.catalog.Listed: {listing_id: {generated: true}}
        sets: {section_id: input.section_id}
views:
  - name: demo.catalog.Listings
    source: demo.catalog.Listing
    consistency: read_your_writes
    fields:
      - {name: listing_id, type: demo.catalog.ListingId}
      - {name: section_id, type: demo.catalog.SectionId}
  - name: demo.catalog.Notices
    source: demo.catalog.Notice
    consistency: read_your_writes
    fields:
      - {name: notice_id, type: demo.catalog.NoticeId}
      - {name: sent, type: Boolean}
bindings:
  - id: list-on-announced
    when: {event: demo.catalog.SectionAnnounced}
    invoke: {command: demo.catalog.ListForSection}
    mapping: {section_id: event.section_id}
    delivery: at_least_once
    on_failure: retry
";

/// The unit's `TRIGGER_NEEDS_ROW`, plus `ReassignRole`, which publishes the same event without
/// refusing a missing role.
const SECOND_PUBLISHER: &str = r"
format: ess/27
system: demo
version: v1
domain: demo.access
types:
  - {name: demo.access.RoleId, kind: newtype, of: Uuid}
  - {name: demo.access.GrantId, kind: newtype, of: Uuid}
  - {name: demo.access.Title, kind: newtype, of: String}
entities:
  - name: demo.access.Role
    identity: {name: role_id, type: demo.access.RoleId}
    fields:
      - {name: title, type: demo.access.Title}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.access.Grant
    identity: {name: grant_id, type: demo.access.GrantId}
    fields:
      - {name: role_id, type: demo.access.RoleId}
    lifecycle: {initial: Held, states: [Held], terminal: [Held]}
events:
  - name: demo.access.RoleDefined
    fields:
      - {name: role_id, type: demo.access.RoleId}
  - name: demo.access.RoleAssigned
    fields:
      - {name: role_id, type: demo.access.RoleId}
  - name: demo.access.Granted
    fields:
      - {name: grant_id, type: demo.access.GrantId}
errors:
  - {name: demo.access.NoRole, fields: []}
actors:
  - name: demo.access.Clerk
    may: [demo.access.DefineRole, demo.access.AssignRole, demo.access.ReassignRole, demo.access.GrantForRole]
commands:
  - name: demo.access.DefineRole
    input:
      - {name: title, type: demo.access.Title}
    outcomes:
      - name: defined
        creates: demo.access.Role
        instance: role_id
        emits: [demo.access.RoleDefined]
        payload:
          demo.access.RoleDefined: {role_id: {generated: true}}
        sets: {title: input.title}
  - name: demo.access.AssignRole
    input:
      - {name: role_id, type: demo.access.RoleId}
    outcomes:
      - {name: role-not-found, when_related: {via: input.role_id, exists: false}, error: demo.access.NoRole}
      - name: assigned
        emits: [demo.access.RoleAssigned]
        payload:
          demo.access.RoleAssigned: {role_id: input.role_id}
  - name: demo.access.ReassignRole
    input:
      - {name: role_id, type: demo.access.RoleId}
    outcomes:
      - name: reassigned
        emits: [demo.access.RoleAssigned]
        payload:
          demo.access.RoleAssigned: {role_id: input.role_id}
  - name: demo.access.GrantForRole
    input:
      - {name: role_id, type: demo.access.RoleId}
    outcomes:
      - {name: role-missing, when_related: {via: input.role_id, exists: false}, error: demo.access.NoRole}
      - name: granted
        creates: demo.access.Grant
        instance: grant_id
        emits: [demo.access.Granted]
        payload:
          demo.access.Granted: {grant_id: {generated: true}}
        sets: {role_id: input.role_id}
views:
  - name: demo.access.Grants
    source: demo.access.Grant
    consistency: read_your_writes
    fields:
      - {name: grant_id, type: demo.access.GrantId}
      - {name: role_id, type: demo.access.RoleId}
bindings:
  - id: grant-on-assigned
    when: {event: demo.access.RoleAssigned}
    invoke: {command: demo.access.GrantForRole}
    mapping: {role_id: event.role_id}
    delivery: at_least_once
    on_failure: {drop: [role-missing], retry: {except: [role-missing]}}
";

fn model(text: &str) -> EssIr {
    let raw =
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("the fixture parses: {error}"));
    let spec = Specification::assemble([(Source::new("fixture.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|d| panic!("the fixture resolves:\n{d}"))
}

fn refused_as(synthesis: &Synthesis, id: &str) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|refusal| refusal.contains(&format!("`{id}`")))
        .collect()
}

fn run(synthesis: &Synthesis, implementation: EssIr) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).expect("the suite is admitted");
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(implementation))
        .into_report()
        .scenarios
        .into_iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

fn not_passed(statuses: &BTreeMap<String, Status>) -> Vec<String> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, status)| format!("{id} {status:?}"))
        .collect()
}

/// Each of `ids` is synthesized, and every scenario of the suite passes against the interpreter
/// running the model itself.
fn assert_witnessed(text: &str, ids: &[&str]) {
    let ir = model(text);
    let synthesis = synthesize(&ir);
    for id in ids {
        assert_eq!(
            refused_as(&synthesis, id),
            Vec::<String>::new(),
            "{id} is refused"
        );
        assert!(
            synthesis
                .suite
                .scenarios
                .keys()
                .any(|key| key.to_string() == *id),
            "{id} is synthesized"
        );
    }
    let statuses = run(&synthesis, ir);
    assert_eq!(not_passed(&statuses), Vec::<String>::new());
}

#[test]
fn a_trigger_input_destination_is_the_row_the_trigger_guard_arranged() {
    // `RequestPairing` refuses a managed device, so its arrangement opens one; the binding's
    // scenarios must send it naming that device, not a freshly registered (managed) one.
    assert_witnessed(
        TRIGGER_INPUT_GUARDED,
        &[
            "pair-on-requested/binding/flow",
            "pair-on-requested/binding/mapping",
            "pair-on-requested/binding/delivery",
        ],
    );
}

#[test]
fn a_per_refusal_unbounded_retry_leaves_no_trigger_scenario_unsupported() {
    let text = UNBOUNDED_RETRY.replacen(
        "    on_failure: retry\n",
        "    on_failure: {retry: [section-missing], drop: {except: [section-missing]}}\n",
        1,
    );
    assert_ne!(text, UNBOUNDED_RETRY);
    assert_witnessed(&text, &["demo.catalog.AnnounceSection/outcome/announced"]);
}

#[test]
fn an_unbounded_retry_from_a_stored_field_leaves_no_scenario_unsupported() {
    assert_witnessed(
        SUBJECT_FIELD_RETRY,
        &[
            "demo.catalog.AnnounceSection/outcome/announced",
            "list-on-announced/binding/mapping",
        ],
    );
}

#[test]
fn an_unguarded_second_publisher_keeps_the_missing_row_refusal_scenario() {
    // `ReassignRole` publishes `RoleAssigned` for any role, so a trigger exists that leaves the
    // role absent when the binding is delivered; the refusal's help names exactly that spelling.
    assert_witnessed(
        SECOND_PUBLISHER,
        &["grant-on-assigned/binding/refusal/role-missing"],
    );
}

// ---- pass 2 -------------------------------------------------------------------------------------

/// `TRIGGER_INPUT_GUARDED` plus a binding on `DeviceOpened` into a command acting on the device: the
/// trigger's arrangement sets off a binding on the row, so its state is not followed.
const TRIGGER_INPUT_GUARDED_SIBLING: &str = r"
format: ess/27
system: demo
version: v1
domain: demo.device
types:
  - {name: demo.device.DeviceId, kind: newtype, of: Uuid}
  - {name: demo.device.Label, kind: newtype, of: String}
  - {name: demo.device.Mode, kind: enum, variants: [managed, open]}
entities:
  - name: demo.device.Device
    identity: {name: device_id, type: demo.device.DeviceId}
    fields:
      - {name: label, type: demo.device.Label}
      - {name: mode, type: demo.device.Mode}
      - {name: paired, type: Boolean}
      - {name: audited, type: Boolean}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
events:
  - name: demo.device.DeviceRegistered
    fields:
      - {name: device_id, type: demo.device.DeviceId}
  - name: demo.device.DeviceOpened
    fields:
      - {name: device_id, type: demo.device.DeviceId}
  - name: demo.device.PairingRequested
    fields:
      - {name: device_id, type: demo.device.DeviceId}
  - name: demo.device.DevicePaired
    fields:
      - {name: device_id, type: demo.device.DeviceId}
  - name: demo.device.DeviceAudited
    fields:
      - {name: device_id, type: demo.device.DeviceId}
errors:
  - {name: demo.device.NoDevice, fields: []}
  - {name: demo.device.Managed, fields: []}
actors:
  - name: demo.device.Clerk
    may: [demo.device.RegisterDevice, demo.device.OpenDevice, demo.device.RequestPairing, demo.device.PairDevice, demo.device.AuditDevice]
commands:
  - name: demo.device.RegisterDevice
    input:
      - {name: label, type: demo.device.Label}
    outcomes:
      - name: registered
        creates: demo.device.Device
        instance: device_id
        emits: [demo.device.DeviceRegistered]
        payload:
          demo.device.DeviceRegistered: {device_id: {generated: true}}
        sets: {label: input.label, mode: managed, paired: false, audited: false}
  - name: demo.device.OpenDevice
    input:
      - {name: device_id, type: demo.device.DeviceId}
    outcomes:
      - name: opened
        updates: demo.device.Device
        instance: device_id
        emits: [demo.device.DeviceOpened]
        payload:
          demo.device.DeviceOpened: {device_id: input.device_id}
        sets: {mode: open}
      - {name: no-device, unknown_instance: true, error: demo.device.NoDevice}
  - name: demo.device.RequestPairing
    input:
      - {name: device_id, type: demo.device.DeviceId}
    outcomes:
      - {name: unknown-device, when_related: {via: input.device_id, exists: false}, error: demo.device.NoDevice}
      - {name: managed, when_related: {via: input.device_id, predicate: mode == managed}, error: demo.device.Managed}
      - name: requested
        emits: [demo.device.PairingRequested]
        payload:
          demo.device.PairingRequested: {device_id: input.device_id}
  - name: demo.device.PairDevice
    input:
      - {name: device_id, type: demo.device.DeviceId}
    outcomes:
      - name: paired
        updates: demo.device.Device
        instance: device_id
        emits: [demo.device.DevicePaired]
        payload:
          demo.device.DevicePaired: {device_id: input.device_id}
        sets: {paired: true}
      - {name: no-device, unknown_instance: true, error: demo.device.NoDevice}
  - name: demo.device.AuditDevice
    input:
      - {name: device_id, type: demo.device.DeviceId}
    outcomes:
      - name: audited
        updates: demo.device.Device
        instance: device_id
        emits: [demo.device.DeviceAudited]
        payload:
          demo.device.DeviceAudited: {device_id: input.device_id}
        sets: {audited: true}
      - {name: no-device, unknown_instance: true, error: demo.device.NoDevice}
views:
  - name: demo.device.Devices
    source: demo.device.Device
    consistency: read_your_writes
    fields:
      - {name: device_id, type: demo.device.DeviceId}
      - {name: mode, type: demo.device.Mode}
      - {name: paired, type: Boolean}
bindings:
  - id: pair-on-requested
    when: {event: demo.device.PairingRequested}
    invoke: {command: demo.device.PairDevice}
    mapping: {device_id: event.device_id}
    delivery: at_least_once
    on_failure: retry
  - id: audit-on-opened
    when: {event: demo.device.DeviceOpened}
    invoke: {command: demo.device.AuditDevice}
    mapping: {device_id: event.device_id}
    delivery: at_least_once
    on_failure: drop
";

/// The d2 shape where the only publisher updates the role and refuses an unknown one
/// (`unknown_instance`) instead of a `when_related` refusal.
const TRIGGER_UPDATES_ROW: &str = r"
format: ess/27
system: demo
version: v1
domain: demo.access
types:
  - {name: demo.access.RoleId, kind: newtype, of: Uuid}
  - {name: demo.access.GrantId, kind: newtype, of: Uuid}
  - {name: demo.access.Title, kind: newtype, of: String}
entities:
  - name: demo.access.Role
    identity: {name: role_id, type: demo.access.RoleId}
    fields:
      - {name: title, type: demo.access.Title}
      - {name: assigned, type: Boolean}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.access.Grant
    identity: {name: grant_id, type: demo.access.GrantId}
    fields:
      - {name: role_id, type: demo.access.RoleId}
    lifecycle: {initial: Held, states: [Held], terminal: [Held]}
events:
  - name: demo.access.RoleDefined
    fields:
      - {name: role_id, type: demo.access.RoleId}
  - name: demo.access.RoleAssigned
    fields:
      - {name: role_id, type: demo.access.RoleId}
  - name: demo.access.Granted
    fields:
      - {name: grant_id, type: demo.access.GrantId}
errors:
  - {name: demo.access.NoRole, fields: []}
actors:
  - name: demo.access.Clerk
    may: [demo.access.DefineRole, demo.access.AssignRole, demo.access.GrantForRole]
commands:
  - name: demo.access.DefineRole
    input:
      - {name: title, type: demo.access.Title}
    outcomes:
      - name: defined
        creates: demo.access.Role
        instance: role_id
        emits: [demo.access.RoleDefined]
        payload:
          demo.access.RoleDefined: {role_id: {generated: true}}
        sets: {title: input.title, assigned: false}
  - name: demo.access.AssignRole
    input:
      - {name: role_id, type: demo.access.RoleId}
    outcomes:
      - name: assigned
        updates: demo.access.Role
        instance: role_id
        sets: {assigned: true}
        emits: [demo.access.RoleAssigned]
        payload:
          demo.access.RoleAssigned: {role_id: input.role_id}
      - {name: role-not-found, unknown_instance: true, error: demo.access.NoRole}
  - name: demo.access.GrantForRole
    input:
      - {name: role_id, type: demo.access.RoleId}
    outcomes:
      - {name: role-missing, when_related: {via: input.role_id, exists: false}, error: demo.access.NoRole}
      - name: granted
        creates: demo.access.Grant
        instance: grant_id
        emits: [demo.access.Granted]
        payload:
          demo.access.Granted: {grant_id: {generated: true}}
        sets: {role_id: input.role_id}
views:
  - name: demo.access.Grants
    source: demo.access.Grant
    consistency: read_your_writes
    fields:
      - {name: grant_id, type: demo.access.GrantId}
      - {name: role_id, type: demo.access.RoleId}
bindings:
  - id: grant-on-assigned
    when: {event: demo.access.RoleAssigned}
    invoke: {command: demo.access.GrantForRole}
    mapping: {role_id: event.role_id}
    delivery: at_least_once
    on_failure: {drop: [role-missing], retry: {except: [role-missing]}}
";

#[test]
fn a_trigger_input_destination_is_not_a_fresh_row_when_a_sibling_binding_moves_the_arranged_one() {
    // The arrangement sets off `audit-on-opened` on the row, whose delivery synthesis does not
    // follow: the scenarios needing the destination's state are refused by name, never sent naming
    // a fresh row the trigger's guard refuses. The mapping needs no destination row and is sent
    // through the trigger's own arrangement.
    let ir = model(TRIGGER_INPUT_GUARDED_SIBLING);
    let synthesis = synthesize(&ir);
    let refused = refused_as(&synthesis, "pair-on-requested/binding/flow");
    assert!(
        refused
            .iter()
            .any(|refusal| refusal.contains("cannot be followed")),
        "the flow is refused by name: {refused:?}"
    );
    let id = "pair-on-requested/binding/mapping";
    assert_eq!(refused_as(&synthesis, id), Vec::<String>::new());
    assert!(
        synthesis
            .suite
            .scenarios
            .keys()
            .any(|key| key.to_string() == id),
        "{id} is synthesized"
    );
    assert_eq!(not_passed(&run(&synthesis, ir)), Vec::<String>::new());
}

#[test]
fn a_missing_row_refusal_is_not_set_up_through_a_trigger_that_updates_the_row() {
    // Witnessed, or refused by name: never synthesized and failed by the model itself.
    let ir = model(TRIGGER_UPDATES_ROW);
    let synthesis = synthesize(&ir);
    assert_eq!(not_passed(&run(&synthesis, ir)), Vec::<String>::new());
}
