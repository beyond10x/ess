---
title: Verify conformance
description: Generate the semantic suite a specification requires, run it, and emit a standalone ESS conformance report.
---

# Verify conformance

A specification declares more than an interface shape. Command outcomes, refused branches, state
transitions, emitted events, and invariants become semantic scenarios an implementation can be
checked against.

## Generate the suite

```shell-session
$ ess verify conform synthesize \
    --path examples/billing \
    --out target/billing-suite.json
```

The suite is deterministic. Its provenance names the model and contract digests from which it was
derived.

Add `--compact` to fresh `synthesize` output to write deterministic JSON without
indentation, followed by one newline. The decoded suite is unchanged; its exact
byte digest changes, so retain that original compact file when producing reports
or selecting child suites. Pretty output remains the default. This option does
not rewrite an already committed suite.

## Select authored scenarios explicitly

The manifest capability described here is available in current source and is unreleased.
`--scenarios` accepts one file or one directory. An immediate `ess-inputs.yaml` in that directory
selects its exact `scenarios` list, including explicitly listed nested files of any extension.
The [mixed-layout example](write-a-specification.md#keep-sources-and-generated-output-together)
can serve both roles:

```shell-session
$ ess verify conform author --path . --scenarios . --suite-format 5 --out output/authored.json
```

Without that immediate manifest, a directory selects only immediate lowercase `.yaml`/`.yml`
entries; subdirectories are not searched. An explicit file bypasses extension filtering and parent
configuration. An explicit empty selection refuses before outputs or execution. Omitted
`--scenarios` selects no authored inputs, even if the model's manifest lists scenarios or the working
directory contains `scenarios/`.

The manifest validates both lists but opens only the active role. It rejects duplicate paths,
paths shared by both roles, escaping paths and selected symlinks. Suite/4 legacy invocations retain
their supported file/root links; manifest invocations require real contained files. Each listed
document still reaches its existing semantic reader, including malformed or foreign documents.

Suite/5 retains the exact listed relative identities and original source text. Relocating the root
or reordering the lists preserves these bytes; changing LF to CRLF changes source evidence. Copies
and hardlinks with distinct paths remain separate requested identities. The manifest itself adds
no suite source entry or provenance digest. Coverage describes this explicit selection, not every
scenario that might exist below the directory.

Committed `run --suite` and `run --suite-input` retain their acquisition bypass and argument
conflicts. Impact still loads both model revisions, and release qualification still loads its
explicit model. Discovery refusals precede output or runner activity; existing document-level
semantic refusals can still retain incomplete diagnostic evidence.

## Establish backend state in an authored scenario

Unreleased `ess-scenario/2` supports typed setup for entities whose rows arrive
from an upstream system. A scenario can establish those rows and query their
view without inventing a creator command:

```yaml
type: ess-scenario/2
domain: calls.history
scenario: recent-call
summary: A stored call appears in history.
arrange:
  - instance: recent
    entity: calls.history.CallRecord
    setup:
      identity: 00000000-0000-4000-8000-000000000001
      fields: {started_at: '2026-01-05T09:00:00Z', duration_seconds: 12}
      state: Completed
assert:
  - view: calls.history.CallHistory
    contains: {call_id: {$instance: recent}, duration_seconds: 12}
```

The model must declare that entity, its field types, lifecycle state and view.
Setup validates identity, required fields, nested values and invariants. Duplicate
qualified identities, null identities and inconclusive invariants refuse.

The adapter must establish actual isolated backend state and make it visible
before acknowledging setup. Setup emits no command or event and claims no
lifecycle path. Rust and generated Go expose an optional setup capability;
unsupported adapters produce a non-passing result. These steps use suite/6 or
coverage suite/7 and require explicit report/2. Source scenario/1 refuses setup.

## Observe outcomes selected by held state

For a command declaring `when_subject_state`, synthesis establishes a real
reachable state, queries a declared immediate view exposing identity and state,
invokes the command, and checks the resulting row. The same input can therefore
prove different outcomes from different held states. An implementation that
returns the expected outcome while incorrectly changing the row fails.

The initial adapter requires an unfiltered immediate view without parameters.
Missing observation authority or an unreachable required state produces an
explicit synthesis refusal. This uses existing command and view assertions;
the state guard alone does not require a newer suite vocabulary. The source
declaration requires `ess/3`.

## Observe selection, periodic activity and clock evidence

Unreleased selection observations compare actual source occurrences and selected
indices, preserving optional absence and occurrence-based exclusion. A host
conversion remains an explicit obligation; declaring the conversion does not
execute or prove it.

Periodic checks exercise the named host under controlled target time: ready,
initially inactive, failed first read and slow first read. The runner examines
actual scoped timer, read and invocation facts, including the complete live
interval and a window after stop acknowledgement. Missing ticks, overlapping
work, stale mapped inputs and post-stop activity fail. Unsupported authority
produces a non-passing result. The bounded check observes at most five live
periods; it does not sleep in the runner.

Clock comparisons use the typed `ExpectReadingOrder` operation with references
to already observed event members. The adapter supplies occurrence-scoped
process, epoch, origin and formatter facts; the runner normalizes and compares.
Declared alternative origins are requirements, not evidence. Wrong occurrence,
unknown offset, differing process/epoch or mutated request requirements cannot
establish success. Authored YAML convenience syntax for this comparison is not
provided; construct and persist the typed operation through the admitted writer.

These operations use suite/6 or declared-coverage suite/7 and explicit report/2.
Previous readers refuse their vocabulary. Controlled adapter tests do not prove
that an unrelated production adapter implements these capabilities.

## Run a supported target

```shell-session
$ ess verify conform run \
    --suite target/billing-suite.json \
    --target billing \
    --report-out target/billing-conformance.json
```

The command executes the generated or committed scenarios against the selected reference target.
The built-in choices are `billing` and `oracle-fixture`; a production adapter must establish its
own execution boundary.
The default standalone report is `ess-conformance-report/1`. Its historical `scenarios_failed`
count includes every non-pass, including Go skips and Rust errors or unsupported results. Those
legacy bytes and meanings remain unchanged.

To run directly from a specification instead of a pre-generated suite:

```shell-session
$ ess verify conform run \
    --path examples/billing \
    --target billing \
    --format json
```

## Opt into explicit outcome counts

```shell-session
$ ess verify conform run \
    --suite target/billing-suite.json \
    --target billing \
    --report-format 2 \
    --allow-incomplete \
    --report-out target/billing-counts.json \
    --format json
```

`--report-format 2` selects `ess-conformance-report/2` for `--report-out`. It records separate
`passed`, `failed`, `error`, `unsupported` and `skipped` counts and sorted scenario IDs. Rust uses
error/unsupported; generated Go uses skipped and keeps ordinary target errors as failed. The report
binds every original suite byte, including its final newline, under `sha256-json-bytes/1`. Retain the
original suite beside the report; reformatting it changes that identity.

Legacy suite versions 1–4 have coverage exactly `unknown`, so even an empty or
all-pass execution has inconclusive conformance. `--allow-incomplete` explicitly chooses diagnostic
execution: Rust exits 0 for passed execution, 1 for failed and 3 for inconclusive execution.
`--strict` requires explicit `--report-format 2` and succeeds only for passed conformance; unknown
coverage therefore exits nonzero. Strict and allow-incomplete cannot be combined. Neither control
changes the report format automatically. Suite/4 and diagnostic report/1 remain the defaults.

With report format 2, `--format json` or `--format yaml` presents a separate closed
`ess-conformance-run/2` object containing `format`, the standalone `summary`, `started_at` and ordered
`scenarios`. `--report-out` always writes the standalone JSON summary. Both new formats preserve
unsigned epoch milliseconds exactly through the full u64 range. The default detailed output remains
the legacy unversioned structure.

Regenerate a Go package explicitly to use the new writer, then run:

```shell-session
$ ESS_REPORT_FORMAT=2 ESS_CONFORMANCE_ALLOW_INCOMPLETE=1 ESS_REPORT_OUT=report.json go test ./...
```

Generated Go defaults remain report/1 and diagnostic execution. `ESS_CONFORMANCE_STRICT=1` requires
report/2 and fails the enclosing test even when every subtest skipped. The strictness variables accept
only unset or `1`; conflicting settings refuse before target construction. Omitted or abnormally
terminated selected subtests, a negative report/2 clock or a failed report write cannot publish a
complete report. These checks also apply when no report destination is requested. Retained generated
packages keep their own runtime behavior until regenerated; upgrading the standalone binary does
not update them. Set `ESS_REPORT_OUT` as well as `ESS_REPORT_FORMAT=2` to retain the count report.
An absolute output path avoids package working-directory differences in `go test ./...`.
A failed conformance run still writes its separate counts; report delivery does not turn failures
or skipped scenarios into passing evidence.

## Opt into declared coverage

The suite/5, original-byte carrier and paired replay features in this section are current-source
changes after the [0.20.0 release observed on 7 September 2026](../status/where-this-stands.md).
They remain unreleased relative to that record, even though the workspace still declares `0.20.0`.
That release establishes the report/2 count surface described above, not these later coverage
features. Use a source build containing them; the default suite/4 and report/1 paths remain unchanged.

```shell-session
$ ess verify conform synthesize --path examples/billing \
    --scenarios examples/billing-scenarios --suite-format 5 --out target/coverage-suite.json
$ ess verify conform run --suite target/coverage-suite.json --target billing \
    --report-format 2 --strict --report-out target/coverage-report.json
```

Suite/5 retains the declared scope, origins, selected IDs, known outside scenarios, every refusal
occurrence and every requested authored source's relative identity and exact byte digest. Generated
and authored origins can be selected independently; `--component` limits fresh synthesis to that
component. `conform author --suite-format 5` inventories only its supplied authored files. Library
callers can explicitly retain known excluded generated inventory with
`coverage_build::build_with_known_generated`.

A nonempty all-pass selection qualifies only when its inventory is complete and has no in-scope
refusal. Unknown inventory, an empty selection or a missing required check remains inconclusive.
Filtering cannot remove a refusal or promote unknown knowledge. Every identical refusal occurrence
remains visible; a useful passing scenario can have missing checks beside it. Strict mode fails
when conformance is inconclusive, while diagnostic mode retains the execution result's exit rules.
Suite/5 always requires explicit report/2, including with `--allow-incomplete` or no report output.

To narrow an admitted suite, supply a JSON array of sorted distinct scenario IDs; `[]` explicitly
selects none:

```shell-session
$ ess verify conform select --suite target/coverage-suite.json \
    --ids selected-ids.json --out target/coverage-input.json
$ ess verify conform run --suite-input target/coverage-input.json --target billing \
    --report-format 2 --strict --report-out target/selected-report.json
```

The closed `ess-conformance-input/1` retains the selected original JSON string and every original
parent string, nearest first. Repeated selection uses `--suite-input` and retains the full chain.
An explicit child cannot be run from its raw suite alone. Admission checks each full surviving
scenario definition, dependency, source map, refusal and selection against the exact parent.
Only the inner selected bytes are hashed; reformatting the carrier preserves that identity.

Authored directory discovery remains shallow. Identities use `/`-separated relative UTF-8 segments;
absolute paths, dot segments, backslashes, colons, controls, invalid UTF-8 filenames and selected
symlinks refuse. Relocating a root preserves identities; changing source newlines changes its digest.
The historical `ConformanceSuite` DTO remains unadmitted: it cannot recover discarded source fields
or become known coverage by changing its version. Original inputs use `AdmittedSuite` or
`coverage::AdmittedInput`; count reports require the immutable actual `ExecutedRun` capability.

Generate Go with `--suite-format 5 --target go`, or call `go::emit_input` with an admitted carrier.
The runtime checks every original parent before creating a target. Wire metadata retains the full
u64 range; only selected execution fields must fit the host's Go `int`. Unrepresentable selected
counts, indices or halt limits refuse before callbacks or reports. An oversized omitted parent
field does not block a representable child. The Go report clock supports nonnegative int64 values;
Rust report/run timestamps support the full u64 range. Payload Number meaning is a separate finite
binary64 boundary. Modeled Binary64 remains unsupported by conformance, including suite/5.

`conform web --suite-format 5` emits a paired `ess-conformance-replay/1` document and matching player.
The actual player checks the closed projection, exact suite reference and full input before creating
replay state, then displays selection and refusals. It emits no execution report. The reduced model
omits literal assignment values and full view evaluation; digest comparison neither reconstructs
the full compiled model nor authenticates its publisher. Existing players do not acquire these
checks when handed new metadata; regenerate and distribute the paired bundle together.

`ess impact --suite-input` accepts complete admitted coverage and reports its selection separately.
Unknown or incomplete inventory and missing parents refuse. Persisted output remains `ess-impact/3`
with its existing fields; invalidation within a selection is not whole-system execution evidence.

## Observe bounded binding accessors

The unreleased `ess/3` binding paths described in
[Write a specification](write-a-specification.md#read-a-field-inside-an-event-envelope)
require new suite vocabulary. An ordinary suite retaining an accessor uses
`ess-conformance/6`; a declared-coverage suite retaining the new accessor
vocabulary uses `ess-conformance/7`. Models that do not retain that vocabulary
keep their existing suite formats. Both new versions require explicit
`--report-format 2` for execution. Report format 1 is refused before the target
runs, even without `--report-out` or when incomplete execution is allowed.

The observation checks the declared path against the triggering event and the
resulting command input. It distinguishes an unavailable path from a terminal
value and rejects missing required members, invalid union discriminators and wrong
payload kinds along the traversal. Whole-value collection copies do not validate
their elements. A conversion declaration supplies a reason
for a type crossing, not an executable conversion algorithm: a mapping whose
expected input cannot be determined receives a capability refusal.

Native Rust and Go values can distinguish nested Optional states that serialize
to the same JSON `null`. When that distinction is necessary to decide whether a
mapping is correct, conformance refuses the ambiguous observation instead of
guessing. Copying the typed native value remains distinct from proving that copy
through a JSON observation.

Ordinary suite/6 retains unknown coverage. Suite/7 uses the same exact-input and
parent-lineage rules as suite/5; selecting a child preserves its version and
requires the original parent chain. A passing run proves only the admitted
selection and does not resolve any recorded refusal.

## What the report proves

The report proves what its scenarios observed against the named target and specification digest. It
does not prove that the target is independently operated, deployed in production, or free from
behavior the suite never exercised. A consumer may translate this report into its own evidence
vocabulary at that consumer’s boundary; ESS itself publishes no workflow or planning record.

## Add a target

The built-in targets demonstrate the runner contract. A new target implements the Rust
`ConformanceTarget` boundary and must preserve scenario identity, request/response correlation,
refusal semantics, and deterministic reporting. It must not report its own unobserved success as a
verifier result.
