import {
  Accessor,
  createEffect,
  createMemo,
  createSignal,
  For,
  JSX,
  Show,
  Signal,
} from 'solid-js';
import { realTimeTicks, smallFlightSegment, TICK_RATE_TPS } from './lib/lib';
import { useAtom } from 'solid-jotai';
import { controlAtom, frequencyAtom, selectedAircraftAtom } from './lib/atoms';
import {
  calculateDistance,
  formatTime,
  hardcodedAirspace,
  nauticalMilesToFeet,
} from './lib/lib';
import { createQuery } from '@tanstack/solid-query';
import { getAircraft, ServerTicks, usePing, useWorld } from './lib/api';
import { Aircraft } from '../bindings/Aircraft';
import { Airspace } from '../bindings/Airspace';
import { makePersisted } from '@solid-primitives/storage';
import { FlightSegment } from '../bindings/FlightSegment';
import { unwrap } from 'solid-js/store';
import fastDeepEqual from 'fast-deep-equal';

import './StripBoard.scss';

const INBOX_ID = 0;

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
  | 'Boarding'
  | 'Parked'
  | 'Ground'
  | 'Takeoff'
  | 'Departure'
  | 'Landing'
  | 'Approach'
  | 'Inbound'
  | 'Selected'
  | 'None';

type CreateOn = {
  inbound: boolean;
  approach: boolean;
  departure: boolean;
  landing: boolean;
  takeoff: boolean;
  boarding: boolean;
  parked: boolean;
  ground: boolean;
};

function newCreateOn(): CreateOn {
  return {
    inbound: false,
    approach: true,
    departure: true,
    landing: true,
    takeoff: true,
    boarding: false,
    parked: true,
    ground: true,
  };
}

function testStatus(createOn: CreateOn, status: StripStatus) {
  return (
    (createOn.inbound && status === 'Inbound') ||
    (createOn.approach && status === 'Approach') ||
    (createOn.departure && status === 'Departure') ||
    (createOn.landing && status === 'Landing') ||
    (createOn.takeoff && status === 'Takeoff') ||
    (createOn.boarding && status === 'Boarding') ||
    (createOn.parked && status === 'Parked') ||
    (createOn.ground && status === 'Ground')
  );
}

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

function newHeader(name: string, id: number): Strip {
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

  const isOurArrival = ourAirspace === aircraft.flight_plan.arriving;
  const isOurDeparture = ourAirspace === aircraft.flight_plan.departing;
  const isOurs = isOurArrival || isOurDeparture;

  const isBoarding = isOurDeparture && aircraft.segment === 'boarding';
  const isParked = isOurDeparture && aircraft.segment === 'parked';
  const isGround =
    (isOurArrival && aircraft.segment === 'taxi-arr') ||
    (isOurDeparture && aircraft.segment === 'taxi-dep');
  const isActive = !!aircraft.flight_time;

  const isTakeoff = aircraft.segment === 'takeoff';
  const isDeparture = aircraft.segment === 'departure';
  const isArriving = aircraft.segment === 'arrival';

  const isLanding = aircraft.segment === 'landing';
  const isApproach = aircraft.segment === 'approach';

  if (isActive) {
    if (isOurs) {
      if (isGround) return 'Ground';
    }

    if (isOurDeparture) {
      if (isBoarding) return 'Boarding';
      if (isParked) return 'Parked';

      if (isTakeoff) return 'Takeoff';
      if (isDeparture) return 'Departure';
    }

    if (isOurArrival) {
      if (isLanding) return 'Landing';
      if (isApproach) return 'Approach';
      if (isArriving) return 'Inbound';
    }

    if (isSelected) return 'Selected';
  }

  return 'None';
}

type StripProps<T extends HeaderStrip | AircraftStrip> = {
  key?: unknown;

  strip: Accessor<T>;
  onmousedown?: () => void;
  onmousemove?: () => void;
  ondelete?: () => void;
  onedit?: (name: string) => void;
  dragged?: Accessor<number | null>;

  deletable?: boolean;
};

function HeaderStripView(props: StripProps<HeaderStrip>) {
  const strip = props.strip;
  const onmousedown = () => props.onmousedown;
  const onmousemove = () => props.onmousemove;
  const ondelete = () => props.ondelete;
  const onedit = () => props.onedit;
  const dragged = () => props.dragged;
  const deletable = () => props.deletable;

  let input!: HTMLInputElement;

  let [editing, setEditing] = createSignal(false);

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
      onedit()?.((e.target as HTMLInputElement).value.trim());
      setEditing(false);
    }
  };

  const handleBlur: JSX.EventHandlerUnion<HTMLInputElement, FocusEvent> = (
    e
  ) => {
    onedit()?.((e.target as HTMLInputElement).value.trim());
    setEditing(false);
  };

  const id = () => strip().id;
  const name = () => strip().name;

  return (
    <div
      classList={{
        strip: true,
        header: true,
        dragged: dragged()?.() === id(),
      }}
      onmousedown={() => onmousedown()?.()}
      onmousemove={() => onmousemove()?.()}
    >
      <div class="vertical">
        <Show when={editing()}>
          <input
            type="text"
            onkeydown={handleKeyDown}
            onblur={handleBlur}
            value={name()}
            ref={input}
          />
        </Show>
        <Show when={!editing()}>
          <span ondblclick={handleDoubleClick}>{name()}</span>
        </Show>
      </div>
      <Show when={deletable()}>
        <div class="vertical end">
          <button class="delete" onclick={() => ondelete()?.()}>
            X
          </button>
        </div>
      </Show>
    </div>
  );
}

function AircraftStripView(props: StripProps<AircraftStrip>) {
  const strip = props.strip;
  const onmousedown = () => props.onmousedown;
  const onmousemove = () => props.onmousemove;
  const ondelete = () => props.ondelete;
  const dragged = () => props.dragged;
  const deletable = () => props.deletable;

  let [ourFrequency] = useAtom(frequencyAtom);
  let [selectedAircraft, setSelectedAircraft] = useAtom(selectedAircraftAtom);

  let [control] = useAtom(controlAtom);
  let [airspace] = useAtom(control().airspace);

  const handleMouseDown = () => {
    setSelectedAircraft(strip().callsign);
    onmousedown()?.();
  };

  let dimmer = createMemo(
    () => strip().frequency !== ourFrequency() || strip().timer.startsWith('-')
  );

  const timer = createMemo(() => {
    // return strip().segment === 'parked' || strip().segment === 'boarding'
    //   ? strip().timer
    //   : smallFlightSegment(strip().segment).toUpperCase();
    return strip().timer;
  });

  const id = () => strip().id;
  const callsign = () => strip().callsign;
  const distance = () => strip().distance;
  const arriving = () => strip().arriving;
  const departing = () => strip().departing;
  const topStatus = () => strip().topStatus;
  const bottomStatus = () => strip().bottomStatus;
  const frequency = () => strip().frequency;
  const selected = () => strip().callsign === selectedAircraft();

  return (
    <div
      classList={{
        strip: true,
        dragged: dragged()?.() === id(),
        theirs: dimmer(),
        selected: selected(),
        departure: airspace() === departing(),
      }}
      onmousedown={handleMouseDown}
      onmousemove={() => onmousemove()?.()}
    >
      <div class="vertical">
        <span class="callsign">{callsign()}</span>
        <span>{distance()}</span>
      </div>
      <div class="vertical">
        <span>{departing()}</span>
        <span>{arriving()}</span>
      </div>
      <div class="vertical">
        <span>{topStatus()}</span>
        <span>{bottomStatus()}</span>
      </div>
      <div class="vertical">
        <span class="frequency">{frequency()}</span>
        <span class="timer">{timer()}</span>
      </div>
      <Show when={deletable()}>
        <div class="vertical end">
          <button
            class="delete"
            onclick={() => ondelete()?.()}
            onmousedown={(e) => e.stopPropagation()}
          >
            X
          </button>
        </div>
      </Show>
    </div>
  );
}

function aircraftToStrip(
  aircraft: Aircraft,
  airspace: Airspace,
  selectedAircraft: string,
  server_ticks: ServerTicks
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
    timer: aircraft.flight_time
      ? formatTime(realTimeTicks(server_ticks, aircraft.flight_time))
      : '--:--',
    status: statusOfAircraft(aircraft, airspace.id, selectedAircraft),
    segment: aircraft.segment,
  };

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

function createStrips() {
  const [_, setNextId] = makePersisted(createSignal(0));
  const [strips, setStrips] = makePersisted(
    createSignal<Array<Signal<Strip>>>([], {
      equals: (a, b) => fastDeepEqual(a, b),
    }),
    {
      serialize: (data) => {
        return JSON.stringify(data.map((s) => s[0]()));
      },
      deserialize: (data) => {
        const strips: Array<Strip> = JSON.parse(data);
        const stripSignals: Array<Signal<Strip>> = strips.map((s) =>
          createSignal(s)
        );

        console.log({ strips });

        return stripSignals;
      },
    }
  );

  function addHeader(name: string) {
    setNextId((nextId) => {
      const id = nextId;
      setStrips((strips) => [...strips, createSignal(newHeader(name, id))]);
      return nextId + 1;
    });
  }

  function addAircraft(strip: AircraftStrip, headerId: number) {
    const index = strips().findIndex((s) => s[0]().id === headerId);
    if (index !== -1) {
      setNextId((nextId) => {
        const id = nextId;
        setStrips((strips) => {
          const before = strips.slice(0, index + 1);
          const after = strips.slice(index + 1);
          const newStrip: Strip = { ...strip, id };
          return [
            ...before,
            createSignal(newStrip),
            ...after,
          ] as Signal<Strip>[];
        });
        return nextId + 1;
      });
    }
  }

  function strip(stripId: number) {
    return strips().find((s) => s[0]().id === stripId);
  }

  function move(fromId: number, toId: number) {
    if (fromId === toId) return;

    const fromStrip = strip(fromId);
    const toStrip = strip(toId);

    if (fromStrip !== undefined && toStrip !== undefined) {
      setStrips((prevStrips) => {
        // fromStrip is captured from the outer scope (const fromStrip = strip(fromId);)
        // It's the actual object reference we want to move.

        // Create a new array without the strip that is being moved.
        const stripsWithoutFrom = prevStrips.filter(
          (s) => s[0]().id !== fromId
        );

        // Insert the fromStrip after the toStrip.
        // flatMap is used here: for each strip, if it's the target (toStrip),
        // output an array of [toStrip, fromStrip], otherwise just [strip].
        // This effectively inserts fromStrip after toStrip.
        const newStrips = stripsWithoutFrom.flatMap((currentStrip) =>
          currentStrip[0]().id === toId
            ? [currentStrip, fromStrip]
            : [currentStrip]
        );

        return newStrips;
      });
    }
  }

  function remove(stripId: number) {
    const index = strips().findIndex((s) => s[0]().id === stripId);
    if (index !== -1) {
      setStrips((strips) => strips.filter((s) => s[0]().id !== stripId));
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
    remove,
    length,
  };
}

export default function StripBoard() {
  const [lenAtDrag, setLenAtDrag] = createSignal<number>(0);
  const [dragged, setDragged] = createSignal<number | null>(null);
  const [separator, setSeparator] = createSignal<number | null>(null);
  const board = createStrips();

  const [showOpts, setShowOpts] = createSignal(false);
  const [createOn, setCreateOn] = makePersisted(
    createSignal<CreateOn>(newCreateOn())
  );

  const aircrafts = createQuery<Aircraft[]>(() => ({
    queryKey: [getAircraft],
    initialData: [],
  }));
  const query = useWorld();
  const [selectedAircraft, setSelectedAircraft] = useAtom(selectedAircraftAtom);

  const serverTicks = usePing();

  // Prefill the strips with default headers.
  createEffect(() => {
    if (board.length() === 0 && aircrafts.data.length > 0) {
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
      .filter((s) => s[0]().type === StripType.Aircraft)
      .map((s) => (s[0]() as AircraftStrip).callsign);

    if (airspace && board.length() > 0) {
      const newStrips: AircraftStrip[] = [];
      for (const aircraft of aircrafts.data) {
        if (
          !existing.includes(aircraft.id) &&
          (aircraft.id === selected ||
            testStatus(
              createOn(),
              statusOfAircraft(aircraft, airspace.id, selected)
            ))
        ) {
          console.log('create:', {
            status: statusOfAircraft(aircraft, airspace.id, selected),
            aircraft: structuredClone(unwrap(aircraft)),
          });
          newStrips.push(
            aircraftToStrip(aircraft, airspace, selected, serverTicks.data!)
          );
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
      let deleteStrips: number[] = [];
      for (const s of board.strips()) {
        const [_, setStrip] = s;
        const strip = s[0]();
        if (strip.type === StripType.Aircraft) {
          const callsign = strip.callsign;
          const aircraft = aircrafts.data.find((a) => a.id === callsign);
          if (aircraft) {
            // updateStrips.push({
            //   ...aircraftToStrip(aircraft, airspace, selected),
            //   id: strip.id,
            // });
            const newStrip: Strip = {
              ...aircraftToStrip(
                aircraft,
                airspace,
                selected,
                serverTicks.data!
              ),
              id: strip.id,
            };
            if (!fastDeepEqual(strip, newStrip)) {
              setStrip(newStrip);
            }
          } else {
            deleteStrips.push(strip.id);
          }
        }
      }

      // Delete strips.
      for (const stripId of deleteStrips) {
        board.remove(stripId);
      }
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
    const s = board.strip(stripId);
    if (s) {
      const strip = s[0]();
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
    if (strip) {
      const s = strip[0]();
      const newStrip = { ...s, name };
      strip[1](newStrip);
    }
  }

  function handleAdd() {
    board.addHeader('New Header');
  }

  function resetDrag() {
    setDragged(null);
    setSeparator(null);
  }

  function toggleCreateOn(prop: keyof CreateOn) {
    setCreateOn((createOn) => {
      createOn[prop] = !createOn[prop];
      return createOn;
    });
  }

  function handleClear() {
    board.setStrips((strips) =>
      strips.filter((s) => {
        const strip = s[0]();
        return strip.type === StripType.Aircraft
          ? testStatus(createOn(), strip.status)
          : true;
      })
    );
  }

  const allYours = createMemo(
    () =>
      board.strips().filter((s) => {
        const strip = s[0]();
        return strip.type === StripType.Aircraft;
      }).length
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
      <div class="row top-buttons">
        <span>
          Yours: {allYours()} of {allFlying()}
        </span>
        <div class="row">
          <button onclick={handleAdd}>Add</button>
          <button onclick={() => setShowOpts((x) => !x)}>Opts</button>
          <button class="delete" onclick={handleClear}>
            Clr
          </button>
        </div>
      </div>
      <Show when={showOpts()}>
        <For each={Object.entries(createOn())}>
          {(item) => (
            <label>
              {item[0]}{' '}
              <input
                type="checkbox"
                checked={item[1]}
                onclick={() => toggleCreateOn(item[0] as keyof CreateOn)}
              />
            </label>
          )}
        </For>
      </Show>
      <For each={board.strips()}>
        {(s) => {
          const strip = s[0];

          return (
            <>
              {strip().type === StripType.Header ? (
                <HeaderStripView
                  strip={strip as Accessor<HeaderStrip>}
                  onmousedown={() => handleMouseDown(strip().id)}
                  onmousemove={() => handleMouseMove(strip().id)}
                  ondelete={() => handleDelete(strip().id)}
                  onedit={(name) => handleEdit(strip().id, name)}
                  dragged={() => (separator() !== null ? dragged() : null)}
                  deletable={strip().id !== INBOX_ID}
                />
              ) : (
                <AircraftStripView
                  strip={strip as Accessor<AircraftStrip>}
                  onmousedown={() => handleMouseDown(strip().id)}
                  onmousemove={() => handleMouseMove(strip().id)}
                  ondelete={() => handleDelete(strip().id)}
                  onedit={(name) => handleEdit(strip().id, name)}
                  dragged={() => (separator() !== null ? dragged() : null)}
                  deletable={strip().id !== INBOX_ID}
                />
              )}
              {strip().id === separator() ? <Separator /> : null}
            </>
          );
        }}
      </For>
    </div>
  );
}
