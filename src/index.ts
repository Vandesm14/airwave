type Ctx = CanvasRenderingContext2D;

const timeScale = 1;

const feetPerPixel = 0.007;
const knotToFeetPerSecond = 1.68781 * timeScale;

function headingToDegrees(heading: number) {
  return (heading + 270) % 360;
}

function degreesToHeading(degrees: number) {
  return (degrees + 90) % 360;
}

type Aircraft = {
  x: number;
  y: number;
  /** In Degrees (0 is up) */
  heading: number;
  /** In Knots */
  speed: number;
};

type Airspace = {
  x: number;
  y: number;
  r: number;
};

let aircrafts: Array<Aircraft> = [];
let lastTime = Date.now();

const canvas = document.getElementById('canvas');
if (canvas instanceof HTMLCanvasElement && canvas !== null) {
  window.addEventListener('resize', () => {
    canvas.width = canvas.clientWidth;
    canvas.height = canvas.clientHeight;

    draw(canvas, false);
  });

  setInterval(() => draw(canvas, false), 1000 / 30);

  canvas.width = canvas.clientWidth;
  canvas.height = canvas.clientHeight;

  draw(canvas, true);
}

function draw(canvas: HTMLCanvasElement, init: boolean) {
  const width = canvas.width;
  const height = canvas.height;

  let dt = Date.now() - lastTime;
  lastTime = Date.now();
  let dts = dt / 1000;

  let ctx = canvas.getContext('2d');
  if (ctx) {
    ctx.fillStyle = 'black';
    ctx.fillRect(0, 0, width, height);

    let airspace = calcAirspace(width, height);
    drawCompass(ctx, airspace);
    drawRunway(ctx, width, height);

    if (init) {
      spawnRandomAircraft(airspace);
    }

    for (let aircraft of aircrafts) {
      let newPos = movePoint(
        aircraft.x,
        aircraft.y,
        aircraft.speed * knotToFeetPerSecond * feetPerPixel * dts,
        headingToDegrees(aircraft.heading)
      );

      aircraft.x = newPos.x;
      aircraft.y = newPos.y;

      drawBlip(ctx, aircraft);
    }
  }
}

function spawnRandomAircraft(airspace: Airspace) {
  let result = getRandomPointOnCircle(airspace.x, airspace.y, airspace.r + 25);
  let heading =
    (getAngle(result.x, result.y, airspace.x, airspace.y) + 90) % 360;

  let aircraft: Aircraft = {
    x: result.x,
    y: result.y,
    heading,
    speed: 220,
  };

  aircrafts.push(aircraft);
}

function calcAirspace(width: number, height: number): Airspace {
  let x = width / 2;
  let y = height / 2;
  let radius = x;
  if (height < width) {
    radius = y;
  }

  radius -= 50;

  return {
    x,
    y,
    r: radius,
  };
}

function drawCompass(ctx: Ctx, airspace: Airspace) {
  ctx.strokeStyle = 'white';
  ctx.beginPath();
  ctx.arc(airspace.x, airspace.y, airspace.r, 0, Math.PI * 2);
  ctx.stroke();
}

function drawRunway(ctx: Ctx, width: number, height: number) {
  let length = feetPerPixel * 7000;

  let x1 = width * 0.5;
  let y1 = height * 0.5 - length * 0.5;

  ctx.translate(x1, y1);
  ctx.rotate(0);

  ctx.fillStyle = 'grey';
  ctx.fillRect(0, 0, 3, length);

  ctx.setTransform(1, 0, 0, 1, 0, 0);
}

function drawBlip(ctx: Ctx, aircraft: Aircraft) {
  ctx.fillStyle = '#00ff00';
  ctx.strokeStyle = '#00ff00';

  ctx.moveTo(aircraft.x, aircraft.y);

  ctx.beginPath();
  ctx.arc(aircraft.x, aircraft.y, 3, 0, Math.PI * 2);
  ctx.fill();

  ctx.beginPath();
  ctx.arc(aircraft.x, aircraft.y, 15, 0, Math.PI * 2);
  ctx.stroke();

  function drawDirection(ctx: Ctx, aircraft: Aircraft) {
    const angleDegrees = (aircraft.heading + 270) % 360;
    const angleRadians = angleDegrees * (Math.PI / 180);
    const length = 40;
    const endX = aircraft.x + length * Math.cos(angleRadians);
    const endY = aircraft.y + length * Math.sin(angleRadians);

    ctx.beginPath();
    ctx.moveTo(aircraft.x, aircraft.y);
    ctx.lineTo(endX, endY);
    ctx.stroke();
  }

  drawDirection(ctx, aircraft);
}

function getRandomPointOnCircle(cx: number, cy: number, r: number) {
  // Generate a random angle in radians
  const randomAngle = Math.random() * 2 * Math.PI;

  // Calculate the coordinates of the point on the circle
  const x = cx + r * Math.cos(randomAngle);
  const y = cy + r * Math.sin(randomAngle);

  return { x, y, angle: randomAngle };
}

function getAngle(x1: number, y1: number, x2: number, y2: number) {
  // Calculate the difference in coordinates
  const dx = x2 - x1;
  const dy = y2 - y1;

  // Calculate the angle in radians
  const angleRadians = Math.atan2(dy, dx);

  // Convert the angle to degrees
  const angleDegrees = angleRadians * (180 / Math.PI);

  return angleDegrees;
}

function movePoint(
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
