---
format: aep.planning-md/3
id: story:the-specification-checks-command-response-wire-names
kind: story
status: draft
title: The specification checks command response wire names
summary: Two response fields of one wire name are refused by the specification pass, so every target is covered, not only Rust and web
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

Two fields of one command response may share a wire name. The specification pass
(`crates/specify/ess-domain/src/wire.rs`) checks the wire names of command input fields only, and
`docs/design/unique-wire-field-identity.md` records that the response namespace is checked only by
Rust and web synthesis (batch 2, beyond10x/ess#659 adversary round). Go and clap synthesis, by
grep and not by run, emit such a response with one JSON member written twice.

## Acceptance (provisional)

- The specification pass refuses two response fields of one command whose wire names are equal,
  with the existing duplicate-declaration code, at the second field.
- Go and clap synthesis no longer reach the case; the Rust and web `wire-collision` refusal stays
  as a second line of defence.
- The design page's namespace table says the specification pass checks the response namespace.
