# Optional input narrowing

Status: implemented (beyond10x/ess#169; `story:optional-input-narrowed-after-refusal`). Admitted
under source format `ess/16`. Below it, a narrowed read is refused as `type_mismatch`, the same
refusal it got before.

## Problem

An endpoint takes an account from an optional parameter. It answers a declared error when the
parameter is absent and otherwise creates the record in that account. After the refusing outcome,
the parameter is known to be present, and the success outcome copies it into a required field. The
type checker read `input.account_id` as `Optional<AccountId>` in every branch and refused the copy.
The repair it suggested, `conversions: [{from: Optional<AccountId>, to: AccountId}]`, is
domain-wide and unconditional. It went on validating after the refusing outcome was deleted, so it
admits the crossing the refusal exists to catch.

## Rule

In a branch `B` of a command, an input `x` declared `Optional<T>` is read as `T` when either holds:

1. **Own guard.** `B`'s condition carries an input predicate that requires `defined(x)`: the
   predicate is `defined(x)`, or an `all:` with such a member (recursively). This covers `when:`
   and the input guard of every subject-guarded condition. Each of those is conjunctive eligibility,
   so the branch is never taken while its predicate fails.
2. **Refused absence.** `B` is the command's default branch written with no condition
   (`OutcomeCondition::Otherwise`), and a sibling with `error:` has the condition
   `when: not defined(x)`. `missing(x)` and `{not: "defined(x)"}` parse to the same predicate. The
   default is taken only when no guarded sibling matched, and that sibling matches every request
   without `x`.

`x` must be a top-level input (a one-segment path), and one `Optional` layer comes off.

Neither rule depends on declaration order. The story said "an earlier-declared sibling". Validation
does not read outcomes as first-match: overlapping guards are a `conflicting_declaration` wherever
the finite analysis can decide them. Entity Runtime does try guarded branches in declared order, but
it moves the branch with no condition to the end. So a default written with no condition is taken
last under both readings, wherever the refusal is written. A branch written `when: true` is not the
default and is not narrowed; see the table below.

### What does not narrow, and why

| Shape | Why not |
|---|---|
| a refusal of `not defined(x)` *and* anything else | an absent `x` that fails the other conjunct reaches the default |
| a sibling that succeeds on absence | it refuses nothing, so it proves nothing about the other branches |
| a guarded branch other than the default, beside a refusal of absence | overlap between guards over unbounded types is not refused, so both guards can hold for one request with `x` absent |
| `any:` containing `defined(x)` | another member can hold with `x` absent |
| `x.y` | presence of a member is not presence of the input |
| a branch written `when: true`, beside a refusal of absence | `when: true` is a guard that holds, not the default. Entity Runtime tries guarded branches in declared order, so one declared before the refusal takes a request without `x`. Coordinator decision, correction round 2: only a default written with no condition is narrowed |

## Where it is checked

| Layer | Site | Narrowing |
|---|---|---|
| `ess-domain` | `CommandSpec::narrowed_input` (`command/narrowing.rs`) | the rule above, format-blind |
| `ess-domain` | `check_payload_entry`, `validate_sets` (`command.rs`) through `CommandSpec::admits_input_read` | the declared type and its declared conversions first; then, from `ess/16`, the narrowed type. A refusal names the declared type and hints a crossing from it |
| `ess-domain` | `check_read` (`command/value_expression.rs`) | the same for a leaf of a nested mapping |
| `ess-compiler` | `Resolver::payload_field` (`resolve.rs`) | the same order, gated on the specification's format; without it the repro validates and then fails as `ESS-COMMAND-002` |

`Resolver::crossing` is unchanged. It checks `{subject: …}` and `{input: …, else: …}` sources, and
neither is an `input.<x>` read. `{input: x, else: …}` already reads the present type.

## IR: the read carries `T`

`ResolvedPayloadValue::InputField.type_ref` is documented as the read's type, "which had to be
assignable to the event field's". It records whichever type admitted the copy, checked in this
order: the declared type (assignable, or through a conversion declared from it), then the narrowed
type. When only the narrowed type admits it, `type_ref` is `T` and `conversion` is whatever admitted
`T`, usually `None`. There is no separate narrowed marker, for three reasons:

- The IR stays consistent with itself. Every consumer that reads `type_ref` to decide how to copy the
  value gets the right answer without learning a new flag. Entity Runtime lowers a non-`Optional`
  input read to an unconditional `$args.input.<x>`, and an `Optional` one to an if-present copy
  that would leave a required field unset.
- A marker beside an `Optional` `type_ref` would state two types for one read, and every consumer
  would have to reconcile them. The shape that avoids this is one type.
- The declared type is not lost: `ResolvedCommand.input` keeps `Optional<T>`.

A model that narrows nothing keeps its IR bytes. A crossing declared from the `Optional` type — the
#169 workaround `Optional<T> -> U` — is still honoured under `ess/16` in validation and in the
compiler, and is still the one recorded. The read keeps its declared `Optional<T>` type and its
`conversion` keeps the declared reason, so such a model keeps its IR bytes as well. Narrowing only
adds a way in. Deleting the conversion from a model whose branches all narrow switches the read to
`T` with no conversion. Where neither type is admitted, the refusal names the declared type and
hints a crossing from it, so the repair it suggests is never a domain-wide crossing from the bare
`T`.

## Conformance

No new suite construct. Synthesis already builds the absent witness for `not defined(x)`, and the
default branch's witness sends `x`: its guard search selects input that no sibling guard matches.
The success scenario then asserts the event field and row value equal to the input sent. An
implementation that answers an absent account with a note fails the refusal scenario. One that
refuses a present account fails the success scenario.

## Format

`ess/16` (`FormatVersion::V16`). The domain checks read the format from the `TypeRegistry` they are
handed (`types_with_lifecycles` records it). The nested-leaf check and the compiler read it from the
specification. Below `ess/16` every read keeps its declared type, and the refusal is the
`type_mismatch` it was, with the same message.
