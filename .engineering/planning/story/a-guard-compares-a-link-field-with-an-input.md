---
format: aep.planning-md/3
id: story:a-guard-compares-a-link-field-with-an-input
kind: story
status: implemented
title: A when_subject guard comparing a link field with an input is refused (ESS-SYNTH-003)
refs:
- provider: github
  reference: beyond10x/ess#193
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-28T21:12:07Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-28T21:12:08Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-09-29T06:23:41Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1,"review_outcome":2,"verification":1}}}
---
## Outcome

A `when_subject` guard that compares a link field with an input (`account_id != input.account_id`) is synthesized: the subject-fact input search can send an arranged instance as an input value, and the `!=` branch arranges a second owner. Today it is refused as ESS-SYNTH-003 (beyond10x/ess#193, part 2).

## Evidence

- The viewfilter unit (fc27494ee9) fixed the view-filter and aggregate cases; the guard case needs instance-valued inputs in `synthesize/subject_fact.rs` input selection, which the view-filter tokens (`synthesize/identity.rs`) do not reach.
- Red test: `~/.cache/ess-wave-n2/viewfilter/guard-case-probe.rs`.

## Acceptance

- the probe, added as a conformance test, synthesizes with no refusal;
- a target whose guard ignores the link, and one comparing the wrong owner, each fail the synthesized suite;
- committed suites regenerate byte-identical.
