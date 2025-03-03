import { useAcceptFlight, useAircraft, useWorld } from './lib/api';
import './Flights.scss';
import { createMemo, createSignal, Show } from 'solid-js';
import { selectedAircraftAtom } from './lib/atoms';
import { useAtom } from 'solid-jotai';
import { Aircraft, World } from './lib/types';
import {
  calculateSquaredDistance,
  HARD_CODED_AIRPORT,
  hardcodedAirspace,
} from './lib/lib';

export function FlightItem({ flight }: { flight: Aircraft }) {
  const acceptFlight = useAcceptFlight(flight.id);

  function handleAccept() {
    acceptFlight.mutate();
  }

  const [selected, setSelected] = useAtom(selectedAircraftAtom);

  const isSelected = createMemo(() => selected() === flight.id);
  const isOutbound = createMemo(
    () => flight.flight_plan.departing === HARD_CODED_AIRPORT
  );

  function handleSelect() {
    setSelected(flight.id);
  }

  return (
    <div class="flight" onClick={handleSelect}>
      <span
        classList={{
          callsign: true,
          outbound: isOutbound(),
          selected: isSelected(),
        }}
      >
        {flight.id}
      </span>
      <button class={'accept'} onClick={handleAccept}>
        Accept
      </button>
    </div>
  );
}

function sortByDistance(
  aircraft: Array<Aircraft>,
  world: World | undefined
): Array<Aircraft> {
  let airspace = hardcodedAirspace(world);
  if (airspace) {
    aircraft.sort((a, b) => {
      let aDist = calculateSquaredDistance(a.pos, airspace.pos);
      let bDist = calculateSquaredDistance(b.pos, airspace.pos);
      return aDist - bDist;
    });
  }

  return aircraft;
}

export default function Flights() {
  const [show, setShow] = createSignal(false);
  const [selectedAircraft] = useAtom(selectedAircraftAtom);

  const aircrafts = useAircraft();
  const world = useWorld();
  // TODO: Don't think this should be a memo, but it doesn't update the
  // mutation when selectedAircraft changes.
  const acceptFlight = createMemo(() => useAcceptFlight(selectedAircraft()));

  const arrivals = createMemo(() =>
    sortByDistance(
      aircrafts.data.filter(
        (a) =>
          !a.accepted &&
          a.flight_plan.arriving === HARD_CODED_AIRPORT &&
          a.segment !== 'parked'
      ),
      world.data
    )
  );
  const departures = createMemo(() =>
    aircrafts.data.filter(
      (a) => !a.accepted && a.flight_plan.departing === HARD_CODED_AIRPORT
    )
  );

  function handleSubmit() {
    acceptFlight().mutate();
  }

  return (
    <div class="container border">
      <div class="flights">
        <Show when={!show()}>
          <button onClick={() => setShow(true)}>Flights</button>
        </Show>
        <Show when={show()}>
          <button onClick={() => setShow(false)}>Close</button>
          <hr />
          <button onClick={handleSubmit}>Accept Selected</button>
          <h2>Arrivals ({arrivals().length})</h2>
          <div class="list">
            {arrivals().map((f) => (
              <FlightItem flight={f} />
            ))}
          </div>
          <hr />
          <h2>Departures ({departures().length})</h2>
          <div class="list">
            {departures().map((f) => (
              <FlightItem flight={f} />
            ))}
          </div>
        </Show>
      </div>
    </div>
  );
}
