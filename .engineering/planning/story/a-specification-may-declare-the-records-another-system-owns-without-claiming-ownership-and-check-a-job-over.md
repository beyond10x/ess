---
format: aep.planning-md/3
id: story:a-specification-may-declare-the-records-another-system-owns-without-claiming-ownership-and-check-a-job-over
kind: story
status: draft
title: A specification may declare the records another system owns, without claiming ownership, and check a job over them
summary: 'No construct declares record shapes without ownership, and provided: true still projects the entity as owned and leaves synthesis refusing ESS-SYNTH-001; waits on decision-blocker:unowned-store-records (gap report, 2026-11-24)'
tags:
- external-request
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: architecture-decision-record:a-deployment-may-provide-an-entitys-rows
- informed_by: story:a-command-writes-only-rows-its-own-component-stores
revision: 2
---
## Need

A downstream application reported on 2026-11-24 (tenth gap report; reporter name kept out). A system-issued job acts on
records in a relational store another system owns, and that system has no specification. The specification must
declare the record shapes it relies on without claiming ownership, read, count and delete records by filter, and still
let a suite check which rows go, in what order, at most how many, and that a dry run changes nothing, against a test
double.

Status: architecture decision pending (SW-D1). The Acceptance below is written for option A and changes with the pick.

## Reproduction

Installed 0.98.0, fixture `demo.ret` (entities `Tenant`, `Batch`, `Entry`), `Entry` marked `provided: true`, no
creating command:

| Probe | Result |
|---|---|
| `ess specify validate` | valid |
| `ess verify conform synthesize` | `ESS-SYNTH-001 ... demo.ret.Entry.tenant_id ... which a filter reads and no declared command writes` |
| authored `ess-scenario/2` with `arrange:` `setup:` rows for `Entry` | compiles, 7 of 7 pass on the interpreted target; the generated refusal stays listed |
| `ess generate --kind schema` | emits `schema/entities/demo.ret.Entry.schema.json`: the entity is projected as owned |
| `ess-transport` with `protocol: mysql`; with a `stores:` key | `unknown variant mysql, expected nats`; `unknown field stores, expected one of type, specification, brokers, channels, streams` |
| a binding to the owner's command | `ESS-BINDING-001 ... is not a declared command` (owner has no specification) |
| a system actor holding a bulk delete | validates; synthesis `ESS-SYNTH-023` |

## Findings covered

- No construct declares record shapes without ownership; `provided` is consumed only in `ess-compiler`, `ess-domain` and
  `ess-conformance`, so generators project the entity as owned.
- Synthesis cannot arrange fields of a provided row (`ProvidedFields`, `Unreachable::Provided`); with no creating command
  every filter field is `ESS-SYNTH-001`.
- Transport is event transport with a closed `Nats` protocol, and composition binds commands of an owner that has a
  specification.

## Fit review

1. Need: rely on rows ESS does not own, perform select, count and delete on them, and check the job against a test
   double.
2. Class: gap, ARCHITECTURE. It reopens the accepted ADR `a-deployment-may-provide-an-entitys-rows` (option B: the
   provider returns `{entity, state}`), and touches the component-ownership rule
   (`story:a-command-writes-only-rows-its-own-component-stores`, draft rev 12).
3. Existing idiom: an owned stub domain with seed commands, or an authored scenario with `setup:` rows. Policy 1 holds
   for neither: the stub claims ownership and generates owner artifacts; the authored scenario passes but leaves the
   generated suite refusing `ESS-SYNTH-001`.
4. Fit: extend `provided` (option A) rather than add a store document (option B). A reader of
   `docs/design/pre-execution-fixture-values.md` § Provided rows finds that provided rows carry fields and that the
   deployment is the writer, not the specification.
5. Second adopter: any specification of a service that reads or cleans another team's table, a legacy store without a
   contract, or a vendor database.
6. Cost: A, no new source key, a change to generated output of provided entities, a records port per generated target,
   synthesis of provided rows with fields; B, a new key, format row, IR field, diff class and a protocol enum; C, none.
7. Alternatives: B and C of SW-D1, and a store section in `ess-transport` (closed enum, event concern).

## Decisions

Deferred. `decision-blocker:unowned-store-records` blocks this story and this story waits on it.

SW-D1 (ARCHITECTURE): how a specification relies on records another system owns. Open, with the owner; no default. A: extend `provided` (rows carry fields to the suite through `setup`, generators stop projecting a provided entity as owned, a records port per provided entity, no creator command needed). B: a new declaration of an external records store with its entity shapes (new source key and format row, closed protocol enum, effects limited to select, count, delete). C: no construct (an owned stub domain with seed commands, the rest listed as unmapped lines). The coordinator recommends A, as the smallest construct that makes the check real and adds no source key; the owner decides. The Acceptance below is written for A and changes with the pick; format follows the pick (A needs no source key, B opens a format by policy 2).

## Acceptance

Written for option A.

- provided_rows_are_arranged_with_their_fields: a command whose filter reads fields of a provided entity yields
  scenarios whose arrange step supplies those rows through `setup`, with no `ESS-SYNTH-001`; selected rows read absent
  and kept rows read as arranged.
- a_command_that_only_selects_counts_or_deletes_provided_rows_needs_no_creator: a specification with no command that
  creates the provided entity validates and synthesizes.
- a_provided_entity_is_not_projected_as_owned: `ess generate --kind schema` and the storage and API targets emit no owner
  artifact for it; the diff classifies the change for an existing provided entity.
- the_records_port_is_checked_against_a_double: the generated target's records port (select, count, delete by filter) is
  exercised by the suite; a delete-nothing mutant and a count-then-delete mutant fail the dry-run scenario.
- effects_beyond_select_count_delete_are_refused: `creates:` or `updates:` of a provided entity is refused by name.
- unimplemented_targets_refuse_by_name: a target with no records port refuses by name.

## Scope

- crates/specify/ess-domain/src/entity.rs  cited — `provided` (1015, 2198)
- crates/specify/ess-compiler/src/ir.rs  cited — `owner_of` (3671)
- crates/verify/ess-conformance/src/synthesize/provided.rs  cited — `serves`, `search`, `ProvidedFields`
- crates/generate/  inferred — every generator that projects an entity (schema, ports, API, Rust, Go)
- docs/design/pre-execution-fixture-values.md § Provided rows, website/docs/guides/synthesize.md:292, `architecture-decision-record/a-deployment-may-provide-an-entitys-rows.md`  cited

## Format

None for A (a change of projection for existing provided entities, classified by the diff). B opens a format by policy 2.
