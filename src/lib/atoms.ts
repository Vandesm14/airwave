import { atom } from 'solid-jotai';
import { Aircraft, RadioMessage, Runway } from './types';
import { createStore } from 'solid-js/store';

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
export let airspaceSizeAtom = atom(1000);
export let runwaysAtom = atom<Array<Runway>>([]);
export let frequencyAtom = atom(118.5);

export let renderAtom = atom({
  lastTime: Date.now(),
  lastDraw: 0,
});

export let messagesAtom = atom<Array<RadioMessage>>([]);
