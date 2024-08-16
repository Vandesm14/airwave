import { useAtom } from 'solid-jotai';
import { frequencyAtom } from './lib/atoms';
import { createSignal } from 'solid-js';

export default function FreqSelector() {
  let [frequency, setFrequency] = useAtom(frequencyAtom);
  let [secondary, setSecondary] = createSignal(frequency());

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

  return (
    <div id="freq-selector">
      <input type="number" disabled value={frequency()} />
      <input type="button" value="⬌" class="swap" onclick={swap} />
      <input type="number" oninput={oninput} value={secondary()} step=".1" />
    </div>
  );
}
