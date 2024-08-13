import axios from 'axios';
import { WhisperSTT } from '../vendor/whisper-speech-to-text/src/';

const whisper = new WhisperSTT();
let isRecording = false;

type Ctx = CanvasRenderingContext2D;

const timeScale = 1.2;

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

function speak(text: string) {
  // if ('speechSynthesis' in window) {
  //   const utterance = new SpeechSynthesisUtterance(text);
  //   window.speechSynthesis.speak(utterance);
  // } else {
  //   console.log("Sorry, your browser doesn't support text to speech!");
  // }
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

const chatbox = document.getElementById('chatbox');
const messageTemplate = document.getElementById('message-template');

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
Available Tasks: ["heading", number], ["speed", number], ["altitude", number], ["land", string]
Available Callsigns: SKW (Skywest), AAL (American Airlines), JBL (Jet Blue)
Examples:
User: Skywest 5-1-3-8 turn left heading 180 and reduce speed to 230.
Assistant: {"reply": "Left turn heading 180, reduce speed to 230, Skywest 5138.", "id": "SKW5138", "tasks":[["heading", 180], ["speed", 230]]}
User: Skywest 5-1-3-8 cleared to land runway 18 left.
Assistant: {"reply": "Cleared to land runway 18 left, Skywest 5138.", "id": "SKW5138", "tasks":[["land", "18L"]]}

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

    if (
      chatbox instanceof HTMLDivElement &&
      messageTemplate instanceof HTMLTemplateElement
    ) {
      let message = messageTemplate.innerHTML
        .replace('{{callsign}}', json.id)
        .replace('{{text}}', json.reply)
        .replace(
          '{{tasks}}',
          JSON.stringify(json.tasks, null, 0).replace(/"/g, '&quot;')
        );

      chatbox.insertAdjacentHTML('beforeend', message);
      chatbox.scrollTo(0, chatbox.scrollHeight);
    }

    speak(json.reply);
    console.log({ text, json });

    let aircraft = aircrafts.find((el) => el.callsign === json.id);
    if (aircraft) {
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
      let runway: Runway = {
        id: '20',
        x: width / 2,
        y: height / 2,
        heading: 200,
        length: 7000,
      };
      runways.push(runway);

      for (let i = 0; i < 1; i++) {
        spawnRandomAircraft(airspace);
      }
    }

    drawCompass(ctx, airspace);

    for (let runway of runways) {
      drawRunway(ctx, runway);
    }

    for (let aircraft of aircrafts) {
      updateAircraftTargets(aircraft, dts);

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

function updateAircraftTargets(aircraft: Aircraft, dts: number) {
  /** In feet per second */
  const climbSpeed = timeScale * Math.round(2000 / 60) * dts;
  /** In degrees per second */
  const turnSpeed = timeScale * 2 * dts;
  /** In knots per second */
  const speedSpeed = timeScale * 1 * dts;

  // Update targets based on target ILS
  if (aircraft.target.runway !== null) {
    const runway = runways.find((r) => r.id === aircraft.target.runway);
    if (runway) {
      let runwayHeading = runway.heading;
      let result = calculatePerpendicularLine(
        runway,
        headingToDegrees(runwayHeading),
        aircraft
      );
      let delta_angle = calcDeltaAngle(
        aircraft.heading,
        degreesToHeading(result.heading)
      );

      let turnPadding = 10;

      let speedInPixels =
        aircraft.speed * knotToFeetPerSecond * feetPerPixel * dts;
      let secondsUntilContact = (result.distance / speedInPixels) * dts;
      let secondsToTurnBack = turnSpeed * turnPadding;

      console.log({
        speedInPixels,
        secondsUntilContact,
        secondsToTurnBack,
      });

      if (secondsToTurnBack >= secondsUntilContact) {
        aircraft.target.heading = runwayHeading;
      } else {
        if (delta_angle > 0) {
          aircraft.target.heading = runwayHeading + turnPadding;
        } else if (delta_angle < 0) {
          aircraft.target.heading = runwayHeading - turnPadding;
        }
      }
    }
  }

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
    -length / 2,
    -lineWidth / 2,
    -milesToFeet * feetPerPixel * 10,
    lineWidth
  );

  for (let i = 2; i <= 6; i += 2) {
    ctx.beginPath();
    ctx.arc(
      -(length / 2 + milesToFeet * feetPerPixel * i),
      -(1 / 2),
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
    let spacing = 16;
    let fontSize = 15;

    ctx.textAlign = 'left';
    ctx.fillStyle = '#55ff55';
    ctx.font = `900 ${fontSize}px monospace`;
    ctx.beginPath();
    ctx.fillText(aircraft.callsign, aircraft.x + spacing, aircraft.y - spacing);
    ctx.fill();

    let altitudeIcon = '➡';
    if (aircraft.altitude < aircraft.target.altitude) {
      altitudeIcon = '⬈';
    } else if (aircraft.altitude > aircraft.target.altitude) {
      altitudeIcon = '⬊';
    }

    ctx.beginPath();
    ctx.fillText(
      Math.round(aircraft.altitude / 100).toString() +
        altitudeIcon +
        Math.round(aircraft.target.altitude / 100).toString(),
      aircraft.x + spacing,
      aircraft.y - spacing + fontSize
    );
    ctx.fill();

    ctx.beginPath();
    ctx.fillText(
      Math.round(aircraft.heading)
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

function calculatePerpendicularLine(
  linePoint: { x: number; y: number },
  lineDirection: number,
  testPoint: { x: number; y: number }
): { heading: number; distance: number } {
  // Convert line direction to radians
  const lineAngleRad = (lineDirection * Math.PI) / 180;

  // Calculate the vector of the line
  const lineVectorX = Math.cos(lineAngleRad);
  const lineVectorY = Math.sin(lineAngleRad);

  // Calculate the vector from the line point to the test point
  const vectorToTestX = testPoint.x - linePoint.x;
  const vectorToTestY = testPoint.y - linePoint.y;

  // Calculate the dot product
  const dotProduct = vectorToTestX * lineVectorX + vectorToTestY * lineVectorY;

  // Calculate the closest point on the line to the test point
  const closestPointX = linePoint.x + dotProduct * lineVectorX;
  const closestPointY = linePoint.y + dotProduct * lineVectorY;

  // Calculate the vector from the test point to the closest point
  const perpendicularVectorX = closestPointX - testPoint.x;
  const perpendicularVectorY = closestPointY - testPoint.y;

  // Calculate the angle of this vector
  let perpendicularAngle = Math.atan2(
    perpendicularVectorY,
    perpendicularVectorX
  );

  // Convert to degrees and normalize to 0-360 range
  let perpendicularHeading = (perpendicularAngle * 180) / Math.PI;
  perpendicularHeading = (perpendicularHeading + 360) % 360;

  // Calculate the distance using the Pythagorean theorem
  const perpendicularDistance = Math.sqrt(
    perpendicularVectorX * perpendicularVectorX +
      perpendicularVectorY * perpendicularVectorY
  );

  return { heading: perpendicularHeading, distance: perpendicularDistance };
}

function calcDeltaAngle(current: number, target: number): number {
  return ((target - current + 540.0) % 360.0) - 180.0;
}

function inverseDegrees(degrees: number): number {
  return (degrees + 180) % 360;
}
