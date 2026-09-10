# Typed collection facts

The ESS-to-ER comparison needs an independent observer of compiled command inputs. The
expression checker admits List cardinality/ordinals and List/Map value quantifiers, but at
`568ee569` the conformance input binder only checks collection containers. Primitive FactStore
probes do not establish that a compiled input can supply those facts.

## Contract

`input::bind` publishes `<collection>.count` for every supplied, non-null List or Map, including
empty collections. It recursively projects each value under `<collection>.<ordinal>`, starting
at zero. Lists retain sequence order. Maps enumerate values in the lexical key order of Node's
BTreeMap; the key is neither a binder field nor part of the fact path. A key named `count`, `0`,
or containing punctuation cannot overwrite cardinality or manufacture nested facts. Direct Map
key/ordinal expressions remain inadmissible; ordinal facts are the primitive evaluator's
internal binding representation for Map quantifiers.

Optional omission/null binds nothing. Null optional elements still occupy their ordinal and
count toward cardinality. Thus empty forall is true, empty exists is false, and an absent
collection or absent scalar operand remains Unknown, subject to the existing Kleene rules.
Nested quantification and lexical scope use the existing primitive evaluator unchanged.

The shared binder recursively checks supplied elements with the existing primitive, enum,
newtype and struct rules and accumulates located errors. This applies to command inputs,
event/error assertions and view-row assertions. Partial binding permits omitted top-level
fields, not malformed supplied elements or incomplete required struct members. No partial
FactStore escapes an erroneous bind. Each element descent counts toward MAX_TYPE_DEPTH.

Input path classification recognizes published cardinality and List scalar paths. View
predicate capability remains separate: runner::row_facts does not publish sequence facts and
cannot be certified by the typed input binder. Authored/synthesized row predicates must retain
their existing refusals for collection reads.

## Existing limits and compatibility

This changes an in-memory projection, not a persisted envelope, source grammar, scalar encoding
or evaluator. Previously admitted malformed collection contents now fail shared shape checks.
Map-key codecs are not validated by this change. Union values retain their existing outer-map
check and do not project a discriminant or selected payload. Binary64 still has no FactValue
projection. Missing binder-body facts can produce an unclassified Unknown diagnostic; they
must never become false or select a default outcome. Witness generation remains bounded search,
not a complete solver. These limits remain explicit in the ER lowering account.

## Evidence required

Tests compile real ESS declarations and evaluate their guards through public flatten, rather
than constructing a FactStore. Cases cover collection counts, List ordinals, map value order,
empty/absent/null distinctions, nested scopes and free variables, malformed nested values,
partial binding and depth refusal. Existing authored row refusals remain regression controls.
A missing count projection must make a focused test fail. Only affected package checks run;
the operator has excluded the full repository gate.
