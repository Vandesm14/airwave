import { createEffect, createMemo, createSignal, Show } from 'solid-js';
import {
  Aircraft,
  Frequencies,
  isAircraftFlying,
  isAircraftTaxiing,
} from './lib/types';
import { useAtom } from 'solid-jotai';
import { controlAtom, frequencyAtom, selectedAircraftAtom } from './lib/atoms';
import {
  angleBetweenPoints,
  calculateDistance,
  ENROUTE_TIME_MULTIPLIER,
  formatTime,
  nauticalMilesToFeet,
  runwayInfo,
} from './lib/lib';
import { createQuery } from '@tanstack/solid-query';
import { getAircraft, useFlightByAircraft, useWorld } from './lib/api';

type Strips = {
  Selected: Array<Aircraft>;
  Colliding: Array<Aircraft>;
  Inbound: Array<Aircraft>;
  Approach: Array<Aircraft>;
  Landing: Array<Aircraft>;
  Departure: Array<Aircraft>;
  Outbound: Array<Aircraft>;
  Parked: Array<Aircraft>;
  Ground: Array<Aircraft>;
  Takeoff: Array<Aircraft>;
  None: Array<Aircraft>;
};

const newStrips = (): Strips => ({
  Selected: [],
  Colliding: [],
  Inbound: [],
  Approach: [],
  Landing: [],
  Departure: [],
  Outbound: [],
  Parked: [],
  Ground: [],
  Takeoff: [],
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
  const isParked = aircraft.state.type === 'parked';

  const isTaxiingToRunway = (() => {
    if (isAircraftTaxiing(aircraft.state)) {
      const firstWaypoint = aircraft.state.value.waypoints[0];

      return (
        (typeof firstWaypoint !== 'undefined' &&
          firstWaypoint.kind === 'runway' &&
          aircraft.state.value.waypoints.length == 1) ||
        aircraft.state.value.current.kind === 'runway'
      );
    } else {
      return false;
    }
  })();

  const isInLocalAirspace =
    aircraft.state.type === 'flying' ? !aircraft.state.value.enroute : true;
  const isDepartingAndInLocalAirspace =
    isInLocalAirspace && ourAirspace === aircraft.flight_plan.departing;
  const isDepartingFromLocalAirspace =
    ourAirspace === aircraft.flight_plan.departing;

  const isArrivingToLocalAirspace =
    ourAirspace === aircraft.flight_plan.arriving;
  const isSelected = aircraft.id === selectedAircraft;

  if (aircraft.is_colliding) {
    return 'Colliding';
  }

  if (isInLocalAirspace && isLanding) {
    return 'Landing';
  } else if (isTaxiing || isParked) {
    if (isTaxiingToRunway && isDepartingAndInLocalAirspace) {
      return 'Takeoff';
    } else if (isInLocalAirspace) {
      if (aircraft.state.type === 'parked') {
        if (aircraft.state.value.active || isSelected) {
          return 'Parked';
        } else {
          return 'None';
        }
      } else {
        return 'Ground';
      }
    } else {
      return 'None';
    }
  } else if (isDepartingFromLocalAirspace) {
    if (isInLocalAirspace) {
      return 'Departure';
    } else {
      return 'Outbound';
    }
  } else if (isArrivingToLocalAirspace) {
    if (isInLocalAirspace) {
      return 'Approach';
    } else {
      return 'Inbound';
    }
  } else {
    if (isSelected) {
      return 'Selected';
    } else {
      return 'None';
    }
  }
}

function Strip({ strip }: StripProps) {
  const flight = useFlightByAircraft(strip.id);

  let [ourFrequency] = useAtom(frequencyAtom);
  let [selectedAircraft, setSelectedAircraft] = useAtom(selectedAircraftAtom);

  let [control] = useAtom(controlAtom);
  let [airspace] = useAtom(control().airspace);

  const query = useWorld();

  if (!query.data) {
    return null;
  }

  let sinceCreated = `--:--`;

  if (isAircraftFlying(strip.state)) {
    if (strip.state.value.enroute && strip.state.value.waypoints.length > 0) {
      let current = strip.pos;
      let distance = 0;
      let waypoints = strip.state.value.waypoints.slice();
      waypoints.reverse();
      waypoints.forEach((waypoint) => {
        distance += calculateDistance(current, waypoint.value.to);
        current = waypoint.value.to;
      });

      let distanceInNm = distance / nauticalMilesToFeet;
      let time =
        (distanceInNm / (strip.speed * ENROUTE_TIME_MULTIPLIER)) *
        1000 *
        60 *
        60;
      sinceCreated = formatTime(time);
    } else if (
      !strip.state.value.enroute &&
      strip.state.value.waypoints.length > 0
    ) {
      let current = strip.pos;
      let distance = 0;
      let waypoint = strip.state.value.waypoints.at(-1);

      if (typeof waypoint !== 'undefined') {
        distance += calculateDistance(current, waypoint.value.to);
      }

      let distanceInNm = distance / nauticalMilesToFeet;
      let time = (distanceInNm / strip.speed) * 1000 * 60 * 60;
      sinceCreated = formatTime(time);
    } else {
      sinceCreated = `--:--`;
    }
  } else if (isAircraftTaxiing(strip.state) && strip.speed > 0) {
    if (strip.state.value.waypoints.length > 0) {
      let current = strip.pos;
      let distance = 0;
      let waypoints = strip.state.value.waypoints.slice();
      waypoints.reverse();
      waypoints.forEach((waypoint) => {
        distance += calculateDistance(current, waypoint.value);
        current = waypoint.value;
      });

      let distanceInNm = distance / nauticalMilesToFeet;
      let time = (distanceInNm / strip.speed) * 1000 * 60 * 60;
      sinceCreated = formatTime(time);
    }
  } else if (strip.state.type === 'landing') {
    let distance = calculateDistance(
      strip.pos,
      runwayInfo(strip.state.value.runway).start
    );

    let distanceInNm = distance / nauticalMilesToFeet;
    let time = (distanceInNm / strip.speed) * 1000 * 60 * 60;

    sinceCreated = formatTime(time);
  }

  let topStatus = '';
  let bottomStatus = '';
  let dimmer = strip.frequency !== ourFrequency();

  if (sinceCreated.startsWith('-') && sinceCreated !== '--:--') {
    dimmer = true;
  }

  if (strip.state.type === 'landing') {
    topStatus = 'ILS';
    bottomStatus = strip.state.value.runway.id;
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
  } else if (strip.state.type === 'parked') {
    topStatus = 'PARK';
    bottomStatus = strip.state.value.at.name;
  }

  let distance = calculateDistance(strip.pos, query.data.airspace.pos);
  let distanceText = '';

  if (strip.state.type === 'flying' || strip.state.type === 'landing') {
    distanceText = (distance / nauticalMilesToFeet).toFixed(1).slice(0, 4);
    if (distanceText.endsWith('.')) {
      distanceText = distanceText.replace('.', ' ');
    }
    distanceText = `${distanceText} NM`;
  }

  if (
    ((strip.state.type === 'parked' && strip.state.value.active) ||
      strip.state.type === 'taxiing') &&
    strip.flight_plan.arriving !== airspace()
  ) {
    const connection = query.data.connections.find(
      (c) => c.id === strip.flight_plan.arriving
    );
    if (connection !== undefined) {
      const rawAngle = angleBetweenPoints([0, 0], connection.pos);
      const angle = (360 - Math.round(rawAngle) + 90) % 360;

      let closestAngle = Infinity;
      let heading = angle;
      for (const runway of query.data.airspace.airports.flatMap(
        (a) => a.runways
      )) {
        let diff = Math.abs(runway.heading - angle);
        if (diff < closestAngle) {
          closestAngle = diff;
          heading = runway.heading;
        }
      }

      distanceText = `FOR ${heading.toString().slice(0, 2)}`;
    }
  } else if (strip.state.type === 'parked' && !strip.state.value.active) {
    dimmer = true;
  }

  function handleMouseDown() {
    setSelectedAircraft(strip.id);
  }

  let flightTimer = '--:--';
  if (flight.data) {
    flightTimer = formatTime(Date.now() - flight.data.spawn_at.secs * 1000);
  }

  return (
    <div
      classList={{
        strip: true,
        theirs: dimmer,
        colliding: strip.is_colliding,
        selected: selectedAircraft() === strip.id,
        departure: airspace() === strip.flight_plan.departing,
      }}
      onmousedown={handleMouseDown}
    >
      <div class="vertical">
        <span class="callsign">{strip.id}</span>
        <span>{distanceText}</span>
      </div>
      <div class="vertical">
        <Show
          when={
            !(
              strip.state.type === 'parked' &&
              strip.state.value.active === false
            )
          }
        >
          <span>{strip.flight_plan.departing}</span>
          <span>{strip.flight_plan.arriving}</span>
        </Show>
      </div>
      <div class="vertical">
        <span>{topStatus}</span>
        <span>{bottomStatus}</span>
      </div>
      <div class="vertical">
        <span class="frequency">{strip.frequency}</span>
        <span class="timer">{sinceCreated}</span>
      </div>
      <div class="vertical">
        <span>FLGHT</span>
        <span class="timer">{flightTimer}</span>
      </div>
    </div>
  );
}

export default function StripBoard() {
  let [strips, setStrips] = createSignal<Strips>(newStrips(), {
    equals: false,
  });

  let stripEntries = createMemo(() => Object.entries(strips()));
  let [selectedAircraft] = useAtom(selectedAircraftAtom);

  let [control] = useAtom(controlAtom);
  let [airspace] = useAtom(control().airspace);
  let [_, setFrequency] = useAtom(frequencyAtom);

  const aircrafts = createQuery<Aircraft[]>(() => ({
    queryKey: [getAircraft],
    initialData: [],
  }));

  const query = useWorld();

  createEffect(() => {
    const found = query.data?.airspace;
    if (!found) {
      return;
    }

    // This is to prevent initial loading state from removing saved strips.
    //
    // When we first load, aircrafts() will be blank, since they havent been
    // loaded from the server yet. So, when we run the purge function to clean
    // up nonexistent callsigns from the strips, all are cleaned up.
    if (aircrafts.data.length > 0) {
      let strips: Strips = newStrips();

      for (let aircraft of aircrafts.data) {
        let category = assignAircraftToStrips(
          aircraft,
          airspace(),
          selectedAircraft()
        );
        strips[category].push(aircraft);
      }

      const nameSorter = (a: Aircraft, b: Aircraft) =>
        ('' + a.id).localeCompare(b.id);
      const distanteToAirportSorter =
        (rev: boolean) => (a: Aircraft, b: Aircraft) => {
          let distance_a = calculateDistance(a.pos, found.pos);
          let distance_b = calculateDistance(b.pos, found.pos);

          if (rev) {
            return distance_a - distance_b;
          } else {
            return distance_b - distance_a;
          }
        };

      Object.entries(strips).forEach(([key, list]) => {
        list.sort(distanteToAirportSorter(false));
        setStrips({ ...strips, [key]: list });
      });
      strips.Departure.sort(distanteToAirportSorter(true));
      strips.Outbound.sort(distanteToAirportSorter(true));
      strips.Parked.sort(nameSorter);
      strips.Ground.sort(nameSorter);

      setStrips(strips);
    }
  });

  function onClickHeader(name: keyof Strips) {
    const found = query.data?.airspace;
    if (!found) {
      return;
    }

    let key = name.toLowerCase() as keyof Frequencies;
    if (name === 'Landing' || name === 'Takeoff') {
      key = 'tower';
    } else if (name === 'Parked') {
      key = 'ground';
    }

    setFrequency(found.frequencies[key]);
  }

  return (
    <div id="stripboard">
      <div class="header">
        Yours: {aircrafts.data.length - strips().None.length} (All:{' '}
        {aircrafts.data.length})
      </div>
      {strips().Colliding.length > 0 ? (
        <>
          <div class="header">Colliding</div>
          {strips().Colliding.map((strip) => (
            <Strip strip={strip}></Strip>
          ))}
        </>
      ) : null}
      {strips().Selected.length > 0 ? (
        <>
          <div class="header">Selected</div>
          {strips().Selected.map((strip) => (
            <Strip strip={strip}></Strip>
          ))}
        </>
      ) : null}
      {stripEntries().map(([key, list]) =>
        key !== 'None' && key !== 'Selected' && key !== 'Colliding' ? (
          <>
            <div
              class="header"
              onmousedown={() => onClickHeader(key as keyof Strips)}
            >
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
