import { World } from '../../bindings/World';

type Duration = {
  secs: number;
  nanos: number;
};

export function newDuration(secs: number, nanos: number): Duration {
  return { secs, nanos };
}

export type FlightSegment =
  | 'parked'
  | 'taxi-dep'
  | 'takeoff'
  | 'departure'
  | 'cruise'
  | 'arrival'
  | 'approach'
  | 'land'
  | 'touchdown'
  | 'taxi-arr';

export function smallFlightSegment(segment: FlightSegment): string {
  switch (segment) {
    case 'parked':
      return 'park';
    case 'taxi-dep':
      return 'txid';
    case 'takeoff':
      return 'tkff';
    case 'departure':
      return 'depr';
    case 'cruise':
      return 'cruz';
    case 'arrival':
      return 'arrv';
    case 'approach':
      return 'appr';
    case 'land':
      return 'land';
    case 'taxi-arr':
      return 'txia';
    default:
      return 'unkn';
  }
}

export function DefaultWorld(): World {
  return {
    airspaces: [],
    waypoints: [],
  };
}

export type RadioMessage = {
  id: string;
  frequency: number;
  reply: string;
  created: Duration;
};

export type Game = {};
