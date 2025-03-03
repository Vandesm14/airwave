import {
  baseAPIPath,
  getAircraft,
  postAcceptFlight,
  useAircraft,
} from './lib/api';
import './Flights.scss';
import { useQueryClient } from '@tanstack/solid-query';
import { createMemo, createSignal, Show } from 'solid-js';
import { selectedAircraftAtom } from './lib/atoms';
import { useAtom } from 'solid-jotai';
import { Aircraft } from './lib/types';
import { HARD_CODED_AIRPORT } from './lib/lib';

async function acceptFlight(id: string) {
  await fetch(`${baseAPIPath}${postAcceptFlight(id)}`, {
    method: 'POST',
  });
}

export function FlightItem({ flight }: { flight: Aircraft }) {
  const client = useQueryClient();

  async function handleDelete() {
    // const res = await fetch(`${baseAPIPath}${deleteFlight(flight.id)}`, {
    //   method: 'DELETE',
    // });
    // if (res.ok) {
    //   await client.cancelQueries({ queryKey: [getFlights] });
    //   await client.setQueryData([getFlights], (old: Flight[]) => {
    //     console.log({ old });
    //     return old.filter((f) => f.id !== flight.id);
    //   });
    // }
  }

  async function handleAccept() {
    await acceptFlight(flight.id);
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

export default function Flights() {
  const [show, setShow] = createSignal(false);
  const client = useQueryClient();
  const [selectedAircraft] = useAtom(selectedAircraftAtom);

  // const aircrafts = createQuery<Aircraft[]>(() => ({
  //   queryKey: [getAircraft],
  //   initialData: [],
  // }));
  const aircrafts = useAircraft(() => 1000);

  const arrivals = createMemo(() =>
    aircrafts.data.filter(
      (a) =>
        !a.accepted &&
        a.flight_plan.arriving === HARD_CODED_AIRPORT &&
        a.segment !== 'parked'
    )
  );
  const departures = createMemo(() =>
    aircrafts.data.filter(
      (a) => !a.accepted && a.flight_plan.departing === HARD_CODED_AIRPORT
    )
  );

  async function handleSubmit(e: Event) {
    acceptFlight(selectedAircraft());

    // Invalidate and refetch aircraft
    await client.cancelQueries({ queryKey: [getAircraft] });
    await client.invalidateQueries({ queryKey: [getAircraft] });
    await client.refetchQueries({ queryKey: [getAircraft] });
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
          <h2>Arrivals</h2>
          <div class="list">
            {arrivals().map((f) => (
              <FlightItem flight={f} />
            ))}
          </div>
          <hr />
          <h2>Departures</h2>
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
