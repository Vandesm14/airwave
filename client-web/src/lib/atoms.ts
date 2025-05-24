import { atom } from 'solid-jotai';
import { HARD_CODED_AIRPORT } from './lib';
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
export let useTTSAtom = atom(false);
export let frequencyAtom = atom(118.5);

export let airportAtom = atom<string>(HARD_CODED_AIRPORT);

export let renderAtom = atom<{
  doInitialDraw: boolean;
  lastDraw: number;
}>({
  doInitialDraw: true,
  lastDraw: 0,
});

export let selectedAircraftAtom = atom<string>('');
