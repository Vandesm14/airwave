import { atom } from 'solid-jotai';
import { Aircraft, RadioMessage, World } from './types';

type RadarConfig = {
  scale: number;
  isDragging: boolean;
  shiftPoint: {
    x: number;
    y: number;
  };
  lastShiftPoint: {
    x: number;
    y: number;
  };
  dragStartPoint: {
    x: number;
    y: number;
  };
};
export let radarAtom = atom<RadarConfig>({
  scale: 1.0,
  isDragging: false,
  shiftPoint: {
    x: 0,
    y: 0,
  },
  lastShiftPoint: {
    x: 0,
    y: 0,
  },
  dragStartPoint: {
    x: 0,
    y: 0,
  },
});

export let isRecordingAtom = atom(false);
export let worldAtom = atom<World>({
  airspaces: [],
  waypoints: [],
});
export let frequencyAtom = atom(118.5);
export let controlAtom = atom({
  airspace: atom('KSFO'),
  frequency: atom(118.5),
});

export let renderAtom = atom<{
  doInitialDraw: boolean;
  lastDraw: number;
  aircrafts: Array<Aircraft>;
}>({
  doInitialDraw: true,
  lastDraw: 0,
  aircrafts: [],
});

function initMessages(): Array<RadioMessage> {
  let json = localStorage.getItem('messages');
  if (json) {
    return JSON.parse(json);
  } else {
    return [];
  }
}
export let messagesAtom = atom<Array<RadioMessage>>(initMessages());
export let selectedAircraftAtom = atom<string>('');
