import {
  Accessor,
  createEffect,
  createMemo,
  createSignal,
  For,
  JSX,
  Show,
} from 'solid-js';
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

import './StripBoard.scss';
import { makePersisted } from '@solid-primitives/storage';
import { FlightSegment } from '../bindings/FlightSegment';

enum StripType {
  Header = 1,
  Aircraft,
}

type HeaderStrip = {
  id: number;
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
  id: number;
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
  segment: FlightSegment;
};

type Strip = HeaderStrip | AircraftStrip;

function newHeader(name: string, id: number): HeaderStrip {
  return {
    id,
    type: StripType.Header,
    name,
    collapsed: false,
  };
}

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
  key: unknown;

  strip: Strip;
  onmousedown?: () => void;
  onmousemove?: () => void;
  ondelete?: () => void;
  onedit?: (name: string) => void;
  dragged?: Accessor<number | null>;

  deletable?: boolean;
};

function Strip({
  strip,
  onmousedown,
  onmousemove,
  ondelete,
  onedit,
  dragged,
  deletable,
}: StripProps) {
  let input!: HTMLInputElement;

  let [ourFrequency] = useAtom(frequencyAtom);
  let [selectedAircraft, setSelectedAircraft] = useAtom(selectedAircraftAtom);

  let [control] = useAtom(controlAtom);
  let [airspace] = useAtom(control().airspace);
  let [editing, setEditing] = createSignal(false);

  if (strip.type === StripType.Header) {
    const handleDoubleClick: JSX.EventHandlerUnion<
      HTMLSpanElement,
      MouseEvent
    > = (e) => {
      e.stopPropagation();
      setEditing(true);

      if (input) {
        input.focus();
      }
    };

    const handleKeyDown: JSX.EventHandlerUnion<
      HTMLInputElement,
      KeyboardEvent
    > = (e) => {
      if (e.key === 'Enter') {
        onedit?.((e.target as HTMLInputElement).value);
        setEditing(false);
      }
    };

    const handleBlur: JSX.EventHandlerUnion<HTMLInputElement, FocusEvent> = (
      e
    ) => {
      onedit?.((e.target as HTMLInputElement).value);
      setEditing(false);
    };

    return (
      <div
        classList={{
          strip: true,
          header: true,
          dragged: dragged?.() === strip.id,
        }}
        onmousedown={() => onmousedown?.()}
        onmousemove={() => onmousemove?.()}
      >
        <div class="vertical">
          <Show when={editing()}>
            <input
              type="text"
              onkeydown={handleKeyDown}
              onblur={handleBlur}
              value={strip.name}
              ref={input}
            />
          </Show>
          <Show when={!editing()}>
            <span ondblclick={handleDoubleClick}>{strip.name}</span>
          </Show>
        </div>
        <Show when={deletable}>
          <div class="vertical end">
            <button class="delete" onclick={() => ondelete?.()}>
              X
            </button>
          </div>
        </Show>
      </div>
    );
  } else if (strip.type === StripType.Aircraft) {
    const handleMouseDown = () => {
      setSelectedAircraft(strip.callsign);
      onmousedown?.();
    };

    let dimmer = createMemo(() => strip.frequency !== ourFrequency());

    return (
      <div
        classList={{
          strip: true,
          dragged: dragged?.() === strip.id,
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
          <span class="timer">
            {smallFlightSegment(strip.segment).toUpperCase()}
          </span>
        </div>
        <Show when={deletable}>
          <div class="vertical end">
            <button
              class="delete"
              onclick={() => ondelete?.()}
              onmousedown={(e) => e.stopPropagation()}
            >
              X
            </button>
          </div>
        </Show>
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
    // Default and will be set by adding it into Strips.
    id: 0,
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
    segment: aircraft.segment,
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
    data.topStatus = '----';
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

const IBNOX = 'Inbox';
const INBOX_ID = 0;

function createStrips() {
  const [_, setNextId] = makePersisted(createSignal(0));
  const [strips, setStrips] = makePersisted(
    createSignal<Strip[]>([], { equals: false })
  );

  function addHeader(name: string) {
    setNextId((nextId) => {
      setStrips((strips) => {
        strips.push(newHeader(name, nextId++));
        return strips;
      });
      return nextId;
    });
  }
  function addAircraft(strip: AircraftStrip, headerId: number) {
    const index = strips().findIndex((s) => s.id === headerId);
    if (index !== -1) {
      setNextId((nextId) => {
        setStrips((strips) => {
          strips.splice(index + 1, 0, { ...strip, id: nextId++ });
          return strips;
        });
        return nextId;
      });
    }
  }

  function strip(stripId: number) {
    return strips().find((s) => s.id === stripId);
  }

  function move(fromId: number, toId: number) {
    if (fromId === toId) return;

    const fromStrip = strip(fromId);
    const toStrip = strip(toId);

    if (fromStrip !== undefined && toStrip !== undefined) {
      setStrips((strips) => {
        const fromIndex = strips.findIndex((s) => s.id === fromId);
        if (fromIndex !== -1) {
          strips.splice(fromIndex, 1);
        }

        const toIndex = strips.findIndex((s) => s.id === toId);
        if (toIndex !== -1) {
          strips.splice(toIndex + 1, 0, fromStrip);
        }

        return strips;
      });
    }
  }

  function update(strip: Strip, stripId: number) {
    setStrips((strips) => strips.map((s) => (s.id === stripId ? strip : s)));
  }

  function remove(stripId: number) {
    const index = strips().findIndex((s) => s.id === stripId);
    if (index !== -1) {
      setStrips((strips) => {
        strips.splice(index, 1);
        return strips;
      });
    }
  }

  function length(): number {
    return strips().length;
  }

  return {
    strips,
    setStrips,

    addHeader,
    addAircraft,
    move,
    strip,
    update,
    remove,
    length,
  };
}

export default function StripBoard() {
  const [lenAtDrag, setLenAtDrag] = createSignal<number>(0);
  const [dragged, setDragged] = createSignal<number | null>(null);
  const [separator, setSeparator] = createSignal<number | null>(null);
  const board = createStrips();

  const aircrafts = createQuery<Aircraft[]>(() => ({
    queryKey: [getAircraft],
    initialData: [],
  }));
  const query = useWorld();
  const [selectedAircraft, setSelectedAircraft] = useAtom(selectedAircraftAtom);

  // Prefill the strips with default headers.
  createEffect(() => {
    const airport = hardcodedAirport(query.data!);

    if (
      board.length() === 0 &&
      aircrafts.data.length > 0 &&
      airport !== undefined
    ) {
      const defaultHeaders: string[] = [
        'Inbox',
        'Approach',
        'Landing',
        'Parked',
        'Ground',
        'Takeoff',
      ];

      defaultHeaders.forEach((h) => board.addHeader(h));
    }
  });

  // Create strips for aircraft that are our responsibility.
  createEffect(() => {
    const airspace = hardcodedAirspace(query.data!);
    const selected = selectedAircraft();
    const existing = board
      .strips()
      .filter((s) => s.type === StripType.Aircraft)
      .map((s) => s.callsign);

    if (airspace && board.length() > 0) {
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
        newStrips.forEach((a) => board.addAircraft(a, INBOX_ID));
      }
    }
  });

  // Update aircraft strips.
  createEffect(() => {
    const airspace = hardcodedAirspace(query.data!);
    const selected = selectedAircraft();

    if (airspace && aircrafts.data.length > 0) {
      board.setStrips((strips) =>
        strips.map((strip) => {
          if (strip.type === StripType.Aircraft) {
            const callsign = strip.callsign;
            const aircraft = aircrafts.data.find((a) => a.id === callsign);
            if (aircraft) {
              return {
                ...aircraftToStrip(aircraft, airspace, selected),
                id: strip.id,
              };
            }
          }

          return strip;
        })
      );
    }
  });

  // Cancel drag if length changes.
  createEffect(() => {
    if (dragged() !== null && board.length() !== lenAtDrag()) {
      setDragged(null);
    } else if (dragged() === null) {
      // Ensure that initial length is correct before the first drag.
      setLenAtDrag(board.length());
    }
  });

  function handleDelete(stripId: number) {
    const strip = board.strip(stripId);
    if (strip) {
      // Deselect aircraft if deleting selected strip.
      if (
        strip.type === StripType.Aircraft &&
        selectedAircraft() === strip.callsign
      ) {
        setSelectedAircraft('');
      }

      board.remove(strip.id);
    }
  }

  function handleMouseDown(stripId: number) {
    setDragged(stripId);
    setLenAtDrag(board.length());
  }

  function handleMouseUp() {
    let fromId = dragged();
    let toId = separator();
    if (toId !== null && fromId !== null) {
      board.move(fromId, toId);
    }

    resetDrag();
  }

  function handleMouseMove(stripId: number) {
    if (dragged()) {
      setSeparator(stripId);
    }
  }

  function handleEdit(stripId: number, name: string) {
    const strip = board.strip(stripId);
    if (strip && strip.type === StripType.Header) {
      board.update({ ...strip, name }, stripId);
    }
  }

  function handleAdd() {
    board.addHeader('New Header');
  }

  function resetDrag() {
    setDragged(null);
    setSeparator(null);
  }

  const allYours = createMemo(
    () => board.strips().filter((s) => s.type === StripType.Aircraft).length
  );
  const allFlying = createMemo(
    () => aircrafts.data.filter((a) => a.state.type === 'flying').length
  );

  return (
    <div
      id="stripboard"
      onmouseleave={() => resetDrag()}
      onmouseup={() => handleMouseUp()}
    >
      <div class="horizontal">
        <span>
          Total: {allYours()} (All: {allFlying()})
        </span>
        <button onclick={handleAdd}>Add</button>
      </div>
      <For each={board.strips()}>
        {(strip, index) => {
          // Prevent the inbox from being deleted or moved.
          if (strip.type === StripType.Header && strip.name === IBNOX) {
            return (
              <>
                <Strip
                  key={index()}
                  strip={strip}
                  onmousemove={() => handleMouseMove(strip.id)}
                  deletable={false}
                />
                {strip.id === separator() ? <Separator /> : null}
              </>
            );
          } else {
            return (
              <>
                <Strip
                  key={index()}
                  strip={strip}
                  onmousedown={() => handleMouseDown(strip.id)}
                  onmousemove={() => handleMouseMove(strip.id)}
                  ondelete={() => handleDelete(strip.id)}
                  onedit={(name) => handleEdit(strip.id, name)}
                  dragged={() => (separator() !== null ? dragged() : null)}
                  deletable
                />
                {strip.id === separator() ? <Separator /> : null}
              </>
            );
          }
        }}
      </For>
    </div>
  );
}
