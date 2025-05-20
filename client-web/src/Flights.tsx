import './Flights.scss';
import { createSignal, Show } from 'solid-js';
import { useAirportStatus, useSetAirportStatus } from './lib/api';
import { HARD_CODED_AIRPORT } from './lib/lib';

export default function Flights() {
  const [show, setShow] = createSignal(false);

  const airportStatus = useAirportStatus(HARD_CODED_AIRPORT);
  const setAirportStatus = useSetAirportStatus();

  return (
    <div class="container border">
      <div class="flights">
        <Show when={!show()}>
          <button onClick={() => setShow(true)}>Airport</button>
        </Show>
        <Show when={show()}>
          <button onClick={() => setShow(false)}>Close</button>
          <hr />
          <div class="row">
            <label>
              Divert Arrivals:{' '}
              <input
                type="checkbox"
                onchange={(e) =>
                  setAirportStatus.mutate({
                    id: HARD_CODED_AIRPORT,
                    status: {
                      ...airportStatus.data,
                      divert_arrivals: e.target.checked,
                    },
                  })
                }
                checked={airportStatus.data.divert_arrivals}
              />
            </label>
          </div>
          <div class="row">
            <label>
              Delay Departures:{' '}
              <input
                type="checkbox"
                onchange={(e) =>
                  setAirportStatus.mutate({
                    id: HARD_CODED_AIRPORT,
                    status: {
                      ...airportStatus.data,
                      delay_departures: e.target.checked,
                    },
                  })
                }
                checked={airportStatus.data.delay_departures}
              />
            </label>
          </div>
          <hr />
          <div class="row">
            <label>
              Automate Air:{' '}
              <input
                type="checkbox"
                onchange={(e) =>
                  setAirportStatus.mutate({
                    id: HARD_CODED_AIRPORT,
                    status: {
                      ...airportStatus.data,
                      automate_air: e.target.checked,
                    },
                  })
                }
                checked={airportStatus.data.automate_air}
              />
            </label>
          </div>
          <div class="row">
            <label>
              Automate Ground:{' '}
              <input
                type="checkbox"
                onchange={(e) =>
                  setAirportStatus.mutate({
                    id: HARD_CODED_AIRPORT,
                    status: {
                      ...airportStatus.data,
                      automate_ground: e.target.checked,
                    },
                  })
                }
                checked={airportStatus.data.automate_ground}
              />
            </label>
          </div>
        </Show>
      </div>
    </div>
  );
}
