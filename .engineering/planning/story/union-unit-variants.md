---
format: aep.planning-md/3
id: story:union-unit-variants
kind: story
status: draft
title: A union variant may be declared without a payload
refs:
- provider: github
  reference: beyond10x/ess#418
relations:
- serves: vision:O2
revision: 1
---
## Outcome

A union variant may be declared without a payload (beyond10x/ess#418).

## Origin

Opened on GitHub as beyond10x/ess#418; added to the bundle on 2026-10-04 under the operator goal that every open issue is solved and merged through the integration branch. Issue text (first part):

> A union variant must carry exactly one type (ess-domain/src/types.rs at 0.52.0: `variants: BTreeMap<String, TypeRef>`); there is no unit type and an empty struct is refused. A specification therefore cannot express a sum type that mixes payload-less and payload-carrying variants, such as `Open | Complete { outcome }`.
> 
> Observed 2026-10-04 in beyond10x/commission story:port-skeleton (probe: `Open:` with no type is refused with `invalid type: unit value, expected a string`). Commission uses a stand-in `Unit` newtype of Boolean until this exists.
> 
> Expected: a union variant may be declared without a payload, generating a unit variant (Rust `Open`, TypeScript/Go equivalents) and round-tripping through the JSON codecs.
> 
> Acceptance idea: a union with one unit variant and one struct variant validates, compiles, synthesizes with 0 refusals and generates a Rust enum with a unit variant.
