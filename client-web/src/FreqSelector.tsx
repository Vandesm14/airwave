import { useAtom } from 'solid-jotai';
import { controlAtom, frequencyAtom, worldAtom } from './lib/atoms';
import { createMemo, createSignal, onMount } from 'solid-js';
import { makePersisted } from '@solid-primitives/storage';

export default function FreqSelector() {
  let [frequency, setFrequency] = useAtom(frequencyAtom);
  let [world] = useAtom(worldAtom);
  let [secondary, setSecondary] = makePersisted(createSignal(frequency()));

  let [control] = useAtom(controlAtom);
  let [airspace] = useAtom(control().airspace);

  let foundAirspace = createMemo(() =>
    world().airspaces.find((a) => a.id === airspace())
  );

  function swap() {
    let swapFreq = frequency();
    setFrequency(secondary());
    setSecondary(swapFreq);
  }

  function oninput(e: InputEvent) {
    if (e.target instanceof HTMLInputElement) {
      setSecondary(parseFloat(e.target.value));
    }
  }

  function onEnter(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      swap();
    }
  }

  function onBackslash(e: KeyboardEvent) {
    if (e.key === '\\') {
      swap();
    }
  }

  onMount(() => {
    document.addEventListener('keydown', onBackslash);
  });

  return (
    <div id="freq-selector">
      <input type="number" disabled value={frequency()} class="live" />
      <input type="button" value="⬌" class="swap" onclick={swap} />
      <select
        name="frequency"
        id=""
        onchange={(e) => setSecondary(parseFloat(e.target.value))}
      >
        {foundAirspace()?.frequencies
          ? Object.entries(foundAirspace()?.frequencies).map(([key, value]) => (
              <option value={value}>{key}</option>
            ))
          : null}
      </select>
    </div>
  );
}
