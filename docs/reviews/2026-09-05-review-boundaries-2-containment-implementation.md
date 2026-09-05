unit:                   story:review-output-containment — Validate output paths and page uniqueness before writing
verdict:                green
cases:                  executed 204→220, red 6 observed, 0 remaining
origin:                 n/a
wrote-outside-worktree: none
needs-coordinator:      no

## 1. Unit and acceptance

story:review-output-containment: An escaping or colliding artifact/page path is refused before any generated output changes inside or outside the requested root.

Base: c1c23b24cee8f527784b7f8467c21a609710c65e, branch impl/review-output-containment. Changes are uncommitted and ready for adversarial review; this report does not claim integration or landing. The graph has no depends_on edge for this story.

## 2. Observed diff and scope confirmation

```text
$ git --no-pager diff --stat
 crates/edge/ess-cli/src/main.rs         | 350 ++++++++++++++++++++++++++++----
 crates/generate/ess-gen/src/artifact.rs | 135 ++++++++++++
 crates/generate/ess-gen/src/document.rs |  21 ++
 crates/generate/ess-gen/src/html.rs     |  60 ++++++
 4 files changed, 525 insertions(+), 41 deletions(-)

$ git diff --no-index --stat /dev/null crates/edge/ess-cli/tests/output_containment.rs
 .../edge/ess-cli/tests/output_containment.rs       | 189 +++++++++++++++++++++
 1 file changed, 189 insertions(+)
```

The second command includes the new, deliberately unstaged integration-test file; exit 1 is the normal result of that diff.

| Scope or hypothesis | Actual paths and evidence | Result |
| --- | --- | --- |
| CLI destinations and caller-controlled includes | crates/edge/ess-cli/src/main.rs: included, generate, write_artifacts; tests/output_containment.rs exercises the shipped CLI | Confirmed |
| Artifact path admission and collisions | crates/generate/ess-gen/src/artifact.rs: additive Artifact.validate, validate_path, validate_paths; in-module matrix checks portable paths and file/directory conflicts in both orders | Confirmed |
| Page identities remain publicly mutable/deserializable | crates/generate/ess-gen/src/document.rs: additive PageId.validate and Document.validate_page_ids; constructors and serde representation unchanged | Confirmed |
| Site collection can discard duplicate pages | crates/generate/ess-gen/src/html.rs: Site.try_render validates page identities and the entire rendered sequence, including assets, before CLI map collection | Confirmed by initial duplicate-page red run |
| Site keys and filesystem paths differ | CLI still keys site output with site/ while writing actual Artifact.path. The valid nested include test checks the existing root-relative layout and repeat bytes | Confirmed; no layout migration |
| Full preflight can prevent partial changes from invalid destinations | write_generated_files collects the sequence, validates all names, inspects all existing root/parent/leaf entries, then writes. Initial CLI run was 1 passed/5 failed; mechanism run is 6 passed/0 failed | Confirmed by measurement |
| Additional sinks stay within CLI scope | compose client Rust output, synthesize_suite Go output, write_projection_files (build/Docker/Helm), project_kubernetes now call the common writer. synthesize and conform_web already call write_artifacts | Confirmed, no producer-package changes |
| Rust docs must state a bounded threat model | artifact.rs documents canonical ASCII destinations and case aliases; main.rs documents symlink/hardlink checks, root resolution and exclusions | Confirmed |
| Caller-selected roots are distinct from generated paths | Coordinator caught the initial overbroad refusal of .. in an output root. A new regression failed before correction; preflight now resolves the requested root while checking every encountered existing component before .. can discard it, and writers use that resolved root | Corrected with red/green evidence |
| A binding design is required for a new construct | No persisted noun, envelope, canonicalization, identity or representation was introduced; this change validates existing Artifact/PageId identities and filesystem destinations | No new design construct required |

The class is all generated relative file destinations owned by these CLI sinks, rather than only included site pages. The lexical checker refuses traversal, noncanonical separators/components, drive/UNC/ADS forms, Windows device names, and non-ASCII aliases. Set validation refuses duplicate/case-aliased files, directory spelling aliases, and file/parent conflicts before any collection can erase the relevant site duplicate. Filesystem checks refuse existing symlink roots, ancestors and destinations (including dangling symlinks), nonregular leaves, nondirectory parents, existing case aliases, and Unix destination files with more than one hard link. Constructors stay source-compatible; callers handling untrusted library values have additive checked APIs.

## 3. Red runs, verbatim

All Cargo commands used:
```text
TMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/target/review-boundaries-2
RUSTC_WRAPPER=/usr/bin/sccache
CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
```

The first six CLI cases were written and executed before production edits. The valid nested layout/byte control passed on the baseline; five refusal cases failed by changing sentinel bytes. Later library and sink matrices extend those cases; their counts are not misrepresented as separate initial red cases.

```text
$ cargo test --locked -p ess-cli --test output_containment
   Compiling ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.20s
     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)

running 6 tests
test a_hardlinked_destination_is_refused_before_other_files_change ... FAILED
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... FAILED
test an_escaping_include_is_refused_before_any_output_changes ... FAILED
test noncanonical_and_platform_paths_are_refused_before_writing ... FAILED
test symlink_roots_parents_and_destinations_are_refused_before_writing ... FAILED
test a_valid_nested_include_keeps_the_existing_site_layout_and_bytes ... ok

failures:

---- a_hardlinked_destination_is_refused_before_other_files_change stdout ----

thread 'a_hardlinked_destination_is_refused_before_other_files_change' (4180689) panicked at crates/edge/ess-cli/tests/output_containment.rs:42:9:
assertion `left == right` failed
  left: "<!DOCTYPE html>\n<!--\n  generated from billing v3\n  model digest 56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942\n  contract digest cb634bd5e6f1afa6ebc8e9dca752e9901a9a68a2e51fc5009d099f155680606c\n  do not edit: regenerate with `ess generate`\n-->\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>Authored page · billing</title>\n<link rel=\"stylesheet\" href=\"../assets/style.css\">\n</head>\n<body>\n<div class=\"shell\">\n<nav class=\"sidebar\">\n<a class=\"masthead\" href=\"../index.html\">\n<span class=\"system\">billing</span><span class=\"version\">v3</span>\n</a>\n<div class=\"nav\">\n<a href=\"../index.html\">billing v3</a>\n<a href=\"../crossings.html\">Type crossings</a>\n<div class=\"group\">domains</div>\n<a class=\"nested\" href=\"../domains/billing-email.html\">email</a>\n<a class=\"nested\" href=\"../domains/billing-invoice.html\">Invoicing</a>\n<a href=\"../interactions.html\">Interactions</a>\n<div class=\"group\">plan</div>\n<a class=\"nested\" href=\"board.html\" aria-current=\"page\">Authored page</a>\n<a href=\"../topology.html\">Topology</a>\n</div>\n</nav>\n<main>\n<article class=\"content\">\n<h1>Authored page</h1>\n<p>Kept verbatim.</p>\n</article>\n<footer class=\"provenance\">\nGenerated from billing v3 · model digest <code>56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942</code> · contract digest <code>cb634bd5e6f1afa6ebc8e9dca752e9901a9a68a2e51fc5009d099f155680606c</code>. Do not edit this file; change the specification and regenerate it with <code>ess generate</code>.\n</footer>\n</main>\n</div>\n<script type=\"module\">\nconst diagrams = document.querySelectorAll(\"pre.mermaid\");\nif (diagrams.length > 0) {\n  // A script element, not `import`: the bundle publishes itself with a top-level `var`, and a\n  // top-level `var` in a module is scoped to the module, so importing it throws.\n  await new Promise((resolve, reject) => {\n    const tag = document.createElement(\"script\");\n    tag.src = \"../assets/mermaid.min.js\";\n    tag.addEventListener(\"load\", resolve);\n    tag.addEventListener(\"error\", reject);\n    document.head.append(tag);\n  });\n  globalThis.mermaid.initialize({\n    startOnLoad: false,\n    securityLevel: \"strict\",\n    theme: window.matchMedia(\"(prefers-color-scheme: dark)\").matches ? \"dark\" : \"default\",\n  });\n  await globalThis.mermaid.run({ querySelector: \"pre.mermaid\" });\n}\n</script>\n</body>\n</html>\n"
 right: "outside sentinel"

---- include_aliases_and_duplicate_generated_pages_are_refused_before_writing stdout ----

thread 'include_aliases_and_duplicate_generated_pages_are_refused_before_writing' (4180692) panicked at crates/edge/ess-cli/tests/output_containment.rs:43:9:
assertion `left == right` failed
  left: "<!DOCTYPE html>\n<!--\n  generated from billing v3\n  model digest 56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942\n  contract digest cb634bd5e6f1afa6ebc8e9dca752e9901a9a68a2e51fc5009d099f155680606c\n  do not edit: regenerate with `ess generate`\n-->\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>Authored page · billing</title>\n<link rel=\"stylesheet\" href=\"assets/style.css\">\n</head>\n<body>\n<div class=\"shell\">\n<nav class=\"sidebar\">\n<a class=\"masthead\" href=\"index.html\">\n<span class=\"system\">billing</span><span class=\"version\">v3</span>\n</a>\n<div class=\"nav\">\n<a href=\"index.html\" aria-current=\"page\">billing v3</a>\n<a href=\"index.html\" aria-current=\"page\">Authored page</a>\n<a href=\"crossings.html\">Type crossings</a>\n<div class=\"group\">domains</div>\n<a class=\"nested\" href=\"domains/billing-email.html\">email</a>\n<a class=\"nested\" href=\"domains/billing-invoice.html\">Invoicing</a>\n<a href=\"interactions.html\">Interactions</a>\n<a href=\"topology.html\">Topology</a>\n</div>\n</nav>\n<main>\n<article class=\"content\">\n<h1>Authored page</h1>\n<p>The single normative example. Every snippet in the ESS design document is meant to be derivable from this directory, and a test in <code>ess-domain</code> parses it — so a change here that the model cannot express fails the build rather than quietly making the document wrong.</p>\n<p>The system it describes is deliberately the smallest one that exercises everything wave 1 must model:</p>\n<pre><code class=\"language-text\">billing.invoice                      billing.email\n  CreateInvoice ──▶ InvoiceCreated     SendEmail ──▶ EmailSent\n        │                                  │\n        ▼                                  ▼\n  Invoice (Draft → Issued → Paid | Cancelled)\n        │\n        ▼\n  InvoiceById (eventual)</code></pre>\n<p>Two bounded contexts, a command with <strong>two outcomes</strong>, a command with an outcome the input cannot decide, events, both consistency levels, a filtered view, actors, and a state machine whose illegal transitions are illegal by absence.</p>\n<p>A test — <code>the_example_exercises_every_construct_the_model_has</code> — asserts that every type body, every primitive, <code>Optional</code>/<code>List</code>/<code>Map</code>, both consistency levels, an actor with grants and one without, an error carrying a payload and a command with an overridden wire name all appear here. <strong>A construct added to the model without reaching this directory fails the build</strong>, because what the normative example leaves out is what nothing checks.</p>\n<p><strong>A component is not a deployment.</strong> <code>invoice-service</code> owning <code>billing.invoice</code> says the invoice context is one unit of ownership; whether it ships as its own process or as a module inside one binary is <code>topology.yaml</code>&#39;s business, and changing that answer changes nothing in <code>domains/</code>. That separation is the point of specifying a system semantically, and it is why the three layers are three files.</p>\n<h2 id=\"what-each-file-is-for\">What each file is for</h2>\n<table>\n<thead>\n<tr><th>File</th><th>Why it exists</th></tr>\n</thead>\n<tbody>\n<tr><td><code>system.yaml</code></td><td>the format version, the system&#39;s identity, and which domains it has</td></tr>\n<tr><td><code>domains/invoice.yaml</code></td><td>the invoice bounded context: every type kind, an entity with a lifecycle, actors, a refusable command, an event, and both kinds of view</td></tr>\n<tr><td><code>domains/email.yaml</code></td><td>the second context, so cross-domain references are exercised rather than assumed, and the command whose failure the input cannot decide</td></tr>\n<tr><td><code>components.yaml</code></td><td>who owns which context, the binding between them, and the one type crossing that binding needs</td></tr>\n<tr><td><code>topology.yaml</code></td><td>what the system needs in order to run — modelled, and deployed by nothing</td></tr>\n</tbody>\n</table>\n<h2 id=\"three-things-worth-reading-closely\">Three things worth reading closely</h2>\n<p><strong><code>CreateInvoice</code> has outcomes, not an <code>emits</code> list.</strong> A command with a precondition has at least two results, and a specification that records only the happy one generates tests that say nothing about the branch where the money does not move.</p>\n<p><strong><code>InvoiceById</code> declares its consistency.</strong> It is a projection, so it is <code>eventual</code>, so a generated scenario must assert it with <code>eventually</code> rather than immediately. Getting this wrong produces a suite that passes on a laptop and flakes in CI, and the usual fix — a sleep — makes the suite test the machine it runs on.</p>\n<p><strong><code>Paid</code> cannot become <code>Cancelled</code>, and no rule says so.</strong> There is simply no transition. A rule would be a second place for the truth to live, and the two would eventually disagree.</p>\n<p><strong><code>Account</code> owns its invoices, and says so.</strong> The <code>relations:</code> block on <code>billing.invoice.Account</code> declares that it <code>owns</code> <code>many</code> <code>billing.invoice.Invoice</code>, carried by the invoice&#39;s own <code>account_id</code> field — one declaration, on the owner, rather than a typed id on the child plus an invariant nobody wrote. <code>ess specify validate</code> refuses that relation if the target is not an entity, if <code>account_id</code> is missing or typed as anything but <code>billing.invoice.AccountId</code>, or if a second entity claims to own invoices too. Every projection carries it under <code>x-ess-relation</code>, on the property holding <code>account_id</code>: see <code>generated/schema/entities/billing.invoice.Invoice.schema.json</code>, the <code>x-ess-entities</code> table of <code>generated/openapi/invoice-service.yaml</code>, and the doc comment on <code>InvoiceData::account_id</code> in <code>generated/rust/billing/</code>.</p>\n<p>Kept verbatim.</p>\n</article>\n<footer class=\"provenance\">\nGenerated from billing v3 · model digest <code>56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942</code> · contract digest <code>cb634bd5e6f1afa6ebc8e9dca752e9901a9a68a2e51fc5009d099f155680606c</code>. Do not edit this file; change the specification and regenerate it with <code>ess generate</code>.\n</footer>\n</main>\n</div>\n<script type=\"module\">\nconst diagrams = document.querySelectorAll(\"pre.mermaid\");\nif (diagrams.length > 0) {\n  // A script element, not `import`: the bundle publishes itself with a top-level `var`, and a\n  // top-level `var` in a module is scoped to the module, so importing it throws.\n  await new Promise((resolve, reject) => {\n    const tag = document.createElement(\"script\");\n    tag.src = \"assets/mermaid.min.js\";\n    tag.addEventListener(\"load\", resolve);\n    tag.addEventListener(\"error\", reject);\n    document.head.append(tag);\n  });\n  globalThis.mermaid.initialize({\n    startOnLoad: false,\n    securityLevel: \"strict\",\n    theme: window.matchMedia(\"(prefers-color-scheme: dark)\").matches ? \"dark\" : \"default\",\n  });\n  await globalThis.mermaid.run({ querySelector: \"pre.mermaid\" });\n}\n</script>\n</body>\n</html>\n"
 right: "inside sentinel"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- an_escaping_include_is_refused_before_any_output_changes stdout ----

thread 'an_escaping_include_is_refused_before_any_output_changes' (4180691) panicked at crates/edge/ess-cli/tests/output_containment.rs:42:9:
assertion `left == right` failed
  left: "<!DOCTYPE html>\n<!--\n  generated from billing v3\n  model digest 56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942\n  contract digest cb634bd5e6f1afa6ebc8e9dca752e9901a9a68a2e51fc5009d099f155680606c\n  do not edit: regenerate with `ess generate`\n-->\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>Authored page · billing</title>\n<link rel=\"stylesheet\" href=\"../assets/style.css\">\n</head>\n<body>\n<div class=\"shell\">\n<nav class=\"sidebar\">\n<a class=\"masthead\" href=\"../index.html\">\n<span class=\"system\">billing</span><span class=\"version\">v3</span>\n</a>\n<div class=\"nav\">\n<a href=\"../index.html\">billing v3</a>\n<div class=\"group\">..</div>\n<a class=\"nested\" href=\"escaped.html\" aria-current=\"page\">Authored page</a>\n<a href=\"../crossings.html\">Type crossings</a>\n<div class=\"group\">domains</div>\n<a class=\"nested\" href=\"../domains/billing-email.html\">email</a>\n<a class=\"nested\" href=\"../domains/billing-invoice.html\">Invoicing</a>\n<a href=\"../interactions.html\">Interactions</a>\n<a href=\"../topology.html\">Topology</a>\n</div>\n</nav>\n<main>\n<article class=\"content\">\n<h1>Authored page</h1>\n<p>Kept verbatim.</p>\n</article>\n<footer class=\"provenance\">\nGenerated from billing v3 · model digest <code>56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942</code> · contract digest <code>cb634bd5e6f1afa6ebc8e9dca752e9901a9a68a2e51fc5009d099f155680606c</code>. Do not edit this file; change the specification and regenerate it with <code>ess generate</code>.\n</footer>\n</main>\n</div>\n<script type=\"module\">\nconst diagrams = document.querySelectorAll(\"pre.mermaid\");\nif (diagrams.length > 0) {\n  // A script element, not `import`: the bundle publishes itself with a top-level `var`, and a\n  // top-level `var` in a module is scoped to the module, so importing it throws.\n  await new Promise((resolve, reject) => {\n    const tag = document.createElement(\"script\");\n    tag.src = \"../assets/mermaid.min.js\";\n    tag.addEventListener(\"load\", resolve);\n    tag.addEventListener(\"error\", reject);\n    document.head.append(tag);\n  });\n  globalThis.mermaid.initialize({\n    startOnLoad: false,\n    securityLevel: \"strict\",\n    theme: window.matchMedia(\"(prefers-color-scheme: dark)\").matches ? \"dark\" : \"default\",\n  });\n  await globalThis.mermaid.run({ querySelector: \"pre.mermaid\" });\n}\n</script>\n</body>\n</html>\n"
 right: "outside sentinel"

---- noncanonical_and_platform_paths_are_refused_before_writing stdout ----

thread 'noncanonical_and_platform_paths_are_refused_before_writing' (4180693) panicked at crates/edge/ess-cli/tests/output_containment.rs:43:9:
assertion `left == right` failed
  left: "<!DOCTYPE html>\n<!--\n  generated from billing v3\n  model digest 56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942\n  contract digest cb634bd5e6f1afa6ebc8e9dca752e9901a9a68a2e51fc5009d099f155680606c\n  do not edit: regenerate with `ess generate`\n-->\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>billing v3 · billing</title>\n<link rel=\"stylesheet\" href=\"assets/style.css\">\n</head>\n<body>\n<div class=\"shell\">\n<nav class=\"sidebar\">\n<a class=\"masthead\" href=\"index.html\">\n<span class=\"system\">billing</span><span class=\"version\">v3</span>\n</a>\n<div class=\"nav\">\n<a href=\"index.html\" aria-current=\"page\">billing v3</a>\n<a href=\".html\">Authored page</a>\n<a href=\"crossings.html\">Type crossings</a>\n<div class=\"group\">domains</div>\n<a class=\"nested\" href=\"domains/billing-email.html\">email</a>\n<a class=\"nested\" href=\"domains/billing-invoice.html\">Invoicing</a>\n<a href=\"interactions.html\">Interactions</a>\n<a href=\"topology.html\">Topology</a>\n</div>\n</nav>\n<main>\n<article class=\"content\">\n<h1>billing v3</h1>\n<p>The single normative example. Every snippet in the ESS design document is meant to be derivable from this directory, and a test in <code>ess-domain</code> parses it — so a change here that the model cannot express fails the build rather than quietly making the document wrong.</p>\n<p>The system it describes is deliberately the smallest one that exercises everything wave 1 must model:</p>\n<pre><code class=\"language-text\">billing.invoice                      billing.email\n  CreateInvoice ──▶ InvoiceCreated     SendEmail ──▶ EmailSent\n        │                                  │\n        ▼                                  ▼\n  Invoice (Draft → Issued → Paid | Cancelled)\n        │\n        ▼\n  InvoiceById (eventual)</code></pre>\n<p>Two bounded contexts, a command with <strong>two outcomes</strong>, a command with an outcome the input cannot decide, events, both consistency levels, a filtered view, actors, and a state machine whose illegal transitions are illegal by absence.</p>\n<p>A test — <code>the_example_exercises_every_construct_the_model_has</code> — asserts that every type body, every primitive, <code>Optional</code>/<code>List</code>/<code>Map</code>, both consistency levels, an actor with grants and one without, an error carrying a payload and a command with an overridden wire name all appear here. <strong>A construct added to the model without reaching this directory fails the build</strong>, because what the normative example leaves out is what nothing checks.</p>\n<p><strong>A component is not a deployment.</strong> <code>invoice-service</code> owning <code>billing.invoice</code> says the invoice context is one unit of ownership; whether it ships as its own process or as a module inside one binary is <code>topology.yaml</code>&#39;s business, and changing that answer changes nothing in <code>domains/</code>. That separation is the point of specifying a system semantically, and it is why the three layers are three files.</p>\n<h2 id=\"what-each-file-is-for\">What each file is for</h2>\n<table>\n<thead>\n<tr><th>File</th><th>Why it exists</th></tr>\n</thead>\n<tbody>\n<tr><td><code>system.yaml</code></td><td>the format version, the system&#39;s identity, and which domains it has</td></tr>\n<tr><td><code>domains/invoice.yaml</code></td><td>the invoice bounded context: every type kind, an entity with a lifecycle, actors, a refusable command, an event, and both kinds of view</td></tr>\n<tr><td><code>domains/email.yaml</code></td><td>the second context, so cross-domain references are exercised rather than assumed, and the command whose failure the input cannot decide</td></tr>\n<tr><td><code>components.yaml</code></td><td>who owns which context, the binding between them, and the one type crossing that binding needs</td></tr>\n<tr><td><code>topology.yaml</code></td><td>what the system needs in order to run — modelled, and deployed by nothing</td></tr>\n</tbody>\n</table>\n<h2 id=\"three-things-worth-reading-closely\">Three things worth reading closely</h2>\n<p><strong><code>CreateInvoice</code> has outcomes, not an <code>emits</code> list.</strong> A command with a precondition has at least two results, and a specification that records only the happy one generates tests that say nothing about the branch where the money does not move.</p>\n<p><strong><code>InvoiceById</code> declares its consistency.</strong> It is a projection, so it is <code>eventual</code>, so a generated scenario must assert it with <code>eventually</code> rather than immediately. Getting this wrong produces a suite that passes on a laptop and flakes in CI, and the usual fix — a sleep — makes the suite test the machine it runs on.</p>\n<p><strong><code>Paid</code> cannot become <code>Cancelled</code>, and no rule says so.</strong> There is simply no transition. A rule would be a second place for the truth to live, and the two would eventually disagree.</p>\n<p><strong><code>Account</code> owns its invoices, and says so.</strong> The <code>relations:</code> block on <code>billing.invoice.Account</code> declares that it <code>owns</code> <code>many</code> <code>billing.invoice.Invoice</code>, carried by the invoice&#39;s own <code>account_id</code> field — one declaration, on the owner, rather than a typed id on the child plus an invariant nobody wrote. <code>ess specify validate</code> refuses that relation if the target is not an entity, if <code>account_id</code> is missing or typed as anything but <code>billing.invoice.AccountId</code>, or if a second entity claims to own invoices too. Every projection carries it under <code>x-ess-relation</code>, on the property holding <code>account_id</code>: see <code>generated/schema/entities/billing.invoice.Invoice.schema.json</code>, the <code>x-ess-entities</code> table of <code>generated/openapi/invoice-service.yaml</code>, and the doc comment on <code>InvoiceData::account_id</code> in <code>generated/rust/billing/</code>.</p>\n<p>Invoicing and the notification that follows it: the smallest system that still exercises every construct the model has — two bounded contexts, a command that can be refused, a command with an outcome its input cannot decide, both consistency levels, a state machine, actors, and a type of every kind.</p>\n<h2 id=\"the-system-as-a-graph\">The system as a graph</h2>\n<pre class=\"mermaid\">flowchart TB\n    subgraph who[&quot;who may ask&quot;]\n        who0[&quot;billing.invoice.Auditor&quot;]\n        who1[&quot;billing.invoice.Customer&quot;]\n    end\n    subgraph unit0[&quot;email-service&quot;]\n        cmd0[&quot;billing.email.SendEmail&quot;]\n        evt0[&quot;billing.email.DeliveryEscalated&quot;]\n        evt1[&quot;billing.email.EmailSent&quot;]\n    end\n    subgraph unit1[&quot;invoice-service&quot;]\n        cmd1[&quot;billing.invoice.CancelInvoice&quot;]\n        cmd2[&quot;billing.invoice.CreateInvoice&quot;]\n        cmd3[&quot;billing.invoice.IssueInvoice&quot;]\n        cmd4[&quot;billing.invoice.PayInvoice&quot;]\n        evt2[&quot;billing.invoice.InvoiceCancelled&quot;]\n        evt3[&quot;billing.invoice.InvoiceCreated&quot;]\n        evt4[&quot;billing.invoice.InvoiceIssued&quot;]\n        evt5[&quot;billing.invoice.InvoicePaid&quot;]\n    end\n    who1 --&gt;|&quot;may invoke&quot;| cmd2\n    cmd0 --&gt;|&quot;sent&quot;| evt1\n    cmd1 --&gt;|&quot;cancelled&quot;| evt2\n    cmd2 --&gt;|&quot;accepted&quot;| evt3\n    cmd3 --&gt;|&quot;issued&quot;| evt4\n    cmd4 --&gt;|&quot;settled&quot;| evt5\n    evt3 -.-&gt;|&quot;notify-on-invoice-created&quot;| cmd0</pre>\n<p>A command is accepted by the component that owns its context, emits the events one of its outcomes declares, and a dashed edge is a binding carrying an event into the next command. Design §9 begins one step earlier, at the actor who invokes the first command, and so does this graph: a solid edge out of an actor is a grant, and an actor drawn with no edge at all may invoke nothing — which is something the model says, not an arrow somebody forgot.</p>\n<h2 id=\"bounded-contexts\">Bounded contexts</h2>\n<ul>\n<li><strong><a href=\"domains/billing-email.html\">email</a></strong> (<code>billing.email</code>) — Sending the notifications other contexts ask for. Three types, no entities, no views, one command, two events, one error and no actors.</li>\n<li><strong><a href=\"domains/billing-invoice.html\">Invoicing</a></strong> (<code>billing.invoice</code>) — Issuing invoices and tracking whether they are paid. Eight types, two entities, two views, four commands, four events, two errors and two actors.</li>\n</ul>\n<h2 id=\"components\">Components</h2>\n<p>A component is a unit of ownership, not a deployment. How many of each runs, and what each needs, is <a href=\"topology.html\">the topology</a>.</p>\n<p><strong><code>email-service</code></strong> — Sends what other contexts ask it to send. It owns <a href=\"domains/billing-email.html\"><code>billing.email</code></a>. It accepts <code>billing.email.SendEmail</code>. It publishes <code>billing.email.DeliveryEscalated</code> and <code>billing.email.EmailSent</code>.</p>\n<p><strong><code>invoice-service</code></strong> — Issues invoices and tracks payment. It owns <a href=\"domains/billing-invoice.html\"><code>billing.invoice</code></a>. It accepts <code>billing.invoice.CancelInvoice</code>, <code>billing.invoice.CreateInvoice</code>, <code>billing.invoice.IssueInvoice</code> and <code>billing.invoice.PayInvoice</code>. It publishes <code>billing.invoice.InvoiceCancelled</code>, <code>billing.invoice.InvoiceCreated</code>, <code>billing.invoice.InvoiceIssued</code> and <code>billing.invoice.InvoicePaid</code>.</p>\n<h2 id=\"the-other-pages\">The other pages</h2>\n<table>\n<thead>\n<tr><th>page</th><th>what is on it</th></tr>\n</thead>\n<tbody>\n<tr><td><a href=\"domains/billing-email.html\">email</a></td><td>the <code>billing.email</code> vocabulary: its types, entities, views, commands, events, errors and actors</td></tr>\n<tr><td><a href=\"domains/billing-invoice.html\">Invoicing</a></td><td>the <code>billing.invoice</code> vocabulary: its types, entities, views, commands, events, errors and actors</td></tr>\n<tr><td><a href=\"interactions.html\">Interactions</a></td><td>every binding, with what it guarantees and what happens when it fails</td></tr>\n<tr><td><a href=\"crossings.html\">Type crossings</a></td><td>every conversion this system permits, and the reason someone gave for it</td></tr>\n<tr><td><a href=\"topology.html\">Topology</a></td><td>what each component needs in order to run</td></tr>\n</tbody>\n</table>\n<ul>\n</ul>\n</article>\n<footer class=\"provenance\">\nGenerated from billing v3 · model digest <code>56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942</code> · contract digest <code>cb634bd5e6f1afa6ebc8e9dca752e9901a9a68a2e51fc5009d099f155680606c</code>. Do not edit this file; change the specification and regenerate it with <code>ess generate</code>.\n</footer>\n</main>\n</div>\n<script type=\"module\">\nconst diagrams = document.querySelectorAll(\"pre.mermaid\");\nif (diagrams.length > 0) {\n  // A script element, not `import`: the bundle publishes itself with a top-level `var`, and a\n  // top-level `var` in a module is scoped to the module, so importing it throws.\n  await new Promise((resolve, reject) => {\n    const tag = document.createElement(\"script\");\n    tag.src = \"assets/mermaid.min.js\";\n    tag.addEventListener(\"load\", resolve);\n    tag.addEventListener(\"error\", reject);\n    document.head.append(tag);\n  });\n  globalThis.mermaid.initialize({\n    startOnLoad: false,\n    securityLevel: \"strict\",\n    theme: window.matchMedia(\"(prefers-color-scheme: dark)\").matches ? \"dark\" : \"default\",\n  });\n  await globalThis.mermaid.run({ querySelector: \"pre.mermaid\" });\n}\n</script>\n</body>\n</html>\n"
 right: "inside sentinel"

---- symlink_roots_parents_and_destinations_are_refused_before_writing stdout ----

thread 'symlink_roots_parents_and_destinations_are_refused_before_writing' (4180695) panicked at crates/edge/ess-cli/tests/output_containment.rs:43:9:
assertion `left == right` failed
  left: "<!DOCTYPE html>\n<!--\n  generated from billing v3\n  model digest 56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942\n  contract digest cb634bd5e6f1afa6ebc8e9dca752e9901a9a68a2e51fc5009d099f155680606c\n  do not edit: regenerate with `ess generate`\n-->\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>billing v3 · billing</title>\n<link rel=\"stylesheet\" href=\"assets/style.css\">\n</head>\n<body>\n<div class=\"shell\">\n<nav class=\"sidebar\">\n<a class=\"masthead\" href=\"index.html\">\n<span class=\"system\">billing</span><span class=\"version\">v3</span>\n</a>\n<div class=\"nav\">\n<a href=\"index.html\" aria-current=\"page\">billing v3</a>\n<a href=\"crossings.html\">Type crossings</a>\n<div class=\"group\">domains</div>\n<a class=\"nested\" href=\"domains/billing-email.html\">email</a>\n<a class=\"nested\" href=\"domains/billing-invoice.html\">Invoicing</a>\n<a href=\"interactions.html\">Interactions</a>\n<div class=\"group\">plan</div>\n<a class=\"nested\" href=\"plan/board.html\">Authored page</a>\n<a href=\"topology.html\">Topology</a>\n</div>\n</nav>\n<main>\n<article class=\"content\">\n<h1>billing v3</h1>\n<p>The single normative example. Every snippet in the ESS design document is meant to be derivable from this directory, and a test in <code>ess-domain</code> parses it — so a change here that the model cannot express fails the build rather than quietly making the document wrong.</p>\n<p>The system it describes is deliberately the smallest one that exercises everything wave 1 must model:</p>\n<pre><code class=\"language-text\">billing.invoice                      billing.email\n  CreateInvoice ──▶ InvoiceCreated     SendEmail ──▶ EmailSent\n        │                                  │\n        ▼                                  ▼\n  Invoice (Draft → Issued → Paid | Cancelled)\n        │\n        ▼\n  InvoiceById (eventual)</code></pre>\n<p>Two bounded contexts, a command with <strong>two outcomes</strong>, a command with an outcome the input cannot decide, events, both consistency levels, a filtered view, actors, and a state machine whose illegal transitions are illegal by absence.</p>\n<p>A test — <code>the_example_exercises_every_construct_the_model_has</code> — asserts that every type body, every primitive, <code>Optional</code>/<code>List</code>/<code>Map</code>, both consistency levels, an actor with grants and one without, an error carrying a payload and a command with an overridden wire name all appear here. <strong>A construct added to the model without reaching this directory fails the build</strong>, because what the normative example leaves out is what nothing checks.</p>\n<p><strong>A component is not a deployment.</strong> <code>invoice-service</code> owning <code>billing.invoice</code> says the invoice context is one unit of ownership; whether it ships as its own process or as a module inside one binary is <code>topology.yaml</code>&#39;s business, and changing that answer changes nothing in <code>domains/</code>. That separation is the point of specifying a system semantically, and it is why the three layers are three files.</p>\n<h2 id=\"what-each-file-is-for\">What each file is for</h2>\n<table>\n<thead>\n<tr><th>File</th><th>Why it exists</th></tr>\n</thead>\n<tbody>\n<tr><td><code>system.yaml</code></td><td>the format version, the system&#39;s identity, and which domains it has</td></tr>\n<tr><td><code>domains/invoice.yaml</code></td><td>the invoice bounded context: every type kind, an entity with a lifecycle, actors, a refusable command, an event, and both kinds of view</td></tr>\n<tr><td><code>domains/email.yaml</code></td><td>the second context, so cross-domain references are exercised rather than assumed, and the command whose failure the input cannot decide</td></tr>\n<tr><td><code>components.yaml</code></td><td>who owns which context, the binding between them, and the one type crossing that binding needs</td></tr>\n<tr><td><code>topology.yaml</code></td><td>what the system needs in order to run — modelled, and deployed by nothing</td></tr>\n</tbody>\n</table>\n<h2 id=\"three-things-worth-reading-closely\">Three things worth reading closely</h2>\n<p><strong><code>CreateInvoice</code> has outcomes, not an <code>emits</code> list.</strong> A command with a precondition has at least two results, and a specification that records only the happy one generates tests that say nothing about the branch where the money does not move.</p>\n<p><strong><code>InvoiceById</code> declares its consistency.</strong> It is a projection, so it is <code>eventual</code>, so a generated scenario must assert it with <code>eventually</code> rather than immediately. Getting this wrong produces a suite that passes on a laptop and flakes in CI, and the usual fix — a sleep — makes the suite test the machine it runs on.</p>\n<p><strong><code>Paid</code> cannot become <code>Cancelled</code>, and no rule says so.</strong> There is simply no transition. A rule would be a second place for the truth to live, and the two would eventually disagree.</p>\n<p><strong><code>Account</code> owns its invoices, and says so.</strong> The <code>relations:</code> block on <code>billing.invoice.Account</code> declares that it <code>owns</code> <code>many</code> <code>billing.invoice.Invoice</code>, carried by the invoice&#39;s own <code>account_id</code> field — one declaration, on the owner, rather than a typed id on the child plus an invariant nobody wrote. <code>ess specify validate</code> refuses that relation if the target is not an entity, if <code>account_id</code> is missing or typed as anything but <code>billing.invoice.AccountId</code>, or if a second entity claims to own invoices too. Every projection carries it under <code>x-ess-relation</code>, on the property holding <code>account_id</code>: see <code>generated/schema/entities/billing.invoice.Invoice.schema.json</code>, the <code>x-ess-entities</code> table of <code>generated/openapi/invoice-service.yaml</code>, and the doc comment on <code>InvoiceData::account_id</code> in <code>generated/rust/billing/</code>.</p>\n<p>Invoicing and the notification that follows it: the smallest system that still exercises every construct the model has — two bounded contexts, a command that can be refused, a command with an outcome its input cannot decide, both consistency levels, a state machine, actors, and a type of every kind.</p>\n<h2 id=\"the-system-as-a-graph\">The system as a graph</h2>\n<pre class=\"mermaid\">flowchart TB\n    subgraph who[&quot;who may ask&quot;]\n        who0[&quot;billing.invoice.Auditor&quot;]\n        who1[&quot;billing.invoice.Customer&quot;]\n    end\n    subgraph unit0[&quot;email-service&quot;]\n        cmd0[&quot;billing.email.SendEmail&quot;]\n        evt0[&quot;billing.email.DeliveryEscalated&quot;]\n        evt1[&quot;billing.email.EmailSent&quot;]\n    end\n    subgraph unit1[&quot;invoice-service&quot;]\n        cmd1[&quot;billing.invoice.CancelInvoice&quot;]\n        cmd2[&quot;billing.invoice.CreateInvoice&quot;]\n        cmd3[&quot;billing.invoice.IssueInvoice&quot;]\n        cmd4[&quot;billing.invoice.PayInvoice&quot;]\n        evt2[&quot;billing.invoice.InvoiceCancelled&quot;]\n        evt3[&quot;billing.invoice.InvoiceCreated&quot;]\n        evt4[&quot;billing.invoice.InvoiceIssued&quot;]\n        evt5[&quot;billing.invoice.InvoicePaid&quot;]\n    end\n    who1 --&gt;|&quot;may invoke&quot;| cmd2\n    cmd0 --&gt;|&quot;sent&quot;| evt1\n    cmd1 --&gt;|&quot;cancelled&quot;| evt2\n    cmd2 --&gt;|&quot;accepted&quot;| evt3\n    cmd3 --&gt;|&quot;issued&quot;| evt4\n    cmd4 --&gt;|&quot;settled&quot;| evt5\n    evt3 -.-&gt;|&quot;notify-on-invoice-created&quot;| cmd0</pre>\n<p>A command is accepted by the component that owns its context, emits the events one of its outcomes declares, and a dashed edge is a binding carrying an event into the next command. Design §9 begins one step earlier, at the actor who invokes the first command, and so does this graph: a solid edge out of an actor is a grant, and an actor drawn with no edge at all may invoke nothing — which is something the model says, not an arrow somebody forgot.</p>\n<h2 id=\"bounded-contexts\">Bounded contexts</h2>\n<ul>\n<li><strong><a href=\"domains/billing-email.html\">email</a></strong> (<code>billing.email</code>) — Sending the notifications other contexts ask for. Three types, no entities, no views, one command, two events, one error and no actors.</li>\n<li><strong><a href=\"domains/billing-invoice.html\">Invoicing</a></strong> (<code>billing.invoice</code>) — Issuing invoices and tracking whether they are paid. Eight types, two entities, two views, four commands, four events, two errors and two actors.</li>\n</ul>\n<h2 id=\"components\">Components</h2>\n<p>A component is a unit of ownership, not a deployment. How many of each runs, and what each needs, is <a href=\"topology.html\">the topology</a>.</p>\n<p><strong><code>email-service</code></strong> — Sends what other contexts ask it to send. It owns <a href=\"domains/billing-email.html\"><code>billing.email</code></a>. It accepts <code>billing.email.SendEmail</code>. It publishes <code>billing.email.DeliveryEscalated</code> and <code>billing.email.EmailSent</code>.</p>\n<p><strong><code>invoice-service</code></strong> — Issues invoices and tracks payment. It owns <a href=\"domains/billing-invoice.html\"><code>billing.invoice</code></a>. It accepts <code>billing.invoice.CancelInvoice</code>, <code>billing.invoice.CreateInvoice</code>, <code>billing.invoice.IssueInvoice</code> and <code>billing.invoice.PayInvoice</code>. It publishes <code>billing.invoice.InvoiceCancelled</code>, <code>billing.invoice.InvoiceCreated</code>, <code>billing.invoice.InvoiceIssued</code> and <code>billing.invoice.InvoicePaid</code>.</p>\n<h2 id=\"the-other-pages\">The other pages</h2>\n<table>\n<thead>\n<tr><th>page</th><th>what is on it</th></tr>\n</thead>\n<tbody>\n<tr><td><a href=\"domains/billing-email.html\">email</a></td><td>the <code>billing.email</code> vocabulary: its types, entities, views, commands, events, errors and actors</td></tr>\n<tr><td><a href=\"domains/billing-invoice.html\">Invoicing</a></td><td>the <code>billing.invoice</code> vocabulary: its types, entities, views, commands, events, errors and actors</td></tr>\n<tr><td><a href=\"interactions.html\">Interactions</a></td><td>every binding, with what it guarantees and what happens when it fails</td></tr>\n<tr><td><a href=\"crossings.html\">Type crossings</a></td><td>every conversion this system permits, and the reason someone gave for it</td></tr>\n<tr><td><a href=\"topology.html\">Topology</a></td><td>what each component needs in order to run</td></tr>\n</tbody>\n</table>\n<ul>\n</ul>\n</article>\n<footer class=\"provenance\">\nGenerated from billing v3 · model digest <code>56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942</code> · contract digest <code>cb634bd5e6f1afa6ebc8e9dca752e9901a9a68a2e51fc5009d099f155680606c</code>. Do not edit this file; change the specification and regenerate it with <code>ess generate</code>.\n</footer>\n</main>\n</div>\n<script type=\"module\">\nconst diagrams = document.querySelectorAll(\"pre.mermaid\");\nif (diagrams.length > 0) {\n  // A script element, not `import`: the bundle publishes itself with a top-level `var`, and a\n  // top-level `var` in a module is scoped to the module, so importing it throws.\n  await new Promise((resolve, reject) => {\n    const tag = document.createElement(\"script\");\n    tag.src = \"assets/mermaid.min.js\";\n    tag.addEventListener(\"load\", resolve);\n    tag.addEventListener(\"error\", reject);\n    document.head.append(tag);\n  });\n  globalThis.mermaid.initialize({\n    startOnLoad: false,\n    securityLevel: \"strict\",\n    theme: window.matchMedia(\"(prefers-color-scheme: dark)\").matches ? \"dark\" : \"default\",\n  });\n  await globalThis.mermaid.run({ querySelector: \"pre.mermaid\" });\n}\n</script>\n</body>\n</html>\n"
 right: "inside sentinel"


failures:
    a_hardlinked_destination_is_refused_before_other_files_change
    an_escaping_include_is_refused_before_any_output_changes
    include_aliases_and_duplicate_generated_pages_are_refused_before_writing
    noncanonical_and_platform_paths_are_refused_before_writing
    symlink_roots_parents_and_destinations_are_refused_before_writing

test result: FAILED. 1 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

error: test failed, to rerun pass `-p ess-cli --test output_containment`
exit: 101
```

The coordinator's requested-root compatibility correction was likewise preceded by this new failing regression on the first implementation:

```text
$ cargo test --locked -p ess-cli --bin ess caller_selected_parent_roots_resolve_without_creating_discarded_directories
   Compiling ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 1.37s
     Running unittests src/main.rs (target/debug/deps/ess-2a340c79ed142cb3)

running 1 test
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... FAILED

failures:

---- tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories stdout ----

thread 'tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories' (70815) panicked at crates/edge/ess-cli/src/main.rs:3279:84:
called `Result::unwrap()` on an `Err` value: output root cannot contain `..`: /home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/target/review-boundaries-2/ess-parent-root-70814-1788606724131523786/existing/../generated
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p ess-cli --bin ess`
exit: 101
```

## 4. Green runs and runner counts

The baseline command was cargo test --locked -p ess-cli -p ess-gen, exit 0. Its runner summary lines are preserved below; full original output remains in baseline.log.

```text
     Running unittests src/main.rs (target/debug/deps/ess-2a340c79ed142cb3)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/command_surface.rs (target/debug/deps/command_surface-f896f6f697ed70aa)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
     Running tests/command_surface_adversary.rs (target/debug/deps/command_surface_adversary-14ea054bad6c3502)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/go_conformance.rs (target/debug/deps/go_conformance-ba60d23811c1c6c2)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.86s
     Running unittests src/lib.rs (target/debug/deps/ess_gen-c9d966b95ec79202)
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/agreement.rs (target/debug/deps/agreement-58dd9a6c86f12a96)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
     Running tests/asyncapi.rs (target/debug/deps/asyncapi-34bc104727d0f83b)
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
     Running tests/corpus.rs (target/debug/deps/corpus-89a4fd9bef615f3d)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/determinism.rs (target/debug/deps/determinism-d4fe73d99eace0bf)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/docs.rs (target/debug/deps/docs-2ad82a61410dd2f8)
test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
     Running tests/openapi.rs (target/debug/deps/openapi-451fc006cd0e00d7)
test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
     Running tests/provenance.rs (target/debug/deps/provenance-8971c2bb396b7183)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/relations.rs (target/debug/deps/relations-acc8784e2326b745)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
     Running tests/schema.rs (target/debug/deps/schema-ecf7530cc8743c86)
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
   Doc-tests ess_gen
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

| Runner lane | Executed before → after | Final exit |
| --- | --- | --- |
| ess-cli unit | 5 → 11 | 0 |
| ess-cli command_surface | 5 → 5 | 0 |
| ess-cli command_surface_adversary | 4 → 4 | 0 |
| ess-cli go_conformance | 7 → 7 | 0 |
| ess-cli output_containment (new lane) | first execution 6, of which 5 red → 6, all green | 0 |
| ess-gen unit | 51 → 55 | 0 |
| ess-gen agreement | 4 → 4 | 0 |
| ess-gen asyncapi | 18 → 18 | 0 |
| ess-gen corpus | 3 → 3 | 0 |
| ess-gen determinism | 2 → 2 | 0 |
| ess-gen docs | 30 → 30 | 0 |
| ess-gen openapi | 35 → 35 | 0 |
| ess-gen provenance | 9 → 9 | 0 |
| ess-gen relations | 4 → 4 | 0 |
| ess-gen schema | 27 → 27 | 0 |
| ess-gen doc-tests | 0 → 0 | 0 |

Totals from these runner summaries: ess-cli 21→33, ess-gen 183→187, combined 204→220. No test was removed, weakened, ignored or skipped. The unchanged lanes contain no added tests. The same combined package selection was also run after the initial implementation (204→218); final exact individual package commands below cover the later two root-resolution cases. Separate ess-gen package selection recompiles a feature-unified dependency variant but executes the same named lanes and case counts.

```text
$ cargo test --locked -p ess-cli --test output_containment
   Compiling ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/generate/ess-gen)
   Compiling ess-conformance v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/verify/ess-conformance)
   Compiling ess-synth v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/generate/ess-synth)
   Compiling ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/verify/ess-diff)
   Compiling ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 13.10s
     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)

running 6 tests
test an_escaping_include_is_refused_before_any_output_changes ... ok
test a_hardlinked_destination_is_refused_before_other_files_change ... ok
test a_valid_nested_include_keeps_the_existing_site_layout_and_bytes ... ok
test symlink_roots_parents_and_destinations_are_refused_before_writing ... ok
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... ok
test noncanonical_and_platform_paths_are_refused_before_writing ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s

exit: 0
```

```text
$ cargo test --locked -p ess-cli --bin ess
   Compiling ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 1.24s
     Running unittests src/main.rs (target/debug/deps/ess-2a340c79ed142cb3)

running 11 tests
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
test tests::every_artifact_destination_is_checked_before_the_first_write ... ok
test tests::the_first_level_is_exactly_the_four_areas ... ok
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... ok
test tests::the_generate_area_answers_to_the_flat_spelling_and_to_its_own ... ok
test tests::every_leaf_is_reachable_by_its_area_path_and_by_its_flat_spelling ... ok
test tests::no_manifest_or_lockfile_depends_on_aep ... ok
test tests::every_command_and_argument_name_is_unambiguous ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

exit: 0
```

```text
$ cargo test --locked -p ess-cli
   Compiling ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 1.83s
     Running unittests src/main.rs (target/debug/deps/ess-2a340c79ed142cb3)

running 11 tests
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... ok
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
test tests::every_artifact_destination_is_checked_before_the_first_write ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
test tests::the_first_level_is_exactly_the_four_areas ... ok
test tests::the_generate_area_answers_to_the_flat_spelling_and_to_its_own ... ok
test tests::no_manifest_or_lockfile_depends_on_aep ... ok
test tests::every_command_and_argument_name_is_unambiguous ... ok
test tests::every_leaf_is_reachable_by_its_area_path_and_by_its_flat_spelling ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/command_surface.rs (target/debug/deps/command_surface-f896f6f697ed70aa)

running 5 tests
test the_help_offers_exactly_the_four_areas ... ok
test the_generate_area_help_offers_the_verbs_options_and_the_areas_subcommands ... ok
test a_clap_refusal_differs_only_in_its_usage_line ... ok
test the_generate_area_refuses_its_arguments_beside_a_sibling_verb ... ok
test a_flat_spelling_prints_what_its_area_path_prints ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s

     Running tests/command_surface_adversary.rs (target/debug/deps/command_surface_adversary-14ea054bad6c3502)

running 4 tests
test the_generate_area_does_not_run_a_sibling_verb_against_a_path_it_was_not_given ... ok
test the_generate_area_honours_the_arguments_it_accepts_or_refuses_them ... ok
test a_flat_spelling_prints_what_its_area_path_prints_when_clap_refuses ... ok
test the_generate_usage_line_admits_the_arguments_the_command_takes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/go_conformance.rs (target/debug/deps/go_conformance-ba60d23811c1c6c2)

running 7 tests
test a_view_returned_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test the_emitted_package_holds_a_correct_go_implementation_to_the_whole_suite ... ok
test one_deliberate_defect_fails_the_scenarios_responsible_for_it_and_no_others ... ok
test the_emitted_runner_reads_a_positional_assertion_and_refuses_one_in_an_unordered_view ... ok
test a_view_that_drops_rows_fails_the_scenarios_that_say_how_many_it_holds ... ok
test the_emitted_runner_stops_a_scan_and_fails_a_target_that_builds_the_whole_listing ... ok
test the_emitted_runner_holds_a_window_and_fails_a_target_whose_clock_never_moves ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.74s

     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)

running 6 tests
test an_escaping_include_is_refused_before_any_output_changes ... ok
test a_hardlinked_destination_is_refused_before_other_files_change ... ok
test a_valid_nested_include_keeps_the_existing_site_layout_and_bytes ... ok
test symlink_roots_parents_and_destinations_are_refused_before_writing ... ok
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... ok
test noncanonical_and_platform_paths_are_refused_before_writing ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s

exit: 0
```

```text
$ cargo test --locked -p ess-gen
   Compiling syn v3.0.4
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.20
   Compiling ref-cast-impl v1.0.27
   Compiling ref-cast v1.0.27
   Compiling thiserror v2.0.20
   Compiling serde v1.0.229
   Compiling schemars v0.8.22
   Compiling serde_yaml v0.9.34+deprecated
   Compiling ahash v0.8.12
   Compiling fluent-uri v0.4.1
   Compiling email_address v0.2.9
   Compiling ess-primitives v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/specify/ess-primitives)
   Compiling referencing v0.52.1
   Compiling jsonschema-value v0.52.1
   Compiling ess-domain v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/specify/ess-domain)
   Compiling jsonschema v0.52.1
   Compiling ess-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/specify/ess-compiler)
   Compiling ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/generate/ess-gen)
    Finished `test` profile [unoptimized] target(s) in 20.96s
     Running unittests src/lib.rs (target/debug/deps/ess_gen-5cfeec7d828080d8)

running 55 tests
test artifact::tests::portable_artifacts_refuse_escape_and_platform_aliases ... ok
test artifact::tests::a_destination_set_rejects_duplicates_case_aliases_and_file_parents_in_any_order ... ok
test authored::tests::a_fence_keeps_its_language_and_loses_its_trailing_newline ... ok
test authored::tests::a_heading_becomes_a_section_with_an_anchor ... ok
test authored::tests::a_paragraph_keeps_its_inline_structure ... ok
test authored::tests::a_link_an_adopter_wrote_stays_theirs ... ok
test authored::tests::a_list_becomes_items_and_a_quote_becomes_a_quote ... ok
test authored::tests::a_leading_title_becomes_the_page_title_and_not_a_second_heading ... ok
test authored::tests::a_top_level_heading_is_demoted_because_the_page_title_is_the_first ... ok
test authored::tests::raw_html_is_dropped_rather_than_passed_through ... ok
test authored::tests::a_table_keeps_its_header_apart_from_its_rows ... ok
test docs::tests::a_gap_that_ships_says_which_crate_closes_it ... ok
test docs::tests::a_heading_and_its_anchor_agree ... ok
test docs::tests::a_lifecycle_renders_as_a_state_diagram_with_its_initial_and_terminal_states_marked ... ok
test docs::tests::a_lifecycle_that_connects_every_pair_says_it_forbids_nothing ... ok
test docs::tests::a_plural_of_entity_is_entities ... ok
test docs::tests::a_lifecycle_with_one_state_forbids_nothing_rather_than_forbidding_everything ... ok
test docs::tests::a_list_of_three_reads_as_a_person_would_write_it ... ok
test docs::tests::a_transition_from_two_states_draws_one_arrow_from_each ... ok
test docs::tests::a_state_no_transition_touches_is_still_drawn ... ok
test graph::tests::a_component_group_is_a_dot_cluster_and_graphviz_only_boxes_clusters ... ok
test docs::tests::the_page_names_every_transition_the_specification_does_not_permit ... ok
test graph::tests::a_dot_label_keeps_its_parts_on_separate_lines ... ok
test document::tests::a_link_names_what_it_points_at_and_never_a_path ... ok
test graph::tests::a_mermaid_label_cannot_close_the_quoted_string_it_sits_in ... ok
test html::tests::a_code_block_is_a_code_listing_and_carries_its_language ... ok
test document::tests::a_page_id_says_how_deep_it_is_so_a_renderer_can_reach_the_root ... ok
test html::tests::a_link_into_the_page_it_is_on_is_a_fragment_and_not_a_round_trip ... ok
test html::tests::a_construct_is_addressed_by_the_section_that_documents_it ... ok
test document::tests::a_document_round_trips_through_its_own_format ... ok
test html::tests::a_link_is_addressed_from_the_page_it_is_written_on ... ok
test html::tests::a_diagram_is_a_pre_the_renderer_draws_into_and_never_a_code_listing ... ok
test html::tests::a_table_is_a_table_with_a_head_and_a_body ... ok
test html::tests::a_page_reaches_its_stylesheet_and_its_renderer_from_wherever_it_sits ... ok
test html::tests::an_adopters_front_page_goes_above_the_index_and_nowhere_else ... ok
test html::tests::markup_in_text_never_reaches_the_browser_as_markup ... ok
test html::tests::the_sidebar_groups_the_nested_pages_and_marks_the_page_the_reader_is_on ... ok
test markdown::tests::a_link_into_the_page_it_is_on_is_a_fragment_and_not_a_round_trip ... ok
test markdown::tests::a_link_is_addressed_from_the_page_it_is_written_on ... ok
test markdown::tests::a_diagram_is_a_fenced_mermaid_block ... ok
test markdown::tests::a_quotation_marks_every_line_it_covers ... ok
test schema::types::tests::a_decimal_is_written_as_an_exact_string_because_a_json_number_is_read_as_a_float ... ok
test schema::types::tests::a_reference_is_a_pointer_into_the_defs_of_the_document_holding_it ... ok
test markdown::tests::a_table_is_written_with_the_separator_a_reader_expects ... ok
test markdown::tests::a_section_flattens_into_the_stream_and_its_children_follow_it ... ok
test schema::types::tests::a_string_keyed_map_publishes_no_property_name_rule_that_checks_nothing ... ok
test schema::types::tests::a_timestamp_and_a_duration_publish_a_format_and_no_pattern_they_could_be_wrong_about ... ok
test schema::types::tests::a_union_tagged_value_moves_its_payload_aside_rather_than_colliding_with_the_tag ... ok
test schema::types::tests::an_integer_key_is_constrained_to_the_text_an_integer_is_spelt_with ... ok
test schema::types::tests::an_optional_outside_a_field_gains_a_null_branch_because_a_list_element_cannot_be_absent ... ok
test schema::types::tests::a_union_branch_pins_its_tag_so_exactly_one_branch_can_match ... ok
test html::tests::the_default_style_is_the_stylesheet_that_is_published ... ok
test html::tests::every_emitted_file_says_what_it_was_generated_from ... ok
test html::tests::checked_rendering_validates_deserialized_page_identities_before_map_collection ... ok
test html::tests::checked_rendering_preserves_valid_parent_and_nested_page_bytes ... ok

test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/agreement.rs (target/debug/deps/agreement-a6d7a7ff380699da)

running 4 tests
test the_agreement_check_compares_the_constructs_the_defect_was_about_rather_than_nothing ... ok
test every_keyword_the_projections_publish_is_classified_as_an_assertion_or_an_annotation ... ok
test every_projection_publishes_the_same_schema_for_a_construct_more_than_one_of_them_describes ... ok
test no_projection_collapses_a_newtype_into_the_representation_it_wraps ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

     Running tests/asyncapi.rs (target/debug/deps/asyncapi-9e439fb4245702f8)

running 18 tests
test a_binding_no_component_handles_still_states_its_failure_policy ... ok
test a_payload_refuses_an_undeclared_field_and_spells_absence_by_leaving_it_out_of_required ... ok
test a_payload_field_carries_the_grammar_the_model_states_and_not_a_note_naming_it ... ok
test a_dropped_failure_is_stated_in_prose_and_not_only_in_an_extension ... ok
test a_union_pins_its_tag_so_exactly_one_branch_matches_rather_than_none_or_both ... ok
test a_collection_says_what_it_holds_and_an_absent_element_is_null_because_it_has_no_key_to_omit ... ok
test the_publisher_of_an_event_sees_who_reacts_to_it_and_under_what_failure_policy ... ok
test the_channel_and_its_message_say_nothing_about_the_binding ... ok
test an_events_channel_address_is_its_declared_wire_name_or_else_its_qualified_name ... ok
test every_event_in_the_billing_example_appears_in_some_document ... ok
test a_bindings_mapping_and_the_reason_for_its_type_crossing_reach_the_document ... ok
test a_bindings_delivery_and_failure_reach_the_receiving_operation ... ok
test every_document_carries_the_provenance_of_the_model_it_came_from ... ok
test a_document_is_a_valid_asyncapi_three_skeleton ... ok
test every_ref_resolves_inside_the_document_that_holds_it ... ok
test regenerating_from_the_same_model_produces_the_same_bytes ... ok
test a_document_shows_what_the_component_publishes_and_what_it_reacts_to ... ok
test every_component_gets_one_document_named_after_it ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

     Running tests/corpus.rs (target/debug/deps/corpus-93718f8b1fc63993)

running 3 tests
test the_gatepass_documentation_is_byte_for_byte_what_is_pinned ... ok
test the_oracle_fixture_documentation_is_byte_for_byte_what_is_pinned ... ok
test the_billing_documentation_is_byte_for_byte_what_is_pinned ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/determinism.rs (target/debug/deps/determinism-9942695ed2e87dec)

running 2 tests
test the_determinism_scan_sees_code_and_not_prose ... ok
test no_generator_reads_a_clock_or_an_unordered_map ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/docs.rs (target/debug/deps/docs-530499ed1f64a736)

running 30 tests
test a_type_nothing_references_is_flagged_rather_than_left_looking_used ... ok
test a_grant_that_crosses_two_contexts_links_to_the_other_contexts_page ... ok
test an_entitys_identity_reaches_the_page_by_name_and_not_only_by_type ... ok
test an_actors_grant_renders_as_an_edge_from_the_actor_to_that_command_in_the_index_graph ... ok
test a_type_reached_only_through_an_entitys_field_is_not_called_unreached ... ok
test a_binding_renders_as_a_flow_and_a_lifecycle_as_a_state_diagram ... ok
test a_bindings_delivery_and_failure_semantics_are_stated_in_words ... ok
test an_actor_that_may_invoke_nothing_is_still_on_the_page ... ok
test an_empty_gap_allowlist_puts_no_cannot_show_section_on_any_page ... ok
test an_entitys_absent_transition_is_named_as_a_move_the_specification_does_not_permit ... ok
test a_declared_conversion_carries_its_reason_everywhere_a_reader_might_start ... ok
test an_entitys_invariant_reaches_the_page_as_a_condition_on_every_instance ... ok
test a_views_eventual_consistency_reads_differently_from_an_immediate_one ... ok
test an_outcome_the_input_cannot_decide_says_so_rather_than_claiming_it_is_unreachable ... ok
test a_commands_refusal_branch_is_documented_and_not_only_its_name ... ok
test a_wrong_state_branch_is_documented_with_the_states_the_document_never_lists ... ok
test a_components_ownership_and_a_workloads_replica_floor_are_both_documented ... ok
test an_outcome_that_changes_an_entity_says_which_instance_and_where_the_identity_is_read ... ok
test a_views_filter_reaches_the_page_rather_than_being_silently_dropped ... ok
test an_entitys_lifecycle_transitions_reach_the_page_as_arrows ... ok
test an_events_payload_and_an_errors_payload_are_both_documented_field_by_field ... ok
test the_provenance_header_is_a_markdown_comment_a_renderer_can_close ... ok
test every_link_between_pages_lands_on_a_page_that_exists_at_the_heading_it_names ... ok
test the_command_that_takes_each_move_reaches_the_page_beside_the_move_itself ... ok
test every_member_of_a_resolved_domain_reaches_the_page_of_the_context_it_belongs_to ... ok
test every_page_says_which_specification_produced_it ... ok
test every_type_kind_reaches_a_page_including_the_tagged_union ... ok
test generating_the_documentation_twice_produces_byte_identical_output ... ok
test an_outcome_says_what_it_does_to_an_entity_and_a_refusal_says_it_changes_none ... ok
test every_name_the_ir_holds_appears_on_some_page ... ok

test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/openapi.rs (target/debug/deps/openapi-cbc5ba4392fca057)

running 35 tests
test a_component_that_accepts_nothing_still_gets_a_document ... ok
test a_command_no_component_accepts_appears_in_no_document ... ok
test a_command_with_no_input_is_exposed_without_a_body ... ok
test a_served_view_declares_its_rows_and_the_consistency_a_caller_gets ... ok
test a_map_with_a_non_string_key_says_the_key_is_still_a_string ... ok
test a_view_is_served_only_where_the_specification_says_something_outside_reads_it ... ok
test every_kind_of_type_the_model_has_projects_into_a_schema ... ok
test each_declared_outcome_is_its_own_response_and_no_status_is_invented ... ok
test a_command_with_no_wire_name_is_exposed_under_the_name_the_model_gives_it ... ok
test a_command_no_binding_invokes_carries_no_idempotency_header ... ok
test a_refusal_the_input_decides_carries_the_declared_error_payload ... ok
test a_command_a_binding_delivers_at_least_once_requires_an_idempotency_key ... ok
test an_external_outcome_is_an_upstream_failure_and_not_a_validation_refusal ... ok
test a_document_is_valid_yaml_with_a_version_an_info_block_and_paths ... ok
test a_newtype_stays_a_schema_of_its_own_rather_than_becoming_its_representation ... ok
test a_command_is_only_ever_a_post ... ok
test a_command_names_the_actors_permitted_to_invoke_it_and_no_authentication_mechanism ... ok
test a_command_is_exposed_at_its_wire_name_under_its_domains ... ok
test a_refusal_the_subjects_state_decides_is_a_conflict_and_not_a_bad_request ... ok
test a_decimal_is_a_string_because_a_json_number_is_a_float ... ok
test several_outcomes_on_one_status_stay_distinguishable ... ok
test two_commands_claiming_one_path_both_move_to_their_qualified_names ... ok
test every_document_carries_its_provenance_as_a_comment_and_as_data ... ok
test every_reference_resolves_inside_the_document_that_makes_it ... ok
test a_command_no_actor_names_carries_no_grant_rather_than_a_grant_to_everybody ... ok
test an_outcome_that_emits_says_so_without_claiming_to_return_the_events ... ok
test every_schema_the_document_declares_is_pointed_at_by_something ... ok
test the_operation_id_is_the_commands_qualified_name ... ok
test a_commands_input_becomes_a_closed_object_over_its_declared_fields ... ok
test every_component_gets_one_document_named_after_it ... ok
test every_document_this_generator_can_produce_is_a_valid_openapi_document ... ok
test every_schema_a_document_embeds_is_valid_in_the_dialect_openapi_31_declares ... ok
test regenerating_from_the_same_ir_produces_the_same_bytes ... ok
test the_entities_published_are_exactly_those_of_the_domains_the_component_owns ... ok
test the_document_a_server_hands_out_is_the_committed_one_in_the_other_dialect ... ok

test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running tests/provenance.rs (target/debug/deps/provenance-9ec9789f25e84015)

running 9 tests
test a_text_without_both_digests_reads_as_nothing ... ok
test a_damaged_digest_reads_as_nothing ... ok
test a_generator_that_stamps_nothing_cannot_ship_an_artifact - should panic ... ok
test the_reader_reads_back_every_form_the_writer_emits ... ok
test the_whole_model_contract_digest_is_not_the_source_digest ... ok
test a_whole_model_slice_is_stamped_as_one ... ok
test a_generator_that_pairs_a_stamp_with_the_wrong_slice_cannot_ship_an_artifact - should panic ... ok
test a_change_no_construct_can_be_named_for_moves_every_contract_digest ... ok
test a_change_outside_an_artifacts_slice_leaves_its_contract_digest_standing ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/relations.rs (target/debug/deps/relations-7cdc743b0b26371d)

running 4 tests
test the_committed_openapi_document_is_byte_for_byte_what_the_projection_writes ... ok
test the_openapi_document_states_the_relation_and_links_the_targets_schema ... ok
test the_entity_document_states_the_relation_on_the_property_that_carries_it ... ok
test the_committed_entity_documents_are_byte_for_byte_what_the_schema_projection_writes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

     Running tests/schema.rs (target/debug/deps/schema-e0945bde8d462715)

running 27 tests
test a_list_element_may_be_null_where_a_field_may_only_be_absent ... ok
test a_timestamp_and_a_duration_publish_a_format_and_no_pattern_they_could_be_wrong_about ... ok
test a_field_carries_its_own_words_beside_the_reference_to_its_type ... ok
test a_field_is_called_what_the_specification_says_it_is_called_on_the_wire ... ok
test a_map_key_that_is_not_the_text_its_key_type_is_spelt_with_is_refused ... ok
test a_bytes_field_refuses_a_string_that_is_not_base64 ... ok
test a_map_is_an_object_whose_keys_are_the_text_its_key_type_is_spelt_with ... ok
test an_optional_field_may_be_absent_and_a_required_field_may_not ... ok
test a_command_input_accepts_a_filled_instance_and_refuses_a_misspelt_field ... ok
test an_amount_is_written_as_an_exact_decimal_string_and_a_float_is_refused ... ok
test a_uuid_newtype_carries_the_format_of_what_it_wraps ... ok
test a_newtype_over_a_string_publishes_no_constraint_the_specification_never_stated ... ok
test an_error_that_carries_nothing_accepts_an_empty_object_and_nothing_else ... ok
test a_tagged_union_round_trips_because_every_branch_pins_its_tag ... ok
test an_event_payload_accepts_what_the_specification_says_it_carries ... ok
test a_newtype_keeps_its_name_instead_of_collapsing_into_its_representation ... ok
test every_command_input_event_payload_error_payload_and_named_type_gets_a_schema ... ok
test a_uuid_is_refused_unless_it_is_the_canonical_hyphenated_form ... ok
test a_decimal_amount_is_refused_when_it_is_not_written_the_way_the_pattern_says ... ok
test every_artifact_is_a_json_schema_document_declaring_the_dialect_it_is_written_in ... ok
test an_invariant_travels_with_the_type_and_says_it_is_not_a_constraint ... ok
test every_reference_resolves_inside_the_document_that_makes_it ... ok
test every_message_accepts_an_instance_of_itself_and_refuses_one_that_is_wrong ... ok
test every_schema_says_which_specification_it_came_from ... ok
test no_schema_uses_a_keyword_outside_the_set_this_projection_publishes ... ok
test every_published_document_is_a_valid_json_schema_in_the_dialect_it_declares ... ok
test generation_is_byte_identical_between_runs ... ok

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s

   Doc-tests ess_gen

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

exit: 0
```

```text
$ cargo fmt -p ess-cli --check
exit: 0
```

```text
$ cargo fmt -p ess-gen --check
exit: 0
```

```text
$ cargo clippy --locked -p ess-cli --all-targets -- -D warnings
    Checking ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/edge/ess-cli)
    Finished `dev` profile [unoptimized] target(s) in 1.00s
exit: 0
```

```text
$ cargo clippy --locked -p ess-gen --all-targets -- -D warnings
    Checking serde v1.0.229
    Checking thiserror v2.0.20
    Checking ref-cast v1.0.27
    Checking schemars v0.8.22
    Checking ahash v0.8.12
    Checking serde_yaml v0.9.34+deprecated
    Checking email_address v0.2.9
    Checking fluent-uri v0.4.1
    Checking ess-primitives v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/specify/ess-primitives)
    Checking ess-domain v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/specify/ess-domain)
    Checking jsonschema-value v0.52.1
    Checking referencing v0.52.1
    Checking jsonschema v0.52.1
    Checking ess-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/specify/ess-compiler)
    Checking ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/generate/ess-gen)
    Finished `dev` profile [unoptimized] target(s) in 6.32s
exit: 0
```

The first combined Clippy run found only a redundant method closure and a needless generic-argument borrow, both fixed without changing assertions. Its original output remains in clippy.log.

## 5. Deliberate boundaries and coordinator decisions

- Legacy Artifact/PageId/Document constructors, public fields and deserialization remain compatible. Legacy Site.render still produces an in-memory sequence without refusal; Site.try_render is the checked entry point used by the CLI. The existing artifact::run return type and exact-duplicate behavior remain unchanged; exact duplicates are already refused there, and other aliases survive its string-keyed map for complete sink validation.
- Valid artifact bytes, persisted document bytes and the special site's existing root-relative layout remain unchanged. Existing corpus/projection byte-equality tests and the valid nested site control remain green.
- Requested roots may contain ..; generated artifact paths may not. Existing components of the requested root are inspected before normalization can erase them, and only the resulting absolute root is used for writing. Missing discarded components are not created. Windows drive-relative roots are explicitly refused.
- Unix existing destination replacement requires exactly one hard link. Non-Unix replacement is conservatively refused because a supported hard-link-count check is not implemented there. The checks were executed on Linux; this report claims no other-platform execution.
- Concurrent filesystem replacement, hostile mounts and filesystem aliases beyond the documented portable rules are outside this preflight threat model. Later I/O failures have no rollback. Stale output retirement, atomic replacement and input discovery remain their separate planned stories.
- Explicit single-file output arguments and tool-owned cache/staging writes are not generated relative artifact trees and remain unchanged. The composition tree is preflighted before its companion JSON output arguments are written.
- No unrelated producer packages, planning artifacts, Git lifecycle, public documentation publication, release/tag or live external systems were changed or invoked.
- The coordinator still owns adversarial review, the full integration gate, any required site build, commits and cleanup.

## 6. Write locations and resource observation

No authored file, scratch file, log or patch was written outside /home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment. Cargo used this tree's target directory; TMPDIR was the assigned scratch directory. The mandated sccache/Cargo tooling may maintain its own shared caches; none was manually edited or removed.

All raw logs and this report are under /home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/target/review-boundaries-2. Temporary test fixtures were created there and removed by their test fixtures. No managed tree, target directory or shared cache was removed.

Final observed resource/check output:
```text
Filesystem        1B-blocks         Used    Available Use% Mounted on
/dev/nvme0n1p2 910126964736 732876529664 130943082496  85% /
1099380	target
```

