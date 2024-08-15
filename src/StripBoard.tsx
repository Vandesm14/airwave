import { createEffect, createSignal, For } from 'solid-js';
import { Aircraft } from './lib/types';
import { useAtom } from 'solid-jotai';
import { gameStore } from './lib/atoms';
import { createStore } from 'solid-js/store';

type StripType =
  | {
      type: 'header';
      value: string;
    }
  | { type: 'strip'; value: Aircraft };

type SeparatorType = { position: 'above' | 'below'; callsign: string };
type StripProps = {
  strip: StripType;
};

const Separator = () => <div class="separator"></div>;

function Strip({ strip }: StripProps) {
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
      <div classList={{ strip: true }}>
        <span class="callsign">{strip.value.callsign}</span>
        <span class="target"> {target()}</span>
      </div>
    );
  } else if (strip.type === 'header') {
    return <div classList={{ header: true }}>{strip.value}</div>;
  }
}

export default function StripBoard() {
  let [game] = gameStore;
  let [dragged, setDragged] = createSignal<string | null>(null);
  let [separator, setSeparator] = createSignal<SeparatorType | null>(null);
  let [strips, setStrips] = createStore<{ inner: Array<StripType> }>({
    inner: [
      { type: 'header', value: 'Approach' },
      { type: 'header', value: 'Landing' },
      { type: 'header', value: 'Takeoff' },
      { type: 'header', value: 'Departure' },
    ],
  });

  createEffect(() => {
    for (let aircraft of game.aircrafts) {
      let index = strips.inner.findIndex(
        (s) => s.type === 'strip' && s.value.callsign === aircraft.callsign
      );
      if (index === -1) {
        setStrips((state) => {
          return {
            inner: [
              state.inner[0],
              { type: 'strip', value: aircraft },
              ...state.inner.slice(1),
            ],
          };
        });
      } else {
        setStrips((state) => {
          state.inner[index].value = aircraft;
          return state;
        });
      }
    }
  });

  return (
    <div id="stripboard">
      {strips.inner.map((s) => (
        <Strip strip={s}></Strip>
      ))}
    </div>
  );
}
