import { useAtom } from 'solid-jotai';
import { baseAPIPath, frequencyAtom } from './lib/atoms';
import { createEffect, createSignal, onMount } from 'solid-js';
import { makePersisted } from '@solid-primitives/storage';
import { Frequencies, World } from './lib/types';
import { createQuery } from '@tanstack/solid-query';

export default function FreqSelector() {
  let [frequency, setFrequency] = useAtom(frequencyAtom);
  let [secondary, setSecondary] = makePersisted(createSignal(frequency()));
  let [key, setKey] = createSignal<keyof Frequencies>('approach');
  const query = createQuery<World>(() => ({
    queryKey: ['/api/world'],
    queryFn: async () => {
      const result = await fetch(`${baseAPIPath}/world`);
      if (!result.ok) return undefined;
      return result.json();
    },
    staleTime: Infinity,
    refetchOnReconnect: 'always',
    throwOnError: true, // Throw an error if the query fails
  }));

  function updateKeyByFreqChange() {
    const found = query.data?.airspace;
    if (!found) {
      return;
    }

    let newKey = Object.entries(found.frequencies).find(
      ([, v]) => v === frequency()
    ) as [keyof Frequencies, number];

    if (newKey) {
      setKey(newKey[0]);
    }
  }

  createEffect(() => updateKeyByFreqChange());

  function changeViaKey(key: keyof Frequencies) {
    setKey(key as keyof Frequencies);

    const found = query.data?.airspace;
    if (!found) {
      return;
    }

    if (found?.frequencies && key in found?.frequencies) {
      setFrequency(found?.frequencies[key]);
    }
  }

  function changeViaValue(value: number) {
    setFrequency(value);

    const found = query.data?.airspace;
    if (!found) {
      return;
    }

    if (found.frequencies) {
      // TODO: Remove uses of as keyof Frequencies.
      setKey(
        Object.keys(found.frequencies).find(
          (k) => found.frequencies[k as keyof Frequencies] === value
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
        <select
          name="frequency"
          onchange={(e) => changeViaKey(e.target.value as keyof Frequencies)}
          value={key()}
        >
          {query.data?.airspace?.frequencies
            ? // TODO: Remove uses of as keyof Frequencies.
              Object.entries(query.data?.airspace!.frequencies).map(
                ([k, v]) => (
                  <option value={k}>
                    {k} - {v}
                  </option>
                )
              )
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
