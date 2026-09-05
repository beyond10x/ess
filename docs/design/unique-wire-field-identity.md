# Unique Wire Field Identity

## Invariant

Before a specification is admitted to any projector, every field in one JSON object
namespace has a distinct effective wire key: `field.naming.wire` when present, otherwise
`field.name`. Renaming must not let ordered-map insertion erase another declaration.
An empty or punctuation-bearing JSON key remains legal; its uniqueness is still checked.
Display names do not participate. Equality is exact and case-sensitive, not a host-language
identifier normalization.

## Namespaces

| Construct | One namespace |
| --- | --- |
| Named struct | Its fields |
| Entity | Identity, declared fields, and the synthetic `state` property |
| Command | Input fields, not outcome names or event mapping expressions |
| Event or error | Payload fields |
| View | Output fields (inline or referenced shape) |
| View parameters | A separate input namespace, never merged with the output row |

Struct fields under a referenced type remain nested, not flattened. Tagged unions
retain the existing adjacent tag/content layout; the existing `value`/`content` fallback
keeps their two generated keys distinct. Relations annotate declared fields and introduce
no additional wire property. No speculative flattening or runtime decoder rules are added.

## Enforcement

One domain-owned validation pass runs in `Specification::validate`, which assembly and
compiler entry both invoke. It reads the complete typed specification, including named
view shapes. It accumulates `DuplicateDeclaration` findings at each later conflicting
field's semantic source location and names the earlier field and effective key. The
synthetic entity state is reserved first, so a conflict points at the authored identity
or field rather than a generated property. Ordinary noncolliding models retain their
canonical bytes and projections; there is no persisted format change.

Keep the accounted model-selection guard as defense in depth, but test that the normal
specification path now refuses collisions before it can reach that guard. Cover all
namespaces, explicit versus default keys, entity identity/state conflicts, named view
shapes, cross-object reuse, valid special keys, and accumulated source-located errors.
