import { useAtom } from 'solid-jotai';
import { controlAtom, frequencyAtom, worldAtom } from './lib/atoms';
import { createEffect, createMemo, createSignal, onMount } from 'solid-js';
import { makePersisted } from '@solid-primitives/storage';
import { Frequencies } from './lib/types';

export default function FreqSelector() {
  let [frequency, setFrequency] = useAtom(frequencyAtom);
  let [world] = useAtom(worldAtom);
  let [secondary, setSecondary] = makePersisted(createSignal(frequency()));

  let [control] = useAtom(controlAtom);
  let [airspace] = useAtom(control().airspace);
  let [key, setKey] = createSignal<keyof Frequencies>('approach');

  let foundAirspace = createMemo(() =>
    world().airspaces.find((a) => a.id === airspace())
  );

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
    // let swapFreq = frequency();
    // setFrequency(secondary());
    // setSecondary(swapFreq);
  }

  function oninput(e: InputEvent) {
    if (e.target instanceof HTMLInputElement) {
      // setSecondary(parseFloat(e.target.value));
      changeViaValue(parseFloat(e.target.value));
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
      <select
        name="frequency"
        id=""
        onchange={(e) => changeViaKey(e.target.value)}
      >
        {foundAirspace()?.frequencies
          ? Object.keys(foundAirspace()?.frequencies).map((k) => (
              <option value={k} selected={k === key()}>
                {k}
              </option>
            ))
          : null}
      </select>
      <input
        type="number"
        value={frequency()}
        class="live"
        oninput={oninput}
        onkeydown={onEnter}
        step=".1"
      />
      {/* <input type="button" value="⬌" class="swap" onclick={swap} /> */}
    </div>
  );
}
