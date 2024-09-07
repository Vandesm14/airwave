import { useAtom } from 'solid-jotai';
import {
  radarAtom,
  renderAtom,
  selectedAircraftAtom,
  worldAtom,
} from './lib/atoms';
import {
  Aircraft,
  Airspace,
  arrToVec2,
  Gate,
  Runway,
  Taxiway,
  Terminal,
  Vec2,
  World,
} from './lib/types';
import {
  Accessor,
  createEffect,
  createMemo,
  createSignal,
  onMount,
} from 'solid-js';
import {
  headingToDegrees,
  knotToFeetPerSecond,
  midpointBetweenPoints,
  movePoint,
  nauticalMilesToFeet,
  runwayInfo,
  toRadians,
} from './lib/lib';

const groundScale = 5.0;

export default function Canvas({
  aircrafts,
}: {
  aircrafts: Accessor<Array<Aircraft>>;
}) {
  let canvas;

  type Ctx = CanvasRenderingContext2D;

  let [radar, setRadar] = useAtom(radarAtom);

  let [world] = useAtom(worldAtom);
  let [render, setRender] = useAtom(renderAtom);
  let [selectedAircraft] = useAtom(selectedAircraftAtom);
  let fontSize = createMemo(() => 16);
  let isGround = createMemo(() => radar().scale > groundScale);
  let [waitingForAircraft, setWaitingForAircraft] = createSignal(true);

  function scaleFeet(num: number): number {
    const FEET_TO_PIXELS = 0.003;
    return num * FEET_TO_PIXELS * radar().scale;
  }

  function scalePoint(vec2: Vec2): Vec2 {
    let x = scaleFeet(vec2.x);
    let y = scaleFeet(vec2.y);

    return {
      x: x,
      y: -y,
    };
  }

  createEffect(() => {
    setRadar((radar) => {
      radar.shiftPoint = {
        x: 0.0,
        y: 0.0,
      };

      return { ...radar };
    });
  });

  createEffect(() => {
    if (waitingForAircraft() && aircrafts().length > 0) {
      setRender((render) => {
        render.doInitialDraw = true;
        return { ...render };
      });
      setWaitingForAircraft(false);
    }
  });

  onMount(() => {
    if (canvas instanceof HTMLCanvasElement && canvas !== null) {
      window.addEventListener('resize', () => {
        canvas.width = canvas.clientWidth;
        canvas.height = canvas.clientHeight;
      });

      setInterval(() => doRender(canvas), 1000 / 30);

      canvas.width = canvas.clientWidth;
      canvas.height = canvas.clientHeight;

      doRender(canvas);

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
          let maxScale = 30.0;
          let minScale = 0.1;

          if (e.deltaY > 0) {
            radar.scale *= 0.9;
          } else {
            radar.scale *= 1.1;
          }

          // radar.scale += e.deltaY * -0.001;
          radar.scale = Math.max(Math.min(radar.scale, maxScale), minScale);

          radar.isZooming = true;

          return { ...radar };
        });
      });
    }
  });

  function doRender(canvas: HTMLCanvasElement) {
    doDraw(canvas);
    setRadar((radar) => {
      radar.isZooming = false;
      return { ...radar };
    });

    setRender((render) => {
      let now = Date.now();
      let duration = isGround() ? 1000 * 0.5 : 1000 * 4;

      if (now - render.lastDraw > duration || render.doInitialDraw) {
        render.lastDraw = now;
        render.aircrafts = aircrafts();

        render.doInitialDraw = false;
      }

      return { ...render };
    });
  }

  function resetTransform(ctx: CanvasRenderingContext2D) {
    ctx.resetTransform();
    ctx.translate(radar().shiftPoint.x, radar().shiftPoint.y);
    // Set 0.0 to be the center of the canvas
    ctx.translate(canvas.width * 0.5, canvas.height * 0.5);

    ctx.lineWidth = 1;
    ctx.strokeStyle = 'white';
    ctx.fillStyle = 'white';
    ctx.font = `900 ${fontSize()}px monospace`;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
  }

  function doDraw(canvas: HTMLCanvasElement) {
    const width = canvas.width;
    const height = canvas.height;

    let ctx = canvas.getContext('2d');
    if (ctx) {
      ctx.font = `900 16px monospace`;
      ctx.fillStyle = 'black';

      ctx.resetTransform();
      ctx.fillRect(0, 0, width, height);
      // drawCompass(ctx);
      resetTransform(ctx);

      if (isGround()) {
        drawGround(ctx, world(), render().aircrafts);
      } else {
        drawTower(ctx, world(), render().aircrafts);
      }
    }
  }

  function drawCompass(ctx: Ctx) {
    let diameter = canvas.height;
    if (canvas.width < canvas.height) {
      diameter = canvas.width;
    }

    let radius = diameter * 0.5;
    let origin = {
      x: canvas.width * 0.5,
      y: canvas.height * 0.5,
    };

    ctx.fillStyle = '#8887';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    let padding = -10;
    for (let i = 0; i < 36; i++) {
      let text = headingToDegrees(i * 10)
        .toString()
        .padStart(3, '0');
      if (text === '000') {
        text = '360';
      }
      ctx.fillText(
        text,
        Math.cos(toRadians(i * 10)) * (radius + padding) + origin.x,
        Math.sin(toRadians(i * 10)) * (radius + padding) + origin.y
      );
    }
  }

  function drawAirspace(ctx: Ctx, airspace: Airspace) {
    resetTransform(ctx);
    let pos = scalePoint(airspace.pos);

    ctx.strokeStyle = 'white';
    ctx.fillStyle = 'white';
    ctx.beginPath();
    ctx.arc(pos.x, pos.y, scaleFeet(airspace.size), 0, Math.PI * 2);
    ctx.stroke();
  }

  function drawRunway(ctx: Ctx, runway: Runway) {
    resetTransform(ctx);
    let info = runwayInfo(runway);
    let start = scalePoint(info.start);
    let end = scalePoint(info.end);
    let ils = {
      end: scalePoint(info.ils.end),
      maxAngle: scalePoint(info.ils.maxAngle),
      minAngle: scalePoint(info.ils.minAngle),
    };

    // Draw the runway
    ctx.strokeStyle = 'grey';
    ctx.fillStyle = 'grey';
    ctx.lineWidth = scaleFeet(1000);
    ctx.beginPath();
    ctx.moveTo(start.x, start.y);
    ctx.lineTo(end.x, end.y);
    ctx.stroke();

    // Draw the localizer beacon
    ctx.fillStyle = '#3087f2';
    ctx.strokeStyle = '#3087f2';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(start.x, start.y);
    ctx.lineTo(ils.end.x, ils.end.y);
    ctx.stroke();

    // Draw the max and min localizer angle
    ctx.strokeStyle = '#444444';
    ctx.beginPath();
    ctx.moveTo(start.x, start.y);
    ctx.lineTo(ils.maxAngle.x, ils.maxAngle.y);
    ctx.stroke();

    ctx.beginPath();
    ctx.moveTo(start.x, start.y);
    ctx.lineTo(ils.minAngle.x, ils.minAngle.y);
    ctx.stroke();

    // Draw the localizer altitude points
    ctx.strokeStyle = '#3087f2';
    for (let p of info.ils.altitudePoints) {
      let point = scalePoint(p);
      ctx.beginPath();
      ctx.arc(point.x, point.y, scaleFeet(1500), 0, Math.PI * 2);
      ctx.stroke();
    }
  }

  function drawBlip(ctx: Ctx, aircraft: Aircraft) {
    resetTransform(ctx);
    let pos = scalePoint(aircraft);

    if (selectedAircraft() == aircraft.callsign) {
      ctx.fillStyle = '#aaaa00';
      ctx.strokeStyle = '#aaaa00';
    } else {
      ctx.fillStyle = '#00aa00';
      ctx.strokeStyle = '#00aa00';
    }

    // Draw the dot
    ctx.beginPath();
    ctx.arc(pos.x, pos.y, Math.min(3, scaleFeet(1000)), 0, Math.PI * 2);
    ctx.fill();

    // Draw the separation circle
    ctx.beginPath();
    ctx.arc(pos.x, pos.y, scaleFeet(nauticalMilesToFeet * 0.8), 0, Math.PI * 2);
    ctx.stroke();

    // Draw the direction
    const length = aircraft.speed * knotToFeetPerSecond * 60;
    const end = movePoint(aircraft.x, aircraft.y, length, aircraft.heading);
    let endPos = scalePoint(end);

    if (selectedAircraft() == aircraft.callsign) {
      ctx.strokeStyle = '#aaaa00';
    } else {
      ctx.strokeStyle = '#00aa00';
    }
    ctx.beginPath();
    ctx.moveTo(pos.x, pos.y);
    ctx.lineTo(endPos.x, endPos.y);
    ctx.stroke();

    // Draw info
    let spacing = scaleFeet(nauticalMilesToFeet * 0.8);
    ctx.textAlign = 'left';
    ctx.fillStyle =
      aircraft.intention.type === 'depart' ||
      aircraft.intention.type === 'flyover'
        ? '#fc67eb'
        : '#44ff44';

    if (selectedAircraft() == aircraft.callsign) {
      ctx.fillStyle = '#FFE045';
    }

    // Draw callsign
    ctx.fillText(aircraft.callsign, pos.x + spacing, pos.y - spacing);

    // Draw altitude
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
      pos.y - spacing + fontSize()
    );

    // Draw heading
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
      pos.x + spacing,
      pos.y - spacing + fontSize() * 2
    );

    // Draw speed
    ctx.fillText(
      Math.round(aircraft.speed).toString(),
      pos.x + spacing,
      pos.y - spacing + fontSize() * 3
    );
  }

  function drawTerminal(ctx: Ctx, terminal: Terminal) {
    let a = scalePoint(terminal.a);
    let b = scalePoint(terminal.b);
    let c = scalePoint(terminal.c);
    let d = scalePoint(terminal.d);

    ctx.fillStyle = '#555';
    ctx.lineWidth = scaleFeet(200);
    ctx.beginPath();
    ctx.moveTo(a.x, a.y);
    ctx.lineTo(b.x, b.y);
    ctx.lineTo(c.x, c.y);
    ctx.lineTo(d.x, d.y);
    ctx.lineTo(a.x, a.y);
    ctx.fill();

    for (let i = 0; i < terminal.gates.length; i++) {
      let gate = terminal.gates[i];
      let id = `${terminal.id}${i + 1}`;
      drawGate(ctx, gate, id);
    }
  }

  function drawGate(ctx: Ctx, gate: Gate, id: string) {
    let pos = scalePoint(gate.pos);

    ctx.fillStyle = 'red';
    ctx.beginPath();
    ctx.arc(pos.x, pos.y, 5, 0, Math.PI * 2);
    ctx.fill();

    let fontSize = 16;
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

  function drawTaxiway(ctx: Ctx, taxiway: Taxiway) {
    resetTransform(ctx);
    let start = scalePoint(taxiway.a);
    let end = scalePoint(taxiway.b);

    ctx.strokeStyle = '#555';
    ctx.lineWidth = scaleFeet(200);
    ctx.beginPath();
    ctx.moveTo(start.x, start.y);
    ctx.lineTo(end.x, end.y);
    ctx.stroke();
  }

  function drawTaxiwayLabel(ctx: Ctx, taxiway: Taxiway) {
    if (taxiway.kind.type === 'normal') {
      let start = scalePoint(taxiway.a);
      let end = scalePoint(taxiway.b);
      let middle = midpointBetweenPoints(start, end);
      let textWidth = ctx.measureText(taxiway.id).width + 10;
      ctx.fillStyle = '#000a';
      ctx.fillRect(
        middle.x - textWidth * 0.5,
        middle.y - fontSize() * 0.5,
        textWidth,
        fontSize()
      );
      ctx.textAlign = 'center';
      ctx.fillStyle = '#dd9904';
      ctx.fillText(taxiway.id, middle.x, middle.y);
    }
  }

  function drawRunwayGround(ctx: Ctx, runway: Runway) {
    resetTransform(ctx);
    let info = runwayInfo(runway);
    let start = scalePoint(info.start);
    let end = scalePoint(info.end);

    ctx.strokeStyle = '#222';
    ctx.lineWidth = scaleFeet(250);
    ctx.beginPath();
    ctx.moveTo(start.x, start.y);
    ctx.lineTo(end.x, end.y);
    ctx.stroke();

    // Draw runway label
    let textWidth = ctx.measureText(runway.id).width + 10;
    ctx.fillStyle = '#000a';
    ctx.fillRect(
      start.x - textWidth * 0.5,
      start.y - fontSize() * 0.5,
      textWidth,
      fontSize()
    );
    ctx.textAlign = 'center';
    ctx.fillStyle = '#dd9904';
    ctx.fillText(runway.id, start.x, start.y);
  }

  function drawBlipGround(ctx: Ctx, aircraft: Aircraft) {
    resetTransform(ctx);
    let pos = scalePoint(aircraft);
    // let taxi_yellow = '#ffff00';
    let taxi_color = '#ffffff';

    if (
      aircraft.state.type === 'taxiing' &&
      selectedAircraft() == aircraft.callsign
    ) {
      ctx.strokeStyle = '#ffff00aa';
      ctx.lineWidth = scaleFeet(50);

      ctx.beginPath();
      ctx.moveTo(pos.x, pos.y);
      for (let wp of aircraft.state.value.waypoints.slice().reverse()) {
        let pos = scalePoint(arrToVec2(wp.value));
        ctx.lineTo(pos.x, pos.y);
      }
      ctx.stroke();

      for (let wp of aircraft.state.value.waypoints.slice().reverse()) {
        ctx.fillStyle = wp.behavior === 'goto' ? '#ffff00' : '#ff0000';
        let pos = scalePoint(arrToVec2(wp.value));
        ctx.beginPath();
        ctx.arc(pos.x, pos.y, 3, 0, Math.PI * 2);
        ctx.fill();
      }
    }

    resetTransform(ctx);

    ctx.fillStyle = taxi_color;
    ctx.strokeStyle = taxi_color;

    // Draw the dot
    ctx.beginPath();
    ctx.arc(pos.x, pos.y, scaleFeet(50), 0, Math.PI * 2);
    ctx.fill();

    // Draw the direction
    ctx.strokeStyle = `${taxi_color}`;
    ctx.lineWidth = 2;
    const length = 400;
    const end = movePoint(aircraft.x, aircraft.y, length, aircraft.heading);
    let endPos = scalePoint(end);

    ctx.strokeStyle = taxi_color;
    ctx.beginPath();
    ctx.moveTo(pos.x, pos.y);
    ctx.lineTo(endPos.x, endPos.y);
    ctx.stroke();

    // Draw info
    let spacing = scaleFeet(100);
    ctx.textAlign = 'left';
    ctx.fillStyle = taxi_color;
    if (selectedAircraft() == aircraft.callsign) {
      ctx.fillStyle = '#FFE045';
    }

    // Draw callsign
    ctx.fillText(aircraft.callsign, pos.x + spacing, pos.y - spacing);

    // Draw speed
    ctx.fillText(
      Math.round(aircraft.speed).toString(),
      pos.x + spacing,
      pos.y - spacing + fontSize()
    );
  }

  function drawTower(ctx: Ctx, world: World, aircrafts: Array<Aircraft>) {
    for (let airspace of world.airspaces) {
      drawAirspace(ctx, airspace);

      for (let airport of airspace.airports) {
        for (let runway of airport.runways) {
          drawRunway(ctx, runway);
        }
      }
    }

    for (let aircraft of aircrafts.filter((a) => a.altitude >= 1000)) {
      drawBlip(ctx, aircraft);
    }
  }

  function drawGround(ctx: Ctx, world: World, aircrafts: Array<Aircraft>) {
    // TODO: only draws selected airspace in ground and approach view
    // center view shows all airspaces
    for (let airspace of world.airspaces) {
      drawAirspace(ctx, airspace);

      for (let airport of airspace.airports) {
        for (let taxiway of airport.taxiways) {
          drawTaxiway(ctx, taxiway);
        }
        for (let taxiway of airport.taxiways) {
          drawTaxiwayLabel(ctx, taxiway);
        }
        for (let runway of airport.runways) {
          drawRunwayGround(ctx, runway);
        }
        for (let terminal of airport.terminals) {
          drawTerminal(ctx, terminal);
        }
      }
    }

    for (let aircraft of aircrafts.filter((a) => a.altitude < 1000)) {
      drawBlipGround(ctx, aircraft);
    }
  }

  return <canvas id="canvas" ref={canvas}></canvas>;
}
