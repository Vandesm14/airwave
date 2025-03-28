import { createEffect, createMemo, createSignal, Show } from 'solid-js';
import {
  Aircraft,
  isAircraftFlying,
  isAircraftLanding,
  isAircraftParked,
  smallFlightSegment,
} from './lib/types';
import { useAtom } from 'solid-jotai';
import { controlAtom, frequencyAtom, selectedAircraftAtom } from './lib/atoms';
import {
  calculateDistance,
  formatTime,
  hardcodedAirspace,
  nauticalMilesToFeet,
  runwayInfo,
} from './lib/lib';
import { createQuery } from '@tanstack/solid-query';
import { getAircraft, useWorld } from './lib/api';

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

type Collapse = {
  [key: string]: boolean;
};

const newCollapse = (): Collapse => {
  const strips = newStrips();
  let collapse: Collapse = {};
  Object.keys(strips).forEach((key) => {
    collapse[key] = false;
  });

  return collapse;
};

type StripProps = {
  strip: Aircraft;
};

function assignAircraftToStrips(
  aircraft: Aircraft,
  ourAirspace: string,
  selectedAircraft: string
): keyof Strips {
  const isSelected = aircraft.id === selectedAircraft;
  const isAccepted = aircraft.accepted;

  const isOurArrival = ourAirspace === aircraft.flight_plan.arriving;
  const isOurDeparture = ourAirspace === aircraft.flight_plan.departing;
  const isOurs = isOurArrival || isOurDeparture;

  const isParked = aircraft.segment === 'parked';
  const isGround = aircraft.segment.startsWith('taxi-');

  const isTakeoff = aircraft.segment === 'takeoff';
  const isDeparture = aircraft.segment === 'departure';
  const isOutbound = aircraft.segment === 'cruise';

  const isLanding = aircraft.segment === 'land';
  const isApproach = aircraft.segment === 'approach';
  const isInbound = true;

  if (aircraft.is_colliding) {
    return 'Colliding';
  }

  if (isAccepted) {
    if (isOurs) {
      if (isParked) return 'Parked';
      if (isGround) return 'Ground';
    }

    if (isOurDeparture) {
      if (isTakeoff) return 'Takeoff';
      if (isDeparture) return 'Departure';
      if (isOutbound) return 'Outbound';
    }

    if (isOurArrival) {
      if (isLanding) return 'Landing';
      if (isApproach) return 'Approach';
      if (isInbound) return 'Inbound';
    }
  }

  if (isSelected) return 'Selected';

  return 'None';
}

function Strip({ strip }: StripProps) {
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
    if (strip.flight_plan.follow) {
      let current = strip.pos;
      let distance = 0;
      strip.flight_plan.waypoints
        .slice(strip.flight_plan.waypoint_index)
        .forEach((waypoint) => {
          distance += calculateDistance(current, waypoint.data.pos);
          current = waypoint.data.pos;
        });

      let distanceInNm = distance / nauticalMilesToFeet;
      let time = (distanceInNm / strip.speed) * 1000 * 60 * 60;
      sinceCreated = formatTime(time);
    }
  } else if (isAircraftLanding(strip.state)) {
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
  let dimmer = createMemo(
    () =>
      strip.frequency !== ourFrequency() ||
      (sinceCreated.startsWith('-') && sinceCreated !== '--:--')
  );

  if (isAircraftLanding(strip.state)) {
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
  } else if (isAircraftParked(strip.state)) {
    topStatus = 'PARK';
    bottomStatus = strip.state.value.at.name;
  } else {
    topStatus = smallFlightSegment(strip.segment).toUpperCase();
  }

  let distance = calculateDistance(
    strip.pos,
    hardcodedAirspace(query.data)!.pos
  );
  let distanceText = '';

  if (isAircraftFlying(strip.state) || isAircraftLanding(strip.state)) {
    distanceText = (distance / nauticalMilesToFeet).toFixed(1).slice(0, 4);

    if (distanceText.endsWith('.')) {
      distanceText = distanceText.replace('.', ' ');
    }

    distanceText = `${distanceText} NM`;
  }

  function handleMouseDown() {
    setSelectedAircraft(strip.id);
  }

  return (
    <div
      classList={{
        strip: true,
        theirs: dimmer(),
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

export default function StripBoard() {
  let [strips, setStrips] = createSignal<Strips>(newStrips(), {
    equals: false,
  });

  let stripEntries = createMemo(() => Object.entries(strips()));
  let [selectedAircraft] = useAtom(selectedAircraftAtom);

  let [control] = useAtom(controlAtom);
  let [airspace] = useAtom(control().airspace);
  let [collapse, setCollapse] = createSignal<Collapse>(newCollapse());

  const aircrafts = createQuery<Aircraft[]>(() => ({
    queryKey: [getAircraft],
    initialData: [],
  }));

  const query = useWorld();

  createEffect(() => {
    const found = hardcodedAirspace(query.data!);
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
    setCollapse({ ...collapse(), [name]: !collapse()[name] });
  }

  const allFlying = createMemo(
    () => aircrafts.data.filter((a) => isAircraftFlying(a.state)).length
  );

  return (
    <div id="stripboard">
      <div class="header">
        Yours: {aircrafts.data.length - strips().None.length} (All:{' '}
        {allFlying()})
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
              onClick={() => onClickHeader(key as keyof Strips)}
            >
              {collapse()[key] ? '- ' : list.length == 0 ? '□ ' : '■ '}
              {key}
              {list.length > 0 ? ` (${list.length})` : ''}
            </div>
            <Show when={!collapse()[key]}>
              {list.map((strip) => (
                <Strip strip={strip}></Strip>
              ))}
            </Show>
          </>
        ) : null
      )}
    </div>
  );
}
