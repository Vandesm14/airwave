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
  onmousedown?: () => void;
  onmousemove?: () => void;
};

function Strip({ strip, onmousedown, onmousemove }: StripProps) {
  let [ourFrequency] = useAtom(frequencyAtom);
  let [selectedAircraft, setSelectedAircraft] = useAtom(selectedAircraftAtom);

  let [control] = useAtom(controlAtom);
  let [airspace] = useAtom(control().airspace);

  let world = useWorld();

  if (strip.type === StripType.Header) {
    return (
      <div
        class="header"
        onmousedown={() => onmousedown?.()}
        onmousemove={() => onmousemove?.()}
      >
        {strip.name}
      </div>
    );
  } else if (strip.type === StripType.Aircraft && airspace()) {
    let found = world.data?.airspaces.find((a) => a.id === airspace());
    if (!found) {
      return null;
    }

    const handleMouseDown = () => {
      setSelectedAircraft(strip.callsign);
      onmousedown?.();
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
        onmousemove={() => onmousemove?.()}
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

function aircraftToStrip(
  aircraft: Aircraft,
  airspace: Airspace,
  selectedAircraft: string
) {
  const data: AircraftStrip = {
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

const Separator = () => <div class="separator"></div>;

const DELETE = 'DELETE';

export default function StripBoard() {
  const [lenAtDrag, setLenAtDrag] = createSignal<number>(0);
  const [dragged, setDragged] = createSignal<number | null>(null);
  const [separator, setSeparator] = createSignal<number | null>(null);
  const [strips, setStrips] = createSignal<Strips>([], {
    equals: false,
  });

  const aircrafts = createQuery<Aircraft[]>(() => ({
    queryKey: [getAircraft],
    initialData: [],
  }));
  const query = useWorld();
  const [selectedAircraft] = useAtom(selectedAircraftAtom);

  // Prefill the strips with default headers.
  createEffect(() => {
    const airport = hardcodedAirport(query.data!);

    if (
      aircrafts.data.length > 0 &&
      airport !== undefined &&
      strips().length === 0
    ) {
      setStrips([
        newHeader('Inbox'),
        newHeader('Approach'),
        newHeader('Landing'),
        newHeader('Parked'),
        newHeader('Ground'),
        newHeader('Takeoff'),
        newHeader(DELETE),
        // ...airport.runways.map((r) => newHeader(`Landing ${r.id}`)),
      ]);
    }
  });

  // Create strips for aircraft that are our responsibility.
  createEffect(() => {
    const airspace = hardcodedAirspace(query.data!);
    const selected = selectedAircraft();
    const existing = strips()
      .filter((s) => s.type === StripType.Aircraft)
      .map((s) => s.callsign);

    if (airspace && strips().length > 0) {
      const newStrips: AircraftStrip[] = [];
      for (const aircraft of aircrafts.data) {
        if (
          !existing.includes(aircraft.id) &&
          statusOfAircraft(aircraft, airspace.id, selected) !== 'None'
        ) {
          newStrips.push(aircraftToStrip(aircraft, airspace, selected));
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

  // Update aircraft strips.
  createEffect(() => {
    const airspace = hardcodedAirspace(query.data!);
    const selected = selectedAircraft();

    if (airspace && aircrafts.data.length > 0) {
      setStrips((strips) =>
        strips.map((strip) => {
          if (strip.type === StripType.Aircraft) {
            const callsign = strip.callsign;
            const aircraft = aircrafts.data.find((a) => a.id === callsign);
            if (aircraft) {
              strip = aircraftToStrip(aircraft, airspace, selected);
            }
          }

          return strip;
        })
      );
    }
  });

  // Cancel drag if length changes.
  createEffect(() => {
    if (dragged() !== null && strips().length !== lenAtDrag()) {
      setDragged(null);
    } else if (dragged() === null) {
      // Ensure that initial length is correct before the first drag.
      setLenAtDrag(strips().length);
    }
  });

  // Remove strips under DELETE.
  createEffect(() => {
    if (strips().length > 0 && strips().at(-1)?.type !== StripType.Header) {
      setStrips((strips) => {
        return strips.slice(0, -1);
      });
    }
  });

  function handleMouseDown(index: number) {
    setDragged(index);
    setLenAtDrag(strips().length);
  }

  function handleMouseUp() {
    if (separator() !== null) {
      setStrips((strips) => {
        let fromIndex = dragged();
        let toIndex = separator();

        if (fromIndex !== null) {
          const newStrips: Strip[] = [];
          for (let i = 0; i < strips.length; i++) {
            if (i !== fromIndex) {
              const strip = strips[i];
              if (strip) {
                newStrips.push(strip);
              }
            }

            if (i === toIndex) {
              const strip = strips[fromIndex];
              if (strip) {
                newStrips.push(strip);
              }
            }
          }

          return newStrips;
        } else {
          return strips;
        }
      });
    }

    resetDrag();
  }

  function handleMouseMove(index: number) {
    if (dragged()) {
      setSeparator(index);
    }
  }

  function resetDrag() {
    setDragged(null);
    setSeparator(null);
  }

  const allYours = createMemo(
    () => strips().filter((s) => s.type === StripType.Aircraft).length
  );

  return (
    <div
      id="stripboard"
      onmouseleave={() => resetDrag()}
      onmouseup={() => handleMouseUp()}
    >
      Total: {allYours()}
      <For each={strips()}>
        {(strip, index) => {
          if (
            index() === 0 ||
            (strip.type === StripType.Header && strip.name === DELETE)
          ) {
            return (
              <>
                <Strip
                  strip={strip}
                  onmousemove={() => {
                    handleMouseMove(index());
                  }}
                />
                {index() === separator() ? <Separator /> : null}
              </>
            );
          } else {
            return (
              <>
                <Strip
                  strip={strip}
                  onmousedown={() => {
                    handleMouseDown(index());
                  }}
                  onmousemove={() => {
                    handleMouseMove(index());
                  }}
                />
                {index() === separator() ? <Separator /> : null}
              </>
            );
          }
        }}
      </For>
    </div>
  );
}
