---
format: aep.planning-md/1
id: blocker:live-pilot-consumer-coverage
kind: blocker
status: open
title: Live conformance entries and changed profiles lack consumer qualification
relations:
- blocks: story:compact-live-authoring
- blocks: story:callable-live-evaluator
withholds: test_result
revision: 1
---
## Observation

The full `task check` at source 65ed702ab2d5ec1162c35571d124750c106e84f5
passes formatting, strict Clippy, all workspace tests including the formerly
failing metadata tests, rustdoc, command smoke, projection and support checks.
It exits 201 at consumer-check with:

`unclassified concrete consumer entry ess_cli::bin(ess)::enum::ConformCommand/variant/Live; finite review required`

The fresh consumer inventory has 145 entries absent from entry-classifications.json,
including the new CLI boundary and compact compilation, recipe, binding and trace
modules. The refusal preserves diagnostic accounting for 2,430 model obligations
and 90 bound profiles, with 66,571 provisional stale/unaccounted messages. No
behavior cases or metadata guards executed in this refused consumer run, and no
completed qualification receipt exists. These provisional messages are not 66,571
executed test failures and must not all be attributed to this change: the original
model-source roots are unchanged from the new upstream base.

Retained evidence: ess-rebased-check.log, ess-consumer-refusal.json and
ess-unclassified-live-entries.json. Native extraction remains under the fresh
consumer run ending in run-1789545179881235728-3544796. The original full-gate
failure is preserved. Independently invoked fuzz-check, release-check and
action-check pass; they do not replace the refused lane.

## Clearing condition

Review and classify the finite new entries under docs/design/review-consumer-coverage.md.
Compare fresh upstream-base accounting with this candidate to distinguish inherited
model drift from changed live-conformance consumer profiles. Preserve the sealed
initial baseline and existing profile claims. Add exact supported/refused cases
for the new obligations and execute them through the native checker. New entries
cannot inherit package-wide support and changed profiles cannot reuse old unknown
eligibility. A corrected classification file alone is not behavioral qualification.

Run task consumer-check and the complete repository gate to zero before landing.
Feature checkpoints remain published for review; no merge or release is claimed.
