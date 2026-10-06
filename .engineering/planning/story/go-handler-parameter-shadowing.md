---
format: aep.planning-md/3
id: story:go-handler-parameter-shadowing
kind: story
status: implemented
title: A Go component handler parameter never shadows an input domain package
refs:
- provider: github
  reference: beyond10x/ess#414
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T19:17:27Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-04T19:17:28Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-06T09:43:34Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

A Go component handler parameter never shadows an input domain package (beyond10x/ess#414).

## Origin

Opened on GitHub as beyond10x/ess#414; added to the bundle on 2026-10-04 under the operator goal that every open issue is solved and merged through the integration branch. Issue text (first part):

> A compiled no-external-port model with domain `input` generates a Go component that does not compile. Found while checking #412 no-use/owed-only boundary cases; this is a pre-existing owning-emitter defect, not a typed callback regression.
> 
> Source: `crates/generate/ess-synth/src/go/port.rs`, handler around lines196–260, unchanged from base2f554561bef25125a93a1fb1d6517d50cb24ed20. It emits `func (c *Port) Method(input input.Command)` and later `case input.CommandOutcomeAccepted:`. The local `input` hides the imported package.
> 
> Real generated build output:
> ```
> components/renewalservice/renewalservice.go:83:13: input.OwedOutcomeAccepted is not a type
> components/renewalservice/renewalservice.go:100:13: input.PlainOutcomeAccepted is not a type
> ```
> 
> Reproducer is the `no_use_and_owed_only_models_compile_without_an_external_port` fixture in the #412 owned branch (`crates/generate/ess-synth/tests/external_request_binding.rs`, source SHA25659be63088f38a633e423011ec135ebabeb296a19267c579acf3eccfe7383b537). Focused command: `cargo test -p ess-synth --test external_request_binding --locked --offline -- no_use_and_owed_only_models_compile_without_an_external_port --test-threads=1 --nocapture`. The observed two-case run exited101,0passed2failed6filtered; full log SHA2565e5c4444bc5c9559e66e8c160a934f490ae132be42cbea4dd9781d7e938c8350.
> 
> Expected: valid domain names compile without local/import collisions, for generated and owed component behavior. Fix allocation in the owning Go emitter an
