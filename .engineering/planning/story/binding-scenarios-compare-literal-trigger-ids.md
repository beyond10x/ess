---
format: aep.planning-md/3
id: story:binding-scenarios-compare-literal-trigger-ids
kind: story
status: draft
title: Every binding scenario compares the literal trigger ids it sent
tags:
- defect
- ess-d368-close
- severity-low
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Finding

Found by wave D368 U3's stricter sweep, out of that story's Acceptance: 15 non-refusal binding scenarios
expect the trigger's event without comparing an id sent as a literal for a field of an entity's identity
type. They are pinned in `literal_trigger_id_residue()`
(`crates/verify/ess-conformance/tests/adversary_273_pass1.rs`): `billing` notify-on-invoice-created,
`binding-arrangement` created-starts, `toolchain` check-on-tag and `set-effect-binding-causes`
sync-on-flag-set. The trigger event is expected by `expect_trigger_event`
(`crates/verify/ess-conformance/src/synthesize.rs`), whose `determined_identities` and
`fixtures::event_values` add captured instances and fixtures but no literal the trigger was sent.

## Problem

Reproduction on 0.92.0, with the in-tree fixture `tests/fixtures/binding-arrangement.yaml`
(`Create` takes `job_id: JobId` (newtype of Uuid) and publishes `Created {job_id: input.job_id}`;
binding `created-starts` turns `Created` into `Start`):

```
$ ess verify conform synthesize --path binding-arrangement.yaml --target ir --out suite.json
21 scenario(s) (0 authored), 1 refusal(s), written to suite.json
```

Every `expect_event` of `jobs.job.Created` in the suite (shape elided):

```
created-starts/binding/delivery {"step": "expect_event", "event": "jobs.job.Created"}
created-starts/binding/flow     {"step": "expect_event", "event": "jobs.job.Created"}
created-starts/binding/mapping  {"step": "expect_event", "event": "jobs.job.Created"}
jobs.job.Create/outcome/created {"step": "expect_event", "event": "jobs.job.Created", "payload": {"job_id": "00000000-0000-4000-8000-01b453fc1f5c"}}
```

Each binding scenario sends `job_id` as that literal but expects `Created` by shape alone, then
captures `job` from the event, so it trusts whatever id the target published. `examples/billing`
shows the same for `notify-on-invoice-created` (delivery, flow, mapping, on-failure) against
`CreateInvoice/outcome/accepted`, which compares `account_id`.

## Acceptance

- `ess verify conform synthesize` over `binding-arrangement.yaml` writes `created-starts/binding/flow`,
  `delivery` and `mapping` with `expect_event jobs.job.Created` carrying
  `payload: {job_id: <the literal Create was sent>}`; the same holds for the four `billing`
  scenarios and the `toolchain` and `set-effect-binding-causes` bindings.
- `literal_trigger_id_residue()` is removed and the sweep in `adversary_273_pass1.rs` asserts an
  empty miss set.
- A test target that publishes the trigger event with a different literal id fails
  `created-starts/binding/flow` on the interpreted target, naming `job_id`.
- The suites whose bytes move are named in the CHANGELOG entry, including `suites/generated/billing/suite.json`.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize.rs
- inferred: crates/verify/ess-conformance/src/fixtures.rs
- cited: crates/verify/ess-conformance/tests/adversary_273_pass1.rs
- inferred: suites/generated/billing/suite.json
- inferred: crates/verify/ess-conformance/tests/fixtures/binding-arrangement.yaml

confidence: high
