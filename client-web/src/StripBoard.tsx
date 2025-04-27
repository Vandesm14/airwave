import { createEffect, createMemo, createSignal, For } from 'solid-js';
import { hardcodedAirport, smallFlightSegment } from './lib/lib';
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
import { Aircraft } from '../bindings/Aircraft';
import { World } from '../bindings/World';
import { Airspace } from '../bindings/Airspace';

enum StripType {
  Header = 1,
  Aircraft,
}

type HeaderStrip = {
  type: StripType.Header;
  name: string;
  collapsed: boolean;
};

type StripStatus =
  | 'Parked'
  | 'Ground'
  | 'Takeoff'
  | 'Departure'
  | 'Outbound'
  | 'Landing'
  | 'Approach'
  | 'Inbound'
  | 'Selected'
  | 'None';

type AircraftStrip = {
  type: StripType.Aircraft;
  data: Aircraft;
};

type Strip = HeaderStrip | AircraftStrip;

function newHeader(name: string): Strip {
  return {
    type: StripType.Header,
    name,
    collapsed: false,
  };
}

type Strips = Array<Strip>;

function statusOfAircraft(
  aircraft: Aircraft,
  ourAirspace: string,
  selectedAircraft: string
): StripStatus {
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

type StripProps = {
  strip: Strip;
};

function Strip({ strip }: StripProps) {
  let [ourFrequency] = useAtom(frequencyAtom);
  let [selectedAircraft, setSelectedAircraft] = useAtom(selectedAircraftAtom);

  let [control] = useAtom(controlAtom);
  let [airspace] = useAtom(control().airspace);

  let world = useWorld();

  if (strip.type === StripType.Header) {
    return <div class="header">{strip.name}</div>;
  } else if (strip.type === StripType.Aircraft && airspace()) {
    let found = world.data?.airspaces.find((a) => a.id === airspace());
    if (!found) {
      return null;
    }

    const data = aircraftToStripData(strip.data, found, selectedAircraft());

    const handleMouseDown = () => {
      setSelectedAircraft(data.callsign);
    };

    let dimmer = createMemo(
      () =>
        data.frequency !== ourFrequency() ||
        (data.timer.startsWith('-') && data.timer !== '--:--')
    );

    return (
      <div
        classList={{
          strip: true,
          theirs: dimmer(),
          selected: selectedAircraft() === data.callsign,
          departure: airspace() === data.departing,
        }}
        onmousedown={handleMouseDown}
      >
        <div class="vertical">
          <span class="callsign">{data.callsign}</span>
          <span>{data.distance}</span>
        </div>
        <div class="vertical">
          <span>{data.departing}</span>
          <span>{data.arriving}</span>
        </div>
        <div class="vertical">
          <span>{data.topStatus}</span>
          <span>{data.bottomStatus}</span>
        </div>
        <div class="vertical">
          <span class="frequency">{data.frequency}</span>
          <span class="timer">{data.timer}</span>
        </div>
      </div>
    );
  } else {
    return null;
  }
}

function aircraftToStripData(
  aircraft: Aircraft,
  airspace: Airspace,
  selectedAircraft: string
) {
  const data = {
    callsign: aircraft.id,
    distance: '',
    arriving: aircraft.flight_plan.arriving,
    departing: aircraft.flight_plan.departing,
    topStatus: '',
    bottomStatus: '',
    frequency: aircraft.frequency,
    timer: '--:--',
    status: statusOfAircraft(aircraft, airspace.id, selectedAircraft),
  };

  if (aircraft.state.type === 'flying') {
    if (aircraft.flight_plan.follow) {
      let current = aircraft.pos;
      let distance = 0;
      aircraft.flight_plan.waypoints
        .slice(aircraft.flight_plan.waypoint_index)
        .forEach((waypoint) => {
          distance += calculateDistance(current, waypoint.data.pos);
          current = waypoint.data.pos;
        });

      let distanceInNm = distance / nauticalMilesToFeet;
      let time = (distanceInNm / aircraft.speed) * 1000 * 60 * 60;
      data.timer = formatTime(time);
    }
  } else if (aircraft.state.type === 'landing') {
    let distance = calculateDistance(
      aircraft.pos,
      runwayInfo(aircraft.state.value.runway).start
    );

    let distanceInNm = distance / nauticalMilesToFeet;
    let time = (distanceInNm / aircraft.speed) * 1000 * 60 * 60;

    data.timer = formatTime(time);
  }

  if (aircraft.state.type === 'landing') {
    data.topStatus = 'ILS';
    data.bottomStatus = aircraft.state.value.runway.id;
  } else if (aircraft.state.type === 'taxiing') {
    let current = aircraft.state.value.current;
    if (current.kind === 'gate') {
      data.topStatus = 'GATE';
    } else if (current.kind === 'runway') {
      data.topStatus = 'RNWY';
    } else if (current.kind === 'taxiway') {
      data.topStatus = 'TXWY';
    } else if (current.kind === 'apron') {
      data.topStatus = 'APRN';
    }

    data.bottomStatus = current.name;
  } else if (aircraft.state.type === 'parked') {
    data.topStatus = 'PARK';
    data.bottomStatus = aircraft.state.value.at.name;
  } else {
    data.topStatus = smallFlightSegment(aircraft.segment).toUpperCase();
  }

  let distance = calculateDistance(aircraft.pos, airspace.pos);

  if (aircraft.state.type === 'flying' || aircraft.state.type === 'landing') {
    data.distance = (distance / nauticalMilesToFeet).toFixed(1).slice(0, 4);

    if (data.distance.endsWith('.')) {
      data.distance = data.distance.replace('.', ' ');
    }

    data.distance = `${data.distance} NM`;
  }

  return data;
}

export default function StripBoard() {
  let [strips, setStrips] = createSignal<Strips>([], {
    equals: false,
  });

  const aircrafts = createQuery<Aircraft[]>(() => ({
    queryKey: [getAircraft],
    initialData: [],
  }));
  const query = useWorld();

  const [selectedAircraft] = useAtom(selectedAircraftAtom);

  createEffect(() => {
    const airport = hardcodedAirport(query.data!);
    const airspace = hardcodedAirspace(query.data!);
    if (
      aircrafts.data.length > 0 &&
      airport !== undefined &&
      strips().length === 0
    ) {
      setStrips([
        newHeader('Inbox'),
        newHeader('Approach'),
        ...airport.runways.map((r) => newHeader(`Landing ${r.id}`)),
      ]);
    }

    const existing = strips()
      .filter((s) => s.type === StripType.Aircraft)
      .map((s) => s.data.id);
    if (airspace && strips().length > 0) {
      const selected = selectedAircraft();

      const newStrips: AircraftStrip[] = [];
      for (const aircraft of aircrafts.data) {
        if (
          !existing.includes(aircraft.id) &&
          statusOfAircraft(aircraft, airspace.id, selected) !== 'None'
        ) {
          newStrips.push({ type: StripType.Aircraft, data: aircraft });
        }
      }

      if (newStrips.length > 0) {
        setStrips((strips) => {
          strips.splice(1, 0, ...newStrips);

          return strips;
        });
      }
    }
  });

  const allYours = createMemo(
    () => strips().filter((s) => s.type === StripType.Aircraft).length
  );
  const allFlying = createMemo(
    () => aircrafts.data.filter((a) => a.state.type === 'flying').length
  );

  return (
    <div id="stripboard">
      Yours: {allYours()} (All: {allFlying()})
      <For each={strips()}>{(strip, _) => <Strip strip={strip} />}</For>
    </div>
  );
}
