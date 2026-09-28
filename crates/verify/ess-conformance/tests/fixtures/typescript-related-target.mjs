// The #166 shipping model of `tests/related_values.rs`: a shipment's region is read from the
// customer it names. Modes: `correct`, `latest-customer`, `other-customer`.
const mode = process.env.ESS_TARGET_MODE ?? 'correct';
let minted = 0;

class Shipping {
  customers = [];
  shipments = new Map();
  identity() {
    return { name: 'shipping-fixture', version: '1' };
  }
  beginScenario() {}
  endScenario() {}
  /** The region this implementation reads for the customer `id`. */
  region(id) {
    const named = ([customer]) => customer === id;
    let found;
    if (mode === 'latest-customer') {
      found = this.customers[this.customers.length - 1];
    } else if (mode === 'other-customer') {
      found = this.customers.find((customer) => !named(customer)) ?? this.customers.find(named);
    } else {
      found = this.customers.find(named);
    }
    return found === undefined ? undefined : found[1];
  }
  executeCommand({ command, input }) {
    minted += 1;
    const consistency = `seq:${minted}`;
    switch (command) {
      case 'demo.shipping.Register': {
        const id = `customer-${minted}`;
        this.customers.push([id, input.region]);
        return {
          outcome: 'registered',
          consistency,
          directEvents: [{ event: 'demo.shipping.CustomerRegistered', payload: { customer_id: id } }],
        };
      }
      case 'demo.shipping.Pack': {
        const region = this.region(input.customer_id);
        if (region === undefined) {
          return { consistency };
        }
        const id = `shipment-${minted}`;
        this.shipments.set(id, { shipment_id: id, customer_id: input.customer_id, region: null });
        return {
          outcome: 'packed',
          consistency,
          directEvents: [
            { event: 'demo.shipping.ShipmentPacked', payload: { shipment_id: id, region } },
          ],
        };
      }
      case 'demo.shipping.Dispatch': {
        const row = typeof input.shipment_id === 'string' ? this.shipments.get(input.shipment_id) : undefined;
        if (row === undefined) {
          return { consistency };
        }
        const region = this.region(row.customer_id);
        if (region === undefined) {
          return { consistency };
        }
        row.region = region;
        return {
          outcome: 'dispatched',
          consistency,
          directEvents: [
            {
              event: 'demo.shipping.ShipmentDispatched',
              payload: { shipment_id: row.shipment_id, region },
            },
          ],
        };
      }
      default:
        throw new Error(`unexpected command ${command}`);
    }
  }
  queryView() {
    return {
      rows: [...this.shipments.entries()]
        .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))
        .map(([, { customer_id: _customer, ...row }]) => row),
    };
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
  return new Shipping();
}
