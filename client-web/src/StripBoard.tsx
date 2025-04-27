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
  callsign: string;
  distance: string;
  arriving: string;
  departing: string;
  topStatus: string;
  bottomStatus: string;
  frequency: number;
  timer: string;
  status: StripStatus;
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

  if (strip.type === StripType.Header) {
    return <div class="header">{strip.name}</div>;
  } else if (strip.type === StripType.Aircraft && airspace()) {
    const handleMouseDown = () => {
      setSelectedAircraft(strip.callsign);
    };

    let dimmer = createMemo(
      () =>
        strip.frequency !== ourFrequency() ||
        (strip.timer.startsWith('-') && strip.timer !== '--:--')
    );

    return (
      <div
        classList={{
          strip: true,
          theirs: dimmer(),
          selected: selectedAircraft() === strip.callsign,
          departure: airspace() === strip.departing,
        }}
        onmousedown={handleMouseDown}
      >
        <div class="vertical">
          <span class="callsign">{strip.callsign}</span>
          <span>{strip.distance}</span>
        </div>
        <div class="vertical">
          <span>{strip.departing}</span>
          <span>{strip.arriving}</span>
        </div>
        <div class="vertical">
          <span>{strip.topStatus}</span>
          <span>{strip.bottomStatus}</span>
        </div>
        <div class="vertical">
          <span class="frequency">{strip.frequency}</span>
          <span class="timer">{strip.timer}</span>
        </div>
      </div>
    );
  } else {
    return null;
  }
}

function aircraftToStrips(
  aircrafts: Aircraft[],
  airspace: Airspace,
  selectedAircraft: string
): AircraftStrip[] {
  let strips: AircraftStrip[] = [];

  for (const aircraft of aircrafts) {
    const strip: AircraftStrip = {
      type: StripType.Aircraft,
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
        strip.timer = formatTime(time);
      }
    } else if (aircraft.state.type === 'landing') {
      let distance = calculateDistance(
        aircraft.pos,
        runwayInfo(aircraft.state.value.runway).start
      );

      let distanceInNm = distance / nauticalMilesToFeet;
      let time = (distanceInNm / aircraft.speed) * 1000 * 60 * 60;

      strip.timer = formatTime(time);
    }

    if (aircraft.state.type === 'landing') {
      strip.topStatus = 'ILS';
      strip.bottomStatus = aircraft.state.value.runway.id;
    } else if (aircraft.state.type === 'taxiing') {
      let current = aircraft.state.value.current;
      if (current.kind === 'gate') {
        strip.topStatus = 'GATE';
      } else if (current.kind === 'runway') {
        strip.topStatus = 'RNWY';
      } else if (current.kind === 'taxiway') {
        strip.topStatus = 'TXWY';
      } else if (current.kind === 'apron') {
        strip.topStatus = 'APRN';
      }

      strip.bottomStatus = current.name;
    } else if (aircraft.state.type === 'parked') {
      strip.topStatus = 'PARK';
      strip.bottomStatus = aircraft.state.value.at.name;
    } else {
      strip.topStatus = smallFlightSegment(aircraft.segment).toUpperCase();
    }

    let distance = calculateDistance(aircraft.pos, airspace.pos);

    if (aircraft.state.type === 'flying' || aircraft.state.type === 'landing') {
      strip.distance = (distance / nauticalMilesToFeet).toFixed(1).slice(0, 4);

      if (strip.distance.endsWith('.')) {
        strip.distance = strip.distance.replace('.', ' ');
      }

      strip.distance = `${strip.distance} NM`;
    }

    strips.push(strip);
  }

  return strips;
}

export default function StripBoard() {
  let [strips, setStrips] = createSignal<Strips>([], {
    equals: false,
  });

  let [selectedAircraft] = useAtom(selectedAircraftAtom);

  const aircrafts = createQuery<Aircraft[]>(() => ({
    queryKey: [getAircraft],
    initialData: [],
  }));
  const query = useWorld();

  createEffect(() => {
    const airport = hardcodedAirport(query.data!);
    const airspace = hardcodedAirspace(query.data!);
    if (
      aircrafts.data.length > 0 &&
      airport !== undefined &&
      airspace !== undefined &&
      strips().length === 0
    ) {
      let newStrips = aircraftToStrips(
        aircrafts.data,
        airspace,
        selectedAircraft()
      );

      setStrips([
        newHeader('Inbox'),
        ...newStrips.filter((s) => s.status !== 'None'),
        newHeader('Approach'),
        ...airport.runways.map((r) => newHeader(`Landing ${r.id}`)),
      ]);
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
