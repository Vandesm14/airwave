import { Runway, Vec2 } from './types';

export const airlines: Record<string, string> = {
  AAL: 'American Airlines',
  SKW: 'Sky West',
  JBL: 'Jet Blue',
};

export const timeScale = 1;

export const feetPerPixel = 0.005;
export const nauticalMilesToFeet = 6076.115;
export const knotToFeetPerSecond = 1.68781 * timeScale;
export const milesToFeet = 6076.12;

export function headingToDegrees(heading: number) {
  return (heading + 270) % 360;
}

export function degreesToHeading(degrees: number) {
  return (degrees + 360 + 90) % 360;
}

export const toDegrees = (degrees: number) => (degrees * 180) / Math.PI;
export const toRadians = (degrees: number) => (degrees * Math.PI) / 180;

export function callsignString(id: string): string {
  return `${airlines[id.slice(0, 3)]} ${id.slice(3, 7)}`;
}

export function movePoint(
  x: number,
  y: number,
  length: number,
  directionDegrees: number
) {
  // Convert direction from degrees to radians
  const directionRadians = directionDegrees * (Math.PI / 180);

  // Calculate the new coordinates
  const newX = x + length * Math.cos(directionRadians);
  const newY = y + length * Math.sin(directionRadians);

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

export function runwayInfo(runway: Runway): {
  start: { x: number; y: number };
  end: { x: number; y: number };
  ils: {
    altitudePoints: { x: number; y: number }[];
    end: { x: number; y: number };
    maxAngle: { x: number; y: number };
    minAngle: { x: number; y: number };
  };
} {
  let start = movePoint(
    runway.x,
    runway.y,
    runway.length * feetPerPixel * 0.5,
    inverseDegrees(headingToDegrees(runway.heading))
  );
  let end = movePoint(
    runway.x,
    runway.y,
    runway.length * feetPerPixel * 0.5,
    headingToDegrees(runway.heading)
  );

  let maxIlsRangeMiles = 10;
  let ilsPoints: { x: number; y: number }[] = [];
  let separate = 6.0 / 4;
  for (let i = 1; i < 4; i += 1) {
    let point = i * separate + separate;
    ilsPoints.push(
      movePoint(
        start.x,
        start.y,
        length + milesToFeet * feetPerPixel * point,
        inverseDegrees(headingToDegrees(runway.heading))
      )
    );
  }

  let ilsStart = movePoint(
    start.x,
    start.y,
    length / 2 + milesToFeet * feetPerPixel * maxIlsRangeMiles,
    inverseDegrees(headingToDegrees(runway.heading))
  );

  let maxAngle = movePoint(
    start.x,
    start.y,
    length / 2 + milesToFeet * feetPerPixel * maxIlsRangeMiles,
    inverseDegrees(headingToDegrees(runway.heading + 5))
  );
  let minAngle = movePoint(
    start.x,
    start.y,
    length / 2 + milesToFeet * feetPerPixel * maxIlsRangeMiles,
    inverseDegrees(headingToDegrees((runway.heading + (360 - 5)) % 360))
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
