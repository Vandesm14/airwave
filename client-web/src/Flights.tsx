import {
  baseAPIPath,
  deleteFlight,
  getAircraft,
  getFlights,
  postCreateFlight,
  useFlights,
} from './lib/api';
import { formatTime } from './lib/lib';
import { Flight } from './lib/types';
import './Flights.scss';
import { useQueryClient } from '@tanstack/solid-query';

export function FlightItem({ flight }: { flight: Flight }) {
  const client = useQueryClient();

  async function handleDelete() {
    const res = await fetch(`${baseAPIPath}${deleteFlight(flight.id)}`, {
      method: 'DELETE',
    });

    if (res.ok) {
      await client.cancelQueries({ queryKey: [getFlights] });
      await client.setQueryData([getFlights], (old: Flight[]) => {
        console.log({ old });
        return old.filter((f) => f.id !== flight.id);
      });
    }
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
  const client = useQueryClient();

  async function createFlight(data: { kind: string; spawn_at: string }) {
    const body = new URLSearchParams(data);

    await fetch(`${baseAPIPath}${postCreateFlight}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body,
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

    await Promise.all(
      Array.from({ length: quantity }).map((_, i) =>
        createFlight({
          spawn_at: (spawnAt + i * stagger).toString(),
          kind,
        })
      )
    );

    // Invalidate and refetch flights
    await client.cancelQueries({ queryKey: [getFlights] });
    await client.invalidateQueries({ queryKey: [getFlights] });
    await client.refetchQueries({ queryKey: [getFlights] });

    if (spawnAt <= 4) {
      // Invalidate and refetch aircraft
      await client.cancelQueries({ queryKey: [getAircraft] });
      await client.invalidateQueries({ queryKey: [getAircraft] });
      await client.refetchQueries({ queryKey: [getAircraft] });
    }
  }

  return (
    <form onSubmit={handleSubmit}>
      <h2>Order:</h2>
      <label>
        <span>Quantity:</span>
        <input type="number" name="quantity" min={1} value={5} />
      </label>
      <label>
        <span>Flight kind:</span>
        <select name="kind">
          <option value="outbound">Outbound</option>
          <option value="inbound">Inbound</option>
        </select>
      </label>
      <label>
        <span>Spawn in (secs):</span>
        <input type="number" name="spawn_at" min={0} step={15} value={0} />
      </label>
      <label>
        <span>Stagger (secs):</span>
        <input type="number" name="stagger" min={0} step={15} value={60} />
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
