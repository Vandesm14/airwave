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
  Connection,
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
  calculateSquaredDistance,
  headingToDegrees,
  knotToFeetPerSecond,
  midpointBetweenPoints,
  movePoint,
  nauticalMilesToFeet,
  runwayInfo,
  toRadians,
} from './lib/lib';
import colors from './lib/colors';

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
  let [selectedAircraft, setSelectedAircraft] = useAtom(selectedAircraftAtom);
  let fontSize = createMemo(() => 16);
  let isGround = createMemo(() => radar().scale > groundScale);
  let [waitingForAircraft, setWaitingForAircraft] = createSignal(true);

  function scaleFeetToPixels(num: number): number {
    const FEET_TO_PIXELS = 0.003;
    return num * FEET_TO_PIXELS * radar().scale;
  }

  function scalePixelsToFeet(num: number): number {
    const FEET_TO_PIXELS = 0.003;
    return num / FEET_TO_PIXELS / radar().scale;
  }

  function scalePoint(vec2: Vec2): Vec2 {
    let x = vec2[0] + radar().shiftPoint.x;
    let y = vec2[1] - radar().shiftPoint.y;

    x = scaleFeetToPixels(x);
    y = scaleFeetToPixels(y);

    return [x, -y];
  }

  function scalePixelPoint(vec2: Vec2): Vec2 {
    let x = scalePixelsToFeet(vec2[0]);
    let y = scalePixelsToFeet(-vec2[1]);

    x -= radar().shiftPoint.x;
    y += radar().shiftPoint.y;

    return [x, y];
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

      document.addEventListener('keydown', (e) => {
        let zoomAmount = 2.0;
        if (e.key === 'PageUp') {
          setRadar((radar) => {
            radar.scale = radar.scale * zoomAmount;
            return { ...radar };
          });
        } else if (e.key === 'PageDown') {
          setRadar((radar) => {
            radar.scale = (radar.scale * 1) / zoomAmount;
            return { ...radar };
          });
        }
      });

      canvas.addEventListener('mousedown', (e) => {
        setRadar((radar) => {
          radar.isDragging = true;
          radar.dragStartPoint = {
            x: e.clientX,
            y: e.clientY,
          };

          radar.lastShiftPoint.x = scaleFeetToPixels(radar.shiftPoint.x);
          radar.lastShiftPoint.y = scaleFeetToPixels(radar.shiftPoint.y);

          return { ...radar };
        });

        // Convert the cursor position to your coordinate system
        const coords: Vec2 = scalePixelPoint([
          e.offsetX - canvas.width * 0.5,
          e.offsetY - canvas.height * 0.5,
        ]);

        // Initialize variables to keep track of the closest aircraft
        let closestAircraft = null;
        let smallestDistance = Infinity;

        // Define the maximum allowable distance squared
        const maxDistanceSquared = Math.pow(scalePixelsToFeet(100), 2);

        // Iterate through all aircraft to find the closest one within the criteria
        for (const aircraft of render().aircrafts) {
          // Calculate the squared distance between the cursor and the aircraft
          const distanceSquared = calculateSquaredDistance(
            coords,
            aircraft.pos
          );

          // Check if the aircraft is within the maximum distance
          if (distanceSquared <= maxDistanceSquared) {
            // Check altitude based on whether it's on the ground or not
            const altitudeCondition = isGround()
              ? aircraft.altitude < 1000
              : aircraft.altitude >= 1000;

            if (altitudeCondition) {
              // If this aircraft is closer than any previously found, update the closestAircraft
              if (distanceSquared < smallestDistance) {
                smallestDistance = distanceSquared;
                closestAircraft = aircraft;
              }
            }
          }
        }

        // After checking all aircraft, select the closest one if any were found
        if (closestAircraft !== null) {
          setSelectedAircraft(closestAircraft.id);
        }
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

            radar.shiftPoint.x = scalePixelsToFeet(x);
            radar.shiftPoint.y = scalePixelsToFeet(y);

            return { ...radar };
          });
        }
      });
      canvas.addEventListener('wheel', (e) => {
        setRadar((radar) => {
          let maxScale = 50.0;
          let minScale = 0.1;

          if (e.deltaY > 0) {
            radar.scale *= 0.9;
          } else {
            radar.scale *= 1.1;
          }

          // radar.scale += e.deltaY * -0.001;
          radar.scale = Math.max(Math.min(radar.scale, maxScale), minScale);

          return { ...radar };
        });
      });
    }
  });

  function doRender(canvas: HTMLCanvasElement) {
    doDraw(canvas);

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
    // ctx.translate(radar().shiftPoint.x, radar().shiftPoint.y);
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

      drawCollodingMessage(ctx, aircrafts());
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
    ctx.strokeStyle = colors.special.airspace;

    ctx.beginPath();
    ctx.arc(pos[0], pos[1], scaleFeetToPixels(airspace.radius), 0, Math.PI * 2);
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
    ctx.lineWidth = scaleFeetToPixels(1000);
    ctx.beginPath();
    ctx.moveTo(start[0], start[1]);
    ctx.lineTo(end[0], end[1]);
    ctx.stroke();

    // Draw the localizer beacon
    ctx.fillStyle = colors.line_blue;
    ctx.strokeStyle = colors.line_blue;
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(start[0], start[1]);
    ctx.lineTo(ils.end[0], ils.end[1]);
    ctx.stroke();

    // Draw the max and min localizer angle
    ctx.strokeStyle = colors.line_grey;
    ctx.beginPath();
    ctx.moveTo(start[0], start[1]);
    ctx.lineTo(ils.maxAngle[0], ils.maxAngle[1]);
    ctx.stroke();

    ctx.beginPath();
    ctx.moveTo(start[0], start[1]);
    ctx.lineTo(ils.minAngle[0], ils.minAngle[1]);
    ctx.stroke();

    // Draw the localizer altitude points
    ctx.strokeStyle = colors.line_blue;
    for (let p of info.ils.altitudePoints) {
      let point = scalePoint(p);
      ctx.beginPath();
      ctx.arc(point[0], point[1], scaleFeetToPixels(1500), 0, Math.PI * 2);
      ctx.stroke();
    }
  }

  function drawWaypoint(ctx: Ctx, name: string, position: Vec2, color: string) {
    let pos = scalePoint(position);
    ctx.fillStyle = color;
    ctx.strokeStyle = color;
    ctx.beginPath();
    ctx.arc(pos[0], pos[1], scaleFeetToPixels(700), 0, Math.PI * 2);
    ctx.fill();

    // Draw the separation circle
    ctx.beginPath();
    ctx.arc(pos[0], pos[1], scaleFeetToPixels(2000), 0, Math.PI * 2);
    ctx.stroke();

    let spacing = scaleFeetToPixels(2000 + nauticalMilesToFeet * 0.2);
    ctx.textAlign = 'center';
    ctx.textBaseline = 'bottom';

    // Draw the label background
    let textWidth = ctx.measureText(name).width + 10;
    ctx.fillStyle = colors.text_background;
    ctx.fillRect(
      pos[0] - textWidth * 0.5,
      pos[1] - spacing - fontSize() * 1,
      textWidth,
      fontSize()
    );

    // Draw the label
    ctx.fillStyle = color;
    ctx.fillText(name, pos[0], pos[1] - spacing);
  }

  function drawConnection(ctx: Ctx, connection: Connection) {
    let activeColor = colors.text_grey;
    let inactiveColor = colors.special.connection.inactive;
    let selectedColor = colors.text_yellow;

    let color = connection.state === 'active' ? activeColor : inactiveColor;

    let aircraft = aircrafts().find((a) => a.id === selectedAircraft());
    if (aircraft) {
      if (aircraft.flight_plan.arriving === connection.id) {
        color = selectedColor;
      }
    }

    // Draw the airport waypoint
    drawWaypoint(ctx, connection.id, connection.pos, color);

    // Draw the transition waypoint
    drawWaypoint(ctx, connection.id, connection.transition, color);
  }

  function drawBlip(ctx: Ctx, aircraft: Aircraft) {
    let pos = scalePoint(aircraft.pos);
    if (aircraft.state.type === 'flying' && selectedAircraft() == aircraft.id) {
      ctx.strokeStyle = '#ffff0033';
      ctx.lineWidth = 3;

      ctx.beginPath();
      ctx.moveTo(pos[0], pos[1]);

      for (let wp of aircraft.state.value.waypoints.slice().reverse()) {
        let pos = scalePoint(wp.value.to);
        ctx.lineTo(pos[0], pos[1]);
      }
      ctx.stroke();

      for (let wp of aircraft.state.value.waypoints.slice().reverse()) {
        ctx.fillStyle = wp.behavior === 'goto' ? '#ffff00' : '#ff0000';
        let pos = scalePoint(wp.value.to);
        ctx.beginPath();
        ctx.arc(pos[0], pos[1], 3, 0, Math.PI * 2);
        ctx.fill();
      }
    }

    resetTransform(ctx);

    if (selectedAircraft() == aircraft.id) {
      ctx.fillStyle = colors.line_yellow;
      ctx.strokeStyle = colors.line_yellow;
    } else if (aircraft.is_colliding) {
      ctx.fillStyle = colors.line_red;
      ctx.strokeStyle = colors.line_red;
    } else {
      ctx.fillStyle = colors.line_green;
      ctx.strokeStyle = colors.line_green;
    }

    // Draw the dot
    ctx.beginPath();
    ctx.arc(
      pos[0],
      pos[1],
      Math.min(3, scaleFeetToPixels(1000)),
      0,
      Math.PI * 2
    );
    ctx.fill();

    // Draw the separation circle
    ctx.beginPath();
    ctx.arc(
      pos[0],
      pos[1],
      scaleFeetToPixels(nauticalMilesToFeet * 1),
      0,
      Math.PI * 2
    );
    ctx.stroke();

    // Draw the collision circle
    if (aircraft.is_colliding) {
      ctx.beginPath();
      ctx.arc(
        pos[0],
        pos[1],
        scaleFeetToPixels(nauticalMilesToFeet * 15),
        0,
        Math.PI * 2
      );
      ctx.stroke();
    }

    // Draw the direction
    const length = aircraft.speed * knotToFeetPerSecond * 60;
    const end = movePoint(aircraft.pos, length, aircraft.heading);
    let endPos = scalePoint(end);

    ctx.beginPath();
    ctx.moveTo(pos[0], pos[1]);
    ctx.lineTo(endPos[0], endPos[1]);
    ctx.stroke();

    // Draw info
    let spacing = scaleFeetToPixels(nauticalMilesToFeet * 1.0);
    ctx.textAlign = 'left';
    ctx.fillStyle = colors.text_green;

    if (selectedAircraft() == aircraft.id) {
      ctx.fillStyle = colors.text_yellow;
    } else if (aircraft.is_colliding) {
      ctx.fillStyle = colors.text_red;
    } else if (aircraft.flight_plan.departing === world().airspace.id) {
      ctx.fillStyle = colors.line_blue;
    }

    // Draw callsign
    ctx.fillText(aircraft.id, pos[0] + spacing, pos[1] - spacing);

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
      pos[0] + spacing,
      pos[1] - spacing + fontSize()
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
      pos[0] + spacing,
      pos[1] - spacing + fontSize() * 2
    );

    // Draw speed
    ctx.fillText(
      Math.round(aircraft.speed).toString(),
      pos[0] + spacing,
      pos[1] - spacing + fontSize() * 3
    );
  }

  function drawTerminal(ctx: Ctx, terminal: Terminal) {
    let a = scalePoint(terminal.a);
    let b = scalePoint(terminal.b);
    let c = scalePoint(terminal.c);
    let d = scalePoint(terminal.d);

    ctx.fillStyle = colors.special.terminal;
    ctx.lineWidth = scaleFeetToPixels(200);
    ctx.beginPath();
    ctx.moveTo(a[0], a[1]);
    ctx.lineTo(b[0], b[1]);
    ctx.lineTo(c[0], c[1]);
    ctx.lineTo(d[0], d[1]);
    ctx.lineTo(a[0], a[1]);
    ctx.fill();

    // TODO: we should show aprons nicer than a debug line
    let apron_a = scalePoint(terminal.apron[0]);
    let apron_b = scalePoint(terminal.apron[1]);

    ctx.strokeStyle = colors.line_green;
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(apron_a[0], apron_a[1]);
    ctx.lineTo(apron_b[0], apron_b[1]);
    ctx.stroke();

    for (let i = 0; i < terminal.gates.length; i++) {
      let gate = terminal.gates[i];
      drawGate(ctx, gate);
    }
  }

  function drawGate(ctx: Ctx, gate: Gate) {
    let id = gate.id;
    let pos = scalePoint(gate.pos);

    ctx.fillStyle = 'red';
    ctx.beginPath();
    ctx.arc(pos[0], pos[1], 5, 0, Math.PI * 2);
    ctx.fill();

    let fontSize = 16;
    let textWidth = ctx.measureText(id).width + 10;
    ctx.fillStyle = colors.text_background;
    ctx.fillRect(
      pos[0] - textWidth * 0.5,
      pos[1] - fontSize * 0.5 - fontSize,
      textWidth,
      fontSize
    );

    ctx.fillStyle = colors.text_orange;
    ctx.fillText(id, pos[0], pos[1] - fontSize);
  }

  function drawTaxiway(ctx: Ctx, taxiway: Taxiway) {
    resetTransform(ctx);
    let start = scalePoint(taxiway.a);
    let end = scalePoint(taxiway.b);

    ctx.strokeStyle = colors.special.taxiway;
    ctx.lineWidth = scaleFeetToPixels(200);
    ctx.beginPath();
    ctx.moveTo(start[0], start[1]);
    ctx.lineTo(end[0], end[1]);
    ctx.stroke();
  }

  function drawTaxiwayLabel(ctx: Ctx, taxiway: Taxiway) {
    let start = scalePoint(taxiway.a);
    let end = scalePoint(taxiway.b);
    let middle = midpointBetweenPoints(start, end);
    let textWidth = ctx.measureText(taxiway.id).width + 10;
    ctx.fillStyle = colors.text_background;
    ctx.fillRect(
      middle[0] - textWidth * 0.5,
      middle[1] - fontSize() * 0.5,
      textWidth,
      fontSize()
    );
    ctx.textAlign = 'center';
    ctx.fillStyle = colors.text_orange;
    ctx.fillText(taxiway.id, middle[0], middle[1]);
  }

  function drawRunwayGround(ctx: Ctx, runway: Runway) {
    resetTransform(ctx);
    let info = runwayInfo(runway);
    let start = scalePoint(info.start);
    let end = scalePoint(info.end);

    ctx.strokeStyle = '#222';
    ctx.lineWidth = scaleFeetToPixels(250);
    ctx.beginPath();
    ctx.moveTo(start[0], start[1]);
    ctx.lineTo(end[0], end[1]);
    ctx.stroke();

    // Draw runway label
    let textWidth = ctx.measureText(runway.id).width + 10;
    ctx.fillStyle = colors.text_background;
    ctx.fillRect(
      start[0] - textWidth * 0.5,
      start[1] - fontSize() * 0.5,
      textWidth,
      fontSize()
    );
    ctx.textAlign = 'center';
    ctx.fillStyle = colors.text_orange;
    ctx.fillText(runway.id, start[0], start[1]);
  }

  function drawBlipGround(ctx: Ctx, aircraft: Aircraft) {
    resetTransform(ctx);
    let pos = scalePoint(aircraft.pos);
    // let taxi_yellow = '#ffff00';
    let taxi_color =
      selectedAircraft() == aircraft.id ? colors.text_yellow : '#ffffff';

    if (
      aircraft.state.type === 'taxiing' &&
      selectedAircraft() == aircraft.id
    ) {
      ctx.strokeStyle = '#ffff0088';
      ctx.lineWidth = scaleFeetToPixels(50);

      ctx.beginPath();
      ctx.moveTo(pos[0], pos[1]);
      for (let wp of aircraft.state.value.waypoints.slice().reverse()) {
        let pos = scalePoint(wp.value);
        ctx.lineTo(pos[0], pos[1]);
      }
      ctx.stroke();

      for (let wp of aircraft.state.value.waypoints.slice().reverse()) {
        ctx.fillStyle = wp.behavior === 'goto' ? '#ffff00' : '#ff0000';
        let pos = scalePoint(wp.value);
        ctx.beginPath();
        ctx.arc(pos[0], pos[1], 3, 0, Math.PI * 2);
        ctx.fill();
      }
    }

    resetTransform(ctx);

    ctx.fillStyle = taxi_color;
    ctx.strokeStyle = taxi_color;

    // Draw the dot
    ctx.beginPath();
    ctx.arc(pos[0], pos[1], scaleFeetToPixels(50), 0, Math.PI * 2);
    ctx.fill();

    // Draw the direction
    ctx.strokeStyle = taxi_color;
    ctx.lineWidth = 2;
    const length = 400;
    const end = movePoint(aircraft.pos, length, aircraft.heading);
    let endPos = scalePoint(end);

    ctx.beginPath();
    ctx.moveTo(pos[0], pos[1]);
    ctx.lineTo(endPos[0], endPos[1]);
    ctx.stroke();

    if (
      aircraft.state.type === 'parked' &&
      aircraft.id !== selectedAircraft()
    ) {
      return;
    }

    // Draw info
    let spacing = scaleFeetToPixels(100);
    ctx.textAlign = 'left';
    ctx.fillStyle = taxi_color;

    // Draw callsign

    let textWidth = ctx.measureText(aircraft.id).width + 10;
    ctx.fillStyle = colors.text_background;
    ctx.fillRect(
      pos[0] + spacing * 0.5,
      pos[1] - spacing - fontSize() * 0.5,
      textWidth,
      fontSize()
    );
    ctx.fillStyle = taxi_color;
    ctx.fillText(aircraft.id, pos[0] + spacing, pos[1] - spacing);

    // Draw altitude
    let drawAlt = aircraft.altitude > 0;
    if (drawAlt) {
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
        pos[0] + spacing,
        pos[1] - spacing + fontSize()
      );
    }

    // Draw speed
    ctx.fillText(
      Math.round(aircraft.speed).toString(),
      pos[0] + spacing,
      pos[1] - spacing + fontSize() * (drawAlt ? 2 : 1)
    );
  }

  function drawFlightPlanWaypoints(ctx: Ctx, aircraft: Aircraft) {
    resetTransform(ctx);
    let pos = scalePoint(aircraft.pos);

    if (
      aircraft.state.type === 'taxiing' &&
      selectedAircraft() == aircraft.id
    ) {
      ctx.strokeStyle = '#ff990033';
      ctx.lineWidth = 3;

      ctx.beginPath();
      ctx.moveTo(pos[0], pos[1]);
    }
  }

  function drawCollodingMessage(ctx: Ctx, aircrafts: Array<Aircraft>) {
    let names: Array<String> = aircrafts
      .filter((a) => a.is_colliding)
      .map((a) => a.id);
    if (names.length > 0) {
      ctx.font = `900 ${24}px monospace`;
      let message = `SEPARATION WARNING: ${names.join(', ')}`;
      ctx.fillStyle = 'red';
      ctx.textAlign = 'center';
      ctx.fillText(message, 0, -canvas.height * 0.5 + 100);
    }
  }

  function drawTower(ctx: Ctx, world: World, aircrafts: Array<Aircraft>) {
    let airspace = world.airspace;
    drawAirspace(ctx, airspace);

    for (let airport of airspace.airports) {
      for (let runway of airport.runways) {
        drawRunway(ctx, runway);
      }
    }

    for (let connection of world.connections) {
      drawConnection(ctx, connection);
    }

    for (let aircraft of aircrafts.filter((a) => a.altitude >= 1000)) {
      drawBlip(ctx, aircraft);
    }

    for (let aircraft of aircrafts.filter((a) => a.id == selectedAircraft())) {
      drawFlightPlanWaypoints(ctx, aircraft);
    }
  }

  function drawGround(ctx: Ctx, world: World, aircrafts: Array<Aircraft>) {
    let airspace = world.airspace;
    drawAirspace(ctx, airspace);

    for (let airport of airspace.airports) {
      for (let taxiway of airport.taxiways) {
        drawTaxiway(ctx, taxiway);
      }
      for (let runway of airport.runways) {
        drawRunwayGround(ctx, runway);
      }
      for (let terminal of airport.terminals) {
        drawTerminal(ctx, terminal);
      }
      for (let taxiway of airport.taxiways) {
        drawTaxiwayLabel(ctx, taxiway);
      }
    }

    for (let aircraft of aircrafts.filter((a) => a.altitude < 1000)) {
      drawBlipGround(ctx, aircraft);
    }
  }

  return <canvas id="canvas" ref={canvas}></canvas>;
}
