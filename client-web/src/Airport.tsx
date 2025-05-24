import './Airport.scss';
import { createEffect, createMemo, createSignal, For, Show } from 'solid-js';
import {
  getAirportStatusKey,
  useAirportStatus,
  useSetAirportStatus,
  useWorld,
} from './lib/api';
import { useStorageAtom } from './lib/hooks';
import { airportAtom } from './lib/atoms';
import { useQueryClient } from '@tanstack/solid-query';

export default function Flights() {
  const [show, setShow] = createSignal(false);

  const [airport, setAirport] = useStorageAtom(airportAtom);

  const world = useWorld();
  const airportStatus = useAirportStatus(airport);
  const setAirportStatus = useSetAirportStatus();

  const client = useQueryClient();

  createEffect(async () => {
    let _ = airport();

    await client.invalidateQueries({
      queryKey: [getAirportStatusKey],
      type: 'all',
      exact: true,
    });
    await client.refetchQueries({
      queryKey: [getAirportStatusKey],
      type: 'all',
      exact: true,
    });
  });

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
                onchange={(e) => setAirport(e.currentTarget.value)}
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
                    id: airport()!,
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
                    id: airport()!,
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
                    id: airport()!,
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
                    id: airport()!,
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
