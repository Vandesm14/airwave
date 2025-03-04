import {
  useAcceptFlight,
  useAircraft,
  useRejectFlight,
  useWorld,
} from './lib/api';
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
  const acceptFlight = useAcceptFlight();

  function handleAccept() {
    acceptFlight.mutate(flight.id);
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
  const acceptFlight = useAcceptFlight();
  const rejectFlight = useRejectFlight();

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

  function acceptSelected() {
    acceptFlight.mutate(selectedAircraft());
  }

  function rejectSelected() {
    rejectFlight.mutate(selectedAircraft());
  }

  function acceptArrivals() {
    arrivals().forEach((a) => acceptFlight.mutate(a.id));
  }

  function acceptDepartures() {
    departures().forEach((a) => acceptFlight.mutate(a.id));
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
          <button onClick={acceptSelected}>Accept Selected</button>
          <button onClick={rejectSelected}>Reject Selected</button>
          <h2>Arrivals ({arrivals().length})</h2>
          <button onClick={acceptArrivals}>Accept Arrivals</button>
          <div class="list">
            {arrivals().map((f) => (
              <FlightItem flight={f} />
            ))}
          </div>
          <hr />
          <h2>Departures ({departures().length})</h2>
          <button onClick={acceptDepartures}>Accept Departures</button>
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
