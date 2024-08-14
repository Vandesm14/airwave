import axios from 'axios';
import { WhisperSTT } from '../vendor/whisper-speech-to-text/src/';

const whisper = new WhisperSTT();
let isRecording = false;

type Ctx = CanvasRenderingContext2D;

const timeScale = 2;

const nauticalMilesToFeet = 6076.115;
const feetPerPixel = 0.005;
const knotToFeetPerSecond = 1.68781 * timeScale;
const milesToFeet = 6076.12;

function headingToDegrees(heading: number) {
  return (heading + 270) % 360;
}

function degreesToHeading(degrees: number) {
  return (degrees + 360 + 90) % 360;
}

function speak(text: string) {
  if ('speechSynthesis' in window) {
    const utterance = new SpeechSynthesisUtterance(
      text.replace(/[0-9]/g, '$& ')
    );
    utterance.volume = 0.01;
    utterance.rate = 1.0;
    utterance.pitch = 1.3;
    window.speechSynthesis.speak(utterance);
  } else {
    console.log("Sorry, your browser doesn't support text to speech!");
  }
}

const airlines: Record<string, string> = {
  AAL: 'American Airlines',
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

type Message = {
  role: string;
  content: string;
};

async function complete(model: string, messages: Array<Message>) {
  let response = await axios.post(
    'http://localhost:8000/complete',
    JSON.stringify({
      model,
      messages,
    }),
    { headers: { 'Content-Type': 'application/json' } }
  );

  return response.data;
}

type Aircraft = {
  x: number;
  y: number;

  target: {
    /** Name of cleared runway to land on */
    runway: null | string;
    /** In Degrees (0 is north; up) */
    heading: number;
    /** In Knots */
    speed: number;
    /** In Feet */
    altitude: number;
  };

  /** In Degrees (0 is north; up) */
  heading: number;
  /** In Knots */
  speed: number;
  /** In Feet */
  altitude: number;
  callsign: string;
};

type Airspace = {
  x: number;
  y: number;
  r: number;
};

type Runway = {
  id: string;
  x: number;
  y: number;
  /** In Degrees (0 is north; up) */
  heading: number;
  /** In Feet */
  length: number;
};

enum TaskType {
  LAND = 'land',
  GOAROUND = 'go-around',
  ALTITUDE = 'altitude',
  HEADING = 'heading',
  SPEED = 'speed',
}

type Task = [TaskType, number];
type CommandResponse = {
  reply: string;
  id: string;
  tasks: Array<Task>;
};

let aircrafts: Array<Aircraft | undefined> = [];
let runways: Array<Runway> = [];
let lastTime = Date.now();
let lastDraw = 0;

const canvas = document.getElementById('canvas');
if (canvas instanceof HTMLCanvasElement && canvas !== null) {
  window.addEventListener('resize', () => {
    canvas.width = canvas.clientWidth;
    canvas.height = canvas.clientHeight;
  });

  setInterval(() => loopMain(canvas, false), 1000 / 30);

  canvas.width = canvas.clientWidth;
  canvas.height = canvas.clientHeight;

  loopMain(canvas, true);
}

const chatbox = document.getElementById('chatbox');
const messageTemplate = document.getElementById('message-template');

function speakAsAircraft(aircraft: Aircraft, reply: string, tasks: Task[]) {
  if (
    chatbox instanceof HTMLDivElement &&
    messageTemplate instanceof HTMLTemplateElement
  ) {
    let callsignString = `${
      airlines[aircraft.callsign.slice(0, 3)]
    } ${aircraft.callsign.slice(3, 7)}`;

    let fullReply = `${reply}, ${callsignString}`;

    let message = messageTemplate.innerHTML
      .replace('{{callsign}}', aircraft.callsign)
      .replace('{{text}}', fullReply)
      .replace(
        '{{tasks}}',
        JSON.stringify(tasks, null, 0).replace(/"/g, '&quot;')
      );

    chatbox.insertAdjacentHTML('beforeend', message);
    chatbox.scrollTo(0, chatbox.scrollHeight);

    speak(fullReply);
  }
}

function doGoAround(aircraft: Aircraft) {
  aircraft.target.runway = null;
  aircraft.target.speed = 220;

  if (aircraft.target.altitude < 2000) {
    aircraft.target.altitude = 2000;
  }
}

async function parseATCMessage(textRaw: string) {
  let text = textRaw.replace(/9\sor\s?/g, '9');

  if (
    chatbox instanceof HTMLDivElement &&
    messageTemplate instanceof HTMLTemplateElement
  ) {
    let message = messageTemplate.innerHTML
      .replace('{{callsign}}', 'ATC')
      .replace('{{text}}', text)
      .replace('{{tasks}}', '');

    chatbox.insertAdjacentHTML('beforeend', message);
    chatbox.scrollTo(0, chatbox.scrollHeight);
  }

  let response = await complete('gpt-4o-mini', [
    {
      role: 'system',
      content: `You are a professional airline pilot. Your job is to listen to ATC commands and format them into a list of tasks for your aircraft. You MUST reply with a JSON array of the tasks that the aircraft is instructed to follow.
Available Tasks: ["heading", number], ["speed", number], ["altitude", number], ["land", string], ["go-around"]
Available Callsigns: SKW (Skywest), AAL ("American Airlines" or "American"), JBL (Jet Blue)
Examples:
User: Skywest 5-1-3-8 turn left heading 180 and reduce speed to 230.
Assistant: {"reply": "Left turn heading 180, reduce speed to 230.", "id": "SKW5138", "tasks":[["heading", 180], ["speed", 230]]}
User: American 1-2-3-4 climb and maintain flight level 050.
Assistant: {"reply": "Climb and maintain flight level 050.", "id": "AAL1234", "tasks":[["altitude", 5000]]}
User: Skywest 5-1-3-8 descend and maintain 5,000 feet.
Assistant: {"reply": "Descend and and maintain 5,000 feet.", "id": "SKW5138", "tasks":[["altitude", 5000]]}
User: Skywest 5-1-3-8 cleared to land runway 18 left.
Assistant: {"reply": "Cleared to land runway 18 left.", "id": "SKW5138", "tasks":[["land", "18L"]]}

If you do not understand the command, use a blank array for "tasks" and ask ATC to clarify in the "reply".
Example:
User: American 0725, turn.
Assistant: {"reply": "Say again, American 0725.", "id": "SKW5138", "tasks":[]}`,
    },
    {
      role: 'user',
      content: text,
    },
  ]);

  if (response.choices instanceof Array) {
    let reply = response.choices[0].message.content;
    let json: CommandResponse = JSON.parse(reply);

    let aircraft = aircrafts.find((el) => el?.callsign === json.id);
    if (aircraft) {
      speakAsAircraft(aircraft, json.reply, json.tasks);
      console.log({ text, json });

      for (let task of json.tasks) {
        let value = task[1];

        if (typeof value === 'number') {
          switch (task[0]) {
            case TaskType.HEADING:
              aircraft.target.heading = value;
              break;
            case TaskType.SPEED:
              aircraft.target.speed = value;
              break;
            case TaskType.ALTITUDE:
              aircraft.target.altitude = value;
              break;
          }
        }

        if (typeof value === 'string') {
          switch (task[0]) {
            case TaskType.LAND:
              aircraft.target.runway = value;
              break;
          }
        }

        switch (task[0]) {
          case TaskType.GOAROUND:
            doGoAround(aircraft);
            break;
        }
      }
    }
  }
}

document.addEventListener('keydown', (e) => {
  if (e.key === 'Insert' && !isRecording) {
    whisper.startRecording();
    isRecording = true;
  } else if (e.key === 'Delete' && isRecording) {
    whisper.abortRecording();
    isRecording = false;
  }

  if (chatbox instanceof HTMLDivElement) {
    if (isRecording) {
      chatbox.classList.add('live');
    } else {
      chatbox.classList.remove('live');
    }
  }
});

document.addEventListener('keyup', (e) => {
  if (e.key === 'Insert' && isRecording) {
    isRecording = false;

    whisper.stopRecording(parseATCMessage);
  }

  if (chatbox instanceof HTMLDivElement) {
    if (isRecording) {
      chatbox.classList.add('live');
    } else {
      chatbox.classList.remove('live');
    }
  }
});

function loopMain(canvas: HTMLCanvasElement, init: boolean) {
  const width = canvas.width;
  const height = canvas.height;

  let airspace = calcAirspace(width, height);

  let dt = Date.now() - lastTime;
  lastTime = Date.now();
  let dts = dt / 1000;

  if (init) {
    loopInit(width, height, calcAirspace(width, height));
  }

  loopUpdate(dts);

  let deltaDrawTime = Date.now() - lastDraw;
  if (lastDraw === 0 || deltaDrawTime >= 1000 / 1) {
    lastDraw = Date.now();
    loopDraw(canvas, airspace, dts);
  }
}

function loopInit(width: number, height: number, airspace: Airspace) {
  let runway: Runway = {
    id: '20',
    x: width / 2,
    y: height / 2,
    heading: 200,
    length: 7000,
  };
  runways.push(runway);

  for (let i = 0; i < 2; i++) {
    spawnRandomAircraft(airspace);
  }
}

function loopUpdate(dts: number) {
  for (let aircraft of aircrafts) {
    if (aircraft) {
      updateAircraftPosition(aircraft, dts);
      updateAircraftILS(aircraft);
      updateAircraftTargets(aircraft, dts);
    }
  }
}

function loopDraw(canvas: HTMLCanvasElement, airspace: Airspace, dts: number) {
  const width = canvas.width;
  const height = canvas.height;

  let ctx = canvas.getContext('2d');
  if (ctx) {
    const fontSize = 15;
    ctx.font = `900 ${fontSize}px monospace`;
    ctx.fillStyle = 'black';
    ctx.fillRect(0, 0, width, height);

    drawCompass(ctx, airspace);

    for (let runway of runways) {
      drawRunway(ctx, runway);
    }

    for (let aircraft of aircrafts) {
      if (aircraft) {
        drawBlip(ctx, aircraft);
      }
    }

    ctx.fillStyle = '#009900';
    ctx.fillText(`${Math.round(1 / dts)} fps`, 10, 20);
  }
}

function updateAircraftPosition(aircraft: Aircraft, dts: number) {
  let newPos = movePoint(
    aircraft.x,
    aircraft.y,
    aircraft.speed * knotToFeetPerSecond * feetPerPixel * dts,
    headingToDegrees(aircraft.heading)
  );

  aircraft.x = newPos.x;
  aircraft.y = newPos.y;
}

function updateAircraftILS(aircraft: Aircraft) {
  // Update targets based on target ILS
  if (aircraft.target.runway !== null) {
    const runway = runways.find((r) => r.id === aircraft.target.runway);
    if (runway) {
      let runwayHeading = runway.heading;
      let info = runwayInfo(runway);
      let deltaAngleStart = calcDeltaAngle(
        calculateAngleBetweenPoints(info.start, aircraft),
        inverseDegrees(headingToDegrees(runwayHeading))
      );

      let distanceToRunway = calculateSquaredDistance(aircraft, info.start);
      let decreaseAltitudeStart = milesToFeet * feetPerPixel * 5.6;
      let decreaseSpeedStart = milesToFeet * feetPerPixel * 10;

      if (Math.abs(deltaAngleStart) <= 10) {
        let turnPadding = Math.min(30, Math.abs(deltaAngleStart) * 6);

        if (
          aircraft.altitude > 4000 &&
          distanceToRunway <= Math.pow(decreaseAltitudeStart, 2)
        ) {
          doGoAround(aircraft);
          speakAsAircraft(
            aircraft,
            `We've lost the localizer. Requesting vectors to re-intercept.`,
            [[TaskType.GOAROUND, 0]]
          );

          return;
        } else if (distanceToRunway <= Math.pow(decreaseAltitudeStart, 2)) {
          aircraft.target.altitude = 0;
        }

        if (Math.round(Math.abs(deltaAngleStart)) === 0) {
          if (distanceToRunway <= Math.pow(decreaseSpeedStart, 2))
            aircraft.target.speed = 170;
        }

        if (deltaAngleStart < 0) {
          aircraft.target.heading = runwayHeading + turnPadding;
        } else if (deltaAngleStart > 0) {
          aircraft.target.heading = runwayHeading - turnPadding;
        }
      } else {
        if (
          Math.round(Math.abs(deltaAngleStart)) === 180 &&
          aircraft.altitude === 0
        ) {
          let index = aircrafts.findIndex(
            (el) => el?.callsign === aircraft.callsign
          );
          if (index >= 0) {
            aircrafts[index] = undefined;
          }
        }
      }
    }
  }
}

function updateAircraftTargets(aircraft: Aircraft, dts: number) {
  /** In feet per second */
  const climbSpeed = timeScale * Math.round(2000 / 60) * dts;
  /** In degrees per second */
  const turnSpeed = timeScale * 2 * dts;
  /** In knots per second */
  const speedSpeed = timeScale * 1 * dts;

  // Set if "close enough"
  if (Math.abs(aircraft.altitude - aircraft.target.altitude) < climbSpeed) {
    aircraft.altitude = aircraft.target.altitude;
  }
  if (Math.abs(aircraft.heading - aircraft.target.heading) < turnSpeed) {
    aircraft.heading = aircraft.target.heading;
  }
  if (Math.abs(aircraft.speed - aircraft.target.speed) < speedSpeed) {
    aircraft.speed = aircraft.target.speed;
  }

  // Change based on speed if not equal
  if (aircraft.altitude !== aircraft.target.altitude) {
    if (aircraft.altitude < aircraft.target.altitude) {
      aircraft.altitude += climbSpeed;
    } else {
      aircraft.altitude -= climbSpeed;
    }
  }
  if (aircraft.heading !== aircraft.target.heading) {
    // let delta_angle =
    //   ((aircraft.target.heading - aircraft.heading + 540.0) % 360.0) - 180.0;
    let delta_angle = calcDeltaAngle(aircraft.heading, aircraft.target.heading);
    if (delta_angle < 0) {
      aircraft.heading -= turnSpeed;
    } else {
      aircraft.heading += turnSpeed;
    }
  }
  if (aircraft.speed !== aircraft.target.speed) {
    if (aircraft.speed < aircraft.target.speed) {
      aircraft.speed += speedSpeed;
    } else {
      aircraft.speed -= speedSpeed;
    }
  }
}

function spawnRandomAircraft(airspace: Airspace) {
  let result = getRandomPointOnCircle(airspace.x, airspace.y, airspace.r + 25);
  let degrees = getAngle(result.x, result.y, airspace.x, airspace.y);

  let heading = degreesToHeading(degrees);
  let speed = 250;
  let altitude = 8000;

  let aircraft: Aircraft = {
    x: result.x,
    y: result.y,

    target: {
      runway: null,
      heading,
      speed,
      altitude,
    },

    heading,
    speed,
    altitude,
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
  ctx.fillStyle = 'white';
  ctx.beginPath();
  ctx.arc(airspace.x, airspace.y, airspace.r, 0, Math.PI * 2);
  ctx.stroke();

  ctx.fillStyle = '#888';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  for (let i = 0; i < 36; i++) {
    let text = degreesToHeading(i * 10)
      .toString()
      .padStart(3, '0');
    if (text === '000') {
      text = '360';
    }

    ctx.beginPath();
    ctx.fillText(
      text,
      Math.cos(toRadians(i * 10)) * (airspace.r + 20) + airspace.x,
      Math.sin(toRadians(i * 10)) * (airspace.r + 20) + airspace.y
    );
    ctx.stroke();
  }
}

function drawRunway(ctx: Ctx, runway: Runway) {
  let length = feetPerPixel * runway.length;
  let width = 5;

  let x1 = runway.x;
  let y1 = runway.y;

  ctx.translate(x1, y1);
  ctx.rotate(toRadians(headingToDegrees(runway.heading)));

  ctx.fillStyle = 'grey';
  ctx.fillRect(-length / 2, -width / 2, length, width);

  ctx.fillStyle = '#3087f2';
  ctx.strokeStyle = '#3087f2';

  ctx.resetTransform();

  let info = runwayInfo(runway);
  ctx.beginPath();
  ctx.moveTo(info.start.x, info.start.y);
  ctx.lineTo(info.ils.end.x, info.ils.end.y);
  ctx.stroke();

  ctx.strokeStyle = '#444444';
  ctx.beginPath();
  ctx.moveTo(info.start.x, info.start.y);
  ctx.lineTo(info.ils.maxAngle.x, info.ils.maxAngle.y);
  ctx.stroke();

  ctx.beginPath();
  ctx.moveTo(info.start.x, info.start.y);
  ctx.lineTo(info.ils.minAngle.x, info.ils.minAngle.y);
  ctx.stroke();

  ctx.strokeStyle = '#3087f2';
  for (let point of info.ils.altitudePoints) {
    ctx.beginPath();
    ctx.arc(point.x, point.y, 6, 0, Math.PI * 2);
    ctx.stroke();
  }
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
    let spacing = 16;
    let fontSize = 15;

    ctx.textAlign = 'left';
    ctx.fillStyle = '#44ff44';
    ctx.beginPath();
    ctx.fillText(aircraft.callsign, aircraft.x + spacing, aircraft.y - spacing);
    ctx.fill();

    let altitudeIcon = ' ';
    if (aircraft.altitude < aircraft.target.altitude) {
      altitudeIcon = '⬈';
    } else if (aircraft.altitude > aircraft.target.altitude) {
      altitudeIcon = '⬊';
    }

    ctx.beginPath();
    ctx.fillText(
      Math.round(aircraft.altitude / 100)
        .toString()
        .padStart(3, '0') +
        altitudeIcon +
        Math.round(aircraft.target.altitude / 100)
          .toString()
          .padStart(3, '0'),
      aircraft.x + spacing,
      aircraft.y - spacing + fontSize
    );
    ctx.fill();

    ctx.beginPath();
    ctx.fillText(
      Math.round(aircraft.heading)
        .toString()
        .padStart(3, '0')
        .replace('360', '000') +
        ' ' +
        Math.round(aircraft.target.heading)
          .toString()
          .padStart(3, '0')
          .replace('360', '000'),
      aircraft.x + spacing,
      aircraft.y - spacing + fontSize * 2
    );
    ctx.fill();

    ctx.beginPath();
    ctx.fillText(
      Math.round(aircraft.speed).toString(),
      aircraft.x + spacing,
      aircraft.y - spacing + fontSize * 3
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

function calcDeltaAngle(current: number, target: number): number {
  return ((target - current + 540.0) % 360.0) - 180.0;
}

function inverseDegrees(degrees: number): number {
  return (degrees + 180) % 360;
}

function calculateSquaredDistance(
  a: { x: number; y: number },
  b: { x: number; y: number }
): number {
  return Math.pow(b.x - a.x, 2) + Math.pow(b.y - a.y, 2);
}

function calculateDistance(
  a: { x: number; y: number },
  b: { x: number; y: number }
): number {
  return Math.sqrt(
    Math.pow(b.x, 2) - Math.pow(a.x, 2) + Math.pow(b.y, 2) - Math.pow(a.y, 2)
  );
}

function runwayInfo(runway: Runway): {
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

  let maxIlsRangeMiles = 12;
  let ilsPoints: { x: number; y: number }[] = [];
  let separate = 5.6 / 4;
  for (let i = 0; i < 4; i += 1) {
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

function calculateAngleBetweenPoints(
  point1: { x: number; y: number },
  point2: { x: number; y: number }
): number {
  // Calculate the differences in coordinates
  const dx = point2.x - point1.x;
  const dy = point2.y - point1.y;

  // Calculate the angle using Math.atan2
  let angle = Math.atan2(dy, dx);

  // Convert the angle from radians to degrees
  angle = angle * (180 / Math.PI);

  // Normalize the angle to be between 0 and 360 degrees
  angle = (angle + 360) % 360;

  return angle;
}
