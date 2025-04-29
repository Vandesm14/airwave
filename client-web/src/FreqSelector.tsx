import { useAtom } from 'solid-jotai';
import { frequencyAtom } from './lib/atoms';
import { Accessor, createSignal, For, Index, Setter } from 'solid-js';
import { makePersisted } from '@solid-primitives/storage';

type FreqRowProps = {
  freq: number;
  setFreq: (freq: number) => void;
};

function FreqRow({ freq, setFreq }: FreqRowProps) {
  return (
    <div class="row">
      <input
        type="number"
        value={freq}
        class="live"
        onchange={(e) => setFreq(parseFloat(e.target.value))}
        step=".1"
      />
    </div>
  );
}

export default function FreqSelector() {
  const [frequency, setFrequency] = useAtom(frequencyAtom);
  const [selected, setSelected] = createSignal(0);
  const [slots, setSlots] = makePersisted(
    createSignal<number[]>([
      118.5, 118.5, 118.5, 118.5, 118.5, 118.5, 118.5, 118.5, 118.5, 118.5,
    ])
  );
  const [count, setCount] = makePersisted(createSignal(2));

  return (
    <div id="freq-selector">
      <div class="row">
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
      <Index each={slots().slice(0, count())}>
        {(slot, index) => (
          <FreqRow
            freq={slot()}
            setFreq={(freq) =>
              setSlots((slots) => slots.map((s, i) => (i === index ? freq : s)))
            }
          />
        )}
      </Index>
    </div>
  );
}
