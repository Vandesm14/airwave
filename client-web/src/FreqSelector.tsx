import { useAtom } from 'solid-jotai';
import { frequencyAtom } from './lib/atoms';
import {
  Accessor,
  createEffect,
  createMemo,
  createSignal,
  Index,
  onMount,
} from 'solid-js';
import { makePersisted } from '@solid-primitives/storage';
import useGlobalShortcuts from './lib/hooks';

import './FreqSelector.scss';
import { useWorld } from './lib/api';
import { hardcodedAirport } from './lib/lib';

type FreqRowProps = {
  index: number;
  freq: Accessor<number>;
  setFreq: (freq: number) => void;
  selected: Accessor<boolean>;
};

function FreqRow({ index, freq, setFreq, selected }: FreqRowProps) {
  const query = useWorld();

  return (
    <div
      classList={{
        row: true,
        selected: selected(),
      }}
    >
      <span>{index + 1}</span>
      <input
        type="number"
        value={freq()}
        class="live"
        onchange={(e) => setFreq(parseFloat(e.target.value))}
        step=".1"
      />
      <select
        name="frequency"
        onchange={(e) => {
          setFreq(parseFloat(e.target.value));
        }}
      >
        <option value={118.5}></option>
        {query.data && hardcodedAirport(query.data)?.frequencies
          ? // TODO: Remove uses of as keyof Frequencies.
            Object.entries(
              query.data && hardcodedAirport(query.data)!.frequencies
            ).map(([k, v]) => (
              <option value={v}>
                {k} - {v}
              </option>
            ))
          : null}
      </select>
    </div>
  );
}

export default function FreqSelector() {
  const [frequency, setFrequency] = useAtom(frequencyAtom);
  const [selected, setSelected] = createSignal(-1);
  const [slots, setSlots] = makePersisted(
    createSignal<number[]>(Array(10).fill(118.6))
  );
  const [count, setCount] = makePersisted(createSignal(2));

  onMount(() => {
    setSelected(0);
  });

  useGlobalShortcuts((e) => {
    if (e.key === '1') {
      setSelected(0);
    } else if (e.key === '2') {
      setSelected(1);
    } else if (e.key === '3') {
      setSelected(2);
    } else if (e.key === '4') {
      setSelected(3);
    } else if (e.key === '5') {
      setSelected(4);
    } else if (e.key === '6') {
      setSelected(5);
    } else if (e.key === '7') {
      setSelected(6);
    } else if (e.key === '8') {
      setSelected(7);
    } else if (e.key === '9') {
      setSelected(8);
    } else if (e.key === '0') {
      setSelected(9);
    }
  });

  createEffect(() => {
    const freq = slots()[selected()];
    if (freq) {
      console.log({
        freq,
        selected: selected(),
      });
      setFrequency(freq);
    }
  });

  const minSlots = createMemo(() => slots().slice(0, count()));

  return (
    <div id="freq-selector">
      <div class="row">
        <span class="frequency">{frequency()}</span>
      </div>
      <div class="row">
        Slots
        <input
          type="number"
          value={count()}
          class="live"
          onchange={(e) => setCount(parseInt(e.target.value))}
          step="1"
          min="1"
          max="10"
        />
      </div>
      <Index each={minSlots()}>
        {(slot, index) => (
          <FreqRow
            index={index}
            freq={slot}
            setFreq={(freq) =>
              setSlots((slots) => slots.map((s, i) => (i === index ? freq : s)))
            }
            selected={() => selected() === index}
          />
        )}
      </Index>
    </div>
  );
}
