import {
  baseAPIPath,
  deleteFlight,
  getFlights,
  postCreateFlight,
  useFlights,
} from './lib/api';
import { formatTime } from './lib/lib';
import { Flight } from './lib/types';
import './Flights.scss';
import { QueryClient } from '@tanstack/solid-query';

export function FlightItem({ flight }: { flight: Flight }) {
  async function handleDelete() {
    const client = new QueryClient();
    await fetch(`${baseAPIPath}${deleteFlight(flight.id)}`, {
      method: 'DELETE',
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
    <div class="flight">
      <span class="kind">{flight.kind.toLocaleUpperCase()}</span>
      <span class="timer">
        {formatTime(flight.spawn_at.secs * 1000 - new Date().getTime())}
      </span>
      <button onClick={handleDelete}>Del</button>
    </div>
  );
}

export function FlightForm() {
  const client = new QueryClient();

  async function createFlight(data: { kind: string; spawn_at: string }) {
    const body = new URLSearchParams(data);

    const res = await fetch(`${baseAPIPath}${postCreateFlight}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body,
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

  async function handleSubmit(e: Event) {
    e.preventDefault();

    const form = e.target as HTMLFormElement;
    const formData = new FormData(form);

    const quantity = parseInt(formData.get('quantity') as string);
    const stagger = parseInt(formData.get('stagger') as string);
    const spawnAt = parseInt(formData.get('spawn_at') as string);
    const kind = formData.get('kind') as string;

    for (let i = 0; i < quantity; i++) {
      await createFlight({
        spawn_at: (spawnAt + i * stagger).toString(),
        kind,
      });
    }
  }

  return (
    <form onSubmit={handleSubmit}>
      <h2>Order:</h2>
      <label>
        <span>Quantity:</span>
        <input type="number" name="quantity" min={1} value={1} />
      </label>
      <label>
        <span>Flight kind:</span>
        <select name="kind">
          <option value="inbound">Inbound</option>
          <option value="outbound">Outbound</option>
        </select>
      </label>
      <label>
        <span>Spawn in (secs):</span>
        <input type="number" name="spawn_at" min={0} step={15} value={0} />
      </label>
      <label>
        <span>Stagger (secs):</span>
        <input type="number" name="stagger" min={0} step={15} value={0} />
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
      <hr />
      <h2>Flights</h2>
      <div class="list">
        {query.data.map((f) => (
          <FlightItem flight={f} />
        ))}
      </div>
    </div>
  );
}
