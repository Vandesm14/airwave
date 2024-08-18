import { useAtom } from 'solid-jotai';
import { radarAtom } from './lib/atoms';
import { createEffect, createMemo } from 'solid-js';

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
