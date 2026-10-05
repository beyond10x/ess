---
format: aep.planning-md/3
id: story:openapi-returns-response
kind: story
status: active
title: 'OpenAPI carries the response of an outcome declaring returns: true'
refs:
- provider: github
  reference: beyond10x/ess#423
relations:
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T18:48:38Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-04T18:48:38Z", actor: "human:timo", revision: 3}
---
## Outcome

OpenAPI carries the response of an outcome declaring returns: true (beyond10x/ess#423).

## Origin

Opened on GitHub as beyond10x/ess#423; added to the bundle on 2026-10-04 under the operator goal that every open issue is solved and merged through the integration branch. Issue text (first part):

> ## What happens
> 
> A command outcome declaring `returns: true` (format `ess/17`) validates, but `ess generate --kind openapi`
> leaves the command's `response:` fields out of the generated document. The success body schema carries only
> `outcome` and `published`, so an HTTP client of a `reached_by: network` component cannot learn what the
> command answers.
> 
> ## Reproduction (ess 0.52.0, also present on `main` at 5323e4c7ce)
> 
> `system.yaml`
> 
> ```yaml
> format: ess/17
> system: catalogue
> version: v1
> 
> domains:
>   - catalogue.search
> ```
> 
> `components.yaml`
> 
> ```yaml
> components:
>   - component: catalogue-reader
>     summary: Answers searches over the catalogue.
>     owns:
>       domains:
>         - catalogue.search
>     accepts:
>       commands:
>         - catalogue.search.FindTitles
>     publishes:
>       events:
>         - catalogue.search.TitlesFound
>     reached_by: network
> ```
> 
> `domains/search.yaml`
> 
> ```yaml
> domain: catalogue.search
> 
> naming:
>   wire: search
>   display: Search
> 
> entities:
>   - name: catalogue.search.Title
>     identity:
>       name: title_id
>       type: Uuid
>     fields:
>       - name: name
>         type: String
>     lifecycle:
>       initial: Listed
>       states: [Listed]
>       terminal: [Listed]
> 
> events:
>   - name: catalogue.search.TitlesFound
>     fields:
>       - name: count
>         type: Integer
> 
> commands:
>   - name: catalogue.search.FindTitles
>     naming:
>       wire: find-titles
>       display: Find titles
>     input:
>       - name: text
>         type: String
>     response:
>       - name: names
>        
