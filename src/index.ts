import { WhisperSTT } from '../vendor/whisper-speech-to-text/src/';

const whisper = new WhisperSTT();

let isRecording = false;
document.addEventListener('keydown', (e) => {
  if (e.key === 'Pause' && !isRecording) {
    whisper.startRecording();
    isRecording = true;
  }
});

// Example: `JetBlue4677, turn left heading 270.`
// Example: `American 0-9-8-6 reduce speed to 240 knots`
document.addEventListener('keyup', (e) => {
  if (e.key === 'Pause') {
    isRecording = false;
    whisper.stopRecording((text) => alert(text));
  }
});

type Ctx = CanvasRenderingContext2D;

const timeScale = 1;

const nauticalMilesToFeet = 6076.115;
const feetPerPixel = 0.005;
const knotToFeetPerSecond = 1.68781 * timeScale;
const milesToFeet = 5280;

function headingToDegrees(heading: number) {
  return (heading + 270) % 360;
}

function degreesToHeading(degrees: number) {
  return (degrees + 360 + 90) % 360;
}

const airlines: Record<string, string> = {
  AAL: 'American',
  SKW: 'Sky West',
  JBL: 'Jet Blue',
};

function randomNumber(from: number, to: number) {
  return Math.round(Math.random() * to - from) + from;
}

function randomAirline() {
  let keys = Object.keys(airlines);
  return keys[Math.floor(Math.random() * keys.length)];
}

function randomCallsign() {
  return `${randomAirline()}${randomNumber(0, 9)}${randomNumber(
    0,
    9
  )}${randomNumber(0, 9)}${randomNumber(0, 9)}`;
}

const toRadians = (degrees: number) => (degrees * Math.PI) / 180;
const toDegrees = (degrees: number) => (degrees / Math.PI) * 180;

type Aircraft = {
  x: number;
  y: number;
  /** In Degrees (0 is north; up) */
  heading: number;
  /** In Knots */
  speed: number;
  callsign: string;
};

type Airspace = {
  x: number;
  y: number;
  r: number;
};

type Runway = {
  x: number;
  y: number;
  /** In Degrees (0 is north; up) */
  heading: number;
  /** In Feet */
  length: number;
};

enum CommandType {
  ALTITUDE = 'Altitude',
  HEADING = 'Heading',
  SPEED = 'Speed',
}

type Command = {
  type: string;
  value: number;
};

type Commands = Array<Command>;

let aircrafts: Array<Aircraft> = [];
let runways: Array<Runway> = [];
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

    if (init) {
      spawnRandomAircraft(airspace);

      runways.push({
        x: width / 2,
        y: height / 2,
        heading: 360,
        length: 7000,
      });
    }

    drawCompass(ctx, airspace);

    for (let runway of runways) {
      drawRunway(ctx, runway);
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
  let degrees = getAngle(result.x, result.y, airspace.x, airspace.y);

  let aircraft: Aircraft = {
    x: result.x,
    y: result.y,
    heading: degreesToHeading(degrees),
    speed: 250,
    callsign: randomCallsign(),
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

function drawRunway(ctx: Ctx, runway: Runway) {
  let length = feetPerPixel * runway.length;
  let width = 5;
  let lineWidth = 1.5;

  let x1 = runway.x;
  let y1 = runway.y;

  ctx.translate(x1, y1);
  ctx.rotate(toRadians(headingToDegrees(runway.heading)));

  ctx.fillStyle = 'grey';
  ctx.fillRect(-length / 2, -width / 2, length, width);

  ctx.fillStyle = '#3087f2';
  ctx.strokeStyle = '#3087f2';
  ctx.fillRect(
    length / 2,
    -lineWidth / 2,
    milesToFeet * feetPerPixel * 10,
    lineWidth
  );

  for (let i = 2; i <= 6; i += 2) {
    ctx.beginPath();
    ctx.arc(
      length / 2 + milesToFeet * feetPerPixel * i,
      1 / 2,
      6,
      0,
      Math.PI * 2
    );
    ctx.stroke();
  }

  ctx.resetTransform();
}

function drawBlip(ctx: Ctx, aircraft: Aircraft) {
  ctx.fillStyle = '#00aa00';
  ctx.strokeStyle = '#00aa00';

  ctx.moveTo(aircraft.x, aircraft.y);

  ctx.beginPath();
  ctx.arc(aircraft.x, aircraft.y, 3, 0, Math.PI * 2);
  ctx.fill();

  ctx.beginPath();
  ctx.arc(
    aircraft.x,
    aircraft.y,
    nauticalMilesToFeet * feetPerPixel * 0.8,
    0,
    Math.PI * 2
  );
  ctx.stroke();

  function drawDirection(ctx: Ctx, aircraft: Aircraft) {
    const angleDegrees = (aircraft.heading + 270) % 360;
    const angleRadians = angleDegrees * (Math.PI / 180);
    const length = aircraft.speed * knotToFeetPerSecond * feetPerPixel * 60;
    const endX = aircraft.x + length * Math.cos(angleRadians);
    const endY = aircraft.y + length * Math.sin(angleRadians);

    ctx.strokeStyle = '#00aa00';
    ctx.beginPath();
    ctx.moveTo(aircraft.x, aircraft.y);
    ctx.lineTo(endX, endY);
    ctx.stroke();
  }

  function drawInfo(ctx: Ctx, aircraft: Aircraft) {
    let spacing = 10;
    let fontSize = 15;

    ctx.fillStyle = '#55ff55';
    ctx.font = `900 ${fontSize}px monospace`;
    ctx.beginPath();
    ctx.fillText(aircraft.callsign, aircraft.x + spacing, aircraft.y - spacing);
    ctx.fill();

    ctx.beginPath();
    ctx.fillText(
      Math.round(aircraft.heading)
        .toString()
        .padStart(3, '0')
        .replace('360', '000'),
      aircraft.x + spacing,
      aircraft.y - spacing + fontSize
    );
    ctx.fill();
  }

  drawDirection(ctx, aircraft);
  drawInfo(ctx, aircraft);
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
