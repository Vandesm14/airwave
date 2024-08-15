import { atom } from 'solid-jotai';
import { Aircraft, RadioMessage, Runway } from './types';

export let radarAtom = atom({
  scale: 1,
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
export let airspaceSizeAton = atom(1000);

export let aircraftsAtom = atom<Array<Aircraft>>([]);
export let runwaysAtom = atom<Array<Runway>>([]);

export let renderAtom = atom({
  lastTime: Date.now(),
  lastDraw: 0,
});

export let messagesAtom = atom<Array<RadioMessage>>([]);
