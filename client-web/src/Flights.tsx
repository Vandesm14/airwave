import { baseAPIPath, getAircraft, postAcceptFlight } from './lib/api';
import './Flights.scss';
import { useQueryClient } from '@tanstack/solid-query';
import { createSignal, Show } from 'solid-js';
import { selectedAircraftAtom } from './lib/atoms';
import { useAtom } from 'solid-jotai';

export default function Flights() {
  const [show, setShow] = createSignal(false);
  const client = useQueryClient();
  const [selectedAircraft] = useAtom(selectedAircraftAtom);

  async function acceptFlight() {
    await fetch(`${baseAPIPath}${postAcceptFlight(selectedAircraft())}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
    });
  }

  async function handleSubmit(e: Event) {
    acceptFlight();

    // Invalidate and refetch flights
    // await client.cancelQueries({ queryKey: [getFlights] });
    // await client.invalidateQueries({ queryKey: [getFlights] });
    // await client.refetchQueries({ queryKey: [getFlights] });

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
          <span>
            <b>Selected:</b>
            <span>{selectedAircraft() ?? ''}</span>
          </span>
          <button onClick={handleSubmit}>Use Selected</button>
        </Show>
      </div>
    </div>
  );
}
