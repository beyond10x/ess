// The JavaScript port of the store in tests/conditional_aggregate_measures.rs: it computes every
// view of fixtures/conditional-aggregate-measures.yaml itself. ESS_CASES_FAULT names the one defect
// switched in (`none` for none).
import { JsonNumber, equal } from './dist/runtime.js';

const fault = process.env.ESS_CASES_FAULT ?? 'none';

const measure = (field, fn, cond) => ({ field, fn, cond });

const VIEWS = [
  {
    name: 'demo.cases.Scorecard',
    fields: [
      measure('total', 'count', 'all'),
      measure('completed', 'count', 'completed'),
      measure('escalated_cost', 'sum:cents', 'escalated'),
    ],
  },
  {
    name: 'demo.cases.Escalations',
    fields: [
      measure('cases', 'count', 'all'),
      measure('escalated', 'count', 'escalated'),
      measure('labels', 'distinct:label', 'escalated'),
      measure('cost', 'sum:cents', 'escalated'),
      measure('cheapest', 'min:cents', 'escalated'),
      measure('dearest', 'max:cents', 'escalated'),
      measure('mean', 'avg:cents', 'high'),
    ],
  },
  {
    name: 'demo.cases.Urgent',
    fields: [
      measure('cases', 'count', 'all'),
      measure('urgent', 'count', 'urgent'),
      measure('urgent_cost', 'sum:cents', 'not-low'),
    ],
  },
  {
    name: 'demo.cases.Tagged',
    fields: [measure('cases', 'count', 'all'), measure('rushed', 'count', 'rush')],
  },
];

const integer = (value) => BigInt(String(value));

const holds = (cond, row) => {
  const completed = row.state === 'Completed';
  const escalated = row.escalated === true;
  const high = row.priority === 'High';
  switch (cond) {
    case 'all':
      return true;
    case 'completed':
      return completed;
    case 'escalated':
      return escalated;
    case 'urgent':
      return fault === 'first-branch' ? high : high || (escalated && completed);
    case 'high':
      return high;
    case 'not-low':
      return row.priority !== 'Low';
    case 'rush': {
      const tags = Array.isArray(row.tags) ? row.tags : [];
      return fault === 'forall-for-exists'
        ? tags.every((tag) => tag === 'rush')
        : tags.some((tag) => tag === 'rush');
    }
    default:
      throw new Error(`no condition ${cond}`);
  }
};

const NAMES = { count: 'count', distinct: 'count-distinct', sum: 'sum', min: 'min', max: 'max', avg: 'avg' };

/** The condition this store reads for one measure, and whether it inverts it. */
const applied = (view, m) => {
  const name = NAMES[m.fn.split(':')[0]];
  if (view.name === 'demo.cases.Escalations' && m.cond !== 'all') {
    if (fault === `drop-${name}`) {
      return ['all', false];
    }
    if (fault === `invert-${name}`) {
      return [m.cond, true];
    }
  }
  if (fault === 'swap' && view.name === 'demo.cases.Scorecard') {
    if (m.field === 'completed') {
      return ['escalated', false];
    }
    if (m.field === 'escalated_cost') {
      return ['completed', false];
    }
  }
  return [m.cond, false];
};

const aggregate = (fn, members) => {
  if (fn === 'count') {
    return BigInt(members.length);
  }
  const [kind, field] = fn.split(':');
  if (kind === 'distinct') {
    const seen = [];
    for (const row of members) {
      if (!seen.some((held) => equal(held, row[field]))) {
        seen.push(row[field]);
      }
    }
    return BigInt(seen.length);
  }
  const values = members.map((row) => integer(row[field]));
  if (kind === 'sum') {
    return values.reduce((a, b) => a + b, 0n);
  }
  if (values.length === 0) {
    return null;
  }
  if (kind === 'avg') {
    const n = BigInt(values.length);
    const scaled = values.reduce((a, b) => a + b, 0n) * 1000000n;
    let q = scaled / n;
    const r = scaled % n;
    if (2n * r > n || (2n * r === n && q % 2n === 1n)) {
      q += 1n;
    }
    const text = `${q / 1000000n}.${String(q % 1000000n).padStart(6, '0')}`;
    return new JsonNumber(text.replace(/0+$/, '').replace(/\.$/, ''));
  }
  return values.reduce((a, b) => (kind === 'max' ? (b > a ? b : a) : b < a ? b : a));
};

class Store {
  rows = [];
  minted = 0;

  identity() {
    return { name: 'conditional-measures-fixture', version: '1' };
  }
  beginScenario() {
    this.rows = [];
  }
  endScenario() {}

  executeCommand({ command, input }) {
    this.minted += 1;
    const id = `00000000-0000-4000-8000-${String(this.minted).padStart(12, '0')}`;
    const result = { consistency: `seq:${this.minted}` };
    if (command === 'demo.cases.Open') {
      const row = {};
      for (const field of ['team', 'cents', 'label', 'escalated', 'priority', 'tags']) {
        row[field] = input[field];
      }
      row.case_id = id;
      row.state = 'Open';
      this.rows.push(row);
      return { ...result, outcome: 'opened', directEvents: [{ event: 'demo.cases.Opened', payload: { case_id: id } }] };
    }
    if (command === 'demo.cases.Complete') {
      const found = this.rows.find((row) => equal(row.case_id, input.case_id));
      if (found === undefined) {
        return result;
      }
      if (found.state !== 'Open') {
        return { ...result, outcome: 'unavailable' };
      }
      if (fault !== 'stale-state') {
        found.state = 'Completed';
      }
      return {
        ...result,
        outcome: 'completed',
        directEvents: [{ event: 'demo.cases.Completed', payload: { case_id: input.case_id } }],
      };
    }
    throw new Error(`unexpected command ${command}`);
  }

  queryView({ view: name }) {
    const view = VIEWS.find((held) => held.name === name);
    if (view === undefined) {
      throw new Error(`unexpected view ${name}`);
    }
    const groups = [];
    for (const row of this.rows) {
      let group = groups.find((held) => equal(held.key, row.team));
      if (group === undefined) {
        group = { key: row.team, members: [] };
        groups.push(group);
      }
      group.members.push(row);
    }
    const first = view.fields.find((m) => m.cond !== 'all')?.cond;
    const rows = [];
    for (const group of groups) {
      let members = group.members;
      if (fault === 'whole-view' && first !== undefined) {
        members = members.filter((row) => holds(first, row));
        if (members.length === 0) {
          continue;
        }
      }
      const out = { team: group.key };
      let selectedAny = false;
      for (const m of view.fields) {
        const [cond, inverted] = applied(view, m);
        const selected = members.filter((row) => holds(cond, row) !== inverted);
        if (m.cond !== 'all' && selected.length > 0) {
          selectedAny = true;
        }
        out[m.field] = aggregate(m.fn, selected);
      }
      if (fault === 'drops-zero-selected' && !selectedAny) {
        continue;
      }
      rows.push(out);
    }
    return { rows };
  }

  observeEvents() {
    return [];
  }
  configureExternalOutcome() {
    throw new Error('nothing here is externally decided');
  }
  redeliverEvent() {
    throw new Error('no bindings');
  }
}

const store = new Store();

export function makeTarget() {
  return store;
}
