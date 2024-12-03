import { useFlights } from './lib/api';
import { formatTime } from './lib/lib';
import { Flight } from './lib/types';
import './Flights.scss';

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

export default function Flights() {
  const query = useFlights();

  return (
    <div class="flights">
      <h2>Flights</h2>
      <div class="list">
        {query.data.map((f) => (
          <FlightItem flight={f} />
        ))}
      </div>
    </div>
  );
}
