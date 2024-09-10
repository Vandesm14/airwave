import { Accessor, createEffect, createMemo, createSignal } from 'solid-js';
import { Aircraft } from './lib/types';
import { useAtom } from 'solid-jotai';
import { controlAtom, frequencyAtom, selectedAircraftAtom } from './lib/atoms';

type Strips = {
  Center: Array<Aircraft>;
  Approach: Array<Aircraft>;
  Landing: Array<Aircraft>;
  Ground: Array<Aircraft>;
  Takeoff: Array<Aircraft>;
  Departure: Array<Aircraft>;
  None: Array<Aircraft>;
};

const Separator = () => <div class="separator"></div>;
const newStrips = (): Strips => ({
  Center: [],
  Approach: [],
  Landing: [],
  Ground: [],
  Takeoff: [],
  Departure: [],
  None: [],
});

type StripProps = {
  strip: Aircraft;
};

function assignAircraftToStrips(
  aircraft: Aircraft,
  ourAirspace: string
): keyof Strips {
  const isLanding = aircraft.state.type === 'landing';
  const isTaxiing = aircraft.state.type === 'taxiing';

  const isTaxiingToRunway = (() => {
    if (aircraft.state.type === 'taxiing') {
      return (
        (aircraft.state.value.waypoints.length === 1 &&
          aircraft.state.value.waypoints[0].kind === 'runway') ||
        aircraft.state.value.current.kind === 'runway'
      );
    } else {
      return false;
    }
  })();

  // TODO: Don't hard-code this, use an airspace selector in the UI
  const isInLocalAirspace = aircraft.airspace === ourAirspace;
  const isDepartingFromLocalAirspace =
    isInLocalAirspace && aircraft.airspace === aircraft.flight_plan[0];

  const isArrivingToLocalAirspace =
    ourAirspace === aircraft.flight_plan[1] &&
    aircraft.airspace !== aircraft.flight_plan[0];

  if (isInLocalAirspace && isLanding) {
    return 'Landing';
  } else if (isTaxiing) {
    if (isTaxiingToRunway && isDepartingFromLocalAirspace) {
      return 'Takeoff';
    } else if (isInLocalAirspace) {
      return 'Ground';
    } else {
      return 'None';
    }
  } else if (isDepartingFromLocalAirspace) {
    return 'Departure';
  } else if (isArrivingToLocalAirspace) {
    if (isInLocalAirspace) {
      return 'Approach';
    } else {
      return 'Center';
    }
  } else {
    return 'None';
  }
}

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
    } else if (current.kind === 'apron') {
      topStatus = 'APRN';
    }

    bottomStatus = current.name;
  } else if (strip.state.type === 'flying') {
    if (strip.state.value.waypoints.length > 0) {
      topStatus = 'DCT';
      bottomStatus = strip.state.value.waypoints.at(-1).name;
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

export default function StripBoard({
  aircrafts,
}: {
  aircrafts: Accessor<Array<Aircraft>>;
}) {
  let [strips, setStrips] = createSignal<Strips>(newStrips(), {
    equals: false,
  });

  let stripEntries = createMemo(() => Object.entries(strips()));

  let [control] = useAtom(controlAtom);
  let [airspace] = useAtom(control().airspace);

  createEffect(() => {
    // This is to prevent initial loading state from removing saved strips.
    //
    // When we first load, aircrafts() will be blank, since they havent been
    // loaded from the server yet. So, when we run the purge function to clean
    // up nonexistent callsigns from the strips, all are cleaned up.
    if (aircrafts().length > 0) {
      let strips: Strips = newStrips();

      for (let aircraft of aircrafts()) {
        let category = assignAircraftToStrips(aircraft, airspace());
        strips[category].push(aircraft);
      }

      const sorter = (a: Aircraft, b: Aircraft) => b.created - a.created;
      Object.entries(strips).forEach(([key, list]) => {
        list.sort(sorter);
        setStrips({ ...strips, [key]: list });
      });

      setStrips(strips);
    }
  });

  return (
    <div id="stripboard">
      <div class="header">
        Yours: {aircrafts().length - strips().None.length} (All:{' '}
        {aircrafts().length})
      </div>
      {stripEntries().map(([key, list]) =>
        key !== 'None' ? (
          <>
            <div class="header">
              {key} ({list.length})
            </div>
            {list.map((strip) => (
              <Strip strip={strip}></Strip>
            ))}
          </>
        ) : null
      )}
    </div>
  );
}
