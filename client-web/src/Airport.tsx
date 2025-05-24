import './Airport.scss';
import { createMemo, createSignal, For, Show } from 'solid-js';
import { useAirportStatus, useSetAirportStatus, useWorld } from './lib/api';
import { HARD_CODED_AIRPORT } from './lib/lib';
import { useStorageAtom } from './lib/hooks';
import { airportAtom } from './lib/atoms';

export default function Flights() {
  const [show, setShow] = createSignal(false);

  const [airport, setAirport] = useStorageAtom(airportAtom);

  const airportStatus = useAirportStatus(HARD_CODED_AIRPORT);
  const setAirportStatus = useSetAirportStatus();

  const world = useWorld();
  const airports = createMemo(() => {
    return world.data !== undefined ? world.data.airports : [];
  });

  return (
    <div class="container border">
      <div class="airport">
        <Show when={!show()}>
          <button onClick={() => setShow(true)}>Airport</button>
        </Show>
        <Show when={show()}>
          <button onClick={() => setShow(false)}>Close</button>
          <hr />
          <div class="row">
            <label>
              Airport:{' '}
              <select
                onchange={(e) => {
                  setAirport(e.currentTarget.value);
                  setAirportStatus.mutate({
                    id: e.currentTarget.value,
                    status: airportStatus.data,
                  });
                }}
                value={airport()}
              >
                <For each={airports()}>
                  {(airport) => (
                    <option value={airport.id}>{airport.id}</option>
                  )}
                </For>
              </select>
            </label>
          </div>
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
