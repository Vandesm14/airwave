import { Aircraft, Runway, Vec2, World } from './types';

export const timeScale = 1;

export const nauticalMilesToFeet = 6076.115;
export const knotToFeetPerSecond = 1.68781 * timeScale;

export const ENROUTE_TIME_MULTIPLIER = 10;

export function isSome<T>(value: T): boolean {
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
    altitudePoints: Vec2[];
    end: Vec2;
    maxAngle: Vec2;
    minAngle: Vec2;
  };
} {
  let pos: Vec2 = runway.pos;
  let length = runway.length;

  let start = movePoint(pos, length * 0.5, inverseDegrees(runway.heading));
  let end = movePoint(pos, length * 0.5, runway.heading);

  let maxIlsRangeMiles = 10;
  let ilsPoints: Vec2[] = [];
  let separate = 6.0 / 4;
  for (let i = 1; i < 4; i += 1) {
    let point = i * separate + separate;
    ilsPoints.push(
      movePoint(
        start,
        length + nauticalMilesToFeet * point,
        inverseDegrees(runway.heading)
      )
    );
  }

  let ilsStart = movePoint(
    start,
    length / 2 + nauticalMilesToFeet * maxIlsRangeMiles,
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

  return {
    start,
    end,
    ils: { altitudePoints: ilsPoints, end: ilsStart, maxAngle, minAngle },
  };
}

export function calculateSquaredDistance(a: Vec2, b: Vec2): number {
  return Math.pow(b[0] - a[0], 2) + Math.pow(b[1] - a[1], 2);
}

export function calculateDistance(a: Vec2, b: Vec2): number {
  return Math.sqrt(Math.pow(b[0] - a[0], 2) + Math.pow(b[1] - a[1], 2));
}
