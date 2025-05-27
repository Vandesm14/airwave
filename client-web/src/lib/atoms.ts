import { atom } from 'solid-jotai';
import { DEFAULT_AIRPORT } from './lib';
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
export const radarAtom = atom<RadarConfig>({
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

export const isRecordingAtom = atom(false);
export const useTTSAtomKey = 'use-tts';
export const useTTSAtom = atom(false);
export const frequencyAtomKey = 'frequency';
export const frequencyAtom = atom(118.6);

export const airportAtomKey = 'airport';
export const airportAtom = atom<string>(DEFAULT_AIRPORT);

export const renderAtom = atom<{
  doInitialDraw: boolean;
  lastDraw: number;
}>({
  doInitialDraw: true,
  lastDraw: 0,
});

export const selectedAircraftAtom = atom<string>('');
