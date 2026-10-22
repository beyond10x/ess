---
format: aep.planning-md/3
id: story:an-input-path-reads-a-list-element-and-a-fallback-chain-tries-several-places
kind: story
status: draft
title: An input path reads a list element, and a fallback chain tries several Optional places
summary: 'An input path stops at a List, else: may not fall back to an Optional member and does not nest, so a value taken from one of several body places cannot be stated (adopter item 20, 2026-10-22); same on 0.67.0.'
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

Reported by an adopter on 2026-10-22 (item 20 of the recheck batch, second team's list L18, relayed by the orchestrator;
adopter names kept out). A stored value comes from one of several places in a request body: a top-level field if present, else
the first element of a list (`emails[0].value`), else a member of an optional object. beyond10x/ess#142 (closed) gave one wire
name per field, not a choice between places.

Needed: an input path into a list element (`input.emails.first.value` or an index), and a fallback chain of more than one step
whose intermediate steps may be Optional, ending in a value that is always present.

## Reproduction

Neutral fixture, `format: ess/29`, domain `demo.server`: struct `EmailEntry {value}`, struct `Profile {email}`, entity
`Contact {email}`, command `AddContact(email: Optional<String>, profile: Optional<Profile>, emails: List<EmailEntry>)` creating
it. `ess specify validate`, 0.68.0 and 0.67.0 byte-identical:

| `sets: {email: …}` | result |
|---|---|
| `{input: email, else: input.emails.first.value}` | ESS-COMMAND-002 `type_mismatch`: "`input.emails.first.value` reads past `input.emails`, which is `List<demo.server.EmailEntry>` and has no members; a path follows struct fields only" |
| `{input: email, else: input.emails.0.value}` | same refusal |
| `input.emails.first.value` | same refusal |
| `{input: email, else: input.profile.email}`, `profile` Optional | ESS-COMMAND-002: "`input.profile.email` is `Optional<String>`, which may be absent, and `else:` promises a value" |
| `{input: email, else: {input: profile.email, else: ""}}` | reader error: "`else:` admits `{generated: true}`, a literal (format ess/16) or `input.<path>` (format ess/22)" |
| `{input: email, else: input.profile.email}`, `profile` required (control) | valid; 1 scenario, 1 passed on the interpreter |

## Where it lives

- `crates/specify/ess-domain/src/command/input_path.rs:213`: the `Opaque` arm, a path stops at a List.
- `crates/specify/ess-domain/src/command/value_expression.rs:1523`: an Optional fallback is refused.
- `crates/specify/ess-domain/src/command.rs:2119`: `else:` takes no nested `{input: …, else: …}`; the same reader message as
  recheck item 11 (`else: {subject: …}`).

## Regression status

Not a 0.68.0 regression: byte-identical outputs and suites on 0.67.0. `value_expression.rs` changed in `0.67.0..52bd110dd`
(`071b98401`, coalescing an Optional candidate field in a selector `where`), but these refusals did not move. Gap. Does not
block 0.68.0.
