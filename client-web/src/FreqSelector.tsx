import { useAtom } from 'solid-jotai';
import { controlAtom, frequencyAtom, worldAtom } from './lib/atoms';
import { createMemo, createSignal, onMount } from 'solid-js';
import { makePersisted } from '@solid-primitives/storage';
import { Frequencies } from './lib/types';

export default function FreqSelector() {
  let [frequency, setFrequency] = useAtom(frequencyAtom);
  let [world] = useAtom(worldAtom);
  let [secondary, setSecondary] = makePersisted(createSignal(frequency()));
  let [key, setKey] = createSignal<keyof Frequencies>('approach');

  let foundAirspace = createMemo(() => world().airspace);

  function changeViaKey(key: string) {
    setKey(key as keyof Frequencies);

    if (foundAirspace()?.frequencies && key in foundAirspace()?.frequencies) {
      setFrequency(foundAirspace()?.frequencies[key]);
    }
  }

  function changeViaValue(value: number) {
    setFrequency(value);
    if (foundAirspace()?.frequencies) {
      setKey(
        Object.keys(foundAirspace()?.frequencies).find(
          (k) => foundAirspace()?.frequencies[k] === value
        ) as keyof Frequencies
      );
    }
  }

  function swap() {
    let swapFreq = frequency();
    changeViaValue(secondary());
    setSecondary(swapFreq);
  }

  function oninput(e: InputEvent) {
    if (e.target instanceof HTMLInputElement) {
      changeViaValue(parseFloat(e.target.value));
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
      <div class="row">
        <select name="frequency" onchange={(e) => changeViaKey(e.target.value)}>
          {foundAirspace()?.frequencies
            ? Object.entries(foundAirspace()?.frequencies).map(([k, v]) => (
                <option value={k} selected={k === key()}>
                  {k} - {v}
                </option>
              ))
            : null}
        </select>
        <input
          type="number"
          value={frequency()}
          class="live"
          oninput={oninput}
          step=".1"
        />
      </div>
      <div class="row">
        <input type="button" value="Swap" onClick={swap} />
        <input
          type="number"
          value={secondary()}
          oninput={(e) => setSecondary(parseFloat(e.target.value))}
          step=".1"
        />
      </div>
    </div>
  );
}
