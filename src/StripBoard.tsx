import { createSignal } from 'solid-js';
import { Aircraft } from './lib/types';

type State = {
  approach: Array<Aircraft>;
  landing: Array<Aircraft>;
  takeoff: Array<Aircraft>;
  departure: Array<Aircraft>;
};

export default function StripBoard() {
  let [state, setState] = createSignal({});
}
