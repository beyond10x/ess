# Closed-enum outcome coverage

Commands may omit a default only when a finite proof establishes exactly one input
branch for every assignment in a closed domain. The initial fragment is required
enum facts (including transparent newtypes and struct paths), equality/inequality
with text literals, membership, and Boolean combinations of those predicates.
It uses the existing expression type environment and predicate evaluator; it does
not derive the domain from the guard's literals. At most 64 joint assignments and
128 predicate nodes are admitted. Every referenced input participates in the joint
domain. Unreferenced inputs do not affect the proof.

Optional paths, open scalar domains, collection selectors, quantifiers, unsupported
operations, invalid literals and exhausted resource bounds cannot establish this
proof. Unknown is never false and never proves coverage. Such commands retain the
requirement for a genuine default. Missing cases name a real enum assignment;
overlapping guards name the conflicting outcomes and assignment. A branch whose
guard is never true remains declared and receives the existing synthesis refusal.

Raw command shape checking has no registry. It may defer the missing-default check
only for the syntactic finite fragment over named input roots. Full typed validation
must establish the proof before the command becomes a validated specification.
Shape checking still rejects multiple defaults and other structural defects.

The finite analysis is transient Rust data, shared by domain validation and resolved
conformance witness construction via the existing TypeEnvironment adapters. Witness
construction applies its declared assignments to the ordinary typed input builder,
then synthesis re-evaluates actual flattened inputs and requires unique branch
selection. No seventh value or invented default is introduced. Completed candidates
are first checked by the same bounded typed-value validator used for entity setup,
including nested newtype and struct invariants. False or Unknown removes that
candidate; it does not stop the remaining bounded search or prove unsatisfiability.
An unreachable outcome retains its named refusal, whose trial count includes only
admitted candidates actually decided against its guard (zero when none was admitted).
The finite domain proof remains conservative over every declared enum value; filtering
execution witnesses does not weaken source coverage validation or add an invariant solver.
Existing commands with defaults retain the order and bytes of their admitted candidates;
previously emitted invariant-invalid inputs now produce honest branch refusals.
External and wrong-state outcomes retain their current strategies; their conditions
are not treated as enum alternatives or as evidence covering input.

This adds no source syntax, persisted fields, new diagnostic code or serialized proof.
Previously rejected exhaustive commands become valid; existing admitted commands
with real defaults keep their semantics. No format migration is required. Held-state
guards and consumer adoption are separate work.

Deciding checks cover both measured six-value report shapes, every branch's concrete
witness, omitted values, duplicate/overlapping guards, transparent wrappers, extra
inputs, Optional absence, Unknown and proof bounds, retained unreachable outcomes,
and compatibility of defaults/external/wrong-state behavior. Source-only consumer
inspection is not adoption evidence.
