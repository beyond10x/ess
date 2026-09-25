---
format: aep.planning-md/2
id: review-result:wave25-unit2-handback
kind: review-result
status: active
title: 'Unit 2 handback: a name is declared where it is written'
relations:
- reviews: story:a-masked-first-declaration-hides-a-duplicate-name
revision: 1
---
# Unit 2 handback: a name is declared where it is written

Worktree `wt-c2495d2e6f3a`, uncommitted over `wave/ess-wave-25` at `e5a97603`. Green.

```
 crates/specify/ess-compiler/tests/typed_diagnostics.rs |  20 ++
 crates/specify/ess-domain/src/spec.rs                  | 337 +++++++++++++---
 docs/design/review-typed-diagnostics.md                |  13 +-
```

## The change

`insert` asked *is this name in the registry* — which a declaration whose own `try_from` failed
never reaches, so the second copy took the name silently. Replaced by `declare` (*has this name been
written*, asked **before** the conversion is attempted) plus `record` (which keeps the first
declaration's meaning). `Collected` gained `declared: BTreeSet<(&'static str, String)>`.

`ValidationError::new` count in the file is unchanged, so the design page's machine-checked
inventory needed no edit.

## Gate, every command's own exit status

```
cargo test   --locked -p ess-domain   --no-fail-fast     0    451 → 453
cargo test   --locked -p ess-compiler --no-fail-fast     0    126 → 126
cargo test --workspace --exclude ess-xtask --locked      0   2907 → 2909  (293 binaries, before and after)
cargo clippy --locked -p ess-domain   --all-targets -- -D warnings    0
cargo clippy --locked -p ess-compiler --all-targets -- -D warnings    0
RUSTDOCFLAGS='-D warnings' cargo doc --locked --no-deps -p ess-domain     0
RUSTDOCFLAGS='-D warnings' cargo doc --locked --no-deps -p ess-compiler   0
cargo fmt --package ess-domain --package ess-compiler -- --check          0
ess validate --path examples/billing                     0    "billing v3 — 5 file(s), valid"
```

`ess-compiler` is flat on purpose: no case was added there, an existing `assert_eq!` was tightened
from five expected refusals to six.

## The class, enumerated

Rule: *a name is declared where it is written, whether or not what is written under it converts.*
Members = every declaration kind absorbed through a `try_from` into a registry.

| kind | masked before | outcome |
|---|---|---|
| entity, command, event, error, view | yes | fixed, with a case |
| actor | no — `ActorSpec::try_from` documented as unable to fail | fixed anyway, control row |
| component, binding | yes | fixed, with a case |
| conversion | no — refused by `ConversionRegistry::insert` without converting anything | reasoned, stated in the test doc |
| topology | no — `topology_source` is set before the conversion | reasoned, stated in the test doc |
| **declared type** | **yes — still masked** | left open, bound written into the test's doc comment |

The masking runs in both directions; `a_name_declared_twice_is_refused_in_all_four_soundness_combinations`
pins all four.

## The judgement the brief asked for

**The fixture keeps its duplicate `shop.repeat.Solo` and the pinned list gains the sixth refusal.**
Deleting the second `Solo` would delete the only thing in the tree that makes a needle genuinely
ambiguous, and answering a finding by removing the evidence is the mistake that fixture's own design
page records twice. The new refusal is itself `<document>`/`None`, so it strengthens the unlocated
half rather than costing it. Measured delta: exactly +1 refusal, the five existing `Cited` entries
byte-identical, `design_page_matches_the_fixture.rs` passing unchanged.

`docs/design/review-typed-diagnostics.md:340-342` said `Solo`'s refusals are "both" unlocated. They
are now three. Changed to "all three", with a paragraph saying where the third comes from. Both
premise strings the tests quote are untouched, and the fixture YAML is untouched — its line numbers
12/35/56 are pinned by two tests and by the page.

## The open member, stated as a bound rather than implied closed

A declared **type** is still masked. Its duplicate is reported by `SystemSpec::merge`/`Assembly::claim`
in `system.rs` — a second reporter, in a file this unit was not given, with a different location
(`types.<name>`) and a message naming both sources. Closing it means moving that report or teaching
`SpecPart` to carry a name whose declaration failed; either changes an existing diagnostic's
contract for every already-reported case. The unit wrote the bound into the test's doc comment
rather than letting eight fixed kinds imply nine, and left no patch: the change needs a design
decision, not a hunk.

## Reported against the tree rather than hidden

One pre-existing flake, on the **untouched base**: `ess-cli` `execution_recovery.rs:2528`
`an_effect_before_failure_and_a_lost_acknowledgement_are_both_indeterminate` failed once
(`left: None, right: Some(101)`), passed when re-run alone, did not recur in either post-fix
workspace run. A load-sensitive exit-code assertion.
