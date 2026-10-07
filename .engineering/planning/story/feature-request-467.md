---
format: aep.planning-md/3
id: story:feature-request-467
kind: story
status: implemented
title: A generated Optional event field goes through a generator port
tags:
- ess-0.55.0
refs:
- provider: github
  reference: beyond10x/ess#467
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-06T11:58:57Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-06T11:58:57Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-10-07T00:53:35Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1,"verification":1}}}
---
# A generated Optional event field goes through a generator port

## Acceptance

`ess generate synthesize --target rust` and `--target go` give a payload field whose value is
`{generated: true}` and whose type is `Optional<T>` a generator port that returns an optional `T`
(`Option<T>` in Rust, the target's optional representation in Go), as a non-optional
`{generated: true}` field gets `generate_<t>`; the generated behaviour builds the event from that
port's answer, and a synthesis test shows a present and an absent answer each reaching the emitted
event. A payload field that nothing sets stays absent, with no port.

## Context

beyond10x/ess#467, filed 2026-10-06 against 0.52.0. A downstream consumer had to leave two
computed optional fields off its event.

## Fit review

1. **Need.** A value the implementation computes, absent only when there is nothing to report,
   cannot reach an event: `{generated: true}` on an `Optional` field always yields absent. Minimal
   reproduction: an event field `{name: rate, type: Optional<Decimal>, generated: true}` emitted by
   any outcome. The requester's proposal: a generator port returning `Option<T>`.
2. **Class: defect.** `{generated: true}` means the implementation supplies the value
   (`crates/generate/ess-synth/src/rust/behaviour.rs:2152`, `generate`), but `assigned`
   (`rust/behaviour.rs:2143-2150`, same in `go/behaviour.rs:2958`) answers `None` for every optional
   target, so the declaration is ignored for that type. It is reached from an explicit
   `ResolvedPayloadValue::Generated` (`rust/behaviour.rs:2299`) and from a field nothing sets
   (`rust/behaviour.rs:2136`).
3. **Already expressible?** No. Leaving the field unset gives the same constant absent value.
4. **Fit.** No new authored surface: the declaration already parses and validates. Only the
   explicit `Generated` caller changes; the unset-field caller keeps `None`. Rust and Go both
   change; the interpreter already mints a value for `Generated` (`ess-conformance/src/interpret/execute.rs:2101`).
5. **Second adopter.** A shipment event with `delivered_at: Optional<Timestamp>, generated: true`,
   set only when the carrier reported a delivery.
6. **Cost.** Generated-API change: a specification that already declares an optional generated
   payload field gains one port method in its generated Rust and Go ports, which an implementor
   must supply. No format bump, no new diagnostic.
7. **Considered.** (a) Change nothing and document "leave it unset": rejected, the value is lost.
   (b) A port returning `T`, absent never: rejected, it cannot say "nothing measured". (c) The
   requester's `Option<T>` port: taken.

## Decisions

**Accept as proposed**, limited to the explicit `{generated: true}` caller; the field nothing sets
keeps its absent value. The changelog names the new port method under Changed.

## Milestone

`release-plan:ess-055`.
