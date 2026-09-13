# Authorship rewrite of 2026-09-13: what every cited hash became


`b10x-gates` refused the push of 63 unpushed commits with `commit
f3ada66f956100b9aca8146621eb4e5dd73262e2 has inadmissible authorship`. Fifteen of the 63 were merge
commits carrying a person's identity rather than `b10x-bot[bot]`, because `b10x-gates bot` accepts
`commit`, `tag`, `push` and `fetch` and has no `merge`, so every wave's merges were made with plain
`git merge` and took the local identity.

The fifteen were rewritten to the bot identity, which changed the hash of every descendant: 53 of the 63.
**No tree changed** — `git diff` between the pre-rewrite tip and the rewritten tip is empty.

None of the old hashes ever reached `origin`. They exist only in this repository's own records, and
that is the problem this page solves: `.engineering/planning/` is append-only, so a commit id written
into a journal line or an evidence record cannot be corrected. The table below is the correction.

## The mapping


| before | after | commit |
|---|---|---|
| `949153ea` | `85fe4297` | plan: say where ess-evolution actually stands, and stop calling two started epics drafts |
| `b58a3b6d` | `759838f6` | chore: release ESS 0.24.0 |
| `4dfdef37` | `d3669f55` | merge: the model-driven interpretation wave, closed on its branch and never landed |
| `dcaa4afc` | `4fe1e41e` | plan: file the accounting baseline the thirteen-gap change never extended |
| `5b55cdc4` | `da590d9d` | fix(plan): the wave page named this workstation's checkout path |
| `777ffc87` | `c753fbaa` | fix(xtask): classify the entries the thirteen-gap change added, and re-pin what it moved |
| `69daf39e` | `4000238f` | plan: move six stories the store still called active and main had shipped |
| `237f1556` | `208f57d9` | Merge main into the model-driven interpretation wave |
| `5b975c78` | `93c72a62` | docs(plan): record what wave 25's close actually ran |
| `9dc23b39` | `14920f95` | merge(wave-25): a name means what the copy that converts says it means, and a component declares its settings |
| `74687b7e` | `284a6ad3` | fix(xtask): re-review the definitions container the new setting changed |
| `23a48900` | `497756de` | plan(wave-25): close the wave, with the scope it learned written back |
| `0bd387e4` | `661362c9` | Merge branch 'main' into wave/ess-wave-25 |
| `525a4a19` | `7e14af91` | test(compiler): run the duplicate-declaration case its story landed for |
| `df57adb6` | `afe67399` | merge(wave-25): a component declares its settings |
| `a63273c5` | `6d74d6c2` | merge(wave-25): a name means what the copy that converts says it means |
| `1e9f48f1` | `8355adf2` | feat(domain): a component declares its settings, and the runtime slots derive from them |
| `0321f0f2` | `dbf5ed4a` | fix(domain): a name means what the copy that converts says it means |
| `ce583ce8` | `4e8c8831` | fix(xtask): match a name in every dress it is worn in, not the one it was registered in |
| `017258d8` | `01a79228` | merge(wave-25): a macro invocation's identity comes from the invocation |
| `6018f4c4` | `847e1fe8` | fix(xtask): give a macro invocation an identity that does not come from a list |
| `080c9f5c` | `e7d19bc6` | merge: the browser fixture's refusal class, with its open defects pinned |
| `dc8c3193` | `5e4d7daf` | test(ess-cli): pin the five refusals this fixture cannot yet answer |
| `dfcaa5c8` | `ac3da4a8` | fix(xtask): the store gained a file, and the new lane exempted one it did not need |
| `e72f124c` | `a27dfc70` | merge: the unread-tree bullet parse, with its open refusals pinned |
| `5cab6810` | `2c520ec0` | test(xtask): pin the three refusals this lane cannot yet answer |
| `751213dc` | `e27c9d2e` | fix(privacy): redact the organisation this repository was written inside, and gate it |
| `c905289a` | `21fd733e` | chore(planning): record waves 22 to 25, and redact what the store should never have carried |
| `7151c6fa` | `3d5ec38e` | docs(plan): the wave pages for 24 and 25, and the ESSv2 first-half gap analysis |
| `f85889ca` | `01b3b642` | feat(models): specify the ESS toolchain in ESS, and gate its enums against the source |
| `715fb688` | `40be38f5` | wip(xtask): read what an unread-tree bullet claims, not just the tree it names |
| `eaf7ca12` | `e1beefb5` | wip(ess-cli): report a lost browser start as a fixture environment refusal |
| `e5a97603` | `ff4520ee` | merge(wave-24): name the enum a refusal is about, and cite the file that holds it |
| `ab2bb7f1` | `592f9b0c` | merge(wave-24): name the enum a refusal is about, and cite the file that holds it |
| `04e94de0` | `4824d08e` | fix(compiler): name the enum a refusal is about, and cite the file that holds it |
| `bd722fa9` | `8aa08818` | merge(wave-23): one uniqueItems decision wherever this crate validates |
| `ecf83b34` | `d0d2f708` | merge(wave-23): read every runner a workflow names |
| `eb240851` | `6f8f5c60` | chore(xtask): classify the consumer entries wave 23's domain change mints |
| `5afd8ab2` | `660a66ea` | fix(schema-contract): decide uniqueItems the same way wherever this crate validates |
| `f54c64f5` | `0d65eeb5` | fix(xtask): read every runner a workflow names, and check every label it yields |
| `3f15c7a8` | `edc95d5d` | merge(wave-23): one pass owns a literal into a type no value inhabits |
| `58ceca04` | `accbfc42` | fix(domain): let one pass own a literal into a type no value inhabits |
| `5764fc37` | `1b8d7f52` | docs(plan): stop the wave-22 page citing a path the repository does not hold |
| `46c7281e` | `d2054c31` | fix(xtask): hold the adversary's demonstration cases to today's behaviour |
| `cc0a114e` | `7f2a4e67` | merge(wave-22): repository-relative fixture roots, and a gate lane that refuses a host path |
| `85566dbc` | `7e67d6f0` | docs(plan): wave 22 — unit 1 merged, the docs class scrubbed, unit 2 committed |
| `2589564b` | `93dfb69d` | docs: replace the workstation's home path with the portable spelling |
| `9403ac9c` | `9d35adce` | fix(gen,compiler): say what literal checking now does, and stop asserting what it did |
| `24fd246b` | `6e91d8ed` | merge(wave-22): every literal a document writes is checked, however deep |
| `d323cc3c` | `00a1a82a` | docs(plan): wave 22 — unit 1 correction green, and the patch the coordinator mis-routed |
| `51133491` | `39a9abd7` | docs(plan): wave 22 — adversary pass 1 routing and the held compiler patch |
| `cd9e121c` | `496d7cb2` | docs(plan): record ESS wave 22 — selection, pre-flight and unit stages |
| `f3ada66f` | `1e2a3ac3` | merge(wave-22): the Integer half of canonical number serialization |

Nine further cited ids are unchanged, because nothing under them was rewritten: `0a21a499`,
`1268e06f`, `1a2effd6`, `3807cb68`, `436fcbfa`, `560b5690`, `946a3b26`, `cd74b055` and `d13c152c`.

## What this does not fix

A reader who follows a citation still reads the old id first and has to come here. Nothing rewrites
the store, and nothing should: an append-only journal that gains an edit is no longer append-only,
and `aep plan artifact validate` reads such an edit as forgery.

The way not to need this page again is for a wave's merges to carry the bot identity when they are
made — `git merge --no-commit` followed by `b10x-gates bot -- commit`, since the wrapper has no merge
verb of its own.

