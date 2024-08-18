import { useAtom } from 'solid-jotai';
import {
  airspaceSizeAtom,
  radarAtom,
  renderAtom,
  runwaysAtom,
  taxiwaysAtom,
  terminalsAtom,
} from './lib/atoms';
import { Aircraft, Gate, Runway, Taxiway, Terminal, Vec2 } from './lib/types';
import {
  degreesToHeading,
  toRadians,
  headingToDegrees,
  feetPerPixel,
  knotToFeetPerSecond,
  nauticalMilesToFeet,
  runwayInfo,
  movePoint,
  projectPoint,
  angleBetweenPoints,
  midpointBetweenPoints,
} from './lib/lib';
import { Accessor, createEffect, createMemo, onMount } from 'solid-js';

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
  let [taxiways] = useAtom(taxiwaysAtom);
  let [terminals] = useAtom(terminalsAtom);
  let [render, setRender] = useAtom(renderAtom);
  let groundScale = createMemo(() => (radar().mode === 'ground' ? 10 : 1));

  createEffect(() => {
    setRadar((radar) => {
      radar.shiftPoint = {
        x: airspaceSize() * 0.5,
        y: airspaceSize() * 0.5,
      };

      return { ...radar };
    });
  });

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

          return { ...radar };
        });
      });
      canvas.addEventListener('mouseup', (_) => {
        setRadar((radar) => {
          radar.isDragging = false;
          return { ...radar };
        });
      });
      canvas.addEventListener('mousemove', (e) => {
        if (radar().isDragging) {
          setRadar((radar) => {
            let x = e.clientX - radar.dragStartPoint.x + radar.lastShiftPoint.x;
            let y = e.clientY - radar.dragStartPoint.y + radar.lastShiftPoint.y;

            radar.shiftPoint.x = x;
            radar.shiftPoint.y = y;

            return { ...radar };
          });
        }
      });
      canvas.addEventListener('wheel', (e) => {
        setRadar((radar) => {
          radar.scale += e.deltaY * -0.0005;
          radar.scale = Math.max(Math.min(radar.scale, 2), 0.6);

          radar.isZooming = true;

          return { ...radar };
        });
      });
    }
  });

  function loopMain(canvas: HTMLCanvasElement, forceRender?: boolean) {
    let dt = Date.now() - render().lastTime;
    let dts = dt / 1000;

    let deltaDrawTime = Date.now() - render().lastDraw;
    if (
      forceRender ||
      radar().isDragging ||
      radar().isZooming ||
      render().lastDraw === 0 ||
      deltaDrawTime >= 1000 / 3
    ) {
      loopDraw(canvas, dts);
      setRadar((radar) => {
        radar.isZooming = false;
        return { ...radar };
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

  function resetTransform(ctx: CanvasRenderingContext2D) {
    ctx.resetTransform();
    ctx.translate(radar().shiftPoint.x, radar().shiftPoint.y);
    ctx.scale(radar().scale, radar().scale);
    ctx.translate(airspaceSize() * -0.5, airspaceSize() * -0.5);
  }

  function loopDraw(canvas: HTMLCanvasElement, dts: number) {
    const width = canvas.width;
    const height = canvas.height;

    let ctx = canvas.getContext('2d');
    if (ctx) {
      const fontSize = 16 * (1 / radar().scale);
      ctx.font = `900 ${fontSize}px monospace`;
      ctx.fillStyle = 'black';

      ctx.fillRect(0, 0, width, height);
      resetTransform(ctx);
      drawCompass(ctx);

      if (radar().mode === 'tower') {
        for (let runway of runways()) {
          drawRunway(ctx, runway);
        }

        for (let aircraft of aircrafts()) {
          if (aircraft.state.type !== 'taxiing') {
            drawBlip(ctx, aircraft);
          }
        }
      } else if (radar().mode === 'ground') {
        for (let taxiway of taxiways()) {
          drawTaxiway(ctx, taxiway);
        }

        for (let runway of runways()) {
          drawRunwayGround(ctx, runway);
        }

        for (let terminal of terminals()) {
          drawTerminal(ctx, terminal);
        }

        for (let aircraft of aircrafts()) {
          if (aircraft.altitude < 1000) {
            drawBlipGround(ctx, aircraft);
          }
        }
      }
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

      ctx.fillText(
        text,
        Math.cos(toRadians(i * 10)) * (airspace_radius + 20) + half_size,
        Math.sin(toRadians(i * 10)) * (airspace_radius + 20) + half_size
      );
    }
  }

  function drawRunway(ctx: Ctx, runway: Runway) {
    let width = 4;
    let info = runwayInfo(runway);

    let startLeft = movePoint(
      info.start.x,
      info.start.y,
      width * 0.5,
      (headingToDegrees(runway.heading) + 270) % 360
    );
    let startRight = movePoint(
      info.start.x,
      info.start.y,
      width * 0.5,
      (headingToDegrees(runway.heading) + 90) % 360
    );

    let endLeft = movePoint(
      info.end.x,
      info.end.y,
      width * 0.5,
      (headingToDegrees(runway.heading) + 270) % 360
    );
    let endRight = movePoint(
      info.end.x,
      info.end.y,
      width * 0.5,
      (headingToDegrees(runway.heading) + 90) % 360
    );

    ctx.fillStyle = 'grey';
    ctx.beginPath();
    ctx.moveTo(startLeft.x, startLeft.y);
    ctx.lineTo(startRight.x, startRight.y);
    ctx.lineTo(endRight.x, endRight.y);
    ctx.lineTo(endLeft.x, endLeft.y);
    ctx.lineTo(startLeft.x, startLeft.y);
    ctx.fill();

    ctx.fillStyle = '#3087f2';
    ctx.strokeStyle = '#3087f2';

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

  function drawRunwayGround(ctx: Ctx, runway: Runway) {
    let origin: Vec2 = {
      x: airspaceSize() * 0.5,
      y: airspaceSize() * 0.5,
    };
    let width = feetPerPixel * 250;
    let projectionScale = 14;

    let info = runwayInfo(runway);

    let startLeft = projectPoint(
      origin,
      movePoint(
        info.start.x,
        info.start.y,
        width * 0.5,
        (headingToDegrees(runway.heading) + 270) % 360
      ),
      projectionScale
    );
    let startRight = projectPoint(
      origin,
      movePoint(
        info.start.x,
        info.start.y,
        width * 0.5,
        (headingToDegrees(runway.heading) + 90) % 360
      ),
      projectionScale
    );

    let endLeft = projectPoint(
      origin,
      movePoint(
        info.end.x,
        info.end.y,
        width * 0.5,
        (headingToDegrees(runway.heading) + 270) % 360
      ),
      projectionScale
    );
    let endRight = projectPoint(
      origin,
      movePoint(
        info.end.x,
        info.end.y,
        width * 0.5,
        (headingToDegrees(runway.heading) + 90) % 360
      ),
      projectionScale
    );

    ctx.fillStyle = '#666';
    ctx.beginPath();
    ctx.moveTo(startLeft.x, startLeft.y);
    ctx.lineTo(startRight.x, startRight.y);
    ctx.lineTo(endRight.x, endRight.y);
    ctx.lineTo(endLeft.x, endLeft.y);
    ctx.lineTo(startLeft.x, startLeft.y);
    ctx.fill();

    let fontSize = 16;
    ctx.font = `900 ${fontSize}px monospace`;
    ctx.textAlign = 'center';
    let middle = midpointBetweenPoints(startLeft, startRight);
    let textWidth = ctx.measureText(runway.id).width + 10;
    ctx.fillStyle = '#000a';
    ctx.fillRect(
      middle.x - textWidth * 0.5,
      middle.y - fontSize * 0.5,
      textWidth,
      fontSize
    );

    ctx.fillStyle = '#dd9904';
    ctx.fillText(runway.id, middle.x, middle.y);
  }

  function drawTaxiway(ctx: Ctx, taxiway: Taxiway) {
    let origin: Vec2 = {
      x: airspaceSize() * 0.5,
      y: airspaceSize() * 0.5,
    };
    let width = feetPerPixel * 200;
    let projectionScale = 14;

    let angle = angleBetweenPoints(taxiway.a, taxiway.b);

    let startLeft = projectPoint(
      origin,
      movePoint(taxiway.a.x, taxiway.a.y, width * 0.5, (angle + 270) % 360),
      projectionScale
    );
    let startRight = projectPoint(
      origin,
      movePoint(taxiway.a.x, taxiway.a.y, width * 0.5, (angle + 90) % 360),
      projectionScale
    );

    let endLeft = projectPoint(
      origin,
      movePoint(taxiway.b.x, taxiway.b.y, width * 0.5, (angle + 270) % 360),
      projectionScale
    );
    let endRight = projectPoint(
      origin,
      movePoint(taxiway.b.x, taxiway.b.y, width * 0.5, (angle + 90) % 360),
      projectionScale
    );

    ctx.fillStyle = '#999';
    ctx.beginPath();
    ctx.moveTo(startLeft.x, startLeft.y);
    ctx.lineTo(startRight.x, startRight.y);
    ctx.lineTo(endRight.x, endRight.y);
    ctx.lineTo(endLeft.x, endLeft.y);
    ctx.lineTo(startLeft.x, startLeft.y);
    ctx.fill();

    if (taxiway.kind.type === 'normal') {
      let start = projectPoint(origin, taxiway.a, projectionScale);
      let end = projectPoint(origin, taxiway.b, projectionScale);
      let middle = midpointBetweenPoints(start, end);

      let fontSize = 16;
      ctx.font = `900 ${fontSize}px monospace`;
      let textWidth = ctx.measureText(taxiway.id).width + 10;
      ctx.fillStyle = '#000a';
      ctx.fillRect(
        middle.x - textWidth * 0.5,
        middle.y - fontSize * 0.5,
        textWidth,
        fontSize
      );

      ctx.textAlign = 'center';
      ctx.fillStyle = '#dd9904';
      ctx.fillText(taxiway.id, middle.x, middle.y);
    }
  }

  function drawTerminal(ctx: Ctx, terminal: Terminal) {
    let origin: Vec2 = {
      x: airspaceSize() * 0.5,
      y: airspaceSize() * 0.5,
    };
    let projectionScale = 14;

    let a = projectPoint(origin, terminal.a, projectionScale);
    let b = projectPoint(origin, terminal.b, projectionScale);

    let c = projectPoint(origin, terminal.c, projectionScale);
    let d = projectPoint(origin, terminal.d, projectionScale);

    ctx.fillStyle = '#999';
    ctx.beginPath();
    ctx.moveTo(a.x, a.y);
    ctx.lineTo(b.x, b.y);
    ctx.lineTo(c.x, c.y);
    ctx.lineTo(d.x, d.y);
    ctx.lineTo(a.x, a.y);
    ctx.fill();

    function drawGate(ctx: Ctx, gate: Gate, id: string) {
      let pos = projectPoint(origin, gate.pos, projectionScale);

      ctx.fillStyle = 'red';
      ctx.beginPath();
      ctx.arc(pos.x, pos.y, 5, 0, Math.PI * 2);
      ctx.fill();

      let fontSize = 16;
      ctx.font = `900 ${fontSize}px monospace`;
      ctx.textAlign = 'center';
      let textWidth = ctx.measureText(id).width + 10;
      ctx.fillStyle = '#000a';
      ctx.fillRect(
        pos.x - textWidth * 0.5,
        pos.y - fontSize * 0.5 - fontSize,
        textWidth,
        fontSize
      );

      ctx.fillStyle = '#dd9904';
      ctx.fillText(id, pos.x, pos.y - fontSize);
    }

    for (let i = 0; i < terminal.gates.length; i++) {
      let gate = terminal.gates[i];
      drawGate(ctx, gate, `${terminal.id}${i}`);
    }
  }

  function drawBlipGround(ctx: Ctx, aircraft: Aircraft) {
    let origin: Vec2 = {
      x: airspaceSize() * 0.5,
      y: airspaceSize() * 0.5,
    };
    let projectionScale = 14;

    ctx.fillStyle = '#ffff00';
    ctx.strokeStyle = '#ffff00';

    let pos = projectPoint(origin, aircraft, projectionScale);

    ctx.beginPath();
    ctx.arc(pos.x, pos.y, 3, 0, Math.PI * 2);
    ctx.fill();

    ctx.beginPath();
    ctx.arc(
      pos.x,
      pos.y,
      nauticalMilesToFeet * feetPerPixel * 0.4,
      0,
      Math.PI * 2
    );
    ctx.stroke();

    function drawDirection(ctx: Ctx, aircraft: Aircraft) {
      const angleDegrees = (aircraft.heading + 270) % 360;
      const angleRadians = angleDegrees * (Math.PI / 180);
      const length = 30;
      const endX = pos.x + length * Math.cos(angleRadians);
      const endY = pos.y + length * Math.sin(angleRadians);

      ctx.strokeStyle = '#ffff00';
      ctx.beginPath();
      ctx.moveTo(pos.x, pos.y);
      ctx.lineTo(endX, endY);
      ctx.stroke();
    }

    drawDirection(ctx, aircraft);

    let spacing = 16;
    ctx.font = `900 ${spacing}px monospace`;
    ctx.textAlign = 'left';
    ctx.fillStyle = '#ffff00';
    ctx.fillText(aircraft.callsign, pos.x + spacing, pos.y - spacing);

    let altitudeIcon = ' ';
    if (aircraft.altitude < aircraft.target.altitude) {
      altitudeIcon = '⬈';
    } else if (aircraft.altitude > aircraft.target.altitude) {
      altitudeIcon = '⬊';
    }

    ctx.fillText(
      Math.round(aircraft.altitude / 100)
        .toString()
        .padStart(3, '0') +
        altitudeIcon +
        Math.round(aircraft.target.altitude / 100)
          .toString()
          .padStart(3, '0'),
      pos.x + spacing,
      pos.y - spacing + spacing
    );

    ctx.fillText(
      Math.round(aircraft.speed).toString(),
      pos.x + spacing,
      pos.y - spacing + spacing * 2
    );

    if (aircraft.state.type === 'taxiing') {
      ctx.strokeStyle = 'red';
      ctx.fillStyle = 'red';
      ctx.beginPath();
      ctx.moveTo(pos.x, pos.y);
      for (let wp of aircraft.state.value.waypoints.slice().reverse()) {
        let point = projectPoint(origin, wp.pos, projectionScale);
        ctx.lineTo(point.x, point.y);
      }
      ctx.stroke();

      for (let wp of aircraft.state.value.waypoints.slice().reverse()) {
        let point = projectPoint(origin, wp.pos, projectionScale);
        ctx.fillStyle = wp.hold ? 'red' : '#00ff00';
        ctx.beginPath();
        ctx.arc(point.x, point.y, 2, 0, Math.PI * 2);
        ctx.fill();
      }
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
      const fontSize = 16 * (1 / radar().scale);
      ctx.font = `900 ${fontSize}px monospace`;

      ctx.textAlign = 'left';
      ctx.fillStyle =
        aircraft.intention.type === 'depart' ||
        aircraft.intention.type === 'flyover'
          ? '#fc67eb'
          : '#44ff44';
      ctx.fillText(
        aircraft.callsign,
        aircraft.x + spacing,
        aircraft.y - spacing
      );

      let altitudeIcon = ' ';
      if (aircraft.altitude < aircraft.target.altitude) {
        altitudeIcon = '⬈';
      } else if (aircraft.altitude > aircraft.target.altitude) {
        altitudeIcon = '⬊';
      }

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

      let targetHeadingInfo =
        aircraft.state.type === 'landing'
          ? 'ILS'
          : Math.round(aircraft.target.heading)
              .toString()
              .padStart(3, '0')
              .replace('360', '000');
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

      ctx.fillText(
        Math.round(aircraft.speed).toString(),
        aircraft.x + spacing,
        aircraft.y - spacing + fontSize * 3
      );
    }

    drawDirection(ctx, aircraft);
    drawInfo(ctx, aircraft);
  }

  return <canvas id="canvas" ref={canvas}></canvas>;
}
