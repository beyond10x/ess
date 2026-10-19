// The wiring model of `tests/fixtures/two-rows-one-branch.yaml` (ess/28, downstream report
// 2026-10-19, finding 11): a wire is refused where both ports face one way, a branch over both rows
// at once. Modes: `correct`, `source-only`, `sink-only`, `ignores`.
const mode = process.env.ESS_TARGET_MODE ?? 'correct';
let minted = 0;

const uuid = (n) => `00000000-0000-4000-8000-${String(n).padStart(12, '0')}`;

class Wiring {
  ports = new Map();
  wires = [];
  identity() {
    return { name: 'wiring-fixture', version: '1' };
  }
  beginScenario() {
    this.ports = new Map();
    this.wires = [];
  }
  endScenario() {}
  /** Whether this implementation refuses a pair facing `from` and `to` as facing one way. */
  same(from, to) {
    if (mode === 'correct') return from === to;
    return mode !== 'ignores';
  }
  executeCommand({ command, input }) {
    minted += 1;
    const consistency = `seq:${minted}`;
    switch (command) {
      case 'wiring.board.AddPort': {
        const id = uuid(minted);
        this.ports.set(id, input.direction);
        return {
          outcome: 'added',
          consistency,
          directEvents: [{ event: 'wiring.board.PortAdded', payload: { port_id: id } }],
        };
      }
      case 'wiring.board.Connect': {
        const from = this.ports.get(input.from_port);
        const to = this.ports.get(input.to_port);
        if (from === undefined) {
          return { outcome: 'no-source', error: 'wiring.board.SourceNotFound', consistency };
        }
        if (to === undefined) {
          return { outcome: 'no-sink', error: 'wiring.board.SinkNotFound', consistency };
        }
        const read = mode === 'sink-only' ? to : from;
        if (this.same(from, to) && read === 'inbound') {
          return { outcome: 'both-inbound', error: 'wiring.board.SameDirection', consistency };
        }
        if (this.same(from, to) && read === 'outbound') {
          return { outcome: 'both-outbound', error: 'wiring.board.SameDirection', consistency };
        }
        const id = uuid(minted);
        this.wires.push({ wire_id: id, from_port: input.from_port, to_port: input.to_port });
        return {
          outcome: 'connected',
          consistency,
          directEvents: [{ event: 'wiring.board.Connected', payload: { wire_id: id } }],
        };
      }
      default:
        throw new Error(`unexpected command ${command}`);
    }
  }
  queryView({ view }) {
    switch (view) {
      case 'wiring.board.Wires':
        return { rows: this.wires };
      case 'wiring.board.Ports':
        return {
          rows: [...this.ports.entries()].map(([port_id, direction]) => ({ port_id, direction })),
        };
      default:
        throw new Error(`unexpected view ${view}`);
    }
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
  return new Wiring();
}
