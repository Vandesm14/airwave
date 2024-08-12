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

    drawBlip(ctx, {
      x: width / 2,
      y: height / 2,
      heading: 0,
    });
  }
}

function drawBlip(ctx: CanvasRenderingContext2D, aircraft: Aircraft) {
  ctx.fillStyle = '#00ff00';
  ctx.strokeStyle = '#00ff00';
  ctx.beginPath();
  ctx.arc(aircraft.x, aircraft.y, 3, 0, Math.PI * 2);
  ctx.fill();

  ctx.beginPath();
  ctx.arc(aircraft.x, aircraft.y, 15, 0, Math.PI * 2);
  ctx.stroke();

  function drawDirection(ctx: CanvasRenderingContext2D, aircraft: Aircraft) {
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
