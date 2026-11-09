---
format: aep.planning-md/3
id: story:the-specification-checks-command-response-wire-names
kind: story
status: draft
title: The specification checks command response wire names
summary: Two response fields of one wire name are refused by the specification pass, so every target is covered, not only Rust and web
tags:
- defect
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-domain-language
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Problem

Two fields of one command response may share a wire name, and `ess specify validate` accepts it.
The specification pass (`crates/specify/ess-domain/src/wire.rs:28-34`) checks the wire names of
command input fields only; `docs/design/unique-wire-field-identity.md:39-41` records that the
response namespace is checked only by Rust and web synthesis. Go and clap synthesis emit such a
response: Go writes the JSON member twice (the second overwrites the first) and the served OpenAPI
lists the member twice in `required`.

Minimal reproduction (ess 0.92.0), `demo.yaml`:

```yaml
format: ess/22
system: demo
version: v1
domain: demo.core
events:
  - name: demo.core.Sent
    fields:
      - {name: id, type: String}
commands:
  - name: demo.core.Send
    input:
      - {name: text, type: String}
    response:
      - {name: reply, type: String}
      - {name: other, type: String, naming: {wire: reply}}
    outcomes:
      - name: sent
        returns: true
        emits: [demo.core.Sent]
        payload:
          demo.core.Sent:
            id: {generated: true}
actors:
  - name: demo.core.Writer
    may: [demo.core.Send]
components:
  - component: messages
    owns: {domains: [demo.core]}
    accepts: {commands: [demo.core.Send]}
    publishes: {events: [demo.core.Sent]}
    reached_by: network
```

- `ess specify validate --path demo.yaml` prints `valid`, exit 0.
- `ess generate synthesize --path demo.yaml --target go --out out` exits 0; `server/wire.go` holds
  `out["reply"] = value.Reply` and `out["reply"] = value.Other`; `server/messages.openapi.json`
  lists `"reply"` twice under `required`.
- `--target rust` exits 1 with `maps multiple fields to JSON member reply`.
- Control: the same two fields under `input:` are refused by validate with `duplicate_declaration`
  / `ESS-COMMAND-006` at `command.demo.core.Send.input[1]`, exit 1.

## Acceptance

- `ess specify validate --path demo.yaml` (the model above) exits 1 with `duplicate_declaration` /
  `ESS-COMMAND-006` at `command.demo.core.Send.response[1]`, naming `response[0]` as the earlier
  field, as the input namespace does today. A test in `crates/specify/ess-domain` asserts it.
- A response may still share a wire name with an input field of the same command (the two are
  separate namespaces); a test asserts that model validates.
- `ess generate synthesize --target go` and `--target clap` no longer reach the case; the Rust and
  web `wire-collision` refusal stays as a second line of defence.
- The design page's namespace table says the specification pass checks the response namespace.

## Scope

- cited: crates/specify/ess-domain/src/wire.rs
- cited: docs/design/unique-wire-field-identity.md
- inferred: crates/specify/ess-domain/tests/ (a wire-namespace test beside the input case)
- inferred: crates/generate/ess-synth/tests/adversary_ess_issues_2_u8.rs (`two_response_fields_of_one_wire_name_do_not_reach_the_codec` then takes its refused-before-synthesis arm)

confidence: high

## Fit review

1. Need: a command response whose two fields serialize to one JSON member is accepted; the
   second value silently replaces the first on Go. Reproduced above (ess 0.92.0).
2. Class: defect. `docs/design/unique-wire-field-identity.md` declares the response a wire
   namespace of its own; validation enforces every other namespace (`wire.rs:9-63`).
3. Already expressible: not applicable; nothing to express, the model is already invalid on Rust.
4. Fit: reuses the existing `check_fields` pass and the `DuplicateDeclaration` code; no new
   vocabulary, all targets benefit at once.
5. Second adopter: any model that renames a response field with `naming.wire` hits it.
6. Cost: no format bump, no new diagnostic code; a model accepted today and refused afterwards was
   already unbuildable on Rust and wrong on Go.
7. Considered: change nothing (Go and clap keep emitting a duplicate member); a Go-side refusal
   only (leaves clap and every later target to repeat it). The specification pass wins.

Found by the adversary round of beyond10x/ess#659 (`review-result:ess-issues-2-u8-adversary`),
not an adopter request.

## Decisions

- Accept as a defect: the specification pass checks `command.<name>.response` like `input`.
