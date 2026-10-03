---
format: aep.planning-md/3
id: runbook:consumer-runtime-source-handoff-20261003
kind: runbook
status: draft
title: Frozen source handoff to the shared ESS integration branch
relations:
- serves: vision:O2
revision: 4
---
# Frozen runtime source handoff, 2026-10-03

Carrier inspected: 9e8894b7f28d4ae7c02cb6e3144ce7022f06e0cf on batch/consumer-runtime-20261002.
Comparison target: 0e00f67201b70e9f02f5ca335ec7c81998a37213, the serial integrator's frozen318 source base. Its later planning-only commits do not alter this source comparison.

The following 57 source commits are selected in dependency order for the ONE held batch/ui-live-apps-complete-20261003 integration branch. This is a source handoff for local integration, not permission to release the incomplete ess21 bundle or proof that the combined candidate passes. The shared coordinator remains sole branch/PR integrator. Do not merge the whole carrier and do not replace existing318 or main's transport/393/394 work.

## Integration requirements

- Start from the integrator's current source, including newer318 and any separately merged main transport changes. Apply semantic deltas, not whole-file snapshots.
- 7b71cf040 changes only rust/system.rs and tests/feasibility.rs. Both files at0e00 are byte-identical to its parent, so this event-unused-parameter fix is distinct from318 and textually applicable. Existing independent review: consumer-binding-unused-event-pass1.
- 40066842c adds a one_time_response guard to each of ess-synth go/mod.rs, rust/mod.rs and lib.rs. These overlap318 by path but are narrow additions; preserve the newer served-entry code. Source refusal remains an honest generation boundary, not proof every product supports all features.
- 81d14bf05 changes Billing source facts and Timestamp input plus its generated corpus. The two overlapping Billing behavior files change model/contract digest and issued_at assignment. Preserve318 behavior/error signatures and regenerate the corpus against the integrated generator rather than accepting an older complete generated file. Related follow-on01ce4b82f depends on that source migration.
- b4b113e9a introduces source21 and suite34/35 authority; later disclosure/runtime work requires it. It remains inside the FULL held ess21 accepted-syntax bundle. No independent source21 release claim.
- 13e40c33f and8b3f128a0 establish initial-state authority plus migration; final9e8894b7f carries the independently approved general invocation correction. The final patch matches worker18f4248ce exactly, SHAa4638d2c404f715012290675f851f6720e53c2d9c24b9888bcfb2fbdbaba2bcb. Owner105 focused tests passed, independent review findings empty, root integrated stored_field_guards15/0 including the previously stale assertion. Counts overlap and are not a package total.
- 30e9e84e7 includes count/report authority and related CLI/docs as one coherent change. Do not cherry-pick only its tests.
- Regenerate schema/projections/examples and run affected-package checks on the final integrated tree. Register source21, suite34/35 and applicable diff versions coherently; reconcile event-root CLI with merged397 rather than our excluded duplicate393.
- Three uncommitted carrier docs (CHANGELOG.md, docs/design/one-time-response-values.md, website/docs/guides/verify/one-time-responses.md) are NOT included. Reconcile their intended final content separately before publishing.
- Native response work remains uncommitted in ess-interpreted-response-values-20261003 and is NOT included. Recovered focused tests and scoped lint pass, but independent review and integrated one_time_execution/one_time_contract verification remain pending. Subsequent import touches interpret/** and response tests; frozen312 synthesis permits serial282/304 work to begin on this held base after the integrator verifies the transfer.
- Nested response observation, browser product response support, retained replay and remaining admitted capability gaps remain required. This handoff does not close389 or the full backlog.

## Exact ordered source commits

- 7b71cf04015de3a7f784deb8aaf3308c7b72bf0b fix(synth): compile bindings that do not read event fields
- c2b661284c650618f156df289e24911f1464668f fix(conformance): witness copied policies and related view filters
- f9c46bedb9481493565276ce9a2426dd9ce18019 fix(conformance): preserve aggregate-owned source arrangement
- 1695a9c793dbe10fd1948fde59e97aa38ff1d476 test(conformance): require complete generated runtime parity
- 30e9e84e7891f793c0c89d4695acf2fac88bf28e feat(conformance): version precise generated runtime report categories
- b4b113e9af7542ab2ceb5c82d3ceafdf819fc84f feat(conformance): define one-time response source and suite authority
- 40066842c077d63c9b41734cd7c876e0695b4e8d feat(projections): preserve one-time response obligations
- cb5234491fed3886c12a459ae04d01558cb59dec fix(conformance): execute Go prerequisite suite parity
- 1b9b8d3a47db5a04bed238363aa3ec14443a2979 fix(conformance): integrate reviewed TypeScript runtime parity
- c9a9dcbffe1cdd9f97d669d673c7888dea317c9a fix(conformance): preserve event observation after eventual reads
- e7c6bcf01f960f14ddf12f85577817a5973fb156 fix(conformance): retain one-time policies across browser projections
- d213671dd3ead7e8747a1cf741caf520f031f0d7 feat(conformance): identify each generated disclosure obligation
- a0ce3e94456c742b9faf35b16eec833084eb6c52 fix(cli): refuse unsafe history import before reading input bytes
- f0092b8b7343b4e13097823c09692582ffe238e0 test(conformance): require precise generated runtime outcomes
- 86cf019bc1bd4b02132acc4cee80a5bf3285ea36 docs(conformance): explain disclosure policy and runtime outcome parity
- 161352189687d0dba403a582275c31a3523a9db9 test(conformance): share reviewed disclosure execution controls
- 0949cfbdc90b99096baf3617ee543ec15d113613 fix(conformance): keep disclosure identity errors compact
- f5f004456de719f63d50b7053d9816efbc9c569a feat(conformance): prepare TypeScript disclosure authority and identities
- 36cc70af3196a43fb80fa66508b215bb1b7b9860 feat(conformance): prepare Go disclosure authority and identities
- 827e16e63f9eb9c0976738b32ae64704ea7169f3 test(conformance): require real disclosure execution inside WebAssembly
- a673dfc13e670733009609aec738c79120f35fd2 test(conformance): pin exact disclosure payload resource boundaries
- 39ce375e0f783a0a3056bcca02b5ee2c0948c078 test(conformance): check exact resource limits inside WebAssembly
- 09c4e3ea861bdbe3e2e59cdcad96d3d0d260b3ca test(conformance): share coverage and multi-field disclosure controls
- 17a26b1d8852988f410ecdcf3f8d0b2fe228e158 test(conformance): cover successful identity disclosure in shared runtimes
- f9b9383434d41e54f19619581af942b218a28aa9 feat(conformance): integrate partial native disclosure execution checkpoint
- 3a11f5ae0098fd1d6f1a6f4ea70b2682216837f8 test(conformance): pin shared multiple-window observations
- a862b261a86bec6cb16a1456e2016a4566c09e37 test(synth): execute all shared disclosure controls through WASM bridge
- 967b30345b0c810cd37fc0aacf7a66a72ac5b094 feat(conformance): execute protected suites in generated Go runners
- 7cac5289100d629c9f094609373929e9f11dcca0 feat(conformance): integrate TypeScript disclosure execution checkpoint
- 862183e48dd8a47eff2c19bbf9e28040859b8cb1 fix(conformance): bound whole TypeScript event callback batches
- d8522ddfbcfeebf263fee2876ddd49b5ce2d7663 refactor(cli): share marked-model history policy predicate
- e7ae7cf93ecd51b9339b036e048613161189a61f fix(conformance): interpret guards against the held subject state
- d2b8827ec0e41dad1cd4daaa1f139417a1ff81c4 feat(conformance): compose stateful disclosure retries and follow-ups
- d170c67f6a59ad6449804a8a30b3d36261388bc2 fix(conformance): keep generated Go prerequisites formatted
- 348e444df6b08e1fd79228996e9cd313af4c7c98 fix(conformance): reconcile runtime parity and stop refused event polling
- 81d14bf059cad5827cf9755d26337ff5acabcde0 fix(examples): declare source facts and migrate billing timestamp callers
- ff128cbdb0952c57faa635b28e346951d189f22b feat(conformance): execute interpreter responses views and scenario facts
- 01ce4b82fac20736d24f822fecff53750bfc77fc test(conformance): migrate derived examples and preserve historical producer contracts
- 7681816bde435e10353bcd4ccc5a6d0efa7eb349 test(go): pin report categories and strict incomplete exits
- 374294a81277d902ff13124cdff26c10dcf7b4b8 fix(explore): draw deterministic Timestamp inputs in concurrent ports
- fe48502f53c04523e321d7cddb24f7ee802f58eb test(interpreter): require complete Billing execution and mutation parity
- 9ce0e593088d18ed8d0f738fcedcf647543f44fa fix(conformance): preserve legacy replay and update executable walkthroughs
- f227d08fb5b131f27569c054a6bfda9231bf06e6 feat(interpreter): execute repeated external controls through exact invocation counts
- 9aefe4f4074e0bed758b1a73b5c8f3615862de94 feat(interpreter): execute typed stored-field predicates and state snapshots
- 6e316e11d10bcbc49f3a0f9997578ab845c1aa72 test(interpreter): require stored guard outcomes and view observations
- 819f7c1a1635a28a64140cb558ed7017b65893de fix(conformance): integrate validated entity setup and creation identities
- b4a98ae06f2c666b0f4c315f06a6a10dff46c1a7 fix(conformance): execute explicit absent-input requests
- c88fa782f9de2a20376672a23d9098cf12e82a46 fix(conformance): integrate actual related-row predicate execution
- ee6962765749d25e019fa819c7b0019799aad464 fix(conformance): execute filtered and secondary set effects
- 2693e534806ebf07cfd59cc0c75608808924e166 fix(conformance): preserve typed interpreter identities
- fc676ff13b8b98d9f2213d151ce1335d5bbe756b fix(conformance): execute commands with invocation-local caller facts
- 641fdfa28abc7d5776af7b96f87ee515e2acb72a fix(conformance): execute supplied identities and existence branches
- 13e40c33f9f413306ace7bcbf741747ae0d27739 fix(conformance): declare isolation and exercise shared caller state
- 73faabfaf925fb47a7d656d5f37ed1e17c0ed68b fix(conformance): execute pre-outcome subject values and exact increments
- eaf7fde99a6bf3b05e94c199a6513bf2fd77b636 fix(conformance): read related values from original typed store
- 8b3f128a0e55419d976e330f93bc51a3d2cde941 fix(conformance): migrate fixtures without weakening legacy boundaries
- 9e8894b7f28d4ae7c02cb6e3144ce7022f06e0cf fix(conformance): integrate per-invocation caller authority

## Excluded candidates

- e9355b003a8c0153d927fbe89c8597cebc667787 feat(synth): emit memory storage and standalone served entries: Superseded318 served-entry implementation.
- 0e7fb770827c10038985105c8f4997ea74545043 fix(recovery): retain observed child termination signals: Separate CLI child-signal correction; outside this runtime handoff.
- 2291c5adfa61baad92b80efb752a841a11710723 chore(synth): regenerate examples with served entry support: Superseded318 generated corpus.
- 2969953014b954127712f01adbb3f95f94504012 test(synth): check fallible context and served entry surfaces: Superseded318 tests.
- d745061d753b2f7275020b0306e65d4676da4e3d docs: reconcile unreleased fixes with the actual source tag: Consolidate changelog once on final carrier.
- f26efc2eb600c9377e3bac5b7c57218712431c23 fix(types): preserve integer bounds and select exact native widths: Third-owner394; consume final main instead.
- da2dc12b5f0bb83f52782b0cdeedd1ac8c5bed40 test(types): execute generated integer codecs at wire boundaries: Third-owner394 tests; consume final main instead.
- 0a5f61d8523df5f38fa00211df1b4333efdc3014 feat(types): generate standalone event payload libraries: Duplicate393; preserve merged397 API instead.

All selected full path lists were checked: none includes .engineering, .github, Taskfile.yml or AGENTS.md. Planning artifacts transfer separately by owner. Private audit files source-transfer-full-paths.txt and integrator-changed-paths.txt retain the exact comparisons. No source cherry-pick/build was performed in the shared integrator by this handoff.

## Integrated native response addendum, 2026-10-03

Append source commit046db8a6805154aa5cafc63b0a4b741bc26e954d after the frozen57-commit source list, yielding58 selected source commits. The original frozen list and its hashes remain unchanged. This commit imports independently reviewed worker e9d543ac4bb5fd5d2a58d51b7fab8d60a62073af, preserving the existing support_versions::legacy_json helper in response_payload.rs. Both author and committer are b10x-bot[bot]. Nine source/test files only; no planning, generated artifact, delivery-control or unrelated documentation edit.

Integrated actual results:97library,9interpreted_existence,9interpreted_response_values,1interpreted_responses,7one_time_contract,21one_time_execution,17one_time_generation,8response_payload,2response_union,21upsert_by_existence:192passed0failed0ignored0filtered. Scoped strict Clippy covering that same library/test selection exits0. Repository task fmt-check exits0. The initial cargo fmt --all --check also reached byte-pinned generated Rust and reported formatting differences; the Taskfile explicitly excludes generated projections from formatter input. No generated bytes were rewritten. The scoped and repository-defined formatting checks both pass; package-wide/full release gates remain pending.

Carrier evidence SHA256: response-integrated-tests.log a8cfdc77f76745efc3baf11fe5e3f8cbe37021509ad619ae22d87fc3345a810b; response-integrated-clippy.log 5a1e3af04e934337e7ca5670f07289d6f796d4d37d654abacd54d8ae50794f9a; response-repository-fmt.log b1b92dbfb232bd4c5604171da6ab65153fb809d08740f37d511dce0c9d1fe085. Logs and terminal exit files are retained under carrier target/backlog-input.

Source tree ess-interpreted-response-values-20261003 remains retained on fix/interpreted-response-values-20261003 with local-only bot commit and review evidence. Root carrier ess-backlog-next-20261002 holds the integrated source on batch/consumer-runtime-20261002. The shared coordinator remains sole owner of batch/ui-live-apps-complete-20261003 and its eventual PR. The source handoff remains local and unacknowledged: cross-session MCP delivery still fails at its endpoint, so no receipt is inferred. No remote gate or component PR was started.

Full ess21 bundle hold remains. Nested response observation probe is independently recorded: healthy37/37 and unequal37/38 and38/37 all pass, while receipt=null fails. Native response execution fixes do not close that observer defect, browser product support, retained replay, or the full consumer backlog. PR402 remains open at9e15e08e9f0201e06bddc268934c3f96a92e5482; refreshed read shows CI progressing, not final Gate success, and unchanged source still lacks the reviewed contradictory-bound correction. Release owner retains that blocker.

## Integrator coordination refresh, 2026-10-03

The integrator reiterated one held ess/21 branch, batch/ui-live-apps-complete-20261003, with no partial PR or merge. Root provided the already-frozen 57-commit runtime manifest plus actual-response046db8a680 addendum (58 selected source commits) and exact local artifact links in the active conversation. Their SHA256 values remain69fde33220abb186d262f0a77b41844dc4da676096956f96c51e75e29f182cfa andfa5a3ae3bcab3677629cd7a4598813a43fa1befc46240b78a81f96e67ff467a7. Reverification agrees. Do not merge the carrier wholesale; original exclusions and narrow318 overlaps apply.

A new direct bridge send still failed at transport; no delivery acknowledgment is inferred. The latest explicit owner message says release398 is consolidated with merged402 and owner-approved403, with403 to close superseded. Its candidate5a5ac7f74 is frozen and full gates are running. It contains no held ess/21 source. This is owner-provided coordination, not root proof of final Gate or release. All transport development remains the third session's. Nested responses,292,293 and the remaining runtime/aggregate/browser work remain unfinished follow-up units for the same integration carrier, not additional delivery PRs.

## Explicit integrator receipt, 2026-10-03

Integrator01a0fc77 explicitly confirmed reading and retaining both frozen handoff files at their exact SHA256 values69fde33220abb186d262f0a77b41844dc4da676096956f96c51e75e29f182cfa andfa5a3ae3bcab3677629cd7a4598813a43fa1befc46240b78a81f96e67ff467a7. All58 selected source commits through046db8a6805 and overlap/exclusion directions are recorded in their shared runbook revision19. This receipt supersedes the earlier unacknowledged state; it is not proof of transfer, validation or published integration.

Native source transfer/verification follows release completion; dependent waves remain held until it passes. No wholesale carrier merge, partial ess21 release or duplicate PR. Root retains every source tree and retained evidence until the integrator explicitly verifies published integration. New local units remain follow-ups on the same carrier.

Owner reports PR398 published at5a5ac7f74,403 closed superseded,402 merged; full local/remote release gates running. No final Gate, tag or artifact success is inferred from that coordination message.
