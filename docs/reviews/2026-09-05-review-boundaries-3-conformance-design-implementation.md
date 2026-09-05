unit:                   story:review-conformance-format-design — Design the conformance suite and report migration
verdict:                green
cases:                  not applicable — design-only source and 36-row matrix review; no compatibility tests executed
origin:                 n/a — implements the brief and review F03 design acceptance
wrote-outside-worktree: none
needs-coordinator:      no source patch outside assignment; later Atlas/AEP/adopter implementation and rollout obligations are named in the design

## 1. Unit and acceptance

Acceptance: a binding migration design specifies unambiguous old/current suite and report semantics for every current producer state and coverage category before a new writer is implemented.

The new document is docs/design/review-conformance-coverage.md in /home/timo/.local/state/worktree/trees/b10x/ess/review-conformance-format-design, based on ESS 45832cc885377b2d61845ee33af14f0293d99e67. Its decisions reserve report/2, suite/5 and detailed run/2, freeze existing meanings/valid bytes, define producer-specific outcome partitions, define exact issued-JSON byte binding, persist selection/coverage, specify strict qualification, and sequence readers before new default writers.

The document distinguishes source observations from specified future behavior. It does not claim an Atlas ADR, downstream adaptation, compatibility test, or implementation release has shipped.

## 2. Observed change and scope

Only the assigned new design file was authored. It remains untracked and unstaged as required, so ordinary git diff --stat does not show it. The no-index diff against /dev/null observes the new file explicitly.

```text
$ git --no-pager diff --stat
exit: 0
```

```text
$ git --no-pager diff --no-index --stat -- /dev/null docs/design/review-conformance-coverage.md
 .../design/review-conformance-coverage.md          | 246 +++++++++++++++++++++
 1 file changed, 246 insertions(+)
exit: 1
```

```text
$ git status --short
?? docs/design/review-conformance-coverage.md
exit: 0
```

The no-index diff exits 1 because a new file differs from /dev/null; it reports 246 added lines, not a failed build. There were no production, test, planning, root configuration, workflow, generated-file, sibling-repository, Git lifecycle, or cleanup mutations.

Scope confirmation:

| Scope or mechanism hypothesis | Result |
| --- | --- |
| New design file is the only implementation surface | Confirmed; no existing file at that reserved path before writing |
| Producer vocabulary differs | Confirmed from Rust report/evidence and Go runtime source; v1 non-pass semantics frozen |
| Suite version increment alone closes old-reader admission | Rejected as a mechanism; Rust typed deserialization/Runner, reduced Go decoder and browser lack version admission |
| AEP has only the optional adapter | Rejected; recorded_from_report independently reads report counts and AEP's domain predicates have their own meanings |
| Exact-suite digest can be reconstructed from Go's execution struct | Rejected; reduced struct drops data, so the design requires the original complete issued JSON bytes |
| Coverage can be inferred from count or model digest | Rejected; refusals/outside/authored inventory and selection are separate source facts |
| Digest references in realization are report readers | Not established; explicitly retained as references without a new verification claim |
| Impact is a report verifier | Rejected; it uses suite provenance/dependencies for invalidation, with no complete-execution claim |
| An unversioned extension preserves the frozen contract | Rejected; separate versioned ESS shapes and Atlas-governed AEP movement are required |

The story's serialized relations name its parent epic and vision and carry no depends_on edge. No planning mutation was needed. The ESS specification skill was read and its typed-contract, explicit-coverage and format-migration guidance applied; this unit creates no authored business-domain specification or executable writer.

## 3. Red execution

Not applicable by the design-only brief. No test was written or executed, and no red/green compatibility outcome is claimed. The brief explicitly substitutes source inspection and a matrix review for meaningless prose tests or package builds. This section is not an omitted implementation test-first step.

The observed source gap is preserved in source citations and the matrix: v1 category ambiguity; missing durable coverage and exact-suite binding; old suite readers missing admission; two independent AEP readers and a closed AEP result/fact boundary.

## 4. Source/matrix validation and exact checks

The document contains 36 cases with one explicit required outcome and staged owner. They cover all Rust/Go final states, mixed Rust aggregation levels and Go teardown, report versions/unknown fields, suite versions/vocabulary, old generated runtimes, empty/complete/partial/component/authored selections, duplicate/count contradictions, exact byte mismatches, lossy Go hashing, detailed CLI output, impact, realization references, AEP domain adaptation and legacy opt-outs. This was a manual source-backed design review; it is not execution evidence.

AEP source reads used Git objects at 00c742e4179593738a2e8aa69e2ecc07d3c89402. The two report reader files are unchanged from cc321f31fa0120b32a5b9f5e7b8c8fdfa55f69f9. The working tree in the sibling repository was not edited or treated as clean/current authority. Atlas's cross-repository requirement was read from Git object 6035d6e1209686ca474a3f43975fde7d8621ba48, as named in the planning authority; the later coordinator must refresh actual authority before migration operations.

Commands below ran from the assigned design worktree; the AEP command explicitly uses -C only for a read-only object diff.

```text
$ git --no-pager diff --check
exit: 0
```

```text
$ git --no-pager diff --no-index --check -- /dev/null docs/design/review-conformance-coverage.md
exit: 1
```

```text
$ rg -n '[ \t]+$' docs/design/review-conformance-coverage.md
exit: 1
```

```text
$ rg -c '^\| M[0-9][0-9] \|' docs/design/review-conformance-coverage.md
36
exit: 0
```

```text
$ git -C /home/timo/beyond10x/aep --no-pager diff cc321f31fa0120b32a5b9f5e7b8c8fdfa55f69f9 00c742e4179593738a2e8aa69e2ecc07d3c89402 -- crates/observe/aep-ess-evidence/src/lib.rs crates/edge/aep-cli/src/planning.rs
exit: 0
```

```text
$ wc -lc docs/design/review-conformance-coverage.md
  246 42000 docs/design/review-conformance-coverage.md
exit: 0
```

```text
$ df -B1 .
Filesystem        1B-blocks         Used    Available Use% Mounted on
/dev/nvme0n1p2 910126964736 723350261760 140469350400  84% /
exit: 0
```

git diff --check exits 0. The new-file no-index check emits no whitespace diagnostic and exits 1 under no-index difference semantics. The independent trailing-whitespace search exits 1 because there are no matches. The matrix count is 36. No package tests, package formatter, Clippy, full gate or site build ran for this document-only unit. The integration coordinator owns applicable integrated gates.

## 5. Deliberate boundaries and compatibility

- No new default writer or runtime code: this unit specifies the contracts before implementation.
- No retroactive v1 count reinterpretation or byte rewrite: failed remains all non-passes in v1, while v2 uses a complete category partition and preserves each producer's aggregate.
- No semantic normalization disguised as exact identity: hash all issued JSON bytes; equivalent reformatting intentionally changes the reference, while canonical writers remain deterministic.
- No guessed legacy coverage or scenario origin: retain legacy suites with unknown report coverage unless rebuilt from source inventory.
- No claim of current remote deployment: exact local advertised Git objects were inspected, not network or live systems.
- No AEP/Atlas changes: both AEP readers, the domain/facts/predicates, the Atlas ADR and real adopter inventory are separate required implementation and rollout work.
- No narrowing of impact/1 or realization digest meanings: internal v5 support can be added later; persisted byte/profile changes require their own decisions.
- No meaningless executable prose checker: all runtime compatibility expectations remain explicitly future tests.

## 6. Writes and handoff

Authored outside-worktree paths: none.

Authored paths:
- /home/timo/.local/state/worktree/trees/b10x/ess/review-conformance-format-design/docs/design/review-conformance-coverage.md
- /home/timo/.local/state/worktree/trees/b10x/ess/review-conformance-format-design/target/review-boundaries-3/implementation-report.md

No compiler target or new cache location was created, and no build ran. Free space at final observation was 140,469,350,400 bytes, above the 8,589,934,592-byte reserve. No worktree/cache/build directory was removed.

The design and report are ready for document review. Source writes are relinquished at handoff; the coordinator owns commits, integration, downstream decomposition, publication and managed cleanup.

