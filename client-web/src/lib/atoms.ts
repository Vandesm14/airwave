import { atom } from 'solid-jotai';
import { RadioMessage, World } from './types';

type RadarConfig = {
  scale: number;
  isDragging: boolean;
  isZooming: boolean;
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
  isZooming: false,
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
});
export let frequencyAtom = atom(118.5);

export let renderAtom = atom({
  lastTime: Date.now(),
  lastDraw: 0,
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
