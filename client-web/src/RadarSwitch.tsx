import { useAtom } from 'solid-jotai';
import { radarAtom } from './lib/atoms';
import { createEffect, createMemo, onMount } from 'solid-js';

export default function RadarSwitch() {
  let [radar, setRadar] = useAtom(radarAtom);
  let isTower = createMemo(() => {
    return radar().mode === 'tower';
  });
  let isGround = createMemo(() => {
    return radar().mode === 'ground';
  });

  function setTower() {
    setRadar((radar) => {
      radar.mode = 'tower';
      return { ...radar };
    });
  }

  function setGround() {
    setRadar((radar) => {
      radar.mode = 'ground';
      return { ...radar };
    });
  }

  function swap() {
    let swapRadar = radar();
    setRadar((radar) => {
      radar.mode = swapRadar.mode === 'tower' ? 'ground' : 'tower';
      return { ...radar };
    });
  }

  onMount(() => {
    document.addEventListener('keydown', (e) => {
      if (e.key === 'PageUp') {
        swap();
      }
    });
  });

  return (
    <div id="radar-tabs">
      <button classList={{ selected: isTower() }} onClick={setTower}>
        Tower
      </button>
      <button classList={{ selected: isGround() }} onClick={setGround}>
        Ground
      </button>
    </div>
  );
}
