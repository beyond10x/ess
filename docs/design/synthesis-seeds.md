# Explicit synthesis seeds

Binding decision for issue #413, part B. Part A (counter reachability refuses incomplete arithmetic
bounds) is a separate correctness fix and is not restated here. This page adds one explicit CLI
option, additive typed Rust library entry points and the suite format pair `ess-conformance/42`
(ordinary) and `/43` (declared coverage). It adds no source-language construct, no authored
scenario spelling, no production entity or API and no execution step.

## Need

A typed monotonic signed-i64 revision counter starts at zero, increments only when the supplied
expected revision matches the stored one, and refuses at `9223372036854775807`. Synthesis must
neither misidentify the counter boundary nor silently omit its exhaustion obligation. Short
ordinary paths stay ordinary command arrangements. An extreme but valid state needs an explicit
conformance fixture arrangement, not billions of increments and not a production reset command.

Existing typed authored setup (`docs/design/authored-entity-state-arrangement.md`) can establish
`MAX`, but it is not a synthesis seed: an authored scenario is appended after synthesis and does not
discharge a generated obligation. Checking `MAX` in a separately authored scenario is expressible;
supplying that state to generated boundary coverage was not. A sequence-number allocator with
compare-and-swap advancement, signed-maximum exhaustion and no arbitrary reset has the same need.

## Selection

```text
ess verify conform synthesize --path model \
  --synthesis-seed max.yaml at-max --synthesis-seed below.yaml below-max --out suite.json
```

`--synthesis-seed FILE INSTANCE` is repeatable with exactly two values per occurrence. FILE names
one explicit authored document (a regular, non-symlink file; no directory scan, manifest extension,
fragment syntax or implicit instance selection). INSTANCE names exactly one arrangement of that
document whose `setup` is present. The same option works with ordinary output, with
`--suite-format 5` declared coverage and with `--component`. `--scenarios` stays independent:
selecting a seed source never appends its authored scenario, and appending a scenario never selects a
seed. Supplying both explicitly is allowed and the two effects stay distinguishable.

A seed supplies **only the nominated initial setup row**. Its document's timeline, assertions and
any state the timeline would reach are never used, and its assertions never count as generated
coverage.

## Admission order

Every refusal below happens before any output or target activity.

1. Load every explicit source once, with the same file and symlink safeguards as existing authored
   coverage discovery. Refuse a missing or ambiguous source (two files sharing one source identity
   with different bytes), a duplicate selector (the same source and instance twice), an unknown
   instance, an instance without setup, and source parse or compile refusals. An empty explicit
   selection is refused rather than read as no request.
2. Compile the complete source against the same IR with the existing authored compiler. This
   validates rather than runs its timeline and assertions. Only the nominated initial setup is
   selected; setup literals cannot reference another arrangement for identity or fields.
3. Convert the setup to a private admitted seed value retaining the model binding, the declared
   entity, the literal identity, state and fields (Optional absence and present null preserved), and
   the diagnostic source and instance. The library accepts source text, not filesystem paths;
   reading files stays with the CLI. The model binding is checked again when the seeds are passed to
   synthesis.
4. Sort the admitted set by qualified entity, canonical row value, source identity and instance,
   independently of argument order. Refuse a duplicate qualified identity (`entity`, `identity`)
   across selections; rows are never merged, fields never guessed, identities never generated or
   coerced. At most 64 selections are admitted. Seed attempts count against the existing
   arrangement budget instead of multiplying it: each row offered to an obligation is one node of
   the 64 its bounded search already spent part of, and a counter-limit side past the bound of
   further rows one branch is witnessed on is not seeded.

## Library surface

```text
SeedSelection { source: authored::Source, instance: InstanceName }
AdmittedSeeds::compile(ir, selections) -> Result<AdmittedSeeds, SeedAdmissionError>
synthesize_with_seeds(ir, &AdmittedSeeds) -> Result<Synthesis, SeedAdmissionError>
synthesize_for_with_seeds(ir, component, &AdmittedSeeds) -> Result<Synthesis, SeedAdmissionError>
coverage_build::build_with_seeds(ir, authored_sources, scope, origins, &AdmittedSeeds)
    -> Result<AdmittedInput, AdmissionError>
```

Existing `synthesize`, `synthesize_for` and `coverage_build::build` keep their signatures and
delegate with an empty set. There is no mutable global or thread-local seed catalogue: the admitted
set travels in the immutable invocation context of the search. A model whose actors carry caller
attributes is synthesized per caller assignment; seeded synthesis of such a model is refused by name
rather than run seed-free. Selecting authored-only coverage while supplying seeds is a contradiction
and is refused.

## Arrangement and obligation

Ordinary arrangement is always attempted first, with no seeds. A seed is tried only at the exact
obligation whose ordinary attempt was unmet:

- the primary witness of a branch that reads the stored subject row, where no bounded arrangement
  selects it;
- one side of a stored counter's limit that the bounded search did not reach, including a side whose
  nearest value lies past the search's reach and was refused before searching.

A successful ordinary witness is never re-ranked, replaced or swapped for a seeded one. Where a
scenario already has its ordinary steps and only one counter-limit side is missing, those steps,
identities, values and assertions are kept byte for byte and only that side's segment is appended.

For the still-unmet goal, each eligible admitted row (same entity as the row the command reads, in
admitted order) is offered as one more initial arrangement. The row must satisfy the goal's exact
predicates, the held state and every known instance or owner constraint. Its literal facts become the
settled row, with Optional absence preserved. The scenario then emits `establish_entity`, observes
the row, invokes the real command with the input grounded from the actual row by the existing input
search (the expected revision of a compare-and-swap is read off the row), selects the outcome through
the existing guard precedence, and asserts error, events, state and preservation as for any other
row. The source scenario's name or outcome is irrelevant.

A seed is not a shortcut through the graph: the command is tested directly at the admitted row; no
transformation is performed and no seed is derived. Only the obligation actually built loses its
refusal occurrence; nothing is removed by matching a scenario name, an outcome label or the existence
of some application. A row needing an owner, a related row, an aggregate over jointly established
rows or historical facts is not seeded; its refusal stands and says that the seed could not be
applied. A seeded row's identity never collides with another row the same scenario establishes, and
it is never renamed: another eligible seed is tried, or the refusal stands. Separate scenarios keep
their begin/end isolation. Where two attribute-free actors are both granted a command, a scenario
is run once more with the callers reversed; a scenario that establishes a row by setup is not run
again, since the row's identity cannot be established twice, and the note `CrossCallerUnswapped`
says so. External outcomes stay explicit controls from the model; a seed is evidence only for
stored state, never for trusted temporal context, authorization or a live continuation.

## Persisted provenance (suite/42 and /43)

One optional `synthesis_seeds` record on the suite provenance, admitted only in suite/42 (ordinary)
and /43 (declared coverage) and required there. Every seed-free suite omits it and keeps its
previous format and bytes, except where exact integer handling now witnesses a guard binary64
could not represent: a literal beyond 2^53 is stepped exactly when inputs are chosen, and a row
value beyond 2^53 is read exactly, so a guard such as `amount >= 9007199254740993` that synthesis
used to refuse with `ESS-SYNTH-003` now gets its scenario. Any explicitly admitted nonempty
selection records its provenance, even where every seed is unused, and so selects 42 or 43. The record is closed; all three members are
required:

| member | content |
|---|---|
| `sources` | map of checked root-relative source identity to `sha256:` digest of the original UTF-8 bytes; nonempty; no absolute host paths; no claim that an authored scenario was appended |
| `selections` | sorted distinct records of `source`, `instance`, `entity`, `identity`, `fields`, `state`: the exact admitted setup values; retained even when unused |
| `applications` | sorted distinct records binding a selection (`source`, `instance`) to a generated `scenario`, its `establish_step` index and its real `command_step` index; may be empty |

Selected seed documents are not listed in `coverage.authored_sources`: that inventory binds an
authored scenario and its disposition, and a seed source produces no authored scenario. A source
passed both as `--scenarios` and as a seed has two independent roles. Generated scenario counts and
origins stay `generated`; no seed count enters execution totals.

An application is proof of use in the emitted suite, not a claim of execution or reachability. Only
applications whose steps are serialized in the suite are recorded; component filtering keeps sources
and selections and drops uses outside emitted scenarios. A coverage selection keeps the parent
provenance unchanged; there, an application may name a scenario the selection filter moved outside.

## Writer and reader checks

Rust, generated Go and generated TypeScript readers refuse, before any target callback:

- `synthesis_seeds` under any suite major other than 42/43, and a 42/43 suite without it;
- a 42 suite carrying a coverage inventory, a 43 suite without one;
- an unknown member, an empty `sources` or `selections`, an invalid digest, a selection naming an
  unknown source, a source no selection uses, duplicated or unsorted selections or applications, and
  two selections with one qualified identity;
- an application naming an unknown selection, a scenario the suite does not hold, an authored
  scenario, an `establish_step` that is not an `establish_entity` step equal to the selection
  (entity, identity, fields and state), or a `command_step` that does not follow it, does not invoke
  a command addressing that instance, or is not followed by the outcome assertion;
- a generated scenario establishing a row no application binds.

Older readers refuse 42/43 by version. A failure to build an obligation remains in the refusals even
when an application exists for the same scenario. Standalone readers check structure and step
equality; the target's setup capability still validates model semantics and actual storage, and
Unsupported setup never counts as passed.

The browser player and coverage replay refuse a seeded suite by name before artifact output: a
browser depiction is not evidence that a backend accepted setup.

## Not in this design

Arbitrary multi-row snapshot seeding, related or owned-parent seed joins, seeds for caller-attribute
models, and a reusable seed-group key in authored documents. Each would need its own minimal
reproduction and format decision.
