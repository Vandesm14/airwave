import { Accessor, createEffect, createSignal, For } from 'solid-js';
import { Aircraft } from './lib/types';
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

export default function StripBoard({
  aircrafts,
}: {
  aircrafts: Accessor<Array<Aircraft>>;
}) {
  let [dragged, setDragged] = createSignal<string | null>(null);
  let [separator, setSeparator] = createSignal<SeparatorType | null>(null);
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

  return (
    <div id="stripboard">
      {strips().map((s) => (
        <Strip strip={s}></Strip>
      ))}
    </div>
  );
}
