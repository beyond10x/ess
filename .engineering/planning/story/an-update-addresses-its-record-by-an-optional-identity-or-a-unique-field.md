---
format: aep.planning-md/3
id: story:an-update-addresses-its-record-by-an-optional-identity-or-a-unique-field
kind: story
status: draft
title: An update addresses its record by an Optional identity, falling back to a unique lookup by another stored field
summary: 'instance: takes only a field name, and branching on the presence of an identity or an Optional identity is refused, so "by id, else by email" with one not-found answer cannot be stated (adopter item 18, 2026-10-22, related to beyond10x/ess#564); same on 0.67.0.'
tags:
- adopter-report
- ess-0.70.0
- ess-0.70.0-first
- gap
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
---
## Need

Reported by an adopter on 2026-10-22 (item 18 of the recheck batch, second team's list L13, relayed by the orchestrator;
adopter names kept out). An update addresses its record by the identity the caller sends, and when the caller sends no
identity, by another stored field that identifies the row (an email address). One command, one answer for "no such row" on
either path.

Needed: `instance:` (or a sibling) addresses the subject by an Optional identity input with a fallback to a unique row-set
lookup, and `unknown_instance:` answers when neither finds a row. Related open request: beyond10x/ess#564 (queries over rows);
none of its stories (`story:row-queries-navigate-a-reference`, `story:navigation-reaches-any-depth-in-view-filters-and-guards`,
`story:a-guard-compares-fields-of-two-named-rows`) addresses the subject itself by a non-identity field.

## Reproduction

Neutral fixture, `format: ess/29`, domain `demo.server`: entity `Account` (identity `account_id`, fields `email`, `label`),
`Register` creating it, and `Relabel(account_id: Optional<AccountId>, email, label)` with an `updates:` branch and
`unknown_instance:`. `ess specify validate`, 0.68.0 and 0.67.0 byte-identical:

| `Relabel` shape | result |
|---|---|
| `instance: {input: account_id, else: {where: email == input.email}}` | `invalid type: map, expected a string` (reader error, no code) |
| `instance: account_id`, `account_id` required (control) | valid |
| two branches: `when: {account_id: {exists: true}}` + `instance: account_id`, else `instances: {where: email == input.email}` | ESS-COMMAND-003 `unobservable_fact`: "an identity is opaque, so no branch may be chosen by reading one" |
| same, decided by a `by_email: Boolean` input instead | ESS-COMMAND-002 `type_mismatch`: `account_id` "is typed `Optional<demo.server.AccountId>`; `demo.server.Account` is identified by `demo.server.AccountId`" |

The nearest expressible form is two commands, one by identity and one by `instances: {where: email == input.email}`. The second
is a set effect: zero matching rows is an accepted answer, so "no such account" is not stated on the email path.

## Where it lives

- `crates/specify/ess-domain/src/command.rs:6897`: `instance: Option<String>` on the raw outcome, hence the reader error.
- `crates/specify/ess-domain/src/command.rs:3475`: the refusal of a branch decided by the identity it addresses.
- `crates/specify/ess-domain/src/entity.rs:1408`: the refusal of an Optional input naming the instance.

## Regression status

Not a 0.68.0 regression: all four shapes give byte-identical output on 0.67.0. Gap. Does not block 0.68.0.
