import { Airport } from '../../bindings/Airport';
import { Airspace } from '../../bindings/Airspace';
import { FlightSegment } from '../../bindings/FlightSegment';
import { LandingState } from '../../bindings/LandingState';
import { Runway } from '../../bindings/Runway';
import { TaxiingState } from '../../bindings/TaxiingState';
import { Vec2 } from '../../bindings/Vec2';
import { World } from '../../bindings/World';

export const timeScale = 1;

export const nauticalMilesToFeet = 6076.115;
export const knotToFeetPerSecond = 1.68781 * timeScale;

// TODO: remove this
export const HARD_CODED_AIRPORT = 'KSFO';
export function hardcodedAirspace(
  world: World | undefined
): Airspace | undefined {
  if (!world) return undefined;
  for (let airspace of world.airspaces) {
    if (airspace.id === HARD_CODED_AIRPORT) {
      return airspace;
    }
  }

  return undefined;
}
export function hardcodedAirport(
  world: World | undefined
): Airport | undefined {
  if (!world) return undefined;
  for (let airspace of world.airspaces) {
    let airport = airspace.airports.find(
      (airport) => airport.id === HARD_CODED_AIRPORT
    );
    if (airport) {
      return airport;
    }
  }

  return undefined;
}

export function dbg<T>(a: T, note?: string): T {
  if (note) {
    console.log(note, a);
  } else {
    console.log(a);
  }
  return a;
}

export function isSome<T>(value: T): value is NonNullable<T> {
  return value !== undefined && value !== null;
}

export function headingToDegrees(heading: number) {
  return (heading + 360 + 90) % 360;
}

export function degreesToHeading(degrees: number) {
  return (degrees + 360 + 90) % 360;
}

export const toDegrees = (degrees: number) => (degrees * 180) / Math.PI;
export const toRadians = (degrees: number) => (degrees * Math.PI) / 180;

export function movePoint(
  point: Vec2,
  length: number,
  directionDegrees: number
): Vec2 {
  // Convert direction from degrees to radians
  const directionRadians = toRadians(directionDegrees);
  // Calculate the new coordinates
  const newX = point[0] + length * Math.sin(directionRadians);
  const newY = point[1] + length * Math.cos(directionRadians);
  return [newX, newY];
}

export function angleBetweenPoints(a: Vec2, b: Vec2): number {
  let dx = b[0] - a[0];
  let dy = b[1] - a[1];

  return (toDegrees(Math.atan2(dy, dx)) + 360) % 360;
}

export function midpointBetweenPoints(a: Vec2, b: Vec2): Vec2 {
  return [(a[0] + b[0]) / 2, (a[1] + b[1]) / 2];
}

export function projectPoint(origin: Vec2, point: Vec2, scale: number): Vec2 {
  let angle = angleBetweenPoints(origin, point);
  let distance = calculateDistance(origin, point);
  return movePoint(point, distance * scale - distance, angle);
}

export function inverseDegrees(degrees: number): number {
  return (degrees + 180) % 360;
}

export function runwayInfo(
  runway: Runway
  // scale: number
): {
  start: Vec2;
  end: Vec2;
  ils: {
    minGlideslope: Vec2;
    end: Vec2;
    maxAngle: Vec2;
    minAngle: Vec2;
  };
} {
  let pos: Vec2 = runway.pos;
  let length = runway.length;

  let start = movePoint(pos, length * 0.5, inverseDegrees(runway.heading));
  let end = movePoint(pos, length * 0.5, runway.heading);

  let maxIlsRangeMiles = 18;
  let ilsStart = movePoint(
    start,
    length / 2 + nauticalMilesToFeet * 20,
    inverseDegrees(runway.heading)
  );

  let maxAngle = movePoint(
    start,
    length / 2 + nauticalMilesToFeet * maxIlsRangeMiles,
    inverseDegrees(runway.heading + 5)
  );
  let minAngle = movePoint(
    start,
    length / 2 + nauticalMilesToFeet * maxIlsRangeMiles,
    inverseDegrees((runway.heading + (360 - 5)) % 360)
  );
  let minGlideslope = movePoint(
    start,
    nauticalMilesToFeet * 6.0,
    inverseDegrees(runway.heading)
  );

  return {
    start,
    end,
    ils: {
      minGlideslope,
      end: ilsStart,
      maxAngle,
      minAngle,
    },
  };
}

export function calculateSquaredDistance(a: Vec2, b: Vec2): number {
  return Math.pow(b[0] - a[0], 2) + Math.pow(b[1] - a[1], 2);
}

export function calculateDistance(a: Vec2, b: Vec2): number {
  return Math.sqrt(Math.pow(b[0] - a[0], 2) + Math.pow(b[1] - a[1], 2));
}

export function formatTime(duration: number): string {
  const isNegative = duration < 0;
  let absDuration = Math.abs(duration);
  let durationSeconds = Math.floor(absDuration / 1000);
  let seconds = (durationSeconds % 60).toString().padStart(2, '0');
  let minutes = Math.floor(durationSeconds / 60)
    .toString()
    .padStart(2, '0');
  let timeString = `${minutes}:${seconds}`;
  if (isNegative) {
    timeString = `-${timeString}`;
  }

  return timeString;
}

export function shortLandingState(state: LandingState): string {
  switch (state) {
    case 'before-turn':
      return 'ILS';
    case 'turning':
      return 'TRN';
    case 'correcting':
      return 'ALN';
    case 'localizer':
      return 'LOC';
    case 'glideslope':
      return 'GLS';

    default:
      return 'UKN';
  }
}

export function shortTaxiingState(state: TaxiingState): string {
  switch (state) {
    case 'armed':
      return 'ARM';
    case 'stopped':
      return 'STP';
    case 'holding':
      return 'HLD';
    case 'override':
      return 'OVR';

    default:
      return 'UKN';
  }
}

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
    airspace_statuses: {},
  };
}
