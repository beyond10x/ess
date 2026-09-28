// The aggregate-views model (`tests/fixtures/aggregate-optional-fields.yaml`, with the `Counted`
// view of `tests/ungrouped_aggregate_delta.rs`): orders, and views that count, sum, average and
// bound their durations, skipping absent ones. Modes: `correct`, `counts-absent-durations` (a
// skipping aggregate reads an absent duration as zero), `drops-absent-groups` (an absent group key
// is no group of its own), `counted-misses-one` (`Counted` counts one row fewer than there are).
const mode = process.env.ESS_TARGET_MODE ?? 'correct';
let minted = 0;

const present = (value) => value !== null && value !== undefined;

/** One aggregate function over the rows of a group, as the model's `aggregate:` declares it. */
function aggregate(rows, spec) {
  if (spec.count) {
    const n = rows.length;
    return spec.missesOne && n > 0 ? n - 1 : n;
  }
  let values = rows.map((row) => row[spec.of]);
  if (mode === 'counts-absent-durations') {
    values = values.map((value) => (present(value) ? value : 0));
  }
  values = values.filter(present);
  switch (spec.fn) {
    case 'sum':
      return values.length === 0 ? null : values.reduce((a, b) => a + b, 0);
    case 'avg':
      // A Decimal average, to six places, as `docs/design/aggregate-views.md` rounds it.
      return values.length === 0
        ? null
        : Number((values.reduce((a, b) => a + b, 0) / values.length).toFixed(6));
    case 'min':
      return values.length === 0 ? null : Math.min(...values);
    case 'max':
      return values.length === 0 ? null : Math.max(...values);
    case 'count_distinct':
      return new Set(values).size;
    default:
      throw new Error(`unknown aggregate ${spec.fn}`);
  }
}

const VIEWS = {
  'demo.orders.DurationTotal': { groupBy: [], fields: { total: { fn: 'sum', of: 'duration' } } },
  'demo.orders.PerGroup': { groupBy: ['group'], fields: { orders: { count: true } } },
  'demo.orders.DurationByCustomer': {
    groupBy: ['customer'],
    fields: {
      orders: { count: true },
      total: { fn: 'sum', of: 'duration' },
      mean: { fn: 'avg', of: 'duration' },
      shortest: { fn: 'min', of: 'duration' },
      longest: { fn: 'max', of: 'duration' },
      durations: { fn: 'count_distinct', of: 'duration' },
    },
  },
  'demo.orders.DurationForCustomer': {
    groupBy: [],
    filter: (row, params) => row.customer === params.customer,
    fields: {
      orders: { count: true },
      total: { fn: 'sum', of: 'duration' },
      mean: { fn: 'avg', of: 'duration' },
    },
  },
  'demo.orders.PerCustomerChannel': {
    groupBy: ['customer', 'channel'],
    fields: { orders: { count: true }, total: { fn: 'sum', of: 'duration' } },
  },
  'demo.orders.Counted': {
    groupBy: [],
    fields: {
      orders: { count: true, missesOne: mode === 'counted-misses-one' },
      longest: { fn: 'max', of: 'duration' },
    },
  },
};

class Orders {
  rows = [];
  identity() {
    return { name: 'aggregate-fixture', version: '1' };
  }
  beginScenario() {}
  endScenario() {}
  executeCommand({ command, input }) {
    if (command !== 'demo.orders.Place') {
      throw new Error(`unexpected command ${command}`);
    }
    minted += 1;
    const id = `00000000-0000-4000-8000-${String(minted).padStart(12, '0')}`;
    this.rows.push({
      order_id: id,
      customer: input.customer,
      duration: present(input.duration) ? input.duration : null,
      group: present(input.group) ? input.group : null,
      channel: present(input.channel) ? input.channel : null,
      state: 'Placed',
    });
    return {
      outcome: 'placed',
      consistency: `seq:${minted}`,
      directEvents: [{ event: 'demo.orders.Placed', payload: { order_id: id } }],
    };
  }
  queryView({ view, params }) {
    const declared = VIEWS[view];
    if (declared === undefined) {
      throw new Error(`unexpected view ${view}`);
    }
    const rows = this.rows.filter((row) => declared.filter?.(row, params) ?? true);
    const groups = new Map();
    for (const row of rows) {
      if (mode === 'drops-absent-groups' && declared.groupBy.some((key) => !present(row[key]))) {
        continue;
      }
      const key = JSON.stringify(declared.groupBy.map((field) => row[field] ?? null));
      if (!groups.has(key)) {
        groups.set(key, []);
      }
      groups.get(key).push(row);
    }
    if (declared.groupBy.length === 0 && groups.size === 0) {
      groups.set('[]', []);
    }
    const answer = [];
    for (const [key, members] of groups) {
      const out = {};
      JSON.parse(key).forEach((value, index) => {
        out[declared.groupBy[index]] = value;
      });
      for (const [field, spec] of Object.entries(declared.fields)) {
        out[field] = aggregate(members, spec);
      }
      answer.push(out);
    }
    return { rows: answer };
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

export function makeTarget() {
  return new Orders();
}
