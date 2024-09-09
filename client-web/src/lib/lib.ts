import { Runway, Vec2 } from './types';

export const timeScale = 1;

export const nauticalMilesToFeet = 6076.115;
export const knotToFeetPerSecond = 1.68781 * timeScale;

export function headingToDegrees(heading: number) {
  return (heading + 360 + 90) % 360;
}

export function degreesToHeading(degrees: number) {
  return (degrees + 360 + 90) % 360;
}

export const toDegrees = (degrees: number) => (degrees * 180) / Math.PI;
export const toRadians = (degrees: number) => (degrees * Math.PI) / 180;

export function movePoint(
  x: number,
  y: number,
  length: number,
  directionDegrees: number
) {
  // Convert direction from degrees to radians
  const directionRadians = toRadians(directionDegrees);
  // Calculate the new coordinates
  const newX = x + length * Math.sin(directionRadians);
  const newY = y + length * Math.cos(directionRadians);
  return { x: newX, y: newY };
}

export function angleBetweenPoints(a: Vec2, b: Vec2): number {
  let dx = b.x - a.x;
  let dy = b.y - a.y;

  return (toDegrees(Math.atan2(dy, dx)) + 360) % 360;
}

export function midpointBetweenPoints(a: Vec2, b: Vec2): Vec2 {
  return {
    x: (a.x + b.x) / 2,
    y: (a.y + b.y) / 2,
  };
}

export function projectPoint(origin: Vec2, point: Vec2, scale: number): Vec2 {
  let angle = angleBetweenPoints(origin, point);
  let distance = calculateDistance(origin, point);
  return movePoint(point.x, point.y, distance * scale - distance, angle);
}

export function inverseDegrees(degrees: number): number {
  return (degrees + 180) % 360;
}

export function runwayInfo(
  runway: Runway
  // scale: number
): {
  start: { x: number; y: number };
  end: { x: number; y: number };
  ils: {
    altitudePoints: { x: number; y: number }[];
    end: { x: number; y: number };
    maxAngle: { x: number; y: number };
    minAngle: { x: number; y: number };
  };
} {
  let pos: Vec2 = {
    x: runway.x,
    y: runway.y,
  };
  let length = runway.length;

  let start = movePoint(
    pos.x,
    pos.y,
    length * 0.5,
    inverseDegrees(runway.heading)
  );
  let end = movePoint(pos.x, pos.y, length * 0.5, runway.heading);

  let maxIlsRangeMiles = 10;
  let ilsPoints: { x: number; y: number }[] = [];
  let separate = 6.0 / 4;
  for (let i = 1; i < 4; i += 1) {
    let point = i * separate + separate;
    ilsPoints.push(
      movePoint(
        start.x,
        start.y,
        length + nauticalMilesToFeet * point,
        inverseDegrees(runway.heading)
      )
    );
  }

  let ilsStart = movePoint(
    start.x,
    start.y,
    length / 2 + nauticalMilesToFeet * maxIlsRangeMiles,
    inverseDegrees(runway.heading)
  );

  let maxAngle = movePoint(
    start.x,
    start.y,
    length / 2 + nauticalMilesToFeet * maxIlsRangeMiles,
    inverseDegrees(runway.heading + 5)
  );
  let minAngle = movePoint(
    start.x,
    start.y,
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
  return Math.pow(b.x - a.x, 2) + Math.pow(b.y - a.y, 2);
}

export function calculateDistance(a: Vec2, b: Vec2): number {
  return Math.sqrt(Math.pow(b.x - a.x, 2) + Math.pow(b.y - a.y, 2));
}
