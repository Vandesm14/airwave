import { WhisperSTT } from '../vendor/whisper-speech-to-text/src/';

const whisper = new WhisperSTT();
let isRecording = false;

type Ctx = CanvasRenderingContext2D;

const timeScale = 1;
let scale = 1;

const feetPerPixel = 0.005 * scale;
const nauticalMilesToFeet = 6076.115;
const knotToFeetPerSecond = 1.68781 * timeScale;
const milesToFeet = 6076.12;

let shiftPoint = {
  x: 0,
  y: 0,
};

function headingToDegrees(heading: number) {
  return (heading + 270) % 360;
}

function degreesToHeading(degrees: number) {
  return (degrees + 360 + 90) % 360;
}

function speak(text: string) {
  if ('speechSynthesis' in window) {
    if (window.speechSynthesis.speaking || isRecording) {
      setTimeout(() => speak(text), 500);
    } else {
      const utterance = new SpeechSynthesisUtterance(
        text.replace(/[0-9]/g, '$& ')
      );
      utterance.volume = 0.01;
      utterance.rate = 1.0;
      utterance.pitch = 1.3;
      window.speechSynthesis.speak(utterance);
    }
  } else {
    console.log("Sorry, your browser doesn't support text to speech!");
  }
}

const airlines: Record<string, string> = {
  AAL: 'American Airlines',
  SKW: 'Sky West',
  JBL: 'Jet Blue',
};

const toRadians = (degrees: number) => (degrees * Math.PI) / 180;

type Aircraft = {
  pos: [number, number];
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
  pos: [number, number];
  x: number;
  y: number;
  /** In Degrees (0 is north; up) */
  heading: number;
  /** In Feet */
  length: number;
};

let airspace_size = 1000;
let aircrafts: Array<Aircraft> = [];
let runways: Array<Runway> = [];
let lastTime = Date.now();
let lastDraw = 0;

let isDragging = false;
let isZooming = false;
let dragStartPoint = {
  x: 0,
  y: 0,
};
let oldShiftPoint = {
  x: 0,
  y: 0,
};

const chatbox = document.getElementById('chatbox');
const messageTemplate = document.getElementById('message-template');

const canvas = document.getElementById('canvas');
if (canvas instanceof HTMLCanvasElement && canvas !== null) {
  window.addEventListener('resize', () => {
    canvas.width = canvas.clientWidth;
    canvas.height = canvas.clientHeight;
  });

  setInterval(() => loopMain(canvas), 1000 / 30);

  canvas.width = canvas.clientWidth;
  canvas.height = canvas.clientHeight;

  loopMain(canvas);
}

function callsignString(id: string): string {
  return `${airlines[id.slice(0, 3)]} ${id.slice(3, 7)}`;
}

function speakAsAircraft(
  callsign: string,
  reply: string,
  withCallsign?: boolean
) {
  if (
    chatbox instanceof HTMLDivElement &&
    messageTemplate instanceof HTMLTemplateElement
  ) {
    let fullReply = withCallsign
      ? `${reply}, ${callsignString(callsign)}`
      : reply;

    let message = messageTemplate.innerHTML
      .replace('{{callsign}}', callsign)
      .replace('{{text}}', fullReply);

    chatbox.insertAdjacentHTML('beforeend', message);
    chatbox.scrollTo(0, chatbox.scrollHeight);

    speak(fullReply);
  }
}

function speakAsATC(reply: string) {
  if (
    chatbox instanceof HTMLDivElement &&
    messageTemplate instanceof HTMLTemplateElement
  ) {
    let message = messageTemplate.innerHTML
      .replace('{{callsign}}', 'ATC')
      .replace('{{text}}', reply);

    chatbox.insertAdjacentHTML('beforeend', message);
    chatbox.scrollTo(0, chatbox.scrollHeight);
  }
}

let socket = new WebSocket(`ws://${window.location.hostname}:9001`);
socket.onopen = function (_) {
  console.log('[open] Connection established');
  console.log('Sending to server');

  socket.send(JSON.stringify({ type: 'connect' }));
};

type Event =
  | {
      type: 'aircraft';
      value: Aircraft[];
    }
  | { type: 'runways'; value: Runway[] }
  | { type: 'atcreply'; value: string }
  | { type: 'reply'; value: { id: string; reply: string } }
  | { type: 'size'; value: number };

function posToXY<T extends { pos: [number, number]; x: number; y: number }>(
  obj: T
): T {
  obj.x = obj.pos[0];
  obj.y = obj.pos[1];

  return obj;
}

socket.onmessage = function (event) {
  // console.log(`[message] Data received from server: ${event.data}`);

  let json: Event = JSON.parse(event.data);
  switch (json.type) {
    case 'aircraft':
      aircrafts = json.value.map(posToXY);
      break;
    case 'runways':
      runways = json.value.map(posToXY);
      break;
    case 'atcreply':
      speakAsATC(json.value);
      break;
    case 'reply':
      speakAsAircraft(json.value.id, json.value.reply, true);
      break;
    case 'size':
      console.log('size', json.value);
      airspace_size = json.value;
      break;
  }
};

socket.onclose = function (event: { wasClean: any; code: any; reason: any }) {
  if (event.wasClean) {
    console.log(
      `[close] Connection closed cleanly, code=${event.code} reason=${event.reason}`
    );
  } else {
    // e.g. server process killed or network down
    // event.code is usually 1006 in this case
    console.log('[close] Connection died');
  }
};

socket.onerror = function () {
  console.log(`[error]`);
};

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

    whisper.stopRecording((blob) => {
      blob.arrayBuffer().then((value) => {
        console.log('send voice request');

        socket.send(
          JSON.stringify({
            type: 'voice',
            value: [...new Uint8Array(value)],
          })
        );

        console.log('sent voice request');
      });
    });
  }

  if (chatbox instanceof HTMLDivElement) {
    if (isRecording) {
      chatbox.classList.add('live');
    } else {
      chatbox.classList.remove('live');
    }
  }
});

canvas?.addEventListener('mousedown', (e) => {
  isDragging = true;
  dragStartPoint = {
    x: e.clientX,
    y: e.clientY,
  };
  oldShiftPoint = {
    x: shiftPoint.x,
    y: shiftPoint.y,
  };
});
canvas?.addEventListener('mouseup', (e) => (isDragging = false));
canvas?.addEventListener('mousemove', (e) => {
  if (isDragging) {
    let x = e.clientX - dragStartPoint.x + oldShiftPoint.x;
    let y = e.clientY - dragStartPoint.y + oldShiftPoint.y;

    shiftPoint = { x, y };
  }
});
canvas?.addEventListener('wheel', (e) => {
  scale += e.deltaY * -0.0005;
  scale = Math.max(Math.min(scale, 2), 0.6);
  isZooming = true;
});

function loopMain(canvas: HTMLCanvasElement) {
  let airspace = calcAirspace(airspace_size, airspace_size);

  let dt = Date.now() - lastTime;
  lastTime = Date.now();
  let dts = dt / 1000;

  let deltaDrawTime = Date.now() - lastDraw;
  if (isDragging || isZooming || lastDraw === 0 || deltaDrawTime >= 1000 / 3) {
    lastDraw = Date.now();
    loopDraw(canvas, airspace, dts);
    isZooming = false;
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

    ctx.translate(shiftPoint.x, shiftPoint.y);
    ctx.scale(scale, scale);
    drawCompass(ctx, airspace);

    for (let runway of runways) {
      drawRunway(ctx, runway);
    }

    for (let aircraft of aircrafts) {
      drawBlip(ctx, aircraft);
    }

    ctx.resetTransform();

    ctx.fillStyle = '#009900';
    ctx.fillText(`${Math.round(1 / dts)} fps`, 10, 20);
  }
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
  ctx.translate(shiftPoint.x, shiftPoint.y);
  ctx.scale(scale, scale);

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

function inverseDegrees(degrees: number): number {
  return (degrees + 180) % 360;
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
