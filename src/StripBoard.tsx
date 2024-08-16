import { Accessor, createEffect, createSignal, For } from 'solid-js';
import { Aircraft } from './lib/types';

type StripType =
  | {
      type: 'header';
      value: string;
    }
  | { type: 'strip'; value: Aircraft };

const Separator = () => <div class="separator"></div>;

type StripProps = {
  strip: StripType;
  onmousedown: () => void;
  onmouseup: () => void;
  onmousemove: () => void;
};

function Strip({ strip, onmousedown, onmouseup, onmousemove }: StripProps) {
  let [target, setTarget] = createSignal('');

  if (strip.type === 'strip') {
    if (strip.value.state.type === 'landing') {
      setTarget(`RW${strip.value.state.value.id}`);
    } else if (strip.value.state.type === 'takeoff') {
      setTarget(`RW${strip.value.state.value.id}`);
    }
  }

  if (strip.type === 'strip') {
    return (
      <div
        classList={{ strip: true }}
        onmousedown={onmousedown}
        onmouseup={onmouseup}
        onmousemove={onmousemove}
      >
        <span class="callsign">{strip.value.callsign}</span>
        <span class="target"> {target()}</span>
      </div>
    );
  } else if (strip.type === 'header') {
    return (
      <div
        classList={{ header: true }}
        onmousedown={onmousedown}
        onmouseup={onmouseup}
        onmousemove={onmousemove}
      >
        {strip.value}
      </div>
    );
  }
}

export default function StripBoard({
  aircrafts,
}: {
  aircrafts: Accessor<Array<Aircraft>>;
}) {
  let [dragged, setDragged] = createSignal<string | null>(null);
  let [separator, setSeparator] = createSignal<number | null>(null);
  let [strips, setStrips] = createSignal<Array<StripType>>(
    [
      { type: 'header', value: 'Approach' },
      { type: 'header', value: 'Landing' },
      { type: 'header', value: 'Takeoff' },
      { type: 'header', value: 'Departure' },
    ],
    { equals: false }
  );

  createEffect(() => {
    let allsigns = new Set(aircrafts().map((s) => s.callsign));
    setStrips((state) => {
      return state.filter((s) => {
        if (s.type === 'header') {
          return true;
        } else if (s.type === 'strip') {
          return allsigns.has(s.value.callsign);
        }
      });
    });

    for (let aircraft of aircrafts()) {
      setStrips((state) => {
        let index = state.findIndex(
          (s) => s.type === 'strip' && s.value.callsign === aircraft.callsign
        );

        if (index === -1) {
          return [
            state[0],
            { type: 'strip', value: aircraft },
            ...state.slice(1),
          ];
        } else {
          return state.map((e, i) =>
            i === index ? { type: 'strip', value: aircraft } : e
          );
        }
      });
    }
  });

  function handleMouseDown(callsign: string) {
    setDragged(callsign);
  }

  function handleMouseUp() {
    setStrips((strips) => {
      let callsign = dragged();
      let fromIndex = strips.findIndex(
        (s) => s.type === 'strip' && s.value.callsign === callsign
      );
      let toIndex = separator();

      if (fromIndex !== -1) {
        let newStrips = [];
        for (let i = 0; i < strips.length; i++) {
          if (i !== fromIndex) {
            newStrips.push(strips[i]);
          }

          if (i === toIndex) {
            newStrips.push(strips[fromIndex]);
          }
        }

        return newStrips;
      } else {
        return strips;
      }
    });

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

  return (
    <div id="stripboard" onmouseleave={() => resetDrag()}>
      {strips().map((s, i) => (
        <>
          <Strip
            strip={s}
            onmousedown={() => {
              s.type === 'strip' ? handleMouseDown(s.value.callsign) : {};
            }}
            onmouseup={() => {
              handleMouseUp();
            }}
            onmousemove={() => {
              handleMouseMove(i);
            }}
          ></Strip>
          {i === separator() ? <Separator /> : null}
        </>
      ))}
    </div>
  );
}
