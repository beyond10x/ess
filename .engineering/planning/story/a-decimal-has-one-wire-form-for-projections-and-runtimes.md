---
format: aep.planning-md/3
id: story:a-decimal-has-one-wire-form-for-projections-and-runtimes
kind: story
status: draft
title: A Decimal has one wire form for projections, generated servers and runtimes
summary: OpenAPI and generated servers read a Decimal as a string; the value table and suites spell it as a JSON number, so a runtime using the table refuses what the contract declares (adopter report)
tags:
- backlog
- external-request
- from-adopter-report
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-wire-and-answers
- informed_by: story:primitive-canonical-serialization
- informed_by: decision-blocker:decimal-canonical-serialization-crosses-repositories
revision: 1
---
## Need

ESS states two forms for a `Decimal`. The OpenAPI and JSON Schema projections and the generated
Rust and Go servers read and write it as an exact decimal string. The value table
(`Primitive::admits`, `crates/specify/ess-domain/src/types.rs`) admits it as a JSON number only,
and a synthesized suite spells a `Decimal` input as a number. A downstream runtime that checks
the JSON of a command input with the value table therefore refuses the value a client generated
from the OpenAPI document sends.

On 0.93.0 (`format: ess/37`, command `demo.t.Make` with input `baseline: Decimal`):

`ess generate --path model --kind openapi --strict --out out/openapi` writes

```yaml
baseline:
  type: string
  format: decimal
  pattern: ^-?(0|[1-9][0-9]*)(\.[0-9]+)?$
```

`ess verify conform synthesize --path model --out out/suite.json` writes the same input as

```json
{"kind": "literal", "value": 1.0}
```

The value table's only `Decimal` row is `(Self::Decimal, Node::Number(number))`; the generated
Rust decoder reads `json::decimal_at(…, "a decimal string")`
(`crates/generate/ess-synth/src/rust/wire.rs:1021`).

Reported by the adopter and not rerun here: a runtime in another repository, with a specification
compiled by 0.90.0, answers 400 "`baseline`: "10" is not a Decimal" for the string and accepts
the JSON number `10`.

## Acceptance

- The design page states one wire form of a `Decimal` in a command input, a view answer and an
  event payload, and which ESS function a runtime outside the generated servers asks whether a
  JSON value is a `Decimal` in that form.
- That function admits what the projection declares: a value the OpenAPI schema of a model
  accepts is admitted, and a value it refuses is refused, for `Decimal` and for every other
  primitive; a test compares the two over a pinned table of values.
- A suite value and a persisted report keep their bytes unless the decision below is reopened.
- The reference page on types says which form a client sends.

## Fit review

Not done. The operator decided on 2026-10-23
(`decision-blocker:decimal-canonical-serialization-crosses-repositories`) that the exact-decimal
writer for suites and reports is not built and that reopening it is a new blocker with
`Decider: operator`. Admitting the declared string form on input moves no persisted bytes; the
fit review says whether that is the whole request or whether the suite's spelling must follow.
Not tagged `defect`: the projection and the generated servers agree with each other, and the
value table is documented for candidates, setup values, payloads and authored examples, not for
wire input.

## Evidence

The OpenAPI schema, the suite value and the two source lines were read for this story on
`ess 0.93.0` and the branch head `bc2035531f`. The 400 is the adopter's observation on 0.90.0.
