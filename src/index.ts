type Ctx = CanvasRenderingContext2D;

const feetPerPixel = 0.005;

const canvas = document.getElementById('canvas');
if (canvas instanceof HTMLCanvasElement && canvas !== null) {
  window.addEventListener('resize', () => {
    canvas.width = canvas.clientWidth;
    canvas.height = canvas.clientHeight;

    draw(canvas);
  });

  canvas.width = canvas.clientWidth;
  canvas.height = canvas.clientHeight;

  draw(canvas);
}

type Aircraft = {
  x: number;
  y: number;
  heading: number;
};

function draw(canvas: HTMLCanvasElement) {
  const width = canvas.width;
  const height = canvas.height;

  let ctx = canvas.getContext('2d');
  if (ctx) {
    ctx.fillStyle = 'black';
    ctx.fillRect(0, 0, width, height);

    let circle = drawCompass(ctx, width, height);
    drawRunway(ctx, width, height);

    let result = getRandomPointOnCircle(circle.x, circle.y, circle.r + 25);
    let heading = (getAngle(result.x, result.y, circle.x, circle.y) + 90) % 360;
    let aircraft: Aircraft = {
      x: result.x,
      y: result.y,
      heading,
    };

    drawBlip(ctx, aircraft);
  }
}

function drawCompass(ctx: Ctx, width: number, height: number) {
  ctx.strokeStyle = 'white';

  let x = width / 2;
  let y = height / 2;
  let radius = x;
  if (height < width) {
    radius = y;
  }

  radius -= 50;

  ctx.beginPath();
  ctx.arc(x, y, radius, 0, Math.PI * 2);
  ctx.stroke();

  return {
    x,
    y,
    r: radius,
  };
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
