---
format: aep.planning-md/3
id: review-result:adversary-c-binding-pass-2
kind: review-result
status: active
title: Adversary pass 2, 0.41 unit binding-context
relations:
- reviews: story:bindings-read-the-delivery-context
revision: 1
---
unit: binding-context (beyond10x/ess#195), correction 1, working tree ~/.local/state/worktree/trees/b10x/ess/ess-c-binding (base 969394c28e + uncommitted diff with the coordinator patches applied)
verdict: NEEDS-CHANGE
cases: executed 1805→1810, red 5
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: 7 paths (listed in part 6)
needs-coordinator: no

## 1. Diff stat

Tracked diff: `46 files changed, 1003 insertions(+), 90 deletions(-)`. This is unchanged from the handover. I added two untracked test files and nothing else:

```
?? crates/verify/ess-conformance/tests/adv2_delivery_context.rs
?? crates/verify/ess-diff/tests/adv2_delivery_context.rs
```

I touched no non-test path.

## 2. Cases added (each file run alone before any suite)

| file | case | asserts | now |
|---|---|---|---|
| ess-conformance/tests/adv2_delivery_context.rs | `adv2_a_three_variant_enum_context_field_keeps_its_mapping_scenario` | Context `priority: Priority` is an enum `[Low, Medium, High]`, and the payload gains `amount: Integer`. `mapping` and `delivery` are synthesized, and the control passes them. | red |
| same | `adv2_a_target_that_reads_a_nested_payload_flag_in_place_of_the_context_fails` | Context `urgent: Boolean`. The payload has `flags: Flags {urgent: Boolean}`. A target that fills `urgent` from `event.flags.urgent` fails some scenario. | red |
| same | `adv2_a_bounded_integer_context_field_keeps_its_flow_scenario` | Context `rating: Rating` is a newtype over Integer with invariants `[value >= 1, value <= 5]`. All 4 binding scenarios are synthesized, and the control passes them. | red |
| ess-diff/tests/adv2_delivery_context.rs | `adv2_a_context_field_wire_rename_says_what_changed` | A context field gains `wire: accountId`. Every reported binding change has a before and an after that differ, both in `describe()` and in the JSON. | red |
| same | `adv2_a_context_field_summary_says_what_changed` | The same assertion, for a `summary:` added to the context field. | red |

Red output, verbatim (`cargo test -p ess-conformance --test adv2_delivery_context`, log `adv2/red-ess-conformance.log`):

```
thread 'adv2_a_bounded_integer_context_field_keeps_its_flow_scenario' (3166893) panicked at crates/verify/ess-conformance/tests/adv2_delivery_context.rs:193:9:
assertion `left == right` failed: `received/binding/flow` is synthesized and the control passes it: {}
                reason: "has no base value its declared invariants admit",
thread 'adv2_a_three_variant_enum_context_field_keeps_its_mapping_scenario' (3166895) panicked at crates/verify/ess-conformance/tests/adv2_delivery_context.rs:193:9:
assertion `left == right` failed: `received/binding/mapping` is synthesized and the control passes it: {
                reason: "DeliveryContext: the context field `priority` (`demo.inbox.Priority`) has one witness value, so two deliveries cannot be told apart by it",
thread 'adv2_a_target_that_reads_a_nested_payload_flag_in_place_of_the_context_fails' (3166894) panicked at crates/verify/ess-conformance/tests/adv2_delivery_context.rs:312:5:
mutant `urgent read from event.flags.urgent` passes every delivery-context scenario: {
    "received/binding/delivery": Passed,
    "received/binding/flow": Passed,
    "received/binding/mapping": Passed,
    "received/binding/on-failure": Passed,
}
test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
EXIT=101
```

Red output, verbatim (`cargo test -p ess-diff --test adv2_delivery_context`, log `adv2/red-ess-diff.log`):

```
assertion `left != right` failed: the rendered change does not say what changed: cause or required host contract changed: `demo.inbox.MessageReceived delivered by account-messages with context (account_id: demo.inbox.AccountId)` → `demo.inbox.MessageReceived delivered by account-messages with context (account_id: demo.inbox.AccountId)`
  "format": "ess-diff/10",
test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
EXIT=101
```

The same assertion text appears for both the wire case and the summary case. In both, the delta also carries `system/demo/unclassified-changed`.

## 3. Suite runs (after the cases existed)

`<before>` is 1595 + 210 = 1805. It comes from the implementor's `binding/c1/test-ess-conformance.log` and `binding/c1/test-ess-diff.log`.

| command | summed | final lines, verbatim |
|---|---|---|
| `cargo test -p ess-conformance --no-fail-fast` | passed 1595, failed 3, ignored 3 | `error: 1 target failed:` / ``    `-p ess-conformance --test adv2_delivery_context` `` / `EXIT=101` |
| `cargo test -p ess-diff --no-fail-fast` | passed 210, failed 2, ignored 0 | `error: 1 target failed:` / ``    `-p ess-diff --test adv2_delivery_context` `` / `EXIT=101` |
| `cargo xtask generate --check` | n/a | `projections are up to date` / `EXIT=0` |
| `cargo xtask schema --check` | n/a | `schemas/generated/ess.schema.json: current` / `EXIT=0` |

The only failures are my 5 cases. No existing case regressed, and the pass-1 adversary cases are green.

The claim "`cargo xtask generate --check` stays clean" holds on my run. The implementor's own evidence for it does not show that: `c1/xtask-generate_--check.log` and `c1/xtask-schema_--check.log` are both `error: unrecognized subcommand 'generate --check'` (the flag was quoted into the subcommand) with `EXIT=2`. `git status` was identical before and after my check runs.

## 4. Findings (tree: working tree above)

| file:line | severity | verdict | finding | fix |
|---|---|---|---|---|
| crates/verify/ess-conformance/src/synthesize/delivery_context.rs:200 | blocker | NEEDS-CHANGE | F2 is not closed for nested values. Every group base is even, and a Boolean or enum inside a payload struct takes the parity or variant of its own slot. So `event.flags.urgent` equals `context.urgent` in both deliveries. `separated()` (:172) compares only top-level `Node`s (a map against a bool), so nothing is refused, and a target reading the payload flag passes all 4 scenarios. The issue rules this out ("never a lookup in the payload"). What reaches it: any payload struct with a small-range leaf (Boolean, enum, Timestamp mod 28, Bytes mod 256) beside a context field of that type. | Compare the mapped context value against every leaf of the payload and context, walking into maps and sequences, and refuse on a match. Alternatively, number the leaves rather than the top-level fields. |
| crates/verify/ess-conformance/src/synthesize/delivery_context.rs:219 | warning | NEEDS-CHANGE | There is a spurious refusal for odd-sized enums. The first occurrence uses distinction `b` and the second uses `first_slots + b + 1`. For a 3-variant enum these are equal mod 3 whenever `first_slots ≡ 2 (mod 3)`, which happens with 4 one-field type groups. Both deliveries then carry `Low`, and `mapping`/`delivery` are refused with "has one witness value", which is false for a type with 3 values. What reaches it: an ordinary model, the fixture plus one Integer payload field and one enum context field. | Pick the second occurrence's value per field so it differs from the first (for example, step by 1 within the field's own value domain), or search for a free value before refusing. Make the reason name the actual collision. |
| crates/verify/ess-conformance/src/synthesize/delivery_context.rs:233 | warning | NEEDS-CHANGE | The widened distinction range breaks newtype invariants. `Rating` (Integer, 1..5) gets distinctions 4 and 11, which give values 5 and 12. `witness::fields` returns a `WitnessGap`, and `synthesize` then refuses the whole binding with `NoWitness`, including `flow` and `on-failure`, which need no second value. What reaches it: any bounded numeric (or length-bounded text) context or payload field in a model with a few fields. | Pick in-range values (the invariants are recorded) or fall back per field. At minimum, refuse only `mapping`/`delivery` and still synthesize `flow`/`on-failure` from a plain witness. |
| crates/verify/ess-diff/src/diff.rs:1762 | warning | NEEDS-CHANGE | This is F4 residue. `compare_bindings` fires on `was.context != is.context`, which includes each field's `naming` (wire, display, summary). `written_cause` (:1084) rebuilds with `Field::new(name, type)` and drops naming. A wire rename, or a documentation-only `summary:`, is therefore reported as a `cause-changed` whose two sides are byte-identical in both the text and the JSON, and the delta is bumped to `ess-diff/10`. What reaches it: any edit to a context field's `wire`/`display`/`summary`. | Carry `naming.wire` (the part that matters to a host) in the written cause, and compare contexts without `summary`/`display`. Or compare on the written cause, so a documentation-only edit reports nothing binding-specific. |

## 5. Attacked, not broken

- `separated()` compares emitted `Node`s, not pre-image distinctions, at the top level. Boolean top-level fields in one group alternate correctly.
- Cross-type top-level collisions: Integer and Decimal both render `Number(1+nth)`, but every slot's `nth` is unique across all groups and both occurrences, so they cannot collide.
- ess-diff/10: `EssDelta::new` takes the max of `minimum_format`, so deltas without `External` keep their earlier format. `to_canonical_json_for` and the raw reader (`raw.rs:105`) both use `minimum_format`. There is no other writer outside the crate (grep: no `to_canonical_json_for` or `DeltaFormat::parse` in non-test code elsewhere). The /9 reader refusal is covered by the implementor's `tests/delivery_context.rs`.
- Display change: only the `External` arm changed, and no committed golden changed. The ess-diff and ess-conformance suites are green apart from my cases.
- Determinism: groups follow declared order (a `Vec` with a linear find), and occurrences are `BTreeMap`s. No hash-order dependence was found.
- `FORMAT_RELEASES` has `ess-diff/10`, `ess-conformance/30` and `/31` as `None`. `xtask generate --check` and `schema --check` are clean.

## 6. Paths written outside the worktree

- ~/.cache/ess-wave-n2/binding/adv2/review.md
- ~/.cache/ess-wave-n2/binding/adv2/red-ess-conformance.log
- ~/.cache/ess-wave-n2/binding/adv2/red-ess-diff.log
- ~/.cache/ess-wave-n2/binding/adv2/suite-ess-conformance.log
- ~/.cache/ess-wave-n2/binding/adv2/suite-ess-diff.log
- ~/.cache/ess-wave-n2/binding/adv2/xtask-generate_--check.log, ~/.cache/ess-wave-n2/binding/adv2/xtask-schema_--check.log
- ~/.cache/b10x-target/ess-c-binding (the assigned build dir; test binaries were added)

Also outside the worktree: a harness copy `~/.cache/ess-wave-n2/binding/adv2/harness.rs` and `~/.cache/ess-wave-n2/binding/adv2-harness.rs` were created and deleted within the session. The worktree lease `adv2-binding-context` was acquired and then released.

## 7. Findings block

```findings
- file: crates/verify/ess-conformance/src/synthesize/delivery_context.rs
  line: 200
  category: mutant
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a Boolean or enum leaf inside a payload struct carries the same value as a same-typed context field in both deliveries and separated() compares only top-level values, so a target reading the nested payload value passes every scenario
- file: crates/verify/ess-conformance/src/synthesize/delivery_context.rs
  line: 219
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: for an odd-sized enum the two occurrence distinctions coincide modulo the variant count, so a three-variant context field is refused as having one witness value
- file: crates/verify/ess-conformance/src/synthesize/delivery_context.rs
  line: 233
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the widened distinctions push a bounded newtype outside its invariants, and the resulting witness gap refuses the whole binding including flow and on-failure
- file: crates/verify/ess-diff/src/diff.rs
  line: 1762
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a wire or summary change on a context field is reported as a cause change with byte-identical sides in text and JSON and bumps the delta to ess-diff/10
```
