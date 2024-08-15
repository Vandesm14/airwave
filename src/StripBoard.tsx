import { createEffect, createMemo, createSignal } from 'solid-js';
import { Aircraft } from './lib/types';
import { useAtom } from 'solid-jotai';
import { aircraftsAtom } from './lib/atoms';

type StripType = {
  group: keyof Sections;
  aircraft: Aircraft;
};

type Sections = {
  approach: Array<StripType>;
  landing: Array<StripType>;
  takeoff: Array<StripType>;
  departure: Array<StripType>;
};

type State = Array<{
  group: keyof Sections;
  aircraft: Aircraft;
}>;

type StripProps = {
  strip: StripType;
  isHovering: (position: 'above' | 'below', callsign: string) => void;
  onmousedown: (callsign: string) => void;
  onmouseup: () => void;
  classList?: Record<string, boolean>;
};

function Strip({
  strip: { aircraft },
  isHovering,
  onmousedown,
  onmouseup,
  classList,
}: StripProps) {
  let div;
  let target = '';

  if (aircraft.state.type === 'landing') {
    target = aircraft.state.value;
  } else if (aircraft.state.type === 'takeoff') {
    target = aircraft.state.value;
  }

  function onmousemove(e: MouseEvent) {
    if (div instanceof HTMLDivElement) {
      let delta = (div.offsetHeight - e.offsetY) / div.offsetHeight;
      if (delta > 0.5) {
        isHovering('above', aircraft.callsign);
      } else {
        isHovering('below', aircraft.callsign);
      }
    }
  }

  return (
    <div
      onmousemove={onmousemove}
      onmousedown={() => onmousedown(aircraft.callsign)}
      onmouseup={onmouseup}
      ref={div}
      classList={{ ...classList, strip: true }}
    >
      <span class="callsign">{aircraft.callsign}</span>
      <span class="target">{target}</span>
    </div>
  );
}

export default function StripBoard() {
  let [aircrafts] = useAtom(aircraftsAtom);
  let [state, setState] = createSignal<State>([]);
  let [isDragging, setIsDragging] = createSignal<string | null>(null);
  let [separator, setSeparator] = createSignal<string | null>(null);

  let sections = createMemo<Sections>(() => {
    return {
      approach: state().filter((s) => s.group === 'approach'),
      landing: state().filter((s) => s.group === 'landing'),
      takeoff: state().filter((s) => s.group === 'takeoff'),
      departure: state().filter((s) => s.group === 'departure'),
    };
  });

  createEffect(() => {
    for (let aircraft of aircrafts()) {
      let has = state().some((s) => s.aircraft.callsign === aircraft.callsign);
      if (!has) {
        setState((state) => {
          return [{ group: 'approach', aircraft }, ...state];
        });
      }
    }
  });

  function isHovering(position: 'above' | 'below', callsign: string) {
    if (isDragging()) {
      console.log({ position, callsign });
      setSeparator(callsign);
    }
  }

  function onmousedown(callsign: string) {
    setIsDragging(callsign);
  }

  function onmouseup() {
    setIsDragging(null);
    setSeparator(null);
  }

  return (
    <div id="stripboard">
      <div class="header">Approach</div>
      {sections().approach.map((s) => (
        <Strip
          strip={s}
          isHovering={isHovering}
          onmousedown={onmousedown}
          onmouseup={onmouseup}
          classList={{ hover: separator() === s.aircraft.callsign }}
        ></Strip>
      ))}
      <div class="header">Landing</div>
      {sections().landing.map((s) => (
        <Strip
          strip={s}
          isHovering={isHovering}
          onmousedown={onmousedown}
          onmouseup={onmouseup}
          classList={{ hover: separator() === s.aircraft.callsign }}
        ></Strip>
      ))}
      <div class="header">Takeoff</div>
      {sections().takeoff.map((s) => (
        <Strip
          strip={s}
          isHovering={isHovering}
          onmousedown={onmousedown}
          onmouseup={onmouseup}
          classList={{ hover: separator() === s.aircraft.callsign }}
        ></Strip>
      ))}
      <div class="header">Departure</div>
      {sections().departure.map((s) => (
        <Strip
          strip={s}
          isHovering={isHovering}
          onmousedown={onmousedown}
          onmouseup={onmouseup}
          classList={{ hover: separator() === s.aircraft.callsign }}
        ></Strip>
      ))}
    </div>
  );
}
