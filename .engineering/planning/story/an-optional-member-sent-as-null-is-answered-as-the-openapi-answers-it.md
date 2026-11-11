---
format: aep.planning-md/3
id: story:an-optional-member-sent-as-null-is-answered-as-the-openapi-answers-it
kind: story
status: active
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
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/rust/wire.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/adversary_w1_gen1_security.rs
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:40:30Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:40:32Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":1}}}
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

All cases are in `cargo test -p ess-synth --locked --test adversary_w1_gen1_security`.

- `an_optional_member_sent_as_null_is_answered_as_the_served_document_answers_it` drives one
  generated Rust server with `{"port": 1, "note": null}` for an `Optional<String>` member with no
  `presence:`: the server answers `400` with a `refused` detail starting `body.note: `, and the
  served document's request schema refuses the same body.
- The same test sends `{"port": 1}`: the server answers `2xx` and the served document admits it.
- The same test sends `{"port": 1, "note": null}` for a member declaring
  `presence: omitted_or_null` (`ess/31`): the server answers `2xx` and the served document, whose
  property is `anyOf [string, null]`, admits it.
- `every_closed_position_refuses_its_undeclared_member_as_the_document_does` passes with its
  `optional-null` request expecting the answer the served document gives.

## Decisions

- The server follows the served document rather than the document admitting `null`: an `Optional`
  field without a policy is published "optional and not nullable"
  (`website/docs/guides/specify/wire-names.md`, "Say whether an absent Optional is sent as null"),
  and `presence: omitted_or_null` is the declared way to admit `null`. A member that a generated
  page may send as `null` (page state of an `Optional` type starts as `null`,
  `crates/ui/ess-ui-react/src/types.rs:140`) declares that policy.
- The generated Go server admits `null` the same way (`optional`,
  `crates/generate/ess-synth/src/go/http.rs:2320`); it is outside this story, whose finding and
  test harness are the Rust server's.

## Scope

- cited: crates/generate/ess-synth/src/rust/wire.rs
- cited: crates/generate/ess-synth/tests/adversary_w1_gen1_security.rs

confidence: medium

Proposes the `CHANGELOG.md` sentence that replaces the known-limit line; moves no other shared
surface.

## Schedule

Backlog (D398): not waved; considered only after approval and after every predefined wave is released.
