---
format: aep.planning-md/3
id: story:unknown-identity-answered-by-an-existing-refusal
kind: story
status: draft
title: An unknown identity is answered by an existing refusal outcome
relations:
- depends_on: story:read-only-command-answers-an-unknown-instance
- depends_on: story:wrong-state-for-an-unknown-identity-states-its-rule-and-repair
revision: 1
---
## Summary

An adopter answers an identity no record carries exactly like a record the caller may not see, so a response never discloses whether a record exists. ESS cannot state this. `unknown_instance:` is a separate outcome that the conformance report compares by name (ESS-CF-OUTCOME), and the validator refuses `unknown_instance: true` beside another condition on an existing refusal such as `denied` (`crates/specify/ess-domain/src/command.rs:6271-6293`, "declares `unknown_instance` beside another condition"; the adopter reports ESS-COMMAND-004 on 0.59.0).

Two things are wanted:

1. A command declares that an unknown identity is answered by an existing refusal outcome. The synthesized `<Command>/outcome/unknown-instance` scenario then expects that outcome.
2. The synthesized `<Command>/outcome/denied` scenario forces the refusal with an arrangement that seeds the record, not only through an empty read.

No GitHub issue: this comes from an adopter's report.

## Spec first

The change is modelled in this repository's ESS specification and validated with the newest `ess` before any code. A design section in `docs/design/typed-literals-and-unknown-instances.md` comes first. It chooses between `unknown_instance: true` on the refusal outcome itself and a command-level key naming the outcome that answers. It also settles how this sits beside a separate `unknown_instance:` outcome, which still means "not found". If either form changes what an existing document means, the section names the format consequence (ess/25).

## Acceptance

Each line names the failing test that is written first:

- `crates/verify/ess-conformance/tests/unknown_instance.rs::an_unknown_identity_answered_by_the_denied_refusal_expects_denied`: the unknown-instance scenario expects the declared refusal outcome, and an implementation that answers "not found" fails it.
- `crates/verify/ess-conformance/tests/unknown_instance.rs::the_denied_scenario_seeds_the_record_it_refuses`: the `denied` scenario arranges the record before it sends the command.
- `crates/specify/ess-domain` validation: a refusal outcome declared to answer an unknown identity validates. A second outcome that also claims the unknown identity is refused by name.

## Order and links

This story comes after 0.60.0 and waves k to n. It shares the unknown-instance code paths of `synthesize.rs` with `story:read-only-command-answers-an-unknown-instance` and `story:wrong-state-for-an-unknown-identity-states-its-rule-and-repair`, so it depends on both.

## Estimate

About 10G (ess-conformance tests). Adversary: yes, because existence disclosure is the property under test.
