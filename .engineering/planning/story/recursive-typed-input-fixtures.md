---
format: aep.planning-md/3
id: story:recursive-typed-input-fixtures
kind: story
status: draft
title: Conformance synthesis admits finite recursive typed input fixtures
refs:
- provider: github
  reference: beyond10x/ess#416
relations:
- serves: vision:O2
revision: 1
---
## Outcome

Conformance synthesis admits finite recursive typed input fixtures (beyond10x/ess#416).

## Origin

Opened on GitHub as beyond10x/ess#416; added to the bundle on 2026-10-04 under the operator goal that every open issue is solved and merged through the integration branch. Issue text (first part):

> ESS 0.52.0 validates a finite recursive input type and generates Rust representations for it, but conformance synthesis refuses a typed fixture reachable through `List<T>` or `Optional<T>`. This blocks EKR's `ImportInterpretation` and `SubmitSchemaProposal` generated scenarios. It is separate from the recursive Rust layout fixes shipped in 0.52.0.
> 
> Reproduced with the released Linux x86_64 binary, SHA-256 `7c0f35fd2c2365c2390113f756eba1275ea25902d04f7ca5a8f8f6ab30395be7`. No ESS source changes or unreleased binary are needed.
> 
> Minimal two-file reproduction follows.
> 
> `system.yaml`:
> 
> ```yaml
> format: ess/14
> system: fixtureprobe
> version: v1
> domains:
>   - fixtureprobe.recursive
> ```
> 
> `domains/recursive.yaml`:
> 
> ```yaml
> domain: fixtureprobe.recursive
> types:
>   - name: fixtureprobe.recursive.Value
>     kind: struct
>     fields:
>       - name: text
>         type: String
>       - name: children
>         type: List<fixtureprobe.recursive.Value>
> actors:
>   - name: fixtureprobe.recursive.Caller
>     may:
>       - fixtureprobe.recursive.Submit
> events:
>   - name: fixtureprobe.recursive.Accepted
>     fields:
>       - name: accepted
>         type: Boolean
> commands:
>   - name: fixtureprobe.recursive.Submit
>     input:
>       - name: value
>         type: fixtureprobe.recursive.Value
>     fixture_inputs:
>       value: finite-value
>     response:
>       - name: accepted
>         type: Boolean
>     outcomes:
>       - name: accepted
>         emits:
>           - fixtureprobe.recursive.Accepted
>         payload:
>           fixture
