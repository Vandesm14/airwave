import { Accessor, createEffect, createMemo, createSignal } from 'solid-js';
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

type SeparatorType = { position: 'above' | 'below'; callsign: string };
type StripProps = {
  strip: StripType;
  isHovering: (separator: SeparatorType) => void;
  onmousedown: (callsign: string) => void;
  onmouseup: () => void;
  dragged: Accessor<string>;
};

const Separator = () => <div class="separator"></div>;

function Strip({
  strip: { aircraft },
  isHovering,
  onmousedown,
  onmouseup,
  dragged,
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
        isHovering({ position: 'above', callsign: aircraft.callsign });
      } else {
        isHovering({ position: 'below', callsign: aircraft.callsign });
      }
    }
  }

  return (
    <div
      classList={{ strip: true, dragged: dragged() === aircraft.callsign }}
      onmousemove={onmousemove}
      onmousedown={() => onmousedown(aircraft.callsign)}
      onmouseup={onmouseup}
      ref={div}
    >
      <span class="callsign">{aircraft.callsign}</span>
      <span class="target">{target}</span>
    </div>
  );
}

export default function StripBoard() {
  let [aircrafts] = useAtom(aircraftsAtom);
  let [state, setState] = createSignal<State>([]);
  let [dragged, setDragged] = createSignal<string | null>(null);
  let [separator, setSeparator] = createSignal<SeparatorType | null>(null);

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

  function isHovering(separator: SeparatorType) {
    if (dragged()) {
      setSeparator(separator);
    }
  }

  function onmousedown(callsign: string) {
    setDragged(callsign);
  }

  function onmouseup() {
    setDragged(null);
    setSeparator(null);
  }

  const spawnSeparator = (
    strip: StripType,
    position: SeparatorType['position']
  ) => {
    return separator() !== null &&
      separator().callsign === strip.aircraft.callsign &&
      separator().position === position
      ? Separator
      : null;
  };

  return (
    <div id="stripboard">
      <div class="header">Approach</div>
      {sections().approach.map((s) => (
        <>
          {spawnSeparator(s, 'above')}
          <Strip
            strip={s}
            isHovering={isHovering}
            onmousedown={onmousedown}
            onmouseup={onmouseup}
            dragged={dragged}
          ></Strip>
          {spawnSeparator(s, 'below')}
        </>
      ))}
      <div class="header">Landing</div>
      {sections().landing.map((s) => (
        <>
          {spawnSeparator(s, 'above')}
          <Strip
            strip={s}
            isHovering={isHovering}
            onmousedown={onmousedown}
            onmouseup={onmouseup}
            dragged={dragged}
          ></Strip>
          {spawnSeparator(s, 'below')}
        </>
      ))}
      <div class="header">Takeoff</div>
      {sections().takeoff.map((s) => (
        <>
          {spawnSeparator(s, 'above')}
          <Strip
            strip={s}
            isHovering={isHovering}
            onmousedown={onmousedown}
            onmouseup={onmouseup}
            dragged={dragged}
          ></Strip>
          {spawnSeparator(s, 'below')}
        </>
      ))}
      <div class="header">Departure</div>
      {sections().departure.map((s) => (
        <>
          {spawnSeparator(s, 'above')}
          <Strip
            strip={s}
            isHovering={isHovering}
            onmousedown={onmousedown}
            onmouseup={onmouseup}
            dragged={dragged}
          ></Strip>
          {spawnSeparator(s, 'below')}
        </>
      ))}
    </div>
  );
}
