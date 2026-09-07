// Declared replay with explicit unavailable knowledge; no implementation is executed.
import { createApp, reactive, computed, watch } from './assets/vue.esm-browser.prod.js'

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
const UNKNOWN_LITERAL = 'Unknown: assignment literal is absent from this replay projection.'
const UNKNOWN_CONVERSION = 'Unknown: assignment types/conversion are absent from this replay projection.'
const UNKNOWN_SUBJECT = 'Unknown: subject identity source is absent from this replay projection.'
const UNKNOWN_FIELD = 'Unknown: no field value is established by this replay projection.'
const UNKNOWN_ID = 'Unknown: the generated identity was not observed.'

// Keep tags and Node values intact. In particular, a literal mapping is never a reference.
function valueText(value) {
  if (value.kind === 'literal') return `literal ${JSON.stringify(value.value)}`
  if (value.kind === 'instance') return `instance reference ${JSON.stringify(value.instance)} (scenario-local alias)`
  return `observed ${value.event}.${value.field} — Unknown: no implementation observation was made.`
}
const grants = new Map()
for (const actor of model.actors) for (const command of actor.may) {
  if (!grants.has(command)) grants.set(command, [])
  grants.get(command).push(short(actor.name))
}

// These are the scenario vocabulary's unsigned metadata owners, never Literal Node owners.
// Coverage admission retains their original decimal tokens; display them without Number conversion.
function declarationText(declaration) {
  const metadata = new Set(['elapsed', 'after', 'expectation.at_least', 'expectation.at_most', 'expectation.position.index'])
  const text = (value, path) => {
    if (metadata.has(path)) return value !== null && typeof value === 'object' ? value.raw : JSON.stringify(value)
    if (Array.isArray(value)) return `[${value.map((v, i) => text(v, `${path}.${i}`)).join(',')}]`
    if (value !== null && typeof value === 'object') return `{${Object.entries(value).map(([k, v]) => `${JSON.stringify(k)}:${text(v, path ? `${path}.${k}` : k)}`).join(',')}}`
    return JSON.stringify(value)
  }
  return text(declaration, '')
}

// Acts are presentation groups, never executions. Preserve every original step and its position,
// including a leading group before the first command. No assertion is folded into an observation.
function groupSteps(steps) {
  const acts = []
  for (const [index, declaration] of steps.entries()) {
    if (declaration.step === 'execute_command' || !acts.length) {
      const command = declaration.step === 'execute_command' ? declaration.command : null
      acts.push({ command, actor: declaration.actor ?? null, input: declaration.input ?? {},
        outcome: null, events: [], captures: [], consequences: [], steps: [], diagnostics: [] })
    }
    const act = acts[acts.length - 1]
    act.steps.push({ index, declaration, text: declarationText(declaration), status: 'Unexecuted declaration' })
  }
  for (const act of acts) {
    const outcomes = act.steps.filter((s) => s.declaration.step === 'expect_outcome').map((s) => s.declaration.outcome)
    if (outcomes.length === 1 && outcomes[0].command === act.command) act.outcome = outcomes[0].outcome
    else if (act.command) act.diagnostics.push('Unknown: a single matching outcome declaration is required.')
    act.captures = act.steps.filter((s) => s.declaration.step === 'capture_instance').map((s) => s.declaration)
    act.events = act.steps.filter((s) => s.declaration.step === 'expect_event').map((s) => s.declaration)
    act.inputs = Object.entries(act.input).map(([name, value]) => ({ name, value, text: valueText(value) }))
    const outcome = commandsByName.get(act.command)?.outcomes.find((o) => o.name === act.outcome)
    for (const event of outcome?.emits ?? []) for (const b of model.bindings.filter((b) => b.event === event)) {
      act.consequences.push({ event, command: b.command, binding: b.name, grantedTo: grants.get(b.command) ?? [] })
    }
  }
  return acts
}
const scenarios = Object.entries(suite.scenarios).map(([name, body]) => {
  const acts = groupSteps(body.steps)
  const lanes = [...new Set(acts.map((a) => a.actor))]
  return { name, short: name.split('/').pop(), group: name.split('/').slice(0, -1).join('/'),
    purpose: body.purpose, acts, lanes, hasBindingLane: acts.some((a) => a.consequences.length) }
})
const groups = [...new Set(scenarios.map((s) => s.group))]
const freshWorld = () => ({ instances: Object.create(null), events: [], notes: [], unknownEffects: [], queries: [] })

function applyAct(world, act, index) {
  const changes = []
  for (const step of act.steps) {
    const d = step.declaration
    if (['query_view', 'eventually_view', 'expect_halt', 'eventually_halt'].includes(d.step)) {
      world.queries.push({ at: index, index: step.index, view: d.view, kind: d.step,
        params: d.params ?? {}, parameters: Object.entries(d.params ?? {}).map(([name, value]) => ({ name, value, text: valueText(value) })) })
    }
  }
  for (const text of act.diagnostics) world.notes.push({ at: index, text })
  if (!act.command) return changes
  const outcome = commandsByName.get(act.command)?.outcomes.find((o) => o.name === act.outcome)
  if (!outcome) {
    world.notes.push({ at: index, text: 'Unknown: the selected command outcome is not established.' })
    return changes
  }
  // `refuses` is only meaningful for wrong-state branches and defaults true elsewhere.
  // The validated compiler gives refusing outcomes no subject or entity effects.
  const subject = outcome.subject
  if (!subject) return changes
  const entity = entitiesByName.get(subject.entity)
  if (!entity) {
    world.notes.push({ at: index, text: 'Unknown: the declared subject entity is unavailable.' })
    return changes
  }
  const effect = (reason, field, instance) => {
    const e = { at: index, entity: entity.name, kind: subject.kind, reason }
    if (field !== undefined) e.field = field
    if (instance !== undefined) e.instance = instance
    if (subject.kind === 'moves') e.transition = { name: subject.transition, from: subject.from, to: subject.to }
    world.unknownEffects.push(e)
  }
  let created
  if (subject.kind === 'creates') {
    const capture = act.captures[0]
    const matches = act.captures.length === 1 && capture.entity === entity.name
      && outcome.emits.includes(capture.event)
    if (matches && !Object.hasOwn(world.instances, capture.instance)) {
      created = { instance: capture.instance, entity: entity.name, declared: true,
        fields: Object.create(null), unknownFields: Object.create(null), path: [], identityUnknown: UNKNOWN_ID }
      if (entity.initial) created.state = entity.initial
      else created.stateUnknown = 'Unknown: no initial lifecycle state is projected.'
      for (const field of entity.fields) created.unknownFields[field] = UNKNOWN_FIELD
      created.unknownFields[entity.identity] = UNKNOWN_ID
      world.instances[capture.instance] = created
      changes.push({ instance: capture.instance, field: 'declaration' })
    } else {
      const reason = 'Unknown: creation requires one unambiguous matching capture and an unused scenario-local alias.'
      effect(reason)
      world.notes.push({ at: index, text: reason })
    }
  } else {
    effect(UNKNOWN_SUBJECT)
    for (const inst of Object.values(world.instances).filter((i) => i.entity === entity.name)) {
      if (subject.kind === 'moves') {
        delete inst.state
        inst.stateUnknown = UNKNOWN_SUBJECT
        changes.push({ instance: inst.instance, field: 'state' })
      }
    }
  }
  // replay/1 drops every literal and all assignment types/conversions, including on moves.
  // A named input is a declaration, never proof of the post-assignment value.
  for (const set of outcome.sets ?? []) {
    const reason = set.from === null ? UNKNOWN_LITERAL : UNKNOWN_CONVERSION
    effect(reason, set.target, created?.instance)
    const entry = world.unknownEffects[world.unknownEffects.length - 1]
    entry.from = set.from
    if (set.from !== null && !Object.hasOwn(act.input, set.from)) {
      entry.missingInput = `Unknown: assignment input ${JSON.stringify(set.from)} is missing.`
    }
    const affected = created ? [created] : subject.kind === 'creates' ? []
      : Object.values(world.instances).filter((i) => i.entity === entity.name)
    for (const inst of affected) {
      delete inst.fields[set.target]
      inst.unknownFields[set.target] = reason
      changes.push({ instance: inst.instance, field: set.target })
    }
  }
  return changes
}

const state = reactive({ selected: scenarios[0]?.name ?? null,
  cursor: -1, playing: false, world: freshWorld(), lastChanges: [], speed: 1000, tab: 'state' })
const api = { state, scenarios, entitiesByName, short, model }
const app = createApp({
  setup() {
    const scenario = computed(() => scenarios.find((s) => s.name === state.selected))
    const acts = computed(() => scenario.value?.acts ?? [])
    const lanes = computed(() => {
      const lanes = (scenario.value?.lanes ?? []).map((a) => ({ key: a, label: a ? short(a) : 'Declarations', kind: 'actor' }))
      if (scenario.value?.hasBindingLane) lanes.push({ key: '@binding', label: 'Binding declarations', kind: 'binding' })
      return lanes
    })
    const done = computed(() => state.cursor >= acts.value.length - 1)
    let timer = null
    let generation = 0
    function cancelTimer() { generation += 1; clearTimeout(timer); timer = null }
    // The existing skin API and Pause button can set playing directly; cancellation is synchronous.
    watch(() => state.playing, (playing) => { if (!playing) cancelTimer() }, { flush: 'sync' })
    function reset() { cancelTimer(); state.playing = false; state.cursor = -1; state.world = freshWorld(); state.lastChanges = [] }
    function step() {
      if (done.value) { state.playing = false; return }
      state.cursor += 1
      state.lastChanges = applyAct(state.world, acts.value[state.cursor], state.cursor)
    }
    function back() { const target = state.cursor - 1; reset(); for (let i = 0; i <= target; i += 1) step() }
    function play() {
      cancelTimer()
      if (done.value) reset()
      state.playing = true
      const current = generation
      const tick = () => {
        if (current !== generation || !state.playing) return
        timer = null
        step()
        if (!done.value) timer = setTimeout(tick, state.speed)
        else state.playing = false
      }
      tick()
    }
    function select(name) { reset(); state.selected = name }
    const instances = computed(() => Object.values(state.world.instances))
    const changed = computed(() => new Set(state.lastChanges.map((c) => `${c.instance}.${c.field}`)))
    const liveViews = computed(() => model.views.map((v) => ({ ...v, short: short(v.name), filterText: v.filter,
      reasons: [
        ...(v.params.length ? ['Unknown: this replay does not evaluate parameterized view results.'] : []),
        ...(v.filter ? ['Unknown: this replay does not evaluate this view filter.'] : []),
        'Unknown: this replay does not evaluate view results.',
        'Unknown: model ordering was not projected.',
      ], queries: state.world.queries.filter((q) => q.view === v.name) })))
    const rowState = (i) => (i < state.cursor ? 'past' : i === state.cursor ? 'now' : 'next')
    const mark = (i) => (i < state.cursor ? '·' : i === state.cursor ? '▶' : '')
    const lifecycle = (inst) => {
      const entity = entitiesByName.get(inst.entity)
      return entity ? { states: entity.states, terminal: entity.terminal } : null
    }
    Object.assign(api, { scenario, acts, lanes, instances, liveViews, lifecycle, play, step, back, reset, select })
    return { state, scenarios, groups, scenario, acts, lanes, done, instances, changed, liveViews,
      play, step, back, reset, select, rowState, mark, lifecycle, short,
      system: model.system, version: model.version, spec: (suite.provenance?.spec_digest ?? '').slice(0, 12), scenarioCount: scenarios.length }
  },
})
try {
  const skin = await import('./skin.js')
  app.component('outer-surface', skin.default)
  api.hasSkin = true
} catch {
  app.component('outer-surface', { template: '<p class="empty">This specification ships no skin. The panels show declarations and unavailable knowledge.</p>' })
  api.hasSkin = false
}
app.provide('player', api)
app.config.errorHandler = (err, _i, info) => boom(`vue (${info})`, err)
try { app.mount('#app') } catch (err) { boom('mount', err) }
export default api
