---
format: aep.planning-md/3
id: review-result:external-request-binding-design-1
kind: review-result
status: active
title: Review typed request binding across generated Rust and Go
relations:
- reviews: story:external-request-binding
revision: 1
---
approve

Design review of story:external-request-binding revision 17, proposed, on base 2f554561bef25125a93a1fb1d6517d50cb24ed20. Reviewed story SHA256: 3e2b5258dbbebd2172a94511a5daa1523649e8683fca0dbdba3cf61df9395462. This report supersedes the earlier Rust-only scope assessment and its sibling-scope addendum; both remain unchanged. The revised design has no unresolved design finding. This is approval of the implementation plan, not a passing implementation or runtime qualification.

The need is a generated API gap. Two requests can have the same addressed identity and revision but different deadlines, payment values or evidence. The existing Rust and Go callbacks receive only command/outcome names, so a context decision prepared for one request cannot inspect a substituted request in a direct generated call. A wrapper around one transport does not close another direct entry path. The owner needs the actual typed input at the external decision boundary.

The proposed Rust `ExternalCommand<'a>` closed enum borrows each generated command input. The required `Context.external(command: ExternalCommand<'_>, outcome: &'static str) -> bool` receives it, and `name()` returns canonical qualified command identity. Go uses its existing sealed-interface convention: `ExternalCommand` exposes `Name()` and a private marker, with concrete typed value wrappers and a required `Context.External(command ExternalCommand, outcome string) bool`. Both retain declaration order, local-guard precedence and first-true branch selection. The input-bearing API enables exact binding; it is not authentication. The owner must still verify authority, evidence, freshness, stable command time and fail-closed decisions. Go retains ordinary nested-value aliasing and does not acquire Rust borrow-checking guarantees.

The accepted refinements close the identified design risks:

- Allocate deterministic collision-free enum/wrapper names. Qualified Pascal flattening alone is not injective. Use the existing Layout/Emit-resolved payload paths and code aliases; use Go's allocated input local rather than hardcoding a potentially shadowing name. Reserve competing generated identifiers and test colliding candidates.
- Emit the request type and required callback only for commands whose actually generated behavior uses external branches. No-use and owed-only specifications must remain valid, including Rust lifetime correctness. Cover deterministic output, Rust crate/workspace layouts, Go package paths and naming.code overrides.
- Establish a semantic RED on the old API that compiles and admits a substituted request, then preserve literal outcome, storage and event assertions while explicitly migrating the context fixture. A future-API compilation failure alone is not that regression. Separately require old callback implementations to fail compilation after regeneration.

The scoped production change is limited to `crates/generate/ess-synth/src/rust/behaviour.rs` and `crates/generate/ess-synth/src/go/behaviour.rs`. Both already collect context use and render external calls. Existing compiled harness implementations and signature expectations migrate in the explicitly listed test files, including both declared-behaviour harnesses. New compiled request-binding cases belong in `crates/generate/ess-synth/tests/external_request_binding.rs`; the binding design belongs in `docs/design/external-request-binding.md`. No authored syntax, generic metadata bag, JSON/Any input conversion, new dependency, plan change or conformance-synthesizer change is needed. Coordinator ownership of both renderers is established before the later overlapping issue319 work.

Acceptance can run against actual generated code. Existing Rust served_publication tests compile and execute both Rust layouts. Existing declared_behaviour Go tests synthesize, format, vet and compile a module, then exercise its component and HTTP paths. Extend those patterns with a small brand-free model: exact request succeeds; changed deadline/evidence or missing proof refuses with no storage/event effects; local guards precede external calls; successive external outcomes receive the same current input; exact large integers and optional/nested values arrive without conversion. Retain old assertions, refusal controls and signature-migration diagnostics. An unavailable required toolchain is an unexecuted requirement, not a pass.

The compatibility decision is explicit: regeneration breaks existing Rust and Go context implementations until they adopt the required signature. Previously generated artifacts remain unchanged until regenerated. A closed Rust enum also means a newly added external command can require exhaustive consumer matches to change. Document these generated API consequences; do not hide them with permissive defaults. There is no authored/IR/suite format change.

Sibling coverage is coherent. Go generates this behavior and is now included. ESS synthesis has Rust, Go, Web and Clap targets; Web hosts the generated Rust behavior and requires supplied context/storage ports. There is no separate TypeScript behavior callback renderer here; TypeScript schema-normalization generation is a different surface. Interpreter, schema and conformance semantics remain unchanged, and no cross-target authentication guarantee is introduced.

Source basis: Rust behaviour.rs:177,263,297,987; Rust layout.rs:159,180; Go behaviour.rs:167,443,478,1350,1584; Go mod.rs:139; Go items.rs:1–12; tests/served_publication.rs:254–324; tests/declared_behaviour.rs:684–856; lib.rs:97–119; web/mod.rs:22–29,337–342, all under crates/generate/ess-synth except where prefixed otherwise.

No source, existing test or planning artifact was edited. No tests, builds, generation or live invocations were run during this design assessment. Only assigned review scratch and the required reviewer lease bookkeeping were written. Implementation remains subject to its semantic regression, focused checks, independent adversary review and the coordinator's required integration gate. No publication or release is authorized by this report.

```findings
[]
```
