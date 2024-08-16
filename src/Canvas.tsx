import { useAtom } from 'solid-jotai';
import {
  airspaceSizeAtom,
  radarAtom,
  renderAtom,
  runwaysAtom,
} from './lib/atoms';
import { Aircraft, Runway } from './lib/types';
import {
  degreesToHeading,
  toRadians,
  headingToDegrees,
  feetPerPixel,
  knotToFeetPerSecond,
  nauticalMilesToFeet,
  runwayInfo,
} from './lib/lib';
import { Accessor, onMount } from 'solid-js';

export default function Canvas({
  aircrafts,
}: {
  aircrafts: Accessor<Array<Aircraft>>;
}) {
  let canvas;

  type Ctx = CanvasRenderingContext2D;

  let [radar, setRadar] = useAtom(radarAtom);

  let [airspaceSize] = useAtom(airspaceSizeAtom);
  let [runways] = useAtom(runwaysAtom);
  let [render, setRender] = useAtom(renderAtom);

  onMount(() => {
    if (canvas instanceof HTMLCanvasElement && canvas !== null) {
      window.addEventListener('resize', () => {
        canvas.width = canvas.clientWidth;
        canvas.height = canvas.clientHeight;
      });

      setInterval(() => loopMain(canvas), 1000 / 30);

      canvas.width = canvas.clientWidth;
      canvas.height = canvas.clientHeight;

      loopMain(canvas);

      canvas.addEventListener('mousedown', (e) => {
        setRadar((radar) => {
          radar.isDragging = true;
          radar.dragStartPoint = {
            x: e.clientX,
            y: e.clientY,
          };

          radar.lastShiftPoint.x = radar.shiftPoint.x;
          radar.lastShiftPoint.y = radar.shiftPoint.y;

          return radar;
        });
      });
      canvas.addEventListener('mouseup', (_) => {
        setRadar((radar) => {
          radar.isDragging = false;
          return radar;
        });
      });
      canvas.addEventListener('mousemove', (e) => {
        if (radar().isDragging) {
          setRadar((radar) => {
            let x = e.clientX - radar.dragStartPoint.x + radar.lastShiftPoint.x;
            let y = e.clientY - radar.dragStartPoint.y + radar.lastShiftPoint.y;

            radar.shiftPoint.x = x;
            radar.shiftPoint.y = y;

            return radar;
          });
        }
      });
      canvas.addEventListener('wheel', (e) => {
        setRadar((radar) => {
          radar.scale += e.deltaY * -0.0005;
          radar.scale = Math.max(Math.min(radar.scale, 2), 0.6);

          radar.isZooming = true;

          return radar;
        });
      });
    }
  });

  function loopMain(canvas: HTMLCanvasElement) {
    let dt = Date.now() - render().lastTime;
    let dts = dt / 1000;

    let deltaDrawTime = Date.now() - render().lastDraw;
    if (
      radar().isDragging ||
      radar().isZooming ||
      render().lastDraw === 0 ||
      deltaDrawTime >= 1000 / 3
    ) {
      loopDraw(canvas, dts);
      setRadar((radar) => {
        radar.isZooming = false;
        return radar;
      });
      setRender((render) => {
        render.lastDraw = Date.now();
        return render;
      });
    }

    setRender((render) => {
      render.lastTime = Date.now();
      return render;
    });
  }

  function loopDraw(canvas: HTMLCanvasElement, dts: number) {
    const width = canvas.width;
    const height = canvas.height;

    let ctx = canvas.getContext('2d');
    if (ctx) {
      const fontSize = 15 * (1 / radar().scale);
      ctx.font = `900 ${fontSize}px monospace`;
      ctx.fillStyle = 'black';
      ctx.fillRect(0, 0, width, height);

      ctx.translate(radar().shiftPoint.x, radar().shiftPoint.y);
      ctx.scale(radar().scale, radar().scale);
      drawCompass(ctx);

      for (let runway of runways()) {
        drawRunway(ctx, runway);
      }

      for (let aircraft of aircrafts()) {
        drawBlip(ctx, aircraft);
      }

      ctx.resetTransform();

      ctx.fillStyle = '#009900';
      ctx.fillText(`${Math.round(1 / dts)} fps`, 10, 20);
    }
  }

  function drawCompass(ctx: Ctx) {
    let half_size = airspaceSize() * 0.5;
    let airspace_radius = half_size - 50;

    ctx.strokeStyle = 'white';
    ctx.fillStyle = 'white';
    ctx.beginPath();
    ctx.arc(half_size, half_size, airspace_radius, 0, Math.PI * 2);
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
        Math.cos(toRadians(i * 10)) * (airspace_radius + 20) + half_size,
        Math.sin(toRadians(i * 10)) * (airspace_radius + 20) + half_size
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
    ctx.translate(radar().shiftPoint.x, radar().shiftPoint.y);
    ctx.scale(radar().scale, radar().scale);

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
      const fontSize = 15 * (1 / radar().scale);

      ctx.textAlign = 'left';
      ctx.fillStyle = '#44ff44';
      ctx.beginPath();
      ctx.fillText(
        aircraft.callsign,
        aircraft.x + spacing,
        aircraft.y - spacing
      );
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

      let targetHeadingInfo =
        aircraft.state.type === 'landing'
          ? 'ILS'
          : Math.round(aircraft.target.heading)
              .toString()
              .padStart(3, '0')
              .replace('360', '000');
      ctx.beginPath();
      ctx.fillText(
        Math.round(aircraft.heading)
          .toString()
          .padStart(3, '0')
          .replace('360', '000') +
          ' ' +
          targetHeadingInfo,
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

  return <canvas id="canvas" ref={canvas}></canvas>;
}
