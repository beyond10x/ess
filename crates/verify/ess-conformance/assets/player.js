// The scenario player. Static machinery: nothing about any particular system is in this file.
//
// Two inputs, both emitted beside it:
//
//   model.json  the projection of the specification this page needs — entities and their
//               lifecycles, commands and what each outcome does, views, actors, bindings
//   suite.json  the compiled `ess-conformance/3` suite: the scenarios, as flat typed steps
//
// Every transition the player applies is read from `model.json` — an outcome names its transition
// and the states it runs from — so the lifecycle walk is the model's rather than this file's
// opinion of it.
//
// What this is not: an execution. A scenario declares which outcome each command takes and this
// replays that declaration. Obligations are unfilled and nothing here decides anything; the page
// says so where a reader will see it.

import { createApp, reactive, computed } from './assets/vue.esm-browser.prod.js'

const boom = (what, err) => {
  const el = document.getElementById('boom')
  if (!el) return
  el.style.display = 'block'
  el.textContent += `${what}: ${err && err.stack ? err.stack : err}\n`
}
window.addEventListener('error', (e) => boom('error', e.error ?? e.message))
window.addEventListener('unhandledrejection', (e) => boom('unhandled rejection', e.reason))

const [model, suite] = await Promise.all([
  fetch('model.json').then((r) => r.json()),
  fetch('suite.json').then((r) => r.json()),
])

const short = (n) => (n ? String(n).split('.').pop() : n)
const commandsByName = new Map(model.commands.map((c) => [c.name, c]))
const entitiesByName = new Map(model.entities.map((e) => [e.name, e]))

// Grants, emissions and reactions, indexed from the model rather than re-derived per render.
const grants = new Map()
for (const a of model.actors) for (const c of a.may) {
  if (!grants.has(c)) grants.set(c, [])
  grants.get(c).push(a.name)
}
const reactions = new Map()
for (const b of model.bindings) {
  if (!reactions.has(b.event)) reactions.set(b.event, [])
  reactions.get(b.event).push(b)
}
const emissionsOf = (command) => {
  const c = commandsByName.get(command)
  if (!c) return []
  const out = []
  for (const o of c.outcomes) for (const e of o.emits ?? []) if (!out.includes(e)) out.push(e)
  return out
}

function literal(node) {
  if (node === null || node === undefined) return null
  if (typeof node !== 'object') return node
  if (node.kind === 'literal') return node.value
  if (node.kind === 'instance') return { instance: node.instance }
  if (Array.isArray(node)) return node.map(literal)
  const out = {}
  for (const [k, v] of Object.entries(node)) out[k] = literal(v)
  return out
}

// ── scenarios ─────────────────────────────────────────────────────────────────
function groupSteps(steps) {
  const acts = []
  for (const step of steps) {
    if (step.step === 'execute_command') {
      acts.push({ command: step.command, actor: step.actor ?? null, input: step.input ?? {},
                  outcome: null, events: [], captures: [], consequences: [] })
      continue
    }
    const act = acts[acts.length - 1]
    if (!act) continue
    if (step.step === 'expect_outcome') act.outcome = step.outcome.outcome
    if (step.step === 'expect_event') act.events.push({ event: step.event, payload: literal(step.payload ?? {}) })
    if (step.step === 'capture_instance') act.captures.push(step)
  }
  // A consequence a binding causes, whether or not the scenario asserted the event that triggers it.
  for (const act of acts) {
    for (const event of emissionsOf(act.command)) {
      for (const b of reactions.get(event) ?? []) {
        act.consequences.push({
          event, command: b.command, binding: b.name,
          grantedTo: (grants.get(b.command) ?? []).map(short),
        })
      }
    }
  }
  return acts
}

const scenarios = Object.entries(suite.scenarios).map(([name, body]) => {
  const acts = groupSteps(body.steps)
  const lanes = []
  for (const a of acts) if (a.actor && !lanes.includes(a.actor)) lanes.push(a.actor)
  return {
    name, short: name.split('/').pop(), group: name.split('/').slice(0, -1).join('/'),
    purpose: body.purpose, acts, lanes,
    hasBindingLane: acts.some((a) => a.consequences.length > 0),
  }
})
const groups = [...new Set(scenarios.map((s) => s.group))]

// ── the world ─────────────────────────────────────────────────────────────────
const freshWorld = () => ({ instances: {}, events: [], notes: [] })

function applyAct(world, act, index) {
  const command = commandsByName.get(act.command)
  const outcome = command?.outcomes?.find((o) => o.name === act.outcome)
  const changes = []
  if (!outcome) {
    world.notes.push({ at: index, text: `no outcome \`${act.outcome}\` on \`${short(act.command)}\`` })
    return changes
  }

  let key = act.captures[0]?.instance ?? null
  if (!key) {
    for (const value of Object.values(act.input)) {
      const lit = literal(value)
      if (lit && typeof lit === 'object' && lit.instance) { key = lit.instance; break }
    }
  }

  const s = outcome.subject
  if (s && key) {
    const entity = entitiesByName.get(s.entity)
    const existing = world.instances[key]
    if (s.kind === 'moves') {
      const before = existing?.state ?? entity?.initial ?? '—'
      if (existing && s.from && !s.from.includes(before)) {
        world.notes.push({ at: index, text: `\`${s.transition}\` runs from ${s.from.join(', ')}; this instance is ${before}` })
      }
      if (existing) {
        existing.state = s.to
        existing.path.push(s.transition)
        changes.push({ instance: key, field: 'state', from: before, to: s.to })
      }
    } else {
      const entry = existing ?? { instance: key, entity: s.entity, state: entity?.initial ?? '—',
                                  fields: {}, path: [], declared: false }
      if (!existing) {
        world.instances[key] = entry
        changes.push({ instance: key, field: 'exists', from: '—', to: entry.state })
      }
      // Only what the outcome says it writes. A field no `sets:` names is not written here, and the
      // model saying nothing about it is the honest answer rather than copying the input over it.
      for (const target of outcome.sets ?? []) {
        const raw = act.input[target.from ?? target.target]
        if (raw === undefined) continue
        const lit = literal(raw)
        if (lit && typeof lit === 'object' && lit.instance) continue
        if (entry.fields[target.target] !== lit) {
          changes.push({ instance: key, field: target.target, from: entry.fields[target.target] ?? '—', to: lit })
          entry.fields[target.target] = lit
        }
      }
    }
  }

  for (const e of act.events) world.events.push({ at: index, ...e })

  for (const c of act.consequences) {
    const cmd = commandsByName.get(c.command)
    const out = cmd?.outcomes?.find((o) => o.subject?.kind === 'creates')
    const ent = out?.subject?.entity
    if (!ent) continue
    const id = `${short(ent).toLowerCase()}·declared`
    if (!world.instances[id]) {
      world.instances[id] = { instance: id, entity: ent, state: entitiesByName.get(ent)?.initial ?? '—',
                              fields: {}, path: [], declared: true }
      changes.push({ instance: id, field: 'exists', from: '—', to: world.instances[id].state, declared: true })
    }
  }
  return changes
}

// ── views ─────────────────────────────────────────────────────────────────────
// A filter arrives as text: `state == Active`, `call_id == param.call_id`, or those joined by `and`.
// The conjunction of `==` comparisons is evaluated and nothing else; a filter this cannot read is
// reported as unevaluated rather than guessed at.
function parseFilter(text) {
  if (!text) return { terms: [], ok: true }
  const parts = text.replace(/^\(|\)$/g, '').split(/\s+and\s+/)
  const terms = []
  for (const part of parts) {
    const m = part.match(/^\s*([A-Za-z_][\w.]*)\s*==\s*(.+?)\s*$/)
    if (!m) return { terms: [], ok: false }
    const right = m[2].trim()
    terms.push(right.startsWith('param.')
      ? { field: m[1], param: right.slice(6) }
      : { field: m[1], value: right })
  }
  return { terms, ok: true }
}

const viewModel = model.views.map((v) => ({
  ...v, short: short(v.name), parsed: parseFilter(v.filter),
}))

function evaluateViews(world) {
  return viewModel.map((v) => {
    const candidates = Object.values(world.instances).filter((i) => i.entity === v.entity)
    if (!v.parsed.ok) return { ...v, unevaluated: true, rows: [] }
    if ((v.params ?? []).length && !candidates.length) {
      return { ...v, unavailable: `nothing has bound ${v.params.join(', ')}`, rows: [] }
    }
    const rows = candidates.filter((i) => v.parsed.terms.every((t) => {
      if (t.param) return true
      if (t.field === 'state') return i.state === t.value
      return String(i.fields[t.field] ?? '') === t.value
    })).map((i) => ({
      instance: i.instance, declared: i.declared,
      cells: (v.fields ?? []).map((f) => [f, f === 'state' ? i.state : (i.fields[f] ?? '—')]),
    }))
    return { ...v, rows }
  })
}

const state = reactive({
  selected: scenarios[0]?.name ?? null,
  cursor: -1, playing: false, world: freshWorld(), lastChanges: [], speed: 1000, tab: 'state',
})

const api = { state, scenarios, entitiesByName, short, model }

const app = createApp({
  setup() {
    const scenario = computed(() => scenarios.find((s) => s.name === state.selected))
    const acts = computed(() => scenario.value?.acts ?? [])
    const lanes = computed(() => {
      const l = (scenario.value?.lanes ?? []).map((a) => ({ key: a, label: short(a), kind: 'actor' }))
      if (scenario.value?.hasBindingLane) l.push({ key: '@binding', label: 'binding', kind: 'binding' })
      return l
    })
    const done = computed(() => state.cursor >= acts.value.length - 1)

    function reset() { state.playing = false; state.cursor = -1; state.world = freshWorld(); state.lastChanges = [] }
    function step() {
      if (done.value) { state.playing = false; return }
      state.cursor += 1
      state.lastChanges = applyAct(state.world, acts.value[state.cursor], state.cursor)
    }
    function back() { const t = state.cursor - 1; reset(); for (let i = 0; i <= t; i += 1) step() }
    function play() {
      if (done.value) reset()
      state.playing = true
      const tick = () => { if (!state.playing) return; step(); if (!done.value) setTimeout(tick, state.speed); else state.playing = false }
      tick()
    }
    function select(name) { state.selected = name; reset() }

    const instances = computed(() => Object.values(state.world.instances))
    const changed = computed(() => new Set(state.lastChanges.map((c) => `${c.instance}.${c.field}`)))
    const liveViews = computed(() => evaluateViews(state.world).filter((v) => v.rows.length || v.unavailable || v.unevaluated))
    const rowState = (i) => (i < state.cursor ? 'past' : i === state.cursor ? 'now' : 'next')
    // Kept in JS: a `<` inside a text-node mustache makes the browser's HTML parser open a tag
    // before Vue compiles the in-DOM template, and the page dies with no message.
    const mark = (i) => (i < state.cursor ? '✓' : i === state.cursor ? '▶' : '')
    const lifecycle = (inst) => {
      const e = entitiesByName.get(inst.entity)
      return e ? { states: e.states ?? [], terminal: e.terminal ?? [] } : null
    }

    Object.assign(api, { scenario, acts, lanes, instances, liveViews, lifecycle })
    return {
      state, scenarios, groups, scenario, acts, lanes, done, instances, changed, liveViews,
      play, step, back, reset, select, rowState, mark, lifecycle, short,
      system: model.system, version: model.version,
      spec: (suite.provenance?.spec_digest ?? '').slice(0, 12),
      scenarioCount: scenarios.length,
    }
  },
})

// A specification may ship a skin: one module beside this file that renders its own outer
// representation from the same reactive state. Absent, the core panels are all there is.
try {
  const skin = await import('./skin.js')
  app.component('outer-surface', skin.default)
  api.hasSkin = true
} catch {
  app.component('outer-surface', {
    template: '<p class="empty">This specification ships no skin. The panels beside this one are the model itself.</p>',
  })
  api.hasSkin = false
}

app.provide('player', api)
app.config.errorHandler = (err, _i, info) => boom(`vue (${info})`, err)
try { app.mount('#app') } catch (err) { boom('mount', err) }
export default api
