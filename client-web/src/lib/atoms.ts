import { atom } from 'solid-jotai';
import { RadioMessage, Runway, Taxiway, Terminal } from './types';
import { nauticalMilesToFeet } from './lib';

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
  mode: 'tower' | 'ground';
};
export const initialRadarScale = 0.004;
export let radarAtom = atom<RadarConfig>({
  scale: initialRadarScale,
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
  mode: 'tower',
});

export let isRecordingAtom = atom(false);
export let airspaceSizeAtom = atom(nauticalMilesToFeet * 1000);
export let runwaysAtom = atom<Array<Runway>>([]);
export let taxiwaysAtom = atom<Array<Taxiway>>([]);
export let terminalsAtom = atom<Array<Terminal>>([]);
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
