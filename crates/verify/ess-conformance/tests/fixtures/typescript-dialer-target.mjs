// The #179/#188 dialer: a nested `lead` struct copied leaf by leaf from the input, `rank` generated.
// Modes: `correct`, `drops-number-from-the-row`, `wrong-number-on-the-event`.
const mode = process.env.ESS_TARGET_MODE ?? 'correct';
let minted = 0;

class Dialer {
  rows = new Map();
  identity() {
    return { name: 'dialer-fixture', version: '1' };
  }
  beginScenario() {}
  endScenario() {}
  executeCommand({ command, input }) {
    minted += 1;
    const consistency = `seq:${minted}`;
    if (command === 'demo.dialer.Join') {
      const id = `agent-${minted}`;
      this.rows.set(id, { agent_id: id, lead: null });
      return {
        outcome: 'joined',
        consistency,
        directEvents: [{ event: 'demo.dialer.Joined', payload: { agent_id: id } }],
      };
    }
    if (command === 'demo.dialer.SetLead') {
      const id = input.agent_id;
      const row = typeof id === 'string' ? this.rows.get(id) : undefined;
      if (row === undefined) {
        return { consistency };
      }
      const lead = {
        id: input.lead_id,
        uid: input.lead_uid,
        number: input.lead_number,
        rank: 0,
        data: input.lead_data,
      };
      const stored = { ...lead };
      if (mode === 'drops-number-from-the-row') {
        delete stored.number;
      }
      const published = { ...lead };
      if (mode === 'wrong-number-on-the-event') {
        published.number = input.lead_data;
      }
      row.lead = stored;
      return {
        outcome: 'lead-set',
        consistency,
        directEvents: [{ event: 'demo.dialer.LeadSet', payload: { agent_id: id, lead: published } }],
      };
    }
    throw new Error(`unexpected command ${command}`);
  }
  queryView() {
    return { rows: [...this.rows.values()] };
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
  return new Dialer();
}
