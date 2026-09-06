# Conformance coverage input and consumer transport

Status: accepted for implementation on 2026-09-06 under story:review-conformance-coverage,
after the two recorded binding reviews and the coordinator's final wording correction.
Owner: `story:review-conformance-coverage`, under the standing remediation approval.
This binding supplements [the accepted coverage contract](review-conformance-coverage.md).
It does not change the frozen outcome rules, refusal multiplicity, suite/1–4 bytes or defaults.

## Current source and typed owners

The source baseline is ESS `be0eefd7ec125d46bb3b664c4b95b8d638a2b1fe`.
`AdmittedSuite` in `crates/verify/ess-conformance/src/admission.rs` retains original bytes,
an immutable execution view and their digest. `ExecutedRun` in the same package's `src/runner.rs`
binds diagnostics to the actually executed digest. `CountReport::from_run` in `src/counts.rs`
checks that binding. Preserve these boundaries and the unadmitted historical `ConformanceSuite`
DTO. A separate suite/5 representation can carry coverage without changing legacy DTO bytes.

These are concrete Rust-owned document and admission values under the existing suite, report,
browser projection and impact owners. They are not independently identified business entities
and do not introduce an authored ESS domain, a resolver registry or a generic metadata bag.

## One original-byte input carrier

Introduce the closed JSON document `ess-conformance-input/1` with exactly these fields:

| Field | Type and meaning |
| --- | --- |
| `format` | Exactly `ess-conformance-input/1`. |
| `suite_json` | Original UTF-8 JSON text of the selected suite/5. |
| `parent_suites` | Array of original UTF-8 suite/5 JSON strings, nearest parent first. |

This carrier is a separate format. The inner suite/5 still has exactly `provenance`, `scenarios`
and `coverage`. A `SuiteReference` hashes the decoded `suite_json` string's UTF-8 bytes, including
all original whitespace, escape spelling and final newline. It never hashes the carrier or a
reserialized execution view. Reformatting the carrier does not change the inner suite identity.

Carrier parsing is structural input, not admission. Reject duplicate keys, unknown fields,
invalid Unicode and wrong types before producing a carrier value. Admit every inner document
through the complete suite/5 boundary. There is no filesystem, working-directory, network or
ambient parent lookup. The admitted capability retains every original parent string immutably;
serialization of raw input cannot restore that capability.

Admission walks the complete lineage before execution:

1. An `all` selection has no parent and requires an empty parent array.
2. Each explicit child references exactly the next parent's version, digest profile and original
   byte digest. Every parent is suite/5; legacy suites cannot supply invented origins or inventory.
3. The final ancestor has `filter: all`. Reject missing, reordered, duplicate or unused parents.
4. Compare full surviving scenario definitions and dependencies, provenance, scope, origins and
   knowledge. A matching ID alone cannot authorize a changed step or a promoted inventory claim.
5. Apply every accepted parent/child invariant: preserve the full source map, every refusal
   occurrence and its fields, previous outside entries and the exact newly omitted IDs.
   Unknown knowledge cannot become complete through filtering.

No extra depth limit is selected by this binding. Traverse iteratively and reject a chain whose
documents fail admission; any implementation resource limit must be explicit and reviewed before
it changes which otherwise valid input is supported. JSON parser nesting limits remain separate
from the number of parent documents in this flat array.

Keep direct original-byte admission for legacy suites and unfiltered suite/5. A direct explicit
suite/5 without parents refuses. Add an explicit carrier admission API. The existing in-memory
legacy path still issues and admits its own once-serialized buffer; it cannot recover discarded
original fields or mint complete coverage by relabeling a DTO.

## Authored input identities

The new coverage builder accepts checked root-relative identities and unchanged UTF-8 source
contents before compilation decides duplicate ownership. `Source.origin` remains display text
for the legacy API; do not silently reinterpret every existing `Source::new` caller.

For the CLI, a directory argument is the input root and a single-file argument uses that file's
parent as the root. Keep existing direct-directory discovery, extensions and explicit-empty
refusal. Do not add recursive discovery or repair unrelated authored-discovery behavior here.

An identity consists of nonempty UTF-8 path segments separated by `/`. Reject absolute paths,
empty segments, `.` and `..`, backslashes, colon and control characters. Do not normalize Unicode,
case, source newlines or source text. Native filenames that cannot be represented under this
profile refuse the new coverage acquisition path. Sort identities by their UTF-8 bytes before
compilation. Repeated input identity refuses before compilation, including identical duplicates;
separate identities with identical contents remain separate requested files.

The new filesystem path refuses symlink files or symlink directory traversal within its selected
root. It reads each selected file once and hashes those exact bytes. Relocating an otherwise
identical root preserves identity and digest. Library callers explicitly supply the checked
relative identity and original bytes; the core chooses no filesystem authority.

The final source disposition is determined after candidate merging. A compiled candidate rejected
by a merge is refused, while the surviving source retains ownership. Keep original cause text,
known IDs, rejected source identities and every duplicate refusal occurrence. These identities
describe the declared input set; they do not authenticate a repository or prove inventory honesty.

## CLI and generated Go

Fresh synthesis, authoring, reference execution and browser emission gain explicit
`--suite-format 4|5`, defaulting to 4. Loaded suites retain their admitted version. The existing
IR/file output remains a raw canonical suite document; suite/5 YAML is presentation of the typed
document, never a substitute for the original JSON hashed by a runner.

Add explicit `--suite-input` acquisition beside `--suite` for execution and impact. The two flags
conflict. A supplied carrier is not combined with a fresh authored-source request or an explicit
fresh suite-format selection. A supplied raw suite/5 is usable only when it needs no parents.

Add `conform select` with exactly one of `--suite` or `--suite-input`, a required `--ids` JSON file
and `--out` carrier file. The IDs document is an array of sorted distinct valid ScenarioIds;
an empty array is an explicit empty selection. Refuse an ID absent from the admitted parent.
Produce a new explicit child and its complete nearest-parent-first chain. Validate all inputs
and destinations before writing. This command never changes scope or origins.

An unfiltered raw suite/5 can be wrapped in the carrier by a checked library constructor or
consumer edge, retaining the exact raw inner string and an empty parent array. This issues a new
carrier, not a claim that some earlier outer carrier bytes were received.

The Go emitter accepts the admitted input, embeds the complete carrier when needed and retains
the selected inner original bytes through admission and execution. Current `runtime.go` hashes
global `suiteJSON` in count production; the new path must hash the admitted inner bytes instead.
Validate the entire lineage before target construction, identity callbacks or scenarios. Keep
the terminal-completion, teardown, host-filter and no-destination guards from the count stage.

**Checked Go execution adaptation (D7, 2026-09-06).** Suite/5 wire admission and complete
lineage comparison retain the exact unsigned integer metadata before converting a selected
execution view. The inherited public Go `Step.After`, count/position fields and
`ScanRequest.StopAfter` use `int`; this work preserves that API. Their narrower range must not
reject otherwise valid original parent bytes or alter a parent comparison. In particular, a
valid oversized value in an omitted parent scenario does not prevent executing a representable
child. After complete wire/lineage admission, check every integer conversion required by the
selected execution view against the actual Go `int` width. An unrepresentable selected value
refuses adaptation before target construction, identity callbacks, replay/execution state or
report/destination creation, identifying the scenario, field and representability failure.
It neither wraps, clamps, converts through binary64 nor becomes an `unsupported` execution
result. Preserve the positive full-u64 wire/lineage vector and add distinct selected-adaptation
refusal and omitted-parent positive controls. The original suite digest remains unchanged;
this adapter restriction does not narrow the shared wire contract or timestamp admission.

Suite/5 with implicit or explicit report/1 refuses before execution, even with `--allow-incomplete`
or no report destination. Explicit report/2 and the existing strict/diagnostic choices retain
their accepted meanings. No default transition is part of this work.

## Preserved model admission after the Binary64 integration

Source refresh on 2026-09-06: published ESS `6c6620b` adds explicit Binary64 model/suite
refusal before conformance output (`ess-conformance/src/admission.rs:280`, `:285`,
`src/authored.rs:1452`, `src/synthesize.rs:926`). Coverage suite/5 adds inventory, not a
finite Binary64 conformance codec. Fresh suite/5 builders and paired web/Go projections
must retain that model admission before constructing inventory or output. Direct typed
and original-byte suite inputs must retain their unsupported-primitive refusal. An invalid
model cannot become a complete empty inventory merely because synthesis emitted no scenario.

The new model-level authored `UnsupportedBinary64` cause has origin `model` and no file
identity. It is an input-model refusal, not an authored source-map entry: do not invent a
source file or admit it as a new suite/5 coverage refusal code. The selected coverage wire
continues to admit only the supported execution vocabulary and baseline coverage causes.
An actual Binary64 conformance codec requires its separately governed contract decision.
The checked `Result` returns now used by suite serialization and runner/projection entries
must be propagated; no compatibility wrapper may fabricate a report on refusal.

## Browser pairing

Introduce a single closed paired document `ess-conformance-replay/1` containing `format`,
`model`, `suite` and `input`. `suite` is the exact selected SuiteReference; `input` is the shared
SuiteInputDocument. `model` is a concrete typed projection with `system`, `version`, `spec_digest`,
`contract_digest`, `entities`, `commands`, `views`, `actors` and `bindings`.

The finite projection fields originate in current `src/web.rs`:

| Record | Existing fields retained by the typed projection |
| --- | --- |
| Entity | name, display, identity, initial, states, terminal, fields. |
| Command | name, display, outcomes. |
| Outcome | name, refuses, nullable subject, emits, sets. |
| Subject | entity, kind, nullable transition, from, nullable to. |
| Assignment | target and nullable input-field source (`from`). |
| View | name, display, entity, consistency, nullable filter text, fields, params. |
| Actor | name, display, may. |
| Binding | name, event, command, delivery, failure. |

Use explicit Rust structs and closed finite enum representations for these existing shapes;
do not put arbitrary JSON properties in the model envelope. The original projection omits literal
assignment values and does not promise full view evaluation. This proposal preserves that declared
replay limitation; the independent browser-fidelity story must decide its stronger representation
and format consequence. Pairing admission alone does not close that finding.

The constructor takes actual EssIr and admitted suite input, checks system, version, model digest
and contract digest agreement, then produces the projection. The browser validates the closed
paired document, exact suite reference and complete parent chain before creating replay state.
Counts use an integer-preserving parser and comparisons; ordinary JSON.parse through Number cannot
prove exact u64 admission. Declared payload numbers retain their separate finite binary64 profile.

The browser displays the admitted selection, unknown/incomplete inventory and refusals. It emits
no execution report or qualifying evidence. Comparing the declared model digest establishes
pairing under a checked producer; the browser cannot recompute a full EssIr digest from this
reduced projection. No publisher authentication or independent inventory proof is claimed.

Retain actual old player bytes as compatibility evidence. Run the generated generic player in an
actual browser for positive pairing and pre-replay rejection cases; emitted-source assertions and
the unrelated billing/WASM lab do not establish this boundary.

## Impact remains version 3

Current `crates/verify/ess-diff/src/impact.rs` emits `ess-impact/3` with `ess-diff/2`, not the older
version 2 cited in the accepted coverage design. Correct those current-baseline references while
preserving the historical version meanings.

Add an admitted-suite entrypoint and an in-memory result context retaining exact suite reference
and selection. Preserve EssImpact's persisted version-3 fields and bytes for existing inputs;
there are no new persisted reference, selection or coverage fields. The CLI presents the declared
selection in separate diagnostics. An impact result is invalidation information within its supplied
selection, never execution evidence or a whole-system coverage assertion.

For suite/5, require complete inventory with no in-scope refusal before narrowing; otherwise
refuse the operation with an explicit coverage reason. Missing exact input or parents also refuse,
without falling back to a suite-free or empty dependency calculation. Existing dependency-walk
obligations remain conservative. The legacy DTO API refuses suite/5 so discarded coverage cannot
enter that path. These new input refusals do not alter old report meanings or add a persisted
WholeAnswer spelling under version 3.

## AEP prerequisite and review requirements

AEP `story:admit-ess-conformance-coverage` owns a new carrier, evidence kind, expectation and reader
binding. Existing `ess_conformance_v2` retains its closed report_json/suite_json count-stage shape
and nonqualifying meaning. The new kind is `ess_conformance_coverage_v1`, retaining original
report JSON and the separately versioned input-carrier JSON. Independent expectations include the
current model, exact suite reference, complete Selection and exact selected IDs.

Both AEP reader routes, typed inspection, checked mutation and offline snapshot/driver replay must
admit the complete original lineage. Publish those reviewed readers before the ESS writer, then
exercise frozen actual Rust/Go producers through them before ESS source publication. A fresh Atlas
ADR records actual relying parties, order, source shipments and unresolved installed adopters.

Review must cover every accepted coverage matrix row plus multi-generation lineage, changed parent
whitespace, changed surviving steps/dependencies, surplus parents, unknown promotion, repeated
identical refusals, merged source ownership and relocated roots. Execute actual Go and generic
browser controls. Keep count-stage exact scalars, legacy bytes, mutation refusal and all original
adversary assertions. No tests, new readers or new writers are claimed by this binding document.

The two immutable ess-conformance-coverage-binding-review records are held in both ESS and AEP.
Their corrections bind AEP's exact observable table and preserve Text semantics, and align the
ESS story with strict/diagnostic pairing. The transport's semantic decisions were unchanged by
those corrections. Acceptance authorizes implementation under the standing remediation grant;
it does not claim that a future reader, writer, browser or integration gate has executed.
