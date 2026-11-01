---
format: aep.planning-md/3
id: story:when-subject-beside-a-row-set-validates-under-the-g2-format
kind: story
status: draft
title: A when_subject guard beside a row-set guard validates under the G2 format
summary: ESS-COMMAND-009 refuses when_subject beside a row set at every format; the G2 record lifts it in its own format.
tags:
- downstream
- follow-up
relations:
- informed_by: story:a-guard-on-the-input-addressed-parent-row-holds-on-every-command-of-its-children
- decomposes: epic:generic-row-queries
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/command/row_set.rs
revision: 2
---
## Problem

- A command that addresses a parent row (`unknown_instance: true`) and reads its child rows through
  a row-set `when_related: {entity, where, exists}` cannot also refuse on the parent row with
  `when_subject:`.
- Validation refuses the pair as `ESS-COMMAND-009`: "one command reads one kind of related row in
  this cut" (`crates/specify/ess-domain/src/command/row_set.rs:39-42`).
- `ess/32` lifted the identity-addressed `when_related:` beside a row set and kept this pair
  refused at every format (`CHANGELOG.md:112`).
- An adopter met it on a tenant rule: "not found when the parent row belongs to another tenant"
  beside "not found when no child row matches". Reproduced on 0.77.0 with a neutral model, in a
  repro outside the repository.
- Split out of `story:a-guard-on-the-input-addressed-parent-row-holds-on-every-command-of-its-children`.
  That story fixes the synthesis half and leaves this validation half here, because it needs a
  source format.
- The design is decided. "ESS-COMMAND-009's one-kind-of-row refusal is lifted with G2"
  (`docs/design/generic-language-features.md:470-471`).
- G2's single source format is allocated when its first wave lands (`:552-556`, wave 4 at `:631`).
- Until then the comparison is written inside the row set's `where:`, which validates,
  synthesizes and passes the interpreted target for a required field.

## Acceptance

- Under G2's format, `when_subject:` beside a row-set `when_related:` validates.
  - The precedence rule orders the pair: the addressed record's guards answer before the related
    rows (`docs/design/generic-language-features.md` G4).
  - Below that format the pair keeps `ESS-COMMAND-009`, its message unchanged and its hint naming
    the format.
- The interpreter follows the rule.
- Synthesis witnesses every branch of such a command or names the branch it cannot.
- Generated Rust and Go keep the command an obligation, named.
- The tests are named when the G2 wave is scoped.
