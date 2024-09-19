import { Accessor, createEffect, createMemo, createSignal } from 'solid-js';
import {
  Aircraft,
  arrToVec2,
  isAircraftFlying,
  isAircraftTaxiing,
} from './lib/types';
import { useAtom } from 'solid-jotai';
import {
  controlAtom,
  frequencyAtom,
  selectedAircraftAtom,
  worldAtom,
} from './lib/atoms';
import {
  calculateDistance,
  distanceToAirspace,
  distanceToDestination,
  nauticalMilesToFeet,
  runwayInfo,
} from './lib/lib';

type Strips = {
  Selected: Array<Aircraft>;
  Center: Array<Aircraft>;
  Approach: Array<Aircraft>;
  Landing: Array<Aircraft>;
  Parked: Array<Aircraft>;
  Ground: Array<Aircraft>;
  Takeoff: Array<Aircraft>;
  Departure: Array<Aircraft>;
  None: Array<Aircraft>;
};

const newStrips = (): Strips => ({
  Selected: [],
  Center: [],
  Approach: [],
  Landing: [],
  Parked: [],
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
  ourAirspace: string,
  selectedAircraft: string
): keyof Strips {
  const isLanding = aircraft.state.type === 'landing';
  const isTaxiing = aircraft.state.type === 'taxiing';

  const isTaxiingToRunway = (() => {
    if (isAircraftTaxiing(aircraft.state)) {
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
  const isDepartingAndInLocalAirspace =
    isInLocalAirspace && aircraft.airspace === aircraft.flight_plan.departing;
  const isDepartingFromLocalAirspace =
    ourAirspace === aircraft.flight_plan.departing;

  const isArrivingToLocalAirspace =
    ourAirspace === aircraft.flight_plan.arriving &&
    aircraft.airspace !== aircraft.flight_plan.departing;

  if (isInLocalAirspace && isLanding) {
    return 'Landing';
  } else if (isTaxiing) {
    if (isTaxiingToRunway && isDepartingAndInLocalAirspace) {
      return 'Takeoff';
    } else if (isInLocalAirspace) {
      if (aircraft.created < Date.now()) {
        return 'Ground';
      } else {
        return 'Parked';
      }
    } else {
      return 'None';
    }
  } else if (isDepartingFromLocalAirspace) {
    if (isInLocalAirspace) {
      return 'Departure';
    } else {
      return 'Center';
    }
  } else if (isArrivingToLocalAirspace) {
    if (isInLocalAirspace) {
      return 'Approach';
    } else {
      return 'Center';
    }
  } else {
    if (aircraft.id === selectedAircraft) {
      return 'Selected';
    } else {
      return 'None';
    }
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

  let [world] = useAtom(worldAtom);

  let [control] = useAtom(controlAtom);
  let [airspace] = useAtom(control().airspace);

  let sinceCreated = formatTime(Date.now() - strip.created);

  if (isAircraftFlying(strip.state)) {
    if (strip.state.value.waypoints.length > 0) {
      let current = strip.pos;
      let distance = 0;
      let waypoints = strip.state.value.waypoints.slice();
      waypoints.reverse();
      waypoints.forEach((waypoint) => {
        distance += calculateDistance(current, arrToVec2(waypoint.value.to));
        current = arrToVec2(waypoint.value.to);
      });

      let distanceInNm = distance / nauticalMilesToFeet;
      let time = (distanceInNm / strip.speed) * 1000 * 60 * 60;

      sinceCreated = formatTime(time);
    } else {
      sinceCreated = `--:--`;
    }
  } else if (strip.state.type === 'landing') {
    let distance = calculateDistance(
      strip.pos,
      runwayInfo(strip.state.value).start
    );

    let distanceInNm = distance / nauticalMilesToFeet;
    let time = (distanceInNm / strip.speed) * 1000 * 60 * 60;

    sinceCreated = formatTime(time);
  }

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

  let distance = distanceToAirspace(strip, world(), airspace());

  function handleMouseDown() {
    setSelectedAircraft(strip.id);
  }

  return (
    <div
      classList={{
        strip: true,
        theirs,
        selected: selectedAircraft() === strip.id,
        departure: airspace() === strip.flight_plan.departing,
      }}
      onmousedown={handleMouseDown}
    >
      <div class="vertical">
        <span class="callsign">{strip.id}</span>
        <span>
          {(distance / nauticalMilesToFeet).toFixed(1).slice(0, 4)} NM
        </span>
      </div>
      <div class="vertical">
        <span>{strip.flight_plan.departing}</span>
        <span>{strip.flight_plan.arriving}</span>
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
  let [selectedAircraft] = useAtom(selectedAircraftAtom);

  let [world] = useAtom(worldAtom);

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
        let category = assignAircraftToStrips(
          aircraft,
          airspace(),
          selectedAircraft()
        );
        strips[category].push(aircraft);
      }

      const timeSorter = (a: Aircraft, b: Aircraft) => b.created - a.created;
      const nameSorter = (a: Aircraft, b: Aircraft) =>
        ('' + a.id).localeCompare(b.id);
      const distanteToAirportSorter = (a: Aircraft, b: Aircraft) => {
        return (
          distanceToAirspace(b, world(), airspace()) -
          distanceToAirspace(a, world(), airspace())
        );
      };

      Object.entries(strips).forEach(([key, list]) => {
        list.sort(distanteToAirportSorter);
        setStrips({ ...strips, [key]: list });
      });
      // strips.Center.sort(nameSorter);
      strips.Parked.sort(timeSorter);
      strips.Ground.sort(timeSorter);

      setStrips(strips);
    }
  });

  return (
    <div id="stripboard">
      <div class="header">
        Yours: {aircrafts().length - strips().None.length} (All:{' '}
        {aircrafts().length})
      </div>
      {strips().Selected.length > 0 ? (
        <>
          <div class="header">Selected</div>
          {strips().Selected.map((strip) => (
            <Strip strip={strip}></Strip>
          ))}
        </>
      ) : null}
      {stripEntries().map(([key, list]) =>
        key !== 'None' && key !== 'Selected' ? (
          <>
            <div class="header">
              {key}
              {list.length > 0 ? ` (${list.length})` : ''}
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
