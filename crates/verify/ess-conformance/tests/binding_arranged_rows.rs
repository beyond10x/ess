//! Rows a binding scenario, or a scenario beside a binding, must arrange before it can pass
//! (a downstream adopter's report of 2026-10-20).
//!
//! Each fixture is one construct the report named. They are written here rather than under
//! `fixtures/`, so the tables that pin every repository model's bytes keep their lines:
//!
//! | fixture | what synthesis must arrange |
//! |---|---|
//! | [`PER_ROW_EVENT`] | a binding into a set effect whose event is published once per changed row: a row the filter selects, before the trigger |
//! | [`RELATED_REFUSAL`] | a refusal-selected policy over a `when_related: {exists: false}` refusal: the trigger naming no row, so the refusal is answered, not injected |
//! | [`ROW_SET`] | a row-set guard in a model with a binding that never writes the guarded entity: the rows, as without the binding |
//! | [`AFFECTS`] | `affects:` rows whose creating command has a `when_related` refusal: the subject the filter ties them to, named in the creation |
//! | [`SUBJECT_FIELD`] | a binding target named by a `{subject: <field>}` payload value: the target row first, then the trigger's row naming it |
//! | [`SUBJECT_FIELD_GUARDED`] | the same, where the trigger row's creator guards the related row's stored fields: the row the trigger's arrangement made selectable is the target |
//! | [`UNBOUNDED_RETRY`] | the outcome scenario of a command whose event an unbounded `retry` binding answers, guarded by nothing: the row the bound command needs, so the retry ends |
//!
//! Every scenario is also run against the interpreter: a scenario synthesized for a construct and
//! failed by the reference implementation is no better than a refusal.

use std::collections::BTreeMap;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::Value;

const PER_ROW_EVENT: &str = r"
format: ess/27
system: demo
version: v1
domain: demo.stock
types:
  - {name: demo.stock.ItemId, kind: newtype, of: Uuid}
  - {name: demo.stock.Sku, kind: newtype, of: String}
entities:
  - name: demo.stock.Item
    identity: {name: item_id, type: demo.stock.ItemId}
    fields:
      - {name: sku, type: demo.stock.Sku}
      - {name: marked, type: Boolean}
    lifecycle:
      initial: Listed
      states: [Listed, Retired]
      terminal: [Retired]
      transitions:
        - {name: retire, from: [Listed], to: Retired}
events:
  - name: demo.stock.ItemAdded
    fields:
      - {name: item_id, type: demo.stock.ItemId}
  - name: demo.stock.ItemRetired
    fields:
      - {name: item_id, type: demo.stock.ItemId}
  - name: demo.stock.Announced
    fields:
      - {name: sku, type: demo.stock.Sku}
  - name: demo.stock.ItemMarked
    fields:
      - {name: item_id, type: demo.stock.ItemId}
actors:
  - name: demo.stock.Clerk
    may: [demo.stock.AddItem, demo.stock.RetireItem, demo.stock.Announce, demo.stock.Mark]
commands:
  - name: demo.stock.AddItem
    input:
      - {name: sku, type: demo.stock.Sku}
    outcomes:
      - name: added
        creates: demo.stock.Item
        instance: item_id
        emits: [demo.stock.ItemAdded]
        payload:
          demo.stock.ItemAdded: {item_id: {generated: true}}
        sets: {sku: input.sku, marked: false}
  - name: demo.stock.RetireItem
    input:
      - {name: item_id, type: demo.stock.ItemId}
    outcomes:
      - name: retired
        moves: demo.stock.Item.retire
        instance: item_id
        emits: [demo.stock.ItemRetired]
        payload:
          demo.stock.ItemRetired: {item_id: input.item_id}
  - name: demo.stock.Announce
    input:
      - {name: sku, type: demo.stock.Sku}
    outcomes:
      - name: announced
        emits: [demo.stock.Announced]
        payload:
          demo.stock.Announced: {sku: input.sku}
  - name: demo.stock.Mark
    input:
      - {name: sku, type: demo.stock.Sku}
    outcomes:
      - name: marked
        updates: demo.stock.Item
        instances: {where: sku == input.sku}
        sets: {marked: true}
        emits: [{event: demo.stock.ItemMarked, each: changed, when_state: Listed}]
        payload:
          demo.stock.ItemMarked: {item_id: {changed: item_id}}
views:
  - name: demo.stock.Items
    source: demo.stock.Item
    consistency: read_your_writes
    fields:
      - {name: item_id, type: demo.stock.ItemId}
      - {name: sku, type: demo.stock.Sku}
      - {name: marked, type: Boolean}
bindings:
  - id: mark-on-announced
    when: {event: demo.stock.Announced}
    invoke: {command: demo.stock.Mark}
    mapping: {sku: event.sku}
    delivery: at_least_once
    on_failure: retry
";
const RELATED_REFUSAL: &str = r"
format: ess/27
system: demo
version: v1
domain: demo.access
types:
  - {name: demo.access.GroupId, kind: newtype, of: Uuid}
  - {name: demo.access.MembershipId, kind: newtype, of: Uuid}
  - {name: demo.access.Title, kind: newtype, of: String}
entities:
  - name: demo.access.Group
    identity: {name: group_id, type: demo.access.GroupId}
    fields:
      - {name: title, type: demo.access.Title}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.access.Membership
    identity: {name: membership_id, type: demo.access.MembershipId}
    fields:
      - {name: group_id, type: demo.access.GroupId}
    lifecycle: {initial: Held, states: [Held], terminal: [Held]}
events:
  - name: demo.access.GroupDefined
    fields:
      - {name: group_id, type: demo.access.GroupId}
  - name: demo.access.Requested
    fields:
      - {name: group_id, type: demo.access.GroupId}
  - name: demo.access.Enrolled
    fields:
      - {name: membership_id, type: demo.access.MembershipId}
errors:
  - {name: demo.access.NoGroup, fields: []}
actors:
  - name: demo.access.Clerk
    may: [demo.access.DefineGroup, demo.access.Request, demo.access.Enrol]
commands:
  - name: demo.access.DefineGroup
    input:
      - {name: title, type: demo.access.Title}
    outcomes:
      - name: defined
        creates: demo.access.Group
        instance: group_id
        emits: [demo.access.GroupDefined]
        payload:
          demo.access.GroupDefined: {group_id: {generated: true}}
        sets: {title: input.title}
  - name: demo.access.Request
    input:
      - {name: group_id, type: demo.access.GroupId}
    outcomes:
      - name: requested
        emits: [demo.access.Requested]
        payload:
          demo.access.Requested: {group_id: input.group_id}
  - name: demo.access.Enrol
    input:
      - {name: group_id, type: demo.access.GroupId}
    outcomes:
      - {name: no-group, when_related: {via: input.group_id, exists: false}, error: demo.access.NoGroup}
      - name: enrolled
        creates: demo.access.Membership
        instance: membership_id
        emits: [demo.access.Enrolled]
        payload:
          demo.access.Enrolled: {membership_id: {generated: true}}
        sets: {group_id: input.group_id}
views:
  - name: demo.access.Memberships
    source: demo.access.Membership
    consistency: read_your_writes
    fields:
      - {name: membership_id, type: demo.access.MembershipId}
      - {name: group_id, type: demo.access.GroupId}
bindings:
  - id: enrol-on-request
    when: {event: demo.access.Requested}
    invoke: {command: demo.access.Enrol}
    mapping: {group_id: event.group_id}
    delivery: at_least_once
    on_failure: {drop: [no-group], retry: {except: [no-group]}}
";
const ROW_SET: &str = r"
format: ess/27
system: demo
version: v1
domain: demo.queue
types:
  - {name: demo.queue.TaskId, kind: newtype, of: Uuid}
  - {name: demo.queue.Assignee, kind: newtype, of: String}
entities:
  - name: demo.queue.Task
    identity: {name: task_id, type: demo.queue.TaskId}
    fields:
      - {name: assignee, type: demo.queue.Assignee}
      - {name: done, type: Boolean}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
events:
  - name: demo.queue.TaskOpened
    fields:
      - {name: task_id, type: demo.queue.TaskId}
  - name: demo.queue.TaskClosed
    fields:
      - {name: task_id, type: demo.queue.TaskId}
  - name: demo.queue.AssigneeReleased
    fields:
      - {name: assignee, type: demo.queue.Assignee}
  - name: demo.queue.Pinged
    fields:
      - {name: note, type: demo.queue.Assignee}
  - name: demo.queue.Ponged
    fields:
      - {name: note, type: demo.queue.Assignee}
errors:
  - {name: demo.queue.StillBusy, fields: []}
  - {name: demo.queue.NotOpen, fields: []}
actors:
  - name: demo.queue.Clerk
    may: [demo.queue.OpenTask, demo.queue.CloseTask, demo.queue.ReleaseAssignee, demo.queue.Ping, demo.queue.Pong]
commands:
  - name: demo.queue.OpenTask
    input:
      - {name: assignee, type: demo.queue.Assignee}
    outcomes:
      - name: opened
        creates: demo.queue.Task
        instance: task_id
        emits: [demo.queue.TaskOpened]
        payload:
          demo.queue.TaskOpened: {task_id: {generated: true}}
        sets: {assignee: input.assignee, done: false}
  - name: demo.queue.CloseTask
    input:
      - {name: task_id, type: demo.queue.TaskId}
    outcomes:
      - name: closed
        moves: demo.queue.Task.close
        instance: task_id
        emits: [demo.queue.TaskClosed]
        payload:
          demo.queue.TaskClosed: {task_id: input.task_id}
        sets: {done: true}
      - {name: not-open, wrong_state: true, error: demo.queue.NotOpen}
  - name: demo.queue.ReleaseAssignee
    input:
      - {name: assignee, type: demo.queue.Assignee}
    outcomes:
      - name: busy
        when_related: {entity: demo.queue.Task, where: {all: [assignee == input.assignee, done == false]}, exists: true}
        error: demo.queue.StillBusy
      - name: released
        emits: [demo.queue.AssigneeReleased]
        payload:
          demo.queue.AssigneeReleased: {assignee: input.assignee}
  - name: demo.queue.Ping
    input:
      - {name: note, type: demo.queue.Assignee}
    outcomes:
      - name: pinged
        emits: [demo.queue.Pinged]
        payload:
          demo.queue.Pinged: {note: input.note}
  - name: demo.queue.Pong
    input:
      - {name: note, type: demo.queue.Assignee}
    outcomes:
      - name: ponged
        emits: [demo.queue.Ponged]
        payload:
          demo.queue.Ponged: {note: input.note}
views:
  - name: demo.queue.Tasks
    source: demo.queue.Task
    consistency: read_your_writes
    fields:
      - {name: task_id, type: demo.queue.TaskId}
      - {name: assignee, type: demo.queue.Assignee}
      - {name: done, type: Boolean}
bindings:
  - id: pong-on-ping
    when: {event: demo.queue.Pinged}
    invoke: {command: demo.queue.Pong}
    mapping: {note: event.note}
    delivery: at_least_once
    on_failure: retry
";
const AFFECTS: &str = r"
format: ess/27
system: demo
version: v1
domain: demo.library
types:
  - {name: demo.library.AccountId, kind: newtype, of: Uuid}
  - {name: demo.library.LoanId, kind: newtype, of: Uuid}
  - {name: demo.library.Name, kind: newtype, of: String}
entities:
  - name: demo.library.Account
    identity: {name: account_id, type: demo.library.AccountId}
    fields:
      - {name: name, type: demo.library.Name}
    lifecycle:
      initial: Active
      states: [Active, Archived]
      terminal: [Archived]
      transitions:
        - {name: archive, from: [Active], to: Archived}
  - name: demo.library.Loan
    identity: {name: loan_id, type: demo.library.LoanId}
    fields:
      - {name: account_id, type: demo.library.AccountId}
    relations:
      - {name: account, kind: references, target: demo.library.Account, cardinality: one, via: account_id}
    lifecycle: {initial: Lent, states: [Lent], terminal: [Lent]}
events:
  - name: demo.library.AccountOpened
    fields:
      - {name: account_id, type: demo.library.AccountId}
  - name: demo.library.LoanOpened
    fields:
      - {name: loan_id, type: demo.library.LoanId}
  - name: demo.library.AccountArchived
    fields:
      - {name: account_id, type: demo.library.AccountId}
errors:
  - {name: demo.library.NoAccount, fields: []}
  - {name: demo.library.NotActive, fields: []}
actors:
  - name: demo.library.Clerk
    may: [demo.library.OpenAccount, demo.library.OpenLoan, demo.library.ArchiveAccount]
commands:
  - name: demo.library.OpenAccount
    input:
      - {name: name, type: demo.library.Name}
    outcomes:
      - name: opened
        creates: demo.library.Account
        instance: account_id
        emits: [demo.library.AccountOpened]
        payload:
          demo.library.AccountOpened: {account_id: {generated: true}}
        sets: {name: input.name}
  - name: demo.library.OpenLoan
    input:
      - {name: account_id, type: demo.library.AccountId}
    outcomes:
      - {name: no-account, when_related: {via: input.account_id, exists: false}, error: demo.library.NoAccount}
      - name: opened
        creates: demo.library.Loan
        instance: loan_id
        emits: [demo.library.LoanOpened]
        payload:
          demo.library.LoanOpened: {loan_id: {generated: true}}
        sets: {account_id: input.account_id}
  - name: demo.library.ArchiveAccount
    input:
      - {name: account_id, type: demo.library.AccountId}
    outcomes:
      - name: archived
        moves: demo.library.Account.archive
        instance: account_id
        emits: [demo.library.AccountArchived]
        payload:
          demo.library.AccountArchived: {account_id: input.account_id}
        affects:
          - {entity: demo.library.Loan, where: account_id == input.account_id, deletes: demo.library.Loan}
      - {name: not-active, wrong_state: true, error: demo.library.NotActive}
views:
  - name: demo.library.Loans
    source: demo.library.Loan
    consistency: read_your_writes
    fields:
      - {name: loan_id, type: demo.library.LoanId}
      - {name: account_id, type: demo.library.AccountId}
";
const SUBJECT_FIELD: &str = r"
format: ess/27
system: demo
version: v1
domain: demo.device
types:
  - {name: demo.device.DeviceId, kind: newtype, of: Uuid}
  - {name: demo.device.TokenId, kind: newtype, of: Uuid}
  - {name: demo.device.Label, kind: newtype, of: String}
entities:
  - name: demo.device.Device
    identity: {name: device_id, type: demo.device.DeviceId}
    fields:
      - {name: label, type: demo.device.Label}
      - {name: paired, type: Boolean}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: demo.device.Token
    identity: {name: token_id, type: demo.device.TokenId}
    fields:
      - {name: device_id, type: demo.device.DeviceId}
    lifecycle:
      initial: Issued
      states: [Issued, Redeemed]
      terminal: [Redeemed]
      transitions:
        - {name: redeem, from: [Issued], to: Redeemed}
events:
  - name: demo.device.DeviceRegistered
    fields:
      - {name: device_id, type: demo.device.DeviceId}
  - name: demo.device.TokenIssued
    fields:
      - {name: token_id, type: demo.device.TokenId}
  - name: demo.device.TokenRedeemed
    fields:
      - {name: token_id, type: demo.device.TokenId}
      - {name: device_id, type: demo.device.DeviceId}
  - name: demo.device.DevicePaired
    fields:
      - {name: device_id, type: demo.device.DeviceId}
errors:
  - {name: demo.device.AlreadyRedeemed, fields: []}
  - {name: demo.device.NoDevice, fields: []}
actors:
  - name: demo.device.Clerk
    may: [demo.device.RegisterDevice, demo.device.IssueToken, demo.device.RedeemToken, demo.device.PairDevice]
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
        sets: {label: input.label, paired: false}
  - name: demo.device.IssueToken
    input:
      - {name: device_id, type: demo.device.DeviceId}
    outcomes:
      - {name: unknown-device, when_related: {via: input.device_id, exists: false}, error: demo.device.NoDevice}
      - name: issued
        creates: demo.device.Token
        instance: token_id
        emits: [demo.device.TokenIssued]
        payload:
          demo.device.TokenIssued: {token_id: {generated: true}}
        sets: {device_id: input.device_id}
  - name: demo.device.RedeemToken
    input:
      - {name: token_id, type: demo.device.TokenId}
    outcomes:
      - name: redeemed
        moves: demo.device.Token.redeem
        instance: token_id
        emits: [demo.device.TokenRedeemed]
        payload:
          demo.device.TokenRedeemed: {token_id: input.token_id, device_id: {subject: device_id}}
      - {name: already-redeemed, wrong_state: true, error: demo.device.AlreadyRedeemed}
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
      - {name: paired, type: Boolean}
  - name: demo.device.Tokens
    source: demo.device.Token
    consistency: read_your_writes
    fields:
      - {name: token_id, type: demo.device.TokenId}
      - {name: device_id, type: demo.device.DeviceId}
      - {name: state, type: demo.device.Token.State}
bindings:
  - id: pair-on-redeemed
    when: {event: demo.device.TokenRedeemed}
    invoke: {command: demo.device.PairDevice}
    mapping: {device_id: event.device_id}
    delivery: at_least_once
    on_failure: retry
";
const SUBJECT_FIELD_GUARDED: &str = r"
format: ess/27
system: demo
version: v1
domain: demo.device
types:
  - {name: demo.device.DeviceId, kind: newtype, of: Uuid}
  - {name: demo.device.TokenId, kind: newtype, of: Uuid}
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
  - name: demo.device.Token
    identity: {name: token_id, type: demo.device.TokenId}
    fields:
      - {name: device_id, type: demo.device.DeviceId}
    lifecycle:
      initial: Issued
      states: [Issued, Redeemed]
      terminal: [Redeemed]
      transitions:
        - {name: redeem, from: [Issued], to: Redeemed}
events:
  - name: demo.device.DeviceRegistered
    fields:
      - {name: device_id, type: demo.device.DeviceId}
  - name: demo.device.DeviceOpened
    fields:
      - {name: device_id, type: demo.device.DeviceId}
  - name: demo.device.TokenIssued
    fields:
      - {name: token_id, type: demo.device.TokenId}
  - name: demo.device.TokenRedeemed
    fields:
      - {name: token_id, type: demo.device.TokenId}
      - {name: device_id, type: demo.device.DeviceId}
  - name: demo.device.DevicePaired
    fields:
      - {name: device_id, type: demo.device.DeviceId}
errors:
  - {name: demo.device.AlreadyRedeemed, fields: []}
  - {name: demo.device.NoDevice, fields: []}
  - {name: demo.device.Managed, fields: []}
actors:
  - name: demo.device.Clerk
    may: [demo.device.RegisterDevice, demo.device.OpenDevice, demo.device.IssueToken, demo.device.RedeemToken, demo.device.PairDevice]
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
  - name: demo.device.IssueToken
    input:
      - {name: device_id, type: demo.device.DeviceId}
    outcomes:
      - {name: unknown-device, when_related: {via: input.device_id, exists: false}, error: demo.device.NoDevice}
      - {name: managed, when_related: {via: input.device_id, predicate: mode == managed}, error: demo.device.Managed}
      - name: issued
        creates: demo.device.Token
        instance: token_id
        emits: [demo.device.TokenIssued]
        payload:
          demo.device.TokenIssued: {token_id: {generated: true}}
        sets: {device_id: input.device_id}
  - name: demo.device.RedeemToken
    input:
      - {name: token_id, type: demo.device.TokenId}
    outcomes:
      - name: redeemed
        moves: demo.device.Token.redeem
        instance: token_id
        emits: [demo.device.TokenRedeemed]
        payload:
          demo.device.TokenRedeemed: {token_id: input.token_id, device_id: {subject: device_id}}
      - {name: already-redeemed, wrong_state: true, error: demo.device.AlreadyRedeemed}
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
  - name: demo.device.Tokens
    source: demo.device.Token
    consistency: read_your_writes
    fields:
      - {name: token_id, type: demo.device.TokenId}
      - {name: device_id, type: demo.device.DeviceId}
      - {name: state, type: demo.device.Token.State}
bindings:
  - id: pair-on-redeemed
    when: {event: demo.device.TokenRedeemed}
    invoke: {command: demo.device.PairDevice}
    mapping: {device_id: event.device_id}
    delivery: at_least_once
    on_failure: retry
";

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

/// [`UNBOUNDED_RETRY`] with the bound command updating the section it names rather than creating a
/// listing beside it, refusing an unknown one.
const UNBOUNDED_RETRY_UPDATES: &str = r"
format: ess/27
system: demo
version: v1
domain: demo.catalog
types:
  - {name: demo.catalog.SectionId, kind: newtype, of: Uuid}
  - {name: demo.catalog.Title, kind: newtype, of: String}
entities:
  - name: demo.catalog.Section
    identity: {name: section_id, type: demo.catalog.SectionId}
    fields:
      - {name: title, type: demo.catalog.Title}
      - {name: listed, type: Boolean}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.catalog.SectionDefined
    fields:
      - {name: section_id, type: demo.catalog.SectionId}
  - name: demo.catalog.SectionAnnounced
    fields:
      - {name: section_id, type: demo.catalog.SectionId}
  - name: demo.catalog.Listed
    fields:
      - {name: section_id, type: demo.catalog.SectionId}
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
        sets: {title: input.title, listed: false}
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
      - name: listed
        updates: demo.catalog.Section
        instance: section_id
        emits: [demo.catalog.Listed]
        payload:
          demo.catalog.Listed: {section_id: input.section_id}
        sets: {listed: true}
      - {name: section-missing, unknown_instance: true, error: demo.catalog.NoSection}
views:
  - name: demo.catalog.Sections
    source: demo.catalog.Section
    consistency: read_your_writes
    fields:
      - {name: section_id, type: demo.catalog.SectionId}
      - {name: listed, type: Boolean}
bindings:
  - id: list-on-announced
    when: {event: demo.catalog.SectionAnnounced}
    invoke: {command: demo.catalog.ListForSection}
    mapping: {section_id: event.section_id}
    delivery: at_least_once
    on_failure: retry
";

fn model(text: &str) -> EssIr {
    let raw =
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("the fixture parses: {error}"));
    let spec = Specification::assemble([(Source::new("fixture.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|d| panic!("the fixture resolves:\n{d}"))
}

fn rewrite(text: &str, from: &str, to: &str) -> String {
    assert_eq!(text.matches(from).count(), 1, "`{from}` occurs once");
    text.replacen(from, to, 1)
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

fn refused_as(synthesis: &Synthesis, id: &str) -> Vec<String> {
    refusals(synthesis)
        .into_iter()
        .filter(|refusal| refusal.contains(&format!("`{id}`")))
        .collect()
}

fn steps_of(synthesis: &Synthesis, id: &str) -> Vec<Value> {
    let document: Value =
        serde_json::from_str(&synthesis.suite.to_canonical_json().expect("serialises"))
            .expect("JSON");
    document["scenarios"][id]["steps"]
        .as_array()
        .unwrap_or_else(|| panic!("`{id}` is synthesized: {:#?}", refused_as(synthesis, id)))
        .clone()
}

/// Every scenario's status against the interpreter running `implementation`.
fn run(synthesis: &Synthesis, implementation: EssIr) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).expect("the suite is admitted");
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(implementation))
        .into_report()
        .scenarios
        .into_iter()
        .map(|scenario| {
            if scenario.status != Status::Passed {
                for diagnostic in scenario.diagnostics() {
                    eprintln!(
                        "{} {:?}: expected {:?}, observed {:?}",
                        scenario.scenario,
                        diagnostic.code,
                        diagnostic.expected,
                        diagnostic.observed
                    );
                }
            }
            (scenario.scenario.to_string(), scenario.status)
        })
        .collect()
}

fn not_passed(statuses: &BTreeMap<String, Status>) -> Vec<String> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, status)| format!("{id} {status:?}"))
        .collect()
}

/// The scenarios are synthesized, and every scenario of the suite passes against the interpreter.
fn witnessed(text: &str, ids: &[&str]) -> (Synthesis, BTreeMap<String, Status>) {
    let ir = model(text);
    let synthesis = synthesize(&ir);
    for id in ids {
        assert_eq!(refused_as(&synthesis, id), Vec::<String>::new(), "{id}");
        steps_of(&synthesis, id);
    }
    let statuses = run(&synthesis, ir);
    assert_eq!(not_passed(&statuses), Vec::<String>::new());
    for id in ids {
        assert_eq!(statuses.get(*id), Some(&Status::Passed), "{id} ran");
    }
    (synthesis, statuses)
}

fn position(steps: &[Value], command: &str) -> usize {
    steps
        .iter()
        .position(|step| step["command"] == command)
        .unwrap_or_else(|| panic!("`{command}` is sent: {steps:#?}"))
}

// ---- 1. a binding into a set effect whose event is published once per changed row -------------

#[test]
fn a_binding_into_a_per_row_event_arranges_a_row_the_filter_selects() {
    let ids = [
        "mark-on-announced/binding/flow",
        "mark-on-announced/binding/delivery",
    ];
    let (synthesis, _) = witnessed(PER_ROW_EVENT, &ids);
    for id in ids {
        let steps = steps_of(&synthesis, id);
        assert!(
            position(&steps, "demo.stock.AddItem") < position(&steps, "demo.stock.Announce"),
            "{id}: an item exists before the trigger"
        );
    }
}

#[test]
fn a_dropped_binding_fails_the_per_row_event_flow() {
    let synthesis = synthesize(&model(PER_ROW_EVENT));
    let without = rewrite(
        PER_ROW_EVENT,
        "  - id: mark-on-announced
    when: {event: demo.stock.Announced}
    invoke: {command: demo.stock.Mark}
    mapping: {sku: event.sku}
    delivery: at_least_once
    on_failure: retry
",
        "",
    )
    .replacen("bindings:\n", "", 1);
    let statuses = run(&synthesis, model(&without));
    assert_ne!(
        statuses.get("mark-on-announced/binding/flow"),
        Some(&Status::Passed)
    );
}

// ---- 2. a refusal-selected policy over a `when_related` refusal --------------------------------

#[test]
fn a_related_row_refusal_is_answered_for_its_selected_policy() {
    let id = "enrol-on-request/binding/refusal/no-group";
    let (synthesis, _) = witnessed(RELATED_REFUSAL, &[id]);
    let steps = steps_of(&synthesis, id);
    // Answered by the honest system, never injected: no group is defined for the identity sent.
    assert!(
        !steps
            .iter()
            .any(|step| step["step"] == "configure_external_outcome"),
        "{steps:#?}"
    );
    assert!(
        !steps
            .iter()
            .any(|step| step["command"] == "demo.access.DefineGroup"),
        "{steps:#?}"
    );
}

#[test]
fn a_binding_that_retries_the_related_refusal_fails_its_drop_scenario() {
    let id = "enrol-on-request/binding/refusal/no-group";
    let synthesis = synthesize(&model(RELATED_REFUSAL));
    let retrying = rewrite(
        RELATED_REFUSAL,
        "on_failure: {drop: [no-group], retry: {except: [no-group]}}",
        "on_failure: retry",
    );
    let statuses = run(&synthesis, model(&retrying));
    assert_ne!(statuses.get(id), Some(&Status::Passed));
}

// ---- 3. a row-set guard beside a binding that never writes its entity ---------------------------

#[test]
fn an_unrelated_binding_leaves_a_row_set_guard_witnessed() {
    witnessed(
        ROW_SET,
        &[
            "demo.queue.ReleaseAssignee/outcome/busy",
            "demo.queue.ReleaseAssignee/outcome/released",
        ],
    );
}

#[test]
fn a_binding_writing_the_guarded_entity_keeps_the_refusal_and_says_why() {
    // The binding now opens a task whenever a ping is published: rows of the selector's entity
    // appear when the transport delivers, which no arrangement follows.
    let text = rewrite(
        ROW_SET,
        "    invoke: {command: demo.queue.Pong}
    mapping: {note: event.note}",
        "    invoke: {command: demo.queue.OpenTask}
    mapping: {assignee: event.note}",
    );
    let synthesis = synthesize(&model(&text));
    let busy = refused_as(&synthesis, "demo.queue.ReleaseAssignee/outcome/busy");
    assert_eq!(busy.len(), 1, "{busy:#?}");
    assert!(
        busy[0].contains("changes rows of the selector's entity")
            && busy[0].contains("authored scenario (ess-scenario/1)")
            && !busy[0].contains("let a command that creates the entity"),
        "{busy:#?}"
    );
}

// ---- 4. `affects:` rows whose creator guards on a related row -----------------------------------

#[test]
fn affects_rows_are_created_through_their_creators_related_guard() {
    let id = "demo.library.ArchiveAccount/outcome/archived";
    let (synthesis, _) = witnessed(AFFECTS, &[id]);
    let steps = steps_of(&synthesis, id);
    assert!(
        steps
            .iter()
            .any(|step| step["command"] == "demo.library.OpenLoan"),
        "{steps:#?}"
    );
}

// ---- 5. a binding target named by a `{subject: <field>}` payload value --------------------------

#[test]
fn a_binding_target_named_by_a_subject_field_is_arranged_before_the_trigger() {
    let ids = [
        "pair-on-redeemed/binding/flow",
        "pair-on-redeemed/binding/mapping",
        "pair-on-redeemed/binding/delivery",
    ];
    let (synthesis, _) = witnessed(SUBJECT_FIELD, &ids);
    let flow = steps_of(&synthesis, "pair-on-redeemed/binding/flow");
    assert!(
        position(&flow, "demo.device.RegisterDevice") < position(&flow, "demo.device.IssueToken"),
        "the device exists before the token naming it: {flow:#?}"
    );
}

#[test]
fn a_subject_field_target_is_arranged_where_the_trigger_row_stores_a_literal() {
    // Without the guard, the token's arrangement sends a literal device identity, which the
    // binding scenarios replace with the device they arrange.
    let text = rewrite(
        SUBJECT_FIELD,
        "      - {name: unknown-device, when_related: {via: input.device_id, exists: false}, error: demo.device.NoDevice}\n",
        "",
    );
    let ir = model(&text);
    let synthesis = synthesize(&ir);
    let statuses = run(&synthesis, ir);
    for aspect in ["flow", "mapping", "delivery"] {
        let id = format!("pair-on-redeemed/binding/{aspect}");
        assert_eq!(refused_as(&synthesis, &id), Vec::<String>::new(), "{id}");
        assert_eq!(statuses.get(&id), Some(&Status::Passed), "{id}");
    }
}

#[test]
fn a_dropped_binding_fails_the_subject_field_flow() {
    let synthesis = synthesize(&model(SUBJECT_FIELD));
    let without = rewrite(
        SUBJECT_FIELD,
        "  - id: pair-on-redeemed
    when: {event: demo.device.TokenRedeemed}
    invoke: {command: demo.device.PairDevice}
    mapping: {device_id: event.device_id}
    delivery: at_least_once
    on_failure: retry
",
        "",
    )
    .replacen("bindings:\n", "", 1);
    let statuses = run(&synthesis, model(&without));
    assert_ne!(
        statuses.get("pair-on-redeemed/binding/flow"),
        Some(&Status::Passed)
    );
}

// ---- 6. the same, where the trigger row's creator reads the target's stored fields --------------

/// The device the token is issued for, as the scenario's steps name it.
fn issued_for(steps: &[Value]) -> Value {
    steps[position(steps, "demo.device.IssueToken")]["input"]["device_id"].clone()
}

#[test]
fn a_binding_trigger_acts_on_the_row_its_setup_arranged() {
    // `IssueToken` refuses a managed device, so the token's arrangement opens the device it issues
    // the token for. The binding pairs that device: it is the one the trigger's row stores.
    let ids = [
        "pair-on-redeemed/binding/flow",
        "pair-on-redeemed/binding/mapping",
        "pair-on-redeemed/binding/delivery",
    ];
    let (synthesis, _) = witnessed(SUBJECT_FIELD_GUARDED, &ids);
    for id in ids {
        let steps = steps_of(&synthesis, id);
        let issued = issued_for(&steps);
        let opened = &steps[position(&steps, "demo.device.OpenDevice")]["input"]["device_id"];
        assert_eq!(&issued, opened, "{id}: the token names the opened device");
        assert_eq!(
            steps
                .iter()
                .filter(|step| step["command"] == "demo.device.RegisterDevice")
                .count(),
            1,
            "{id}: no second device is arranged beside it: {steps:#?}"
        );
    }
}

#[test]
fn a_dropped_binding_fails_the_guarded_subject_field_flow() {
    let synthesis = synthesize(&model(SUBJECT_FIELD_GUARDED));
    let without = rewrite(
        SUBJECT_FIELD_GUARDED,
        "  - id: pair-on-redeemed
    when: {event: demo.device.TokenRedeemed}
    invoke: {command: demo.device.PairDevice}
    mapping: {device_id: event.device_id}
    delivery: at_least_once
    on_failure: retry
",
        "",
    )
    .replacen("bindings:\n", "", 1);
    let statuses = run(&synthesis, model(&without));
    assert_ne!(
        statuses.get("pair-on-redeemed/binding/flow"),
        Some(&Status::Passed)
    );
}

/// [`SUBJECT_FIELD_GUARDED`] plus a binding on `DeviceOpened` into a command acting on the device:
/// the token's arrangement sets that binding off on the device it opens, so the state the
/// arrangement leaves the device in is not followed.
fn subject_field_guarded_sibling() -> String {
    let text = rewrite(
        SUBJECT_FIELD_GUARDED,
        "      - {name: paired, type: Boolean}\n    lifecycle: {initial: Active",
        "      - {name: paired, type: Boolean}\n      - {name: audited, type: Boolean}\n    \
         lifecycle: {initial: Active",
    );
    let text = rewrite(
        &text,
        "mode: managed, paired: false}",
        "mode: managed, paired: false, audited: false}",
    );
    let text = rewrite(
        &text,
        "      - {name: device_id, type: demo.device.DeviceId}\nerrors:",
        "      - {name: device_id, type: demo.device.DeviceId}\n  - name: demo.device.DeviceAudited\n    \
         fields:\n      - {name: device_id, type: demo.device.DeviceId}\nerrors:",
    );
    let text = rewrite(
        &text,
        "demo.device.PairDevice]",
        "demo.device.PairDevice, demo.device.AuditDevice]",
    );
    let text = rewrite(
        &text,
        "views:\n",
        "  - name: demo.device.AuditDevice
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
",
    );
    format!(
        "{text}  - id: audit-on-opened
    when: {{event: demo.device.DeviceOpened}}
    invoke: {{command: demo.device.AuditDevice}}
    mapping: {{device_id: event.device_id}}
    delivery: at_least_once
    on_failure: drop
"
    )
}

#[test]
fn a_subject_field_target_is_not_a_fresh_row_when_a_sibling_binding_acts_on_the_arranged_one() {
    // `IssueToken` reads the device's mode, so only the device the token's arrangement opened is
    // one it issues for; that arrangement sets off `audit-on-opened` on the device, whose delivery
    // synthesis does not follow. The flow is refused by name, never sent with the token issued
    // for a fresh (managed) device the trigger refuses.
    let ir = model(&subject_field_guarded_sibling());
    let synthesis = synthesize(&ir);
    let refused = refused_as(&synthesis, "pair-on-redeemed/binding/flow");
    assert!(
        refused
            .iter()
            .any(|refusal| refusal.contains("cannot be followed")),
        "the flow is refused by name: {refused:?}"
    );
    assert_eq!(not_passed(&run(&synthesis, ir)), Vec::<String>::new());
}

// ---- 7. the outcome scenario of a command an unbounded retry binding answers ------------------

#[test]
fn an_unbounded_retry_binding_leaves_no_trigger_scenario_unsupported() {
    // `AnnounceSection` reads no row, but the binding it sets off retries until `ListForSection`
    // finds the section: the trigger's outcome scenario names a defined one, or no run ends.
    for text in [UNBOUNDED_RETRY, UNBOUNDED_RETRY_UPDATES] {
        let id = "demo.catalog.AnnounceSection/outcome/announced";
        let (synthesis, _) = witnessed(text, &[id]);
        let steps = steps_of(&synthesis, id);
        assert!(
            position(&steps, "demo.catalog.DefineSection")
                < position(&steps, "demo.catalog.AnnounceSection"),
            "{id}: a section exists before the trigger: {steps:#?}"
        );
        let announced =
            &steps[position(&steps, "demo.catalog.AnnounceSection")]["input"]["section_id"];
        assert_eq!(announced["kind"], "instance", "{id}: {steps:#?}");
    }
}
