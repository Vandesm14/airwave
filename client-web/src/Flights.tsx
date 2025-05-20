import './Flights.scss';
import { createSignal, Show } from 'solid-js';
import {
  useAirportStatus,
  useArrivalStatus,
  useDepartureStatus,
} from './lib/api';
import { HARD_CODED_AIRPORT } from './lib/lib';
import { ArrivalStatus } from '../bindings/ArrivalStatus';
import { DepartureStatus } from '../bindings/DepartureStatus';

export default function Flights() {
  const [show, setShow] = createSignal(false);

  const airportStatus = useAirportStatus(HARD_CODED_AIRPORT);

  const setArrivalStatus = useArrivalStatus();
  const setDepartureStatus = useDepartureStatus();

  return (
    <div class="container border">
      <div class="flights">
        <Show when={!show()}>
          <button onClick={() => setShow(true)}>Flights</button>
        </Show>
        <Show when={show()}>
          <button onClick={() => setShow(false)}>Close</button>
          <hr />
          <div class="row">
            <h2>Arrivals:</h2>
            <select
              value={airportStatus.data.arrival}
              onchange={(e) =>
                setArrivalStatus.mutate({
                  id: HARD_CODED_AIRPORT,
                  status: e.target.value as ArrivalStatus,
                })
              }
            >
              <option value="normal">Normal</option>
              <option value="divert">Divert</option>
              <option value="automated">Automated</option>
            </select>
          </div>
          <div class="row">
            <h2>Departures:</h2>
            <select
              value={airportStatus.data.departure}
              onchange={(e) =>
                setDepartureStatus.mutate({
                  id: HARD_CODED_AIRPORT,
                  status: e.target.value as DepartureStatus,
                })
              }
            >
              <option value="normal">Normal</option>
              <option value="delay">Delay</option>
              <option value="automated">Automated</option>
            </select>
          </div>
        </Show>
      </div>
    </div>
  );
}
