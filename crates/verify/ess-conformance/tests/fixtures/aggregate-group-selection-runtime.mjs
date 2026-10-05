// The JavaScript port of the store in tests/aggregate_group_selection.rs: it computes every view
// of the group-selection fixtures itself. ESS_GROUPS_MODEL names the fixture and ESS_GROUPS_FAULT
// the one defect switched in (`none` for none).
import { JsonNumber, equal } from './dist/runtime.js';

const model = process.env.ESS_GROUPS_MODEL;
const fault = process.env.ESS_GROUPS_FAULT ?? 'none';

const view = (name, groupBy, params, fields, openOnly = false) => ({
  name,
  groupBy,
  params,
  fields,
  openOnly,
});

const MODELS = {
  work: [
    view('demo.work.ByTeam', ['team'], ['team'], [['count', 'count'], ['total', 'sum:cents'], ['top', 'max:cents']]),
    view('demo.work.OpenByTeam', ['team'], ['team'], [['count', 'count'], ['total', 'sum:cents']], true),
    view('demo.work.ByTeamState', ['team', 'state'], ['team'], [['count', 'count']]),
    view('demo.work.ByTeamChannel', ['team', 'channel'], ['team'], [['count', 'count'], ['total', 'sum:cents']]),
    view('demo.work.ByUrgency', ['urgent'], ['urgent'], [['count', 'count'], ['total', 'sum:cents']]),
    view('demo.work.ByBucket', ['bucket'], ['bucket'], [['count', 'count']]),
    view('demo.work.ByState', ['state'], [], [['count', 'count'], ['total', 'sum:cents'], ['low', 'min:cents']]),
    view('demo.work.TeamsInLane', ['team'], ['lane', 'team'], [['count', 'count']]),
    view('demo.work.ByTeamUrgency', ['team', 'urgent'], ['team', 'urgent'], [['count', 'count']]),
    view('demo.work.ByKind', ['kind'], [], [['count', 'count'], ['mean', 'avg:cents'], ['lanes', 'distinct:lane']]),
    view('demo.work.ByUrgent', ['urgent'], [], [['count', 'count']]),
    view('demo.work.TopOpen', [], [], [['top', 'max:cents'], ['low', 'min:cents']], true),
  ],
  depots: [
    view('demo.work.ByTeam', ['team'], ['team'], [['count', 'count'], ['total', 'sum:cents']]),
    view('demo.work.ItemsByState', ['state'], [], [['count', 'count']]),
  ],
  direct: [view('demo.work.ByTeam', ['team'], ['team'], [['count', 'count']])],
  copied: [view('demo.work.ByTeam', ['team'], ['team'], [['count', 'count'], ['total', 'sum:cents']])],
  states: [view('demo.work.ByState', ['state'], [], [['count', 'count']])],
  ledger: [view('demo.ledger.ByAccount', ['account_id'], ['account_id'], [['count', 'count'], ['total', 'sum:cents']])],
  goals: [
    view('demo.goals.EvidenceEvaluations', ['objective_id', 'goal'], ['objective_id'], [['count', 'count']]),
    view('demo.goals.EvaluationsByGoal', ['goal'], [], [['count', 'count'], ['total', 'sum:cents']]),
  ],
};
if (MODELS[model] === undefined) {
  throw new Error(`ESS_GROUPS_MODEL ${model} names no model`);
}

const absent = (value) => value === null || value === undefined;

/** A number as an exact integer, whatever carries it. */
const integer = (value) => BigInt(String(value));

/** A number as a binary64 carries it, where the fault says so. */
const kept = (value) => {
  if (fault !== 'lossy-numbers') {
    return value;
  }
  if (value instanceof JsonNumber || typeof value === 'number' || typeof value === 'bigint') {
    return Number(String(value));
  }
  return value;
};

// The set of commands the model declares.
const family = { depots: 'depots', copied: 'depots', ledger: 'ledger', goals: 'goals' }[model] ?? 'work';

class Store {
  rows = [];
  depots = [];
  minted = 0;

  identity() {
    return { name: 'group-selection-fixture', version: '1' };
  }
  beginScenario() {
    if (fault !== 'retains-rows') {
      this.rows = [];
      this.depots = [];
    }
  }
  endScenario() {}

  executeCommand({ command, input }) {
    this.minted += 1;
    const id = `00000000-0000-4000-8000-${String(this.minted).padStart(12, '0')}`;
    const consistency = `seq:${this.minted}`;
    const read = (field) => (absent(input[field]) ? null : kept(input[field]));
    if (command === 'demo.work.Open' && family === 'work') {
      const row = { id, bucket: kept(new JsonNumber('9007199254740993')), state: 'Open' };
      for (const field of ['team', 'lane', 'cents', 'urgent', 'channel', 'kind']) {
        row[field] = read(field);
      }
      this.rows.push(row);
      return { outcome: 'opened', consistency, directEvents: [{ event: 'demo.work.Opened', payload: { id } }] };
    }
    if (command === 'demo.work.Finish' && family === 'work') {
      const row = this.rows.find((held) => equal(held.id, input.id));
      if (row === undefined) {
        return { consistency };
      }
      if (row.state !== 'Open') {
        return { outcome: 'unavailable', consistency };
      }
      if (fault !== 'stale-state') {
        row.state = 'Done';
      }
      return {
        outcome: 'finished',
        consistency,
        directEvents: [{ event: 'demo.work.Finished', payload: { id: input.id } }],
      };
    }
    if (command === 'demo.work.OpenDepot' && family === 'depots') {
      this.depots.push([id, read('team')]);
      return {
        outcome: 'opened',
        consistency,
        directEvents: [{ event: 'demo.work.DepotOpened', payload: { depot_id: id } }],
      };
    }
    if (command === 'demo.work.OpenItem' && family === 'depots') {
      const depot = this.related(input.depot_id);
      if (depot === undefined) {
        return { consistency };
      }
      this.rows.push({ item_id: id, depot_id: input.depot_id, team: depot[1], cents: read('cents'), state: 'Open' });
      return {
        outcome: 'opened',
        consistency,
        directEvents: [{ event: 'demo.work.ItemOpened', payload: { item_id: id } }],
      };
    }
    if (command === 'demo.ledger.OpenAccount' && family === 'ledger') {
      this.depots.push([id, read('holder')]);
      return {
        outcome: 'opened',
        consistency,
        directEvents: [{ event: 'demo.ledger.AccountOpened', payload: { account_id: id } }],
      };
    }
    if (command === 'demo.ledger.PostEntry' && family === 'ledger') {
      if (!this.depots.some(([held]) => equal(held, input.account_id))) {
        return { consistency };
      }
      this.rows.push({ entry_id: id, account_id: input.account_id, cents: read('cents'), state: 'Posted' });
      return {
        outcome: 'posted',
        consistency,
        directEvents: [{ event: 'demo.ledger.EntryPosted', payload: { entry_id: id } }],
      };
    }
    if (command === 'demo.goals.OpenObjective' && family === 'goals') {
      this.depots.push([id, read('goal')]);
      return {
        outcome: 'opened',
        consistency,
        directEvents: [{ event: 'demo.goals.ObjectiveOpened', payload: { objective_id: id } }],
      };
    }
    if (command === 'demo.goals.IntegrateCandidate' && family === 'goals') {
      // A missing objective is the `no-objective` refusal, which no aggregate scenario sends.
      const objective = this.related(input.objective_id);
      if (objective === undefined) {
        return { consistency };
      }
      this.rows.push({
        evaluation_id: id,
        objective_id: input.objective_id,
        goal: objective[1],
        cents: read('cents'),
        state: 'Recorded',
      });
      return {
        outcome: 'integrated',
        consistency,
        directEvents: [{ event: 'demo.goals.Evaluated', payload: { evaluation_id: id } }],
      };
    }
    throw new Error(`unexpected command ${command}`);
  }

  /** The related row named — or, under a copying fault, the first or last one created. */
  related(named) {
    if (fault === 'copies-first-related') {
      return this.depots[0];
    }
    if (fault === 'copies-last-related') {
      return this.depots[this.depots.length - 1];
    }
    return this.depots.find(([held]) => equal(held, named));
  }

  selected(held, param, first) {
    switch (fault) {
      case 'ignores-param':
        return true;
      case 'selects-first-group':
        return first !== undefined && equal(first, held);
      case 'prefix-param':
        if (typeof held === 'string' && typeof param === 'string') {
          return held.startsWith(param);
        }
        return equal(held, param);
      default:
        return !absent(held) && equal(held, kept(param));
    }
  }

  admits(declared, row, params) {
    const first = this.rows.length > 0 ? this.rows[0] : undefined;
    for (const param of declared.params) {
      const held = absent(row[param]) ? null : row[param];
      const wanted = absent(params[param]) ? null : params[param];
      if (declared.groupBy.includes(param)) {
        const firstHeld = first === undefined ? undefined : absent(first[param]) ? null : first[param];
        if (!this.selected(held, wanted, firstHeld)) {
          return false;
        }
      } else if (!equal(held, kept(wanted))) {
        return false;
      }
    }
    return !declared.openOnly || row.state === 'Open';
  }

  aggregate(fn, all, members) {
    if (fn === 'count') {
      let counted = fault === 'counts-all-rows' ? all.length : members.length;
      if (fault === 'wrong-count') {
        counted += 1;
      }
      return BigInt(counted);
    }
    const [kind, field] = fn.split(':');
    if (kind === 'distinct') {
      const seen = [];
      for (const row of members) {
        if (!seen.some((held) => equal(held, row[field]))) {
          seen.push(row[field]);
        }
      }
      return BigInt(fault === 'distinct-counts-rows' ? members.length : seen.length);
    }
    const values = members.map((row) => integer(row[field]));
    if (kind === 'avg') {
      if (values.length === 0) {
        return null;
      }
      // Six places, half-even, as the specification rounds a mean; or truncated.
      const n = BigInt(values.length);
      const scaled = values.reduce((a, b) => a + b, 0n) * 1000000n;
      let q = scaled / n;
      const r = scaled % n;
      if (fault !== 'truncates-avg' && (2n * r > n || (2n * r === n && q % 2n === 1n))) {
        q += 1n;
      }
      const text = `${q / 1000000n}.${String(q % 1000000n).padStart(6, '0')}`;
      return new JsonNumber(text.replace(/0+$/, '').replace(/\.$/, ''));
    }
    if (kind === 'sum') {
      return values.reduce((a, b) => a + b, 0n) + (fault === 'wrong-sum' ? 1n : 0n);
    }
    if (values.length === 0) {
      return null;
    }
    const max = (kind === 'max') !== (fault === 'wrong-extreme');
    return values.reduce((a, b) => (max ? (b > a ? b : a) : b < a ? b : a));
  }

  queryView({ view: name, params }) {
    const declared = MODELS[model].find((held) => held.name === name);
    if (declared === undefined) {
      throw new Error(`unexpected view ${name}`);
    }
    let keys = declared.groupBy;
    if (fault === 'merges-groups') {
      keys = keys.length > 1 ? keys.slice(0, 1) : [];
    }
    const groups = [];
    for (const row of this.rows) {
      const key = keys.map((field) => (absent(row[field]) ? null : row[field]));
      if (fault === 'drops-absent-group' && key.some(absent)) {
        continue;
      }
      let group = groups.find((held) => held.key.every((value, index) => equal(value, key[index])));
      if (group === undefined) {
        group = { key, all: [], members: [] };
        groups.push(group);
      }
      group.all.push(row);
      if (this.admits(declared, row, params ?? {})) {
        group.members.push(row);
      }
    }
    const rows = [];
    for (const group of groups) {
      if (group.members.length === 0) {
        continue;
      }
      const out = {};
      for (const key of declared.groupBy) {
        out[key] = absent(group.members[0][key]) ? null : group.members[0][key];
      }
      for (const [field, fn] of declared.fields) {
        out[field] = this.aggregate(fn, group.all, group.members);
      }
      rows.push(out);
    }
    // An ungrouped view is one row, over no admitted row too.
    if (declared.groupBy.length === 0 && rows.length === 0) {
      const out = {};
      for (const [field, fn] of declared.fields) {
        out[field] = this.aggregate(fn, [], []);
      }
      rows.push(out);
    }
    if (fault === 'spurious-group' && declared.groupBy.length > 0 && rows.length > 0) {
      const extra = { ...rows[0] };
      for (const key of declared.groupBy) {
        extra[key] = 'spurious';
      }
      rows.push(extra);
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

// One store behind every scenario's target, as one deployed implementation is.
const store = new Store();

export function makeTarget() {
  return store;
}
