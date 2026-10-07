---
format: aep.planning-md/3
id: review-result:consumer-aggregate-wire-pass2
kind: review-result
status: active
title: Aggregate revision defines native admission and retains causal completion dependency
relations:
- reviews: story:feature-request-361
- reviews: story:feature-request-362
revision: 1
---
approve

Root reviewed the immutable aggregate-result-observation-wire-candidate-v2.md, SHA256 a32c5a65d92d132b6356aab3c3a0b2db553f507a9ef066e761d8cbe2fabc64cf, as a normative delta to candidate64d60937c33fe7f0af2cb56a40a8d8c19b3b7fefb823b9019d547a748f472c8d. Source review only: zero adapter builds, round-trip tests or target executions in this pass. The existing actual aggregate and precondition reds remain separate evidence.

```findings
[]
```

The two pass1 design findings are addressed. V2 defines the checked RawSpecFile construction, Specification::assemble and existing compiler path to compiler-minted EssIr, with semantic re-projection equality rather than unchecked deserialization. Public raw structs and assembly entrypoints exist in the inspected carrier; private RawInvariant uses its existing checked Deserialize path. The mapping inventory explicitly covers source-format semantics, lifecycle-derived enums, condition families, nominal/presence rules, conversion rederivation, effects, actor/binding context and typed literal ambiguity. This is a concrete design route, not proof its implementation round-trips every model.

V2 removes delivery_cut/frontier from executable grammar. Binding-affected exact query cuts remain an explicit unresolved same-bundle completion dependency. Unrelated bindings are not a fence, and unsupported partial inventory cannot close the story. No transport interface is invented; all ess-transports development stays with its assigned owner. This satisfies the pass1 alternative of separating an unresolved completion dependency from admitted executable vocabulary, not the eventual binding-support requirement itself.

Approval covers these design choices only. Before implementation, bind the full design page, exact machine-readable scopes, source-to-Contract completeness inventory, finite foreign admission parity inventory, runtime ownership and the shared #292 adapter interface. The first implementation evidence must be actual round-trip/mutation admission across native/Go/TypeScript, followed by complete-prefix execution and actual WASM. Resource proposals remain unmeasured. The full six-function, dynamic fixture/observation, related/set-effect and binding-dependent acceptance stays intact. Recheck source-format publication authority before changing held suite bytes. This review allocates no version and authorizes no production worker by itself.
