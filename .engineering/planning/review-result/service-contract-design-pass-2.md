---
format: aep.planning-md/2
id: review-result:service-contract-design-pass-2
kind: review-result
status: active
title: Final concrete selected ServiceIr design examination
relations:
- reviews: design:reusable-service-contract
revision: 1
---
approve

Covered design: `docs/design/ess-evolution/service-contract.md` at
`sha256:14ce1677ae0f201d0751e335e69eee91edbe4f8145080d9e4e03987991932159`.
Prior review: `service-contract-design-review-1.md` at
`sha256:6a54e5f8c29b3b32b81e136ac258f7bd2807a3ca7fff0b41d087992003e493c4`.

## Prior finding dispositions

1. **Resolved — incoming ownership closure.** The corrected rule starts from owned, subject, and
   view-source entities, follows outgoing relation targets, and uses
   `EssIr::relations_carried_by` to add the declaring source of each incoming `owns` relation as a
   contextual entity, continuing both directions to a visited-set fixpoint
   (`docs/design/ess-evolution/service-contract.md:76`). This matches the actual representation:
   relations live on their declaring source, an `owns` carrier lives on the target, and
   `relations_carried_by` scans the complete source registry to recover that reverse fact
   (`crates/specify/ess-compiler/src/ir.rs:386`, `crates/specify/ess-compiler/src/ir.rs:403`, and
   `crates/specify/ess-compiler/src/ir.rs:1613`). The foreign source remains contextual and does not
   enter `owned_entities()`. The acceptance contract now requires the asymmetric foreign-owner
   fixture and asserts its lifecycle/types, carried relation, and unchanged local ownership surface
   (`docs/design/ess-evolution/service-contract.md:125`).

2. **Resolved — binding selection versus publication.** The corrected rule selects binding
   capabilities when a binding invokes an accepted operation or its event cause belongs to the
   selected component's exact `publishes` set. It explicitly includes published-but-not-emitted
   causes and excludes emitted-but-unpublished causes unless the binding independently invokes an
   accepted operation (`docs/design/ess-evolution/service-contract.md:88`). This agrees with the
   independent `ResolvedComponent::publishes` surface
   (`crates/specify/ess-compiler/src/ir.rs:1310`), the component model's admitted translation and
   republication shape (`crates/specify/ess-domain/src/component.rs:100`), and the existing Rust
   transport filtering and event roster (`crates/generate/ess-synth/src/rust/port.rs:271` and
   `crates/generate/ess-synth/src/rust/system.rs:145`). The acceptance contract now distinguishes
   both asymmetric cases and retains the independently selected incoming-binding case
   (`docs/design/ess-evolution/service-contract.md:127`).

## Final examination

The corrected design provides one implementable rule for input admission, selected component
surface, complete borrowed operations/outcomes/results/errors/events, contextual entity and type
closure, conversions, actors, bindings, obligations/refusals, ordering, purity, and unchanged format
semantics. The stated tests now cover the two source-confirmed asymmetric cases that were absent in
the first revision. No concrete remaining contradiction prevents implementation of this bounded pure
extraction, and no new generic facility or later ER/runtime semantic is required by this verdict.

Read the complete corrected design, immutable first report, repository/worktree rules, and the
relevant compiler, domain, and synthesis sources needed to recheck both findings and the surrounding
contract. No tests, builds, services, credentials, SQL/database operations, network operations, or
source/store edits were executed. The sole outside write was
`home-path:sha256:0035c821691cdc42a64b54e28a0ab3161c4e03be3ebfd49665732cd4907d3176`.
Lease `ess-evolution-service-contract-design-review-2-sol-20260915` was released on completion.

```findings
[]
```
