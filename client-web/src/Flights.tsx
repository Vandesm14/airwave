import { useFlights } from './lib/api';
import { formatTime } from './lib/lib';
import { Flight } from './lib/types';

export function FlightItem({ flight }: { flight: Flight }) {
  return (
    <li>
      {flight.kind}{' '}
      {formatTime(new Date().getTime() - flight.spawn_at.secs * 1000)}
    </li>
  );
}

export default function Flights() {
  const query = useFlights();

  return (
    <div class="flights">
      <h2>Flights</h2>
      <ul>
        {query.data.map((f) => (
          <FlightItem flight={f} />
        ))}
      </ul>
    </div>
  );
}
