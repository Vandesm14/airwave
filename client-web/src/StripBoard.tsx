import {
  Accessor,
  createEffect,
  createMemo,
  createSignal,
  For,
  onMount,
} from 'solid-js';
import { Aircraft } from './lib/types';
import { useAtom } from 'solid-jotai';
import { frequencyAtom, selectedAircraftAtom } from './lib/atoms';

type Strips = {
  Approach: Array<Aircraft>;
  Tower: Array<Aircraft>;
  Ground: Array<Aircraft>;
  Departure: Array<Aircraft>;
  Center: Array<Aircraft>;
};

const Separator = () => <div class="separator"></div>;

type StripProps = {
  strip: Aircraft;
};

function formatTime(duration: number): string {
  const isNegative = duration < 0;
  let absDuration = Math.abs(duration);
  let durationSeconds = Math.floor(absDuration / 1000);
  let seconds = (durationSeconds % 60).toString().padStart(2, '0');
  let minutes = Math.floor(durationSeconds / 60)
    .toString()
    .padStart(2, '0');
  let timeString = `${minutes}:${seconds}`;
  if (isNegative) {
    timeString = `-${timeString}`;
  }

  return timeString;
}

function Strip({ strip }: StripProps) {
  let [ourFrequency] = useAtom(frequencyAtom);
  let [selectedAircraft, setSelectedAircraft] = useAtom(selectedAircraftAtom);

  let sinceCreated = formatTime(Date.now() - strip.created);
  let overtime = !(
    sinceCreated.startsWith('0') ||
    sinceCreated.startsWith('1') ||
    sinceCreated.startsWith('-')
  );
  let topStatus = '';
  let bottomStatus = '';
  let theirs = strip.frequency !== ourFrequency();

  if (sinceCreated.startsWith('-')) {
    theirs = true;
  }

  if (strip.state.type === 'landing') {
    topStatus = 'ILS';
    bottomStatus = strip.state.value.id;
  } else if (strip.state.type === 'taxiing') {
    let current = strip.state.value.current;
    if (current.kind === 'gate') {
      topStatus = 'GATE';
    } else if (current.kind === 'runway') {
      topStatus = 'RNWY';
    } else if (current.kind === 'taxiway') {
      topStatus = 'TXWY';
    }

    bottomStatus = current.name;
  } else if (strip.state.type === 'flying') {
    if (strip.state.value.waypoints.length > 0) {
      topStatus = 'DCT';
      bottomStatus = strip.state.value.waypoints[0].name;
    }
  }

  function handleMouseDown() {
    setSelectedAircraft(strip.callsign);
  }

  return (
    <div
      classList={{
        strip: true,
        theirs,
        overtime,
        selected: selectedAircraft() === strip.callsign,
      }}
      onmousedown={handleMouseDown}
    >
      <div class="vertical">
        <span class="callsign">{strip.callsign}</span>
        <span></span>
      </div>
      <div class="vertical">
        <span>{strip.flight_plan[0]}</span>
        <span>{strip.flight_plan[1]}</span>
      </div>
      <div class="vertical">
        <span>{topStatus}</span>
        <span>{bottomStatus}</span>
      </div>
      <div class="vertical">
        <span class="frequency">{strip.frequency}</span>
        <span class="timer">{sinceCreated}</span>
      </div>
    </div>
  );
}

// function getStripsLocalStorage() {
//   let item = localStorage.getItem('strips');
//   if (typeof item === 'string') {
//     return JSON.parse(item);
//   } else {
//     return [
//       { type: 'header', value: 'Approach' },
//       { type: 'header', value: 'Landing RW20' },
//       { type: 'header', value: 'Landing RW27' },
//       { type: 'header', value: 'Ground' },
//       { type: 'header', value: 'Departure' },
//     ];
//   }
// }

export default function StripBoard({
  aircrafts,
}: {
  aircrafts: Accessor<Array<Aircraft>>;
}) {
  let [strips, setStrips] = createSignal<Strips>(
    { Approach: [], Tower: [], Ground: [], Departure: [], Center: [] },
    {
      equals: false,
    }
  );

  let stripEntries = createMemo(() => Object.entries(strips()));

  // createEffect(() => {
  //   localStorage.setItem('strips', JSON.stringify(strips()));
  // });

  createEffect(() => {
    // This is to prevent initial loading state from removing saved strips.
    //
    // When we first load, aircrafts() will be blank, since they havent been
    // loaded from the server yet. So, when we run the purge function to clean
    // up nonexistent callsigns from the strips, all are cleaned up.
    if (aircrafts().length > 0) {
      let strips: Strips = {
        Approach: [],
        Tower: [],
        Ground: [],
        Departure: [],
        Center: [],
      };

      for (let aircraft of aircrafts()) {
        if (aircraft.state.type === 'landing') {
          strips.Tower.push(aircraft);
        } else if (aircraft.state.type === 'taxiing') {
          if (
            ((aircraft.state.value.waypoints.length === 1 &&
              aircraft.state.value.waypoints[0].kind === 'runway') ||
              aircraft.state.value.current.kind === 'runway') &&
            aircraft.flight_plan[1] !== 'KSFO'
          ) {
            strips.Tower.push(aircraft);
          } else {
            strips.Ground.push(aircraft);
          }
        } else if (
          // TODO: Don't hard-code this, use an airspace selector in the UI
          aircraft.airspace === 'KSFO' &&
          aircraft.airspace === aircraft.flight_plan[0]
        ) {
          if (aircraft.altitude >= 2000) {
            strips.Departure.push(aircraft);
          } else {
            strips.Tower.push(aircraft);
          }
        } else if (
          // TODO: Don't hard-code this, use an airspace selector in the UI
          'KSFO' === aircraft.flight_plan[1] &&
          aircraft.airspace !== aircraft.flight_plan[0]
        ) {
          strips.Approach.push(aircraft);
        } else {
          strips.Center.push(aircraft);
        }
      }

      const sorter = (a: Aircraft, b: Aircraft) => b.created - a.created;
      strips.Approach.sort(sorter);
      strips.Departure.sort(sorter);
      strips.Ground.sort(sorter);
      strips.Tower.sort(sorter);
      strips.Center.sort(sorter);

      setStrips(strips);
    }
  });

  return (
    <div id="stripboard">
      <div class="header">All ({aircrafts().length})</div>
      {stripEntries().map(([key, list]) => (
        <>
          <div class="header">
            {key} ({list.length})
          </div>
          {list.map((strip) => (
            <Strip strip={strip}></Strip>
          ))}
        </>
      ))}
    </div>
  );
}
