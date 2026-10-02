# Scenario initial state and cross-caller witnesses

Binding decision for issue #312. This extends the unreleased ordinary/coverage suite formats
34/35. It introduces no source-language construct, authored-scenario spelling, report/2 field or
additional format major. Previously released suite formats 1–33 retain their original bytes and
unspecified initial-state contract.

## Initial-state authority

Every newly synthesized suite declares typed provenance `scenario_initial_state: empty`. Before
each scenario's setup and commands, its target must provide an empty logical namespace of modeled
instances, publication observations and invocation history. This does not require deleting a
physical database or unrelated tenant data. A fresh target, a fresh scenario namespace or another
equivalent isolation mechanism can satisfy it. Externally provisioned resources remain explicit
obligations, and declared upstream setup populates the namespace after scenario begin.

Successful `begin_scenario` accepts this requirement; it does not prove that a physical reset
occurred. An adapter unable to establish that context must return Unsupported before command
callbacks. Runners do not invent a reset operation or silently remove fixture resources.

Formats 34/35 require the declared value `empty`; missing, null or unknown values are invalid.
Legacy formats 1–33 must not acquire this member or a retroactive claim of shared-target safety.
Fresh ordinary synthesis selects format34 even when its steps need no other recent feature;
coverage production selects35. Selection preserves the requirement and the exact parent binding.
The full run report retains suite provenance. CountReport/2 stays closed and unchanged: its exact
paired suite remains the authority. Human run diagnostics display the requirement before target
execution, and browser catalog/replay displays it without claiming a real backend reset.

The public Rust provenance struct gains a typed optional member to represent legacy absence.
This is a source-compatibility change for external struct literals and must be documented; serde
defaults preserve old wire input, not Rust struct-literal compilation. Generated readers must
admit the same closed vocabulary and bounds. Old readers reject the new major before callbacks.

## Same-row caller evidence

The model's actor and caller declarations are the only principal authority. Where two eligible
callers are expressible, synthesis arranges a row as one and acts on that exact row as the other.
It does not reset, reinstall or create a replacement between those operations. This applies to
direct subjects and related-row dependencies. A credential-partitioned store must therefore fail
when the source describes one shared row.

Caller-sensitive branches are synthesized under their actual mixed assignment. Merely rewriting
an actor after choosing an outcome could turn a legitimate refusal into a fabricated success.
Existing owner/nonowner refusals and input/subject/related guard precedence remain authoritative.
Use one deterministic pair and role reversal where expressible to detect implementations that
special-case the first principal; arbitrary caller permutations are not required.

Attributed actors use distinct declared attribute values. Two attribute-free declared actors
remain distinct even with identical grants. One attribute-free actor does not authorize inventing
a second principal. Existing singleton reuse stays intact: finite identity exhaustion must not
cause a duplicate singleton installation. Uncomposable aggregate/count/order, replay, fixture,
binding or periodic families retain a specific coverage explanation, never a false shared-row
claim from two independently created rows. Existing scenario identifiers are reused where their
observation can be extended honestly; any additional identifier design needs separate review.

## Decisive validation

The retained UUID partition-by-caller adversary is the deciding regression: establish its failure
against the original synthesis before changing production. Require the healthy store to pass,
the partitioned store to fail a named scenario, and a trace showing distinct arranging/acting
callers with one identical row identity and no intervening installation. Preserve singleton and
caller-owned refusal controls, add equal-grant attribute-free actors, and verify swapped roles.

Initial-state controls retain unrelated physical sentinel data while isolating scenario rows;
an incapable adapter refuses before commands. Legacy documents, malformed new provenance,
coverage selection/parent bytes and human/browser output are checked explicitly. Execute the same
serialized suites with real callbacks through native, generated Go, generated TypeScript and
supported WASM paths, comparing all status categories and diagnostic codes without normalization.
Interpreted execution consumes its separately owned caller/related capabilities; this change
does not add another interpreter. Browser replay remains explicitly replay.
