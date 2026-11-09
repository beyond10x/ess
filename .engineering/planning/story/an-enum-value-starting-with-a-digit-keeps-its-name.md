---
format: aep.planning-md/3
id: story:an-enum-value-starting-with-a-digit-keeps-its-name
kind: story
status: draft
title: An enum value that starts with a digit keeps its name in Rust and has a TypeScript value object
summary: 'spindle wave 12: 2xx..5xx become V0..V3 and get no TS value object (ess 0.88.0)'
tags:
- backlog
- defect
- from-consumer
- queue-0.91.0
- rank-5
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/generate/ess-gen/tests/enum_variant_names.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/ts.rs
- confidence: inferred
  path: website/docs/guides/generate-artifacts.md
revision: 5
---
## Need

Reported by a downstream consumer against ess 0.88.0, reproduced on 0.92.0. An enum whose values
start with a digit is generated as the Rust variants `V0`..`V3`, and the TypeScript target emits no
value object for it, so the consumer keeps its own copy of the values.

```yaml
format: ess/7
system: demo
version: v1
domain: demo.http
types:
  - name: demo.http.StatusClass
    kind: enum
    variants: [2xx, 3xx, 4xx, 5xx]
  - name: demo.http.Colour
    kind: enum
    variants: [red, green]
```

```
$ ess generate types --path spec.yaml --target rust --package demo-types --all-types --out rs
$ ess generate types --path spec.yaml --target typescript --all-types --out ts
```

`rs/types.rs` holds `DemoHttpStatusClass { V0, V1, V2, V3 }` (each `#[serde(rename = "2xx")]` …);
`ts/types.ts` holds `export const DemoHttpColour = {…} as const` but only the type alias for
`DemoHttpStatusClass`.

Cause: `enum_variants` (`crates/generate/schema-contract/src/realize.rs:1464-1471`) records no
declared names for the whole enum as soon as one identifier starts with an ASCII digit;
`variant_identifiers` (`realize.rs:870`) then falls back to `V{index}`, and `members`
(`realize/ts.rs:257`) emits no value object because `names_variants` is false.

Workaround today: `variants: [{name: S2xx, wire: 2xx}, …]` under `format: ess/32` names the variants
and emits the value object.

## Acceptance

- `ess generate types --target rust` over the enum above generates the variants `V2xx`, `V3xx`,
  `V4xx`, `V5xx` (the `PascalCase` identifier of the value, prefixed with `V` when it starts with a
  digit), each `#[serde(rename = "<value>")]`. The `ess generate types` guide (`website/docs/guides/generate-artifacts.md`) states the rule.
- `ess generate types --target typescript` over it emits
  `export const DemoHttpStatusClass = { V2xx: "2xx", … } as const;`, as it does for any other enum;
  Go names the constants by the same identifiers.
- An enum where the prefixed identifiers collide with another variant's identifier keeps `V<index>`
  as before.
- Generated output for enums whose values do not start with a digit is byte-identical; a test in
  `crates/generate/schema-contract` or `crates/generate/ess-gen/tests/enum_variant_names.rs` pins
  both cases.

## Scope

- cited: crates/generate/schema-contract/src/realize.rs
- cited: crates/generate/schema-contract/src/realize/ts.rs
- inferred: crates/generate/ess-gen/tests/enum_variant_names.rs
- inferred: website/docs/guides/generate-artifacts.md

confidence: high
