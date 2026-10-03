---
format: aep.planning-md/3
id: story:feature-request-400
kind: story
status: draft
title: Preserve wire labels and recursive values in generated Rust contracts
refs:
- provider: github
  reference: beyond10x/ess#400
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

An adopter can generate Rust contracts preserving punctuation in enum wire labels and finite recursive Optional value shapes. Distinguish the existing standalone data-library route from the remaining component/server synthesis layout limitation; neither is proof of the other.

## Origin

GitHub beyond10x/ess#400, refreshed2026-10-03. The report says a valid baseline fails Rust synthesis on0.36.0 and0.51.0, with invalid-identifier enum variants and recursive-layout for Optional self-reference. Its implementation plan needs generated contracts, preserved serialized labels and a verified release. The report explicitly does not request an authored Box keyword. Full downstream files were not copied into this repository.

## Fit review

1. Need independent of syntax: preserve a wire literal such as demo.packet/1 while producing a legal host declaration; represent finite nested values whose element field is Optional of the same named record. Brand-free probes enum-literal.yaml, enum-named.yaml and recursive.yaml in private carrier ess-400-fit-20261003 each validate with installed ESS0.51.0, exit0. Literal enum and recursive component synthesis each refuse, exit1, reproducing the report's two diagnostic classes.

2. Classification: enum host naming is already expressible, so that part is a convenience/idiom, not a new language gap. Explicit {name: PacketV1, wire: demo.packet/1} validates and synthesizes Rust successfully, exit0. Recursive component layout is a target capability gap with an intentional existing refusal, not evidence of invalid ESS semantics: ess-synth/tests/feasibility.rs719/724 pin self/mutual recursion, rust/feasibility.rs934 explains Optional preserving size, and website/docs/reference/formats.md192 lists recursive-layout as an admitted target failure.

3. Existing expression: docs/design/enum-variant-wire-names.md32–40 already defines separate semantic name and wire spelling. rust/wire.rs143–151 uses variant.wire() for encoding. For standalone generated contracts, released0.51.0 ess generate types succeeds for BOTH explicit enum labels and recursive Optional records. schema-contract/src/realize/rust.rs290 emits boxed named references. Generated enum library uses serde rename demo.packet/1; recursive library uses EssPresence<Box<DemoCoreValue>>. An actual private Rust probe compiled both generated libraries offline and ran2tests,2passed0failed: exact enum encode/decode with rejection of the host label as wire, and exact round-trip of {}, {element:{}}, and {element:{element:{}}}. This is measured codec proof for the minimal cases, not validation of the adopter's entire application.

4. Fit and siblings: keep existing name/wire vocabulary and the existing data-library command. Do not add Box or another authored transport/serialization keyword. If the consumer also needs component/server generation, use deterministic internal indirection for recursive layout and preserve source/wire meaning; inspect all Rust type construction, codec, behavior, binding, view and Web-dependent paths before adopting an implementation. Go and other targets must continue explicit capability reporting; no silent omission. Runtime behavior, conformance and diff semantics are unchanged merely by host allocation. Scope and design must be confirmed before source dispatch.

5. Second adopter: a document editor's recursive value descriptor and a query service's recursive predicate tree both require finite optional child records. A versioned message envelope and a file-format discriminator independently need wire strings containing punctuation while host variants remain identifiers. These examples require no adopter-specific names.

6. Cost: the demonstrated idioms require no ESS upgrade, source-format bump or implementation change. Component synthesis support would change output for models currently refused; preserve bytes/API for already supported nonrecursive models. Boxing may affect generated Rust APIs, internal copying, constructors and codecs, so compiled recursive and mutual-recursive consumer tests are required. No version number is reserved by this draft.

7. Alternatives: change nothing and keep both target refusals does not help an adopter selecting the wrong generator; explicit enum name/wire plus generate types already satisfies the minimal data-contract need and is preferred there. Adding an authored Box keyword leaks target representation into semantic source and is rejected. Automatic deterministic indirection is the candidate for the distinct component-layout gap, subject to a binding design and full affected-target tests; it is not claimed implemented by the standalone library proof.

## Decisions

Accept, redesigned: split the request into the existing released contract-generation idiom and the remaining component-layout capability. Preserve the complete issue scope; do not close the issue merely because two minimal contracts work. No implementation starts from this draft before actual component requirements and scope are reconciled with the generator owners. Current UI/server coordinator confirms400 is outside its five owned units; transport ownership390–395 remains elsewhere. No owner of400 source implementation has yet been assigned.

## Acceptance

Retain the three source-admission/synthesis probes and two real Rust codec tests. Demonstrate equivalent explicit wire labels in generated component codecs if that component path is selected. A future recursive component change must compile/run finite self-recursive and mutual-recursive records through constructors and codecs, preserve exact wire data and nonrecursive output, and keep uninhabited/unsupported structures as truthful named refusals. Reconcile the consumer's required generated artifacts against these capabilities before an issue disposition or release claim.

## Scope

Cited for existing idiom: enum-variant-wire-names design, ess-synth/src/rust/wire.rs and schema-contract/src/realize/rust.rs. Inferred component change: ess-synth/src/rust/feasibility.rs, rust/items.rs, shared type renderer and constructor/codec consumers; actual files require scoping. No changes to transport390–395, current318 bundle or runtime312/interpreter work are authorized by this intake.

## Evidence

Private carrier ess-400-fit-20261003: all three validate.exit files0; enum-literal.synthesize.exit1, enum-named.synthesize.exit0, recursive.synthesize.exit1; enum-named.types.exit0 and recursive.types.exit0; codec-proof.exit0,2tests passed. Probe executable source is Rust outside every repository. No remote write, source commit, gate or release was performed for this assessment.
