import { useAtom } from 'solid-jotai';
import { frequencyAtom } from './lib/atoms';
import { createSignal, onMount } from 'solid-js';
import { makePersisted } from '@solid-primitives/storage';

export default function FreqSelector() {
  let [frequency, setFrequency] = useAtom(frequencyAtom);
  let [secondary, setSecondary] = makePersisted(createSignal(frequency()));

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
      <input
        type="number"
        oninput={oninput}
        onkeydown={onEnter}
        value={secondary()}
        class="standby"
        step=".1"
      />
    </div>
  );
}
