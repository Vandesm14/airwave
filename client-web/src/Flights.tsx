import {
  baseAPIPath,
  getFlights,
  postCreateFlight,
  useFlights,
} from './lib/api';
import { formatTime } from './lib/lib';
import { Flight } from './lib/types';
import './Flights.scss';
import { QueryClient } from '@tanstack/solid-query';

export function FlightItem({ flight }: { flight: Flight }) {
  return (
    <div class="flight">
      <span class="kind">{flight.kind}</span>
      <span class="timer">
        {formatTime(flight.spawn_at.secs * 1000 - new Date().getTime())}
      </span>
    </div>
  );
}

export function FlightForm() {
  const client = new QueryClient();

  async function handleSubmit(e: Event) {
    e.preventDefault();

    const form = e.target as HTMLFormElement;
    const formData = new FormData(form);
    const data = new URLSearchParams({
      kind: formData.get('kind') as string,
      spawn_at: formData.get('spawn_at') as string,
    });

    const res = await fetch(`${baseAPIPath}${postCreateFlight}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: data,
    });

    await client.invalidateQueries({
      queryKey: [getFlights],
      type: 'all',
      exact: true,
    });
    await client.refetchQueries({
      queryKey: [getFlights],
      type: 'all',
      exact: true,
    });
  }

  return (
    <form onSubmit={handleSubmit}>
      <label>
        Kind:
        <select name="kind">
          <option value="inbound">Inbound</option>
          <option value="outbound">Outbound</option>
        </select>
      </label>
      <label>
        Spawn in (secs):
        <input type="number" name="spawn_at" placeholder="seconds" />
      </label>
      <button type="submit">Add</button>
    </form>
  );
}

export default function Flights() {
  const query = useFlights();

  return (
    <div class="flights">
      <FlightForm />
      <h2>Flights</h2>
      <div class="list">
        {query.data.map((f) => (
          <FlightItem flight={f} />
        ))}
      </div>
    </div>
  );
}
