---
format: aep.planning-md/3
id: story:an-optional-member-sent-as-null-is-answered-as-the-openapi-answers-it
kind: story
status: draft
title: An Optional member sent as null is answered as the served OpenAPI answers it
summary: Rust server admits null for an Optional member the served schema refuses (W1 GEN1 review, pre-existing, backlog)
tags:
- backlog
- defect
- review-finding
- severity-medium
- zero-open-issues
relations:
- informed_by: review-result:security-w1-gen1-pass-1
- decomposes: epic:zero-open-issues-2026-11
revision: 4
---
## Need

An `Optional` command member sent as `null` is taken as absent by the generated Rust server
(`decode_member`, `crates/generate/ess-synth/src/rust/wire.rs:915-930`, pre-existing, same arm at
b81deac78 `:883`) and answered 202, while the served OpenAPI request schema has no `null` branch and
refuses the body. The CHANGELOG records it as a known limit (beyond10x/ess#707 release notes,
"Known limit (not a fix)"). Found by review-result:security-w1-gen1-pass-1.

Reproduction on 0.92.0 (generated output only, no build needed). A command with an `Optional`
member, served over the network:

```yaml
commands:
  - name: probe.net.CheckPort
    input:
      - {name: port, type: Integer}
      - {name: note, type: Optional<String>}
    ...
components:
  - component: checker
    owns: {domains: [probe.net]}
    accepts: {commands: [probe.net.CheckPort]}
    reached_by: network
```

```text
$ ess generate synthesize --path spec --target rust --out out
5 capabilities: 5 generated, 0 obligation(s), 0 refused
# out/crates/probe-server/src/checker.openapi.json
components.schemas.probe.net.CheckPort.Input.properties.note = {"type": "string"}   required = ['port']
# out/crates/probe-server/src/wire.rs
        note: match value.member("note") {
            None | Some(json::Value::Null) => None,
```

A body `{"port": 1, "note": null}` is refused by the served schema and admitted by the server.

## Acceptance

- For an `Optional` member sent as `null`, the generated Rust server and its served `openapi.json`
  give the same answer: either the request schema of the member admits `null` (`anyOf` with
  `{"type": "null"}`, or `type: [..., "null"]`), or the server answers `{"port": 1, "note": null}`
  with `400` (or the binding's `refusals.malformed`) naming the path `note` at decode. The design
  records which.
- `an_optional_member_sent_as_null_is_answered_as_the_served_document_answers_it` in
  `crates/generate/ess-synth/tests/adversary_w1_gen1_security.rs` drives one generated Rust server
  with that body, compares its answer with the served document, and passes.
- An absent `Optional` member is still admitted, and the CHANGELOG known-limit line is replaced by
  the fix.

## Scope

- cited: crates/generate/ess-synth/src/rust/wire.rs
- cited: crates/generate/ess-synth/tests/adversary_w1_gen1_security.rs
- inferred: crates/generate/ess-gen/src/openapi.rs
- inferred: crates/generate/ess-synth/src/go/http.rs
- inferred: CHANGELOG.md

confidence: medium

## Schedule

Backlog (D398): not waved; considered only after approval and after every predefined wave is released.
