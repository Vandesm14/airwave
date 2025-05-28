import { useAtom } from 'solid-jotai';
import {
  airportAtom,
  frequencyAtom,
  radarAtom,
  renderAtom,
  selectedAircraftAtom,
} from './lib/atoms';
import { Vec2 } from '../bindings/Vec2';
import {
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
} from 'solid-js';
import {
  AIRSPACE_RADIUS,
  calculateSquaredDistance,
  getAirport,
  headingToDegrees,
  knotToFeetPerSecond,
  midpointBetweenPoints,
  movePoint,
  nauticalMilesToFeet,
  runwayInfo,
  shortLandingState,
  shortTaxiingState,
  toRadians,
} from './lib/lib';
import colors from './lib/colors';
import { useAircraftWithRate, useWorld } from './lib/api';
import { Runway } from '../bindings/Runway';
import { Aircraft } from '../bindings/Aircraft';
import { Terminal } from '../bindings/Terminal';
import { Gate } from '../bindings/Gate';
import { Taxiway } from '../bindings/Taxiway';
import { World } from '../bindings/World';
import { Airport } from '../bindings/Airport';
import { useDemoMode } from './lib/hooks';

const groundScale = 5.0;
const FEET_TO_PIXELS = 0.003;

function scaleFeetToPixels(num: number, scale: number): number {
  return num * FEET_TO_PIXELS * scale;
}

function scalePixelsToFeet(num: number, scale: number): number {
  return num / FEET_TO_PIXELS / scale;
}

function scalePoint(
  vec2: Vec2,
  scale: number,
  shiftPoint: { x: number; y: number }
): Vec2 {
  let x = vec2[0] + shiftPoint.x;
  let y = vec2[1] - shiftPoint.y;

  x = scaleFeetToPixels(x, scale);
  y = scaleFeetToPixels(y, scale);

  return [x, -y];
}

function scalePixelPoint(
  vec2: Vec2,
  scale: number,
  shiftPoint: { x: number; y: number }
): Vec2 {
  let x = scalePixelsToFeet(vec2[0], scale);
  let y = scalePixelsToFeet(-vec2[1], scale);

  x -= shiftPoint.x;
  y += shiftPoint.y;

  return [x, y];
}

export default function Canvas() {
  let canvas!: HTMLCanvasElement;

  type Ctx = CanvasRenderingContext2D;

  let [radar, setRadar] = useAtom(radarAtom);

  let [render, setRender] = useAtom(renderAtom);
  let [selectedAircraft, setSelectedAircraft] = useAtom(selectedAircraftAtom);
  let [frequency] = useAtom(frequencyAtom);
  let fontSize = createMemo(() => 16);
  let isGround = createMemo(() => radar().scale > groundScale);
  // let [aircraftTrails, setAircraftTrails] = createSignal<
  //   Map<string, Array<{ pos: Vec2; now: number }>>
  // >(new Map());
  let [mod, setMod] = createSignal(false);

  let renderRate = createMemo(() => (isGround() ? 1000 * 0.5 : 1000 * 4));

  const aircrafts = useAircraftWithRate(renderRate);
  const world = useWorld();

  // FPS variables
  let [lastFpsUpdate, setLastFpsUpdate] = createSignal(Date.now());
  let [frameCount, setFrameCount] = createSignal(0);
  let [currentFps, setCurrentFps] = createSignal(0);

  const [airportId] = useAtom(airportAtom);

  const isDemoMode = useDemoMode();

  createEffect(() => {
    const id = airportId();
    const airport = world.data.airports.find((a) => a.id === id);
    if (airport) {
      setRadar((radar) => {
        radar.scale = 1;
        radar.shiftPoint = {
          x: -airport.center[0],
          y: airport.center[1],
        };
        return { ...radar };
      });
    }
  });

  function clickToSelectAircraft(e: MouseEvent) {
    // Convert the cursor position to your coordinate system
    const coords: Vec2 = scalePixelPoint(
      [e.offsetX - canvas.width * 0.5, e.offsetY - canvas.height * 0.5],
      radar().scale,
      radar().shiftPoint
    );

    // Initialize variables to keep track of the closest aircraft
    let closestAircraft = null;
    let smallestDistance = Infinity;

    // Define the maximum allowable distance squared
    const maxDistanceSquared = Math.pow(
      scalePixelsToFeet(100, radar().scale),
      2
    );

    // Iterate through all aircraft to find the closest one within the criteria
    for (const aircraft of aircrafts.data) {
      // Calculate the squared distance between the cursor and the aircraft
      const distanceSquared = calculateSquaredDistance(coords, aircraft.pos);

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
    } else {
      setSelectedAircraft('');
    }
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

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'PageUp') {
      e.preventDefault();
      const id = airportId();
      const airport = world.data.airports.find((a) => a.id === id);
      if (airport) {
        setRadar((radar) => {
          radar.scale = 1;
          radar.shiftPoint = {
            x: -airport.center[0],
            y: airport.center[1],
          };
          return { ...radar };
        });
      }
    } else if (e.key === 'PageDown') {
      e.preventDefault();
      const id = airportId();
      const airport = world.data.airports.find((a) => a.id === id);
      if (airport) {
        setRadar((radar) => {
          radar.scale = groundScale * 8.0;
          radar.shiftPoint = {
            x: -airport.center[0],
            y: airport.center[1],
          };
          return { ...radar };
        });
      }
    } else if (e.key === 'Control') {
      setMod((mod) => !mod);
    }
  }

  function onResize() {
    canvas.width = canvas.clientWidth;
    canvas.height = canvas.clientHeight;
  }

  onMount(() => {
    const maxScale = 100.0;
    const minScale = 0.1;

    window.addEventListener('resize', onResize);
    document.addEventListener('keydown', onKeydown);

    if (canvas instanceof HTMLCanvasElement && canvas !== null) {
      setInterval(() => doRender(canvas), 1000 / 60);

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

          radar.lastShiftPoint.x = scaleFeetToPixels(
            radar.shiftPoint.x,
            radar.scale
          );
          radar.lastShiftPoint.y = scaleFeetToPixels(
            radar.shiftPoint.y,
            radar.scale
          );

          return { ...radar };
        });
      });
      canvas.addEventListener('mouseup', (e) => {
        setRadar((radar) => {
          if (
            e.clientX === radar.dragStartPoint.x &&
            e.clientY === radar.dragStartPoint.y
          ) {
            clickToSelectAircraft(e);
          }
          radar.isDragging = false;
          return { ...radar };
        });
      });
      canvas.addEventListener('mousemove', (e) => {
        if (radar().isDragging) {
          setRadar((radar) => {
            let x = e.clientX - radar.dragStartPoint.x + radar.lastShiftPoint.x;
            let y = e.clientY - radar.dragStartPoint.y + radar.lastShiftPoint.y;

            radar.shiftPoint.x = scalePixelsToFeet(x, radar.scale);
            radar.shiftPoint.y = scalePixelsToFeet(y, radar.scale);

            return { ...radar };
          });
        }
      });
      canvas.addEventListener('wheel', (e) => {
        setRadar((radar) => {
          if (e.deltaY > 0) {
            radar.scale *= 0.9;
          } else {
            radar.scale *= 1.1;
          }

          radar.scale = Math.max(Math.min(radar.scale, maxScale), minScale);

          return { ...radar };
        });
      });
    }
  });

  onCleanup(() => {
    document.removeEventListener('keydown', onKeydown);
    window.removeEventListener('resize', onResize);
  });

  function doRender(canvas: HTMLCanvasElement) {
    doDraw(canvas);

    // FPS counter update
    setFrameCount((count) => count + 1);
    let now = Date.now();
    if (now - lastFpsUpdate() >= 1000) {
      setCurrentFps(frameCount());
      setFrameCount(0);
      setLastFpsUpdate(now);
    }

    if (now - render().lastDraw > renderRate() || render().doInitialDraw) {
      setRender((render) => {
        // TODO: fix or remove trails
        // setAircraftTrails((map) => {
        //   for (let aircraft of aircrafts.data) {
        //     const trail = map.get(aircraft.id);

        //     if (typeof trail !== 'undefined') {
        //       let last = trail.at(-1);

        //       if (
        //         typeof last !== 'undefined' &&
        //         now - last.now > 1000 * 4 * 2
        //       ) {
        //         trail.push({ now, pos: aircraft.pos });
        //       }

        //       if (trail.length > 10) {
        //         map.set(aircraft.id, trail.slice(1));
        //       }
        //     } else {
        //       map.set(aircraft.id, [{ now, pos: aircraft.pos }]);
        //     }
        //   }

        //   return map;
        // });

        render.doInitialDraw = false;

        return { ...render };
      });
    }
  }

  function resetTransform(ctx: CanvasRenderingContext2D) {
    ctx.resetTransform();
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
      resetTransform(ctx);

      if (world.data === undefined) {
        return;
      }
      if (isGround()) {
        drawGround(ctx, world.data, aircrafts.data);
      } else {
        drawTower(ctx, world.data, aircrafts.data);
        drawCompass(ctx);
      }

      if (!isDemoMode()) {
        drawFPS(ctx);
      }
    }
  }

  // FPS counter
  function drawFPS(ctx: Ctx) {
    ctx.resetTransform();
    ctx.font = `900 ${fontSize()}px monospace`;
    ctx.fillStyle = 'white';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'top';
    ctx.fillText(`FPS: ${currentFps()}`, canvas.width / 2, 10);
  }

  function drawCompass(ctx: Ctx) {
    let diameter = 250;
    let radius = diameter * 0.5;

    if (!mod()) {
      return;
    }

    let aircraft = aircrafts.data.find((a) => a.id === selectedAircraft());
    if (
      aircraft &&
      (aircraft.state.type === 'flying' || aircraft.state.type === 'landing')
    ) {
      let origin = scalePoint(aircraft.pos, radar().scale, radar().shiftPoint);

      ctx.fillStyle = '#888';
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      let padding = -10;
      const increment = 30;
      const count = 360 / increment;
      for (let i = 0; i < count; i++) {
        let text = headingToDegrees(i * increment)
          .toString()
          .padStart(3, '0');
        if (text === '000') {
          text = '360';
        }
        ctx.fillText(
          text,
          Math.cos(toRadians(i * increment)) * (radius + padding) + origin[0],
          Math.sin(toRadians(i * increment)) * (radius + padding) + origin[1]
        );
      }
    }
  }

  function drawAirspace(ctx: Ctx, airport: Airport) {
    resetTransform(ctx);
    let pos = scalePoint(airport.center, radar().scale, radar().shiftPoint);
    ctx.strokeStyle = colors.special.airspace;

    let aircraft = aircrafts.data.find((a) => a.id === selectedAircraft());
    if (
      aircraft &&
      (aircraft.flight_plan.departing === airport.id ||
        aircraft.flight_plan.arriving === airport.id)
    ) {
      ctx.strokeStyle = colors.line_yellow;
    }

    ctx.beginPath();
    ctx.arc(
      pos[0],
      pos[1],
      scaleFeetToPixels(AIRSPACE_RADIUS, radar().scale),
      0,
      Math.PI * 2
    );
    ctx.stroke();

    // Draw airport name
    ctx.fillStyle = '#777';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(
      airport.id,
      pos[0],
      pos[1] - scaleFeetToPixels(AIRSPACE_RADIUS, radar().scale) - 20
    );
  }

  function drawRunway(ctx: Ctx, runway: Runway) {
    resetTransform(ctx);
    let info = runwayInfo(runway);
    let start = scalePoint(info.start, radar().scale, radar().shiftPoint);
    let end = scalePoint(info.end, radar().scale, radar().shiftPoint);
    let ils = {
      minGlideslope: scalePoint(
        info.ils.minGlideslope,
        radar().scale,
        radar().shiftPoint
      ),
      end: scalePoint(info.ils.end, radar().scale, radar().shiftPoint),
      maxAngle: scalePoint(
        info.ils.maxAngle,
        radar().scale,
        radar().shiftPoint
      ),
      minAngle: scalePoint(
        info.ils.minAngle,
        radar().scale,
        radar().shiftPoint
      ),
    };

    ctx.strokeStyle = 'grey';
    ctx.fillStyle = 'grey';
    ctx.lineWidth = scaleFeetToPixels(1000, radar().scale);
    ctx.beginPath();
    ctx.moveTo(start[0], start[1]);
    ctx.lineTo(end[0], end[1]);
    ctx.stroke();

    ctx.fillStyle = colors.line_blue;
    ctx.strokeStyle = colors.line_blue;
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(start[0], start[1]);
    ctx.lineTo(ils.end[0], ils.end[1]);
    ctx.stroke();

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

    ctx.beginPath();
    ctx.arc(
      ils.minGlideslope[0],
      ils.minGlideslope[1],
      scaleFeetToPixels(1500, radar().scale),
      0,
      Math.PI * 2
    );
    ctx.stroke();
  }

  function drawWaypoint(ctx: Ctx, name: string, position: Vec2, color: string) {
    let pos = scalePoint(position, radar().scale, radar().shiftPoint);
    ctx.fillStyle = color;
    ctx.strokeStyle = color;
    ctx.beginPath();
    ctx.arc(
      pos[0],
      pos[1],
      scaleFeetToPixels(700, radar().scale),
      0,
      Math.PI * 2
    );
    ctx.fill();

    // Draw the separation circle
    ctx.beginPath();
    ctx.arc(
      pos[0],
      pos[1],
      scaleFeetToPixels(2000, radar().scale),
      0,
      Math.PI * 2
    );
    ctx.stroke();

    let spacing = scaleFeetToPixels(
      2000 + nauticalMilesToFeet * 0.2,
      radar().scale
    );
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

  function drawBlip(ctx: Ctx, aircraft: Aircraft) {
    const isSelected = selectedAircraft() === aircraft.id;
    const isLanding =
      aircraft.state.type === 'landing' &&
      aircraft.state.value.state !== 'before-turn';

    let isActive = aircraft.frequency === frequency();
    // TODO: This limits the "Center" view to our aircraft. We can remove once
    // we have better tooling for ARTCC.
    if (
      ['climb', 'cruise', 'arrival'].includes(aircraft.segment) &&
      aircraft.flight_plan.arriving !== airportId() &&
      aircraft.flight_plan.departing !== airportId()
    ) {
      isActive = false;
    }

    const isTcas = aircraft.tcas !== 'idle';
    const isTcasTaRa =
      aircraft.tcas === 'climb' ||
      aircraft.tcas === 'descend' ||
      aircraft.tcas === 'hold';

    // Draw trail
    // let trail = aircraftTrails().get(aircraft.id);
    // if (trail) {
    //   const dotSize = Math.max(2, scaleFeetToPixels(750));

    //   // If selected
    //   if (selectedAircraft() == aircraft.id) {
    //     ctx.fillStyle = colors.text_yellow;
    //     // If colliding
    //   } else if (isTcas) {
    //     ctx.fillStyle = colors.text_red;
    //     // If departure
    //   } else if (aircraft.flight_plan.departing === HARD_CODED_AIRPORT) {
    //     ctx.fillStyle = colors.line_blue;
    //   } else {
    //     // Else, arrival
    //     ctx.fillStyle = colors.line_green;
    //   }

    //   for (let point of trail) {
    //     ctx.fillRect(
    //       scalePoint(point.pos)[0] - dotSize * 0.5,
    //       scalePoint(point.pos)[1] - dotSize * 0.5,
    //       dotSize,
    //       dotSize
    //     );
    //   }
    // }

    // Draw waypoints
    let pos = scalePoint(aircraft.pos, radar().scale, radar().shiftPoint);
    if (
      aircraft.state.type === 'flying' &&
      aircraft.flight_plan.follow &&
      selectedAircraft() == aircraft.id
    ) {
      ctx.strokeStyle = '#ffff0033';
      ctx.lineWidth = 3;

      ctx.beginPath();
      ctx.moveTo(pos[0], pos[1]);

      for (let wp of aircraft.flight_plan.waypoints.slice(
        aircraft.flight_plan.waypoint_index
      )) {
        let pos = scalePoint(wp.data.pos, radar().scale, radar().shiftPoint);
        ctx.lineTo(pos[0], pos[1]);
      }
      ctx.stroke();

      for (let wp of aircraft.flight_plan.waypoints.slice(
        aircraft.flight_plan.waypoint_index
      )) {
        drawWaypoint(ctx, wp.name, wp.data.pos, colors.text_yellow);
      }
    }

    resetTransform(ctx);

    if (isSelected) {
      ctx.fillStyle = colors.line_yellow;
      ctx.strokeStyle = colors.line_yellow;
    } else if (isTcasTaRa) {
      ctx.fillStyle = colors.line_red;
      ctx.strokeStyle = colors.line_red;
    } else if (!isActive) {
      ctx.fillStyle = colors.text_light_grey;
      ctx.strokeStyle = colors.text_light_grey;
    } else {
      ctx.fillStyle = colors.line_green;
      ctx.strokeStyle = colors.line_green;
    }

    // Draw the dot
    const dotSize = Math.max(6, scaleFeetToPixels(3000, radar().scale));
    ctx.fillRect(
      pos[0] - dotSize * 0.5,
      pos[1] - dotSize * 0.5,
      dotSize,
      dotSize
    );

    let spacing = scaleFeetToPixels(nauticalMilesToFeet * 1.0, radar().scale);
    if (!isActive && !isSelected) {
      ctx.textAlign = 'left';
      ctx.fillStyle = colors.text_grey;

      ctx.fillText(
        Math.round(aircraft.altitude / 1000)
          .toString()
          .padStart(2, '0'),
        pos[0] + spacing,
        pos[1] - spacing
      );

      return;
    }

    // Draw the direction
    const length = aircraft.speed * knotToFeetPerSecond * 60;
    const end = movePoint(aircraft.pos, length, aircraft.heading);
    let endPos = scalePoint(end, radar().scale, radar().shiftPoint);

    ctx.beginPath();
    ctx.moveTo(pos[0], pos[1]);
    ctx.lineTo(endPos[0], endPos[1]);
    ctx.stroke();

    // Draw info
    ctx.textAlign = 'left';
    ctx.fillStyle = colors.text_green;

    if (isSelected) {
      ctx.fillStyle = colors.text_yellow;
    } else if (isTcasTaRa) {
      ctx.fillStyle = colors.text_red;
    } else if (!isActive) {
      ctx.fillStyle = colors.text_dark_grey;
    } else if (aircraft.frequency !== frequency()) {
      ctx.fillStyle = colors.text_grey;
    } else if (aircraft.flight_plan.departing === airportId()) {
      ctx.fillStyle = colors.line_blue;
    }

    // Draw callsign
    let callsign = aircraft.id;
    if (isTcasTaRa) {
      callsign += ' TA/RA';
    } else if (isTcas) {
      callsign += ' TA';
    }
    ctx.fillText(callsign, pos[0] + spacing, pos[1] - spacing);

    if (isLanding && !isSelected) {
      return;
    }

    // Draw altitude
    let altitudeIcon = ' ';
    if (aircraft.altitude < aircraft.target.altitude) {
      altitudeIcon = '⬈';
    } else if (aircraft.altitude > aircraft.target.altitude) {
      altitudeIcon = '⬊';
    }

    let targetAltitude =
      aircraft.target.altitude !== aircraft.altitude
        ? altitudeIcon +
          Math.round(aircraft.target.altitude / 100)
            .toString()
            .padStart(3, '0')
        : '';

    if (aircraft.tcas === 'climb') {
      targetAltitude = '⬈' + 'CLB';
    } else if (aircraft.tcas === 'descend') {
      targetAltitude = '⬊' + 'DES';
    } else if (aircraft.tcas === 'hold') {
      targetAltitude = '⬌' + 'HLD';
    }

    if (isLanding) {
      targetAltitude = '';
    }

    ctx.fillText(
      Math.round(aircraft.altitude / 100)
        .toString()
        .padStart(3, '0') + targetAltitude,
      pos[0] + spacing,
      pos[1] - spacing + fontSize()
    );

    if (selectedAircraft() !== aircraft.id) {
      return;
    }

    // Draw speed or ILS status
    let text = '';
    if (aircraft.state.type === 'landing') {
      text = shortLandingState(aircraft.state.value.state);
    } else {
      text = Math.round(aircraft.speed).toString();
    }
    ctx.fillText(text, pos[0] + spacing, pos[1] - spacing + fontSize() * 2);
  }

  function drawTerminal(ctx: Ctx, terminal: Terminal) {
    let a = scalePoint(terminal.a, radar().scale, radar().shiftPoint);
    let b = scalePoint(terminal.b, radar().scale, radar().shiftPoint);
    let c = scalePoint(terminal.c, radar().scale, radar().shiftPoint);
    let d = scalePoint(terminal.d, radar().scale, radar().shiftPoint);

    ctx.fillStyle = colors.special.terminal;
    ctx.lineWidth = scaleFeetToPixels(200, radar().scale);
    ctx.beginPath();
    ctx.moveTo(a[0], a[1]);
    ctx.lineTo(b[0], b[1]);
    ctx.lineTo(c[0], c[1]);
    ctx.lineTo(d[0], d[1]);
    ctx.lineTo(a[0], a[1]);
    ctx.fill();

    // TODO: we should show aprons nicer than a debug line
    let apron_a = scalePoint(
      terminal.apron[0],
      radar().scale,
      radar().shiftPoint
    );
    let apron_b = scalePoint(
      terminal.apron[1],
      radar().scale,
      radar().shiftPoint
    );

    ctx.strokeStyle = colors.line_green;
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(apron_a[0], apron_a[1]);
    ctx.lineTo(apron_b[0], apron_b[1]);
    ctx.stroke();

    for (let i = 0; i < terminal.gates.length; i++) {
      let gate = terminal.gates[i];

      if (typeof gate !== 'undefined') {
        drawGate(ctx, gate);
      }
    }
  }

  function drawGate(ctx: Ctx, gate: Gate) {
    let id = gate.id;
    let pos = scalePoint(gate.pos, radar().scale, radar().shiftPoint);

    let gate_size = scaleFeetToPixels(175, radar().scale);

    ctx.fillStyle = '#222';
    ctx.strokeStyle = colors.text_red;
    ctx.save();
    ctx.translate(pos[0], pos[1]);
    ctx.rotate(toRadians(gate.heading));
    ctx.fillRect(-gate_size / 2, -gate_size / 2, gate_size, gate_size);
    ctx.strokeRect(-gate_size / 2, -gate_size / 2, gate_size, gate_size);
    ctx.restore();

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
    let start = scalePoint(taxiway.a, radar().scale, radar().shiftPoint);
    let end = scalePoint(taxiway.b, radar().scale, radar().shiftPoint);

    ctx.strokeStyle = colors.special.taxiway;
    ctx.lineWidth = scaleFeetToPixels(200, radar().scale);
    ctx.beginPath();
    ctx.moveTo(start[0], start[1]);
    ctx.lineTo(end[0], end[1]);
    ctx.stroke();
  }

  function drawTaxiwayLabel(ctx: Ctx, taxiway: Taxiway) {
    let start = scalePoint(taxiway.a, radar().scale, radar().shiftPoint);
    let end = scalePoint(taxiway.b, radar().scale, radar().shiftPoint);
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
    let start = scalePoint(info.start, radar().scale, radar().shiftPoint);
    let end = scalePoint(info.end, radar().scale, radar().shiftPoint);
    let ils = {
      end: scalePoint(info.ils.end, radar().scale, radar().shiftPoint),
      maxAngle: scalePoint(
        info.ils.maxAngle,
        radar().scale,
        radar().shiftPoint
      ),
      minAngle: scalePoint(
        info.ils.minAngle,
        radar().scale,
        radar().shiftPoint
      ),
    };

    ctx.strokeStyle = '#222';
    ctx.lineWidth = scaleFeetToPixels(250, radar().scale);
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

    // Draw the localizer beacon
    ctx.fillStyle = colors.line_blue;
    ctx.strokeStyle = colors.line_blue;
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(start[0], start[1]);
    ctx.lineTo(ils.end[0], ils.end[1]);
    ctx.stroke();
  }

  function drawBlipGround(ctx: Ctx, aircraft: Aircraft) {
    const isSelected = selectedAircraft() === aircraft.id;
    const airport = getAirport(world.data, airportId());

    resetTransform(ctx);
    let pos = scalePoint(aircraft.pos, radar().scale, radar().shiftPoint);
    // let taxi_yellow = '#ffff00';
    let taxi_color =
      aircraft.segment !== 'dormant' ? '#ffffff' : colors.text_dark_grey;
    taxi_color =
      aircraft.segment === 'boarding' ? colors.text_light_grey : taxi_color;
    taxi_color = isSelected ? colors.text_yellow : taxi_color;

    let callsign_color =
      aircraft.frequency !== frequency() ? colors.text_grey : '#fff';
    if (airport !== undefined) {
      callsign_color =
        aircraft.flight_plan.arriving === airport.id
          ? colors.text_green
          : colors.line_blue;
    }
    callsign_color = isSelected ? colors.text_yellow : callsign_color;

    // Draw taxi waypoints
    if (aircraft.state.type === 'taxiing' && isSelected) {
      ctx.strokeStyle = '#ffff0088';
      ctx.lineWidth = scaleFeetToPixels(50, radar().scale);

      // Draw waypoint lines
      ctx.beginPath();
      ctx.moveTo(pos[0], pos[1]);
      for (let wp of aircraft.state.value.waypoints.slice().reverse()) {
        let pos = scalePoint(wp.data, radar().scale, radar().shiftPoint);
        ctx.lineTo(pos[0], pos[1]);
      }
      ctx.stroke();

      // Draw waypoint dots
      for (let wp of aircraft.state.value.waypoints.slice().reverse()) {
        if (wp.behavior === 'goto') {
          ctx.fillStyle = colors.line_green;
        } else if (wp.behavior === 'park') {
          ctx.fillStyle = colors.line_green;
        } else if (wp.behavior === 'holdshort') {
          ctx.fillStyle = colors.line_red;
        } else {
          ctx.fillStyle = colors.line_yellow;
        }
        let pos = scalePoint(wp.data, radar().scale, radar().shiftPoint);
        ctx.beginPath();
        ctx.arc(
          pos[0],
          pos[1],
          scaleFeetToPixels(40, radar().scale),
          0,
          Math.PI * 2
        );
        ctx.fill();
      }

      // Draw the hold short waypoints above the normal waypoints
      for (let wp of aircraft.state.value.waypoints
        .slice()
        .reverse()
        .filter((wp) => wp.behavior === 'holdshort')) {
        if (wp.behavior === 'goto') {
          ctx.fillStyle = colors.line_yellow;
        } else if (wp.behavior === 'park') {
          ctx.fillStyle = colors.line_yellow;
        } else if (wp.behavior === 'holdshort') {
          ctx.fillStyle = colors.line_red;
        }
        let pos = scalePoint(wp.data, radar().scale, radar().shiftPoint);
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
    ctx.arc(
      pos[0],
      pos[1],
      scaleFeetToPixels(50, radar().scale),
      0,
      Math.PI * 2
    );
    ctx.fill();

    // Draw the direction
    ctx.strokeStyle = taxi_color;
    ctx.lineWidth = 2;
    const length = 150;
    const end = movePoint(aircraft.pos, length, aircraft.heading);
    let endPos = scalePoint(end, radar().scale, radar().shiftPoint);

    ctx.beginPath();
    ctx.moveTo(pos[0], pos[1]);
    ctx.lineTo(endPos[0], endPos[1]);
    ctx.stroke();

    if (aircraft.state.type === 'parked' && !isSelected) {
      return;
    }

    // Draw info
    let spacing = scaleFeetToPixels(100, radar().scale);
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
    ctx.fillStyle = callsign_color;
    ctx.fillText(aircraft.id, pos[0] + spacing, pos[1] - spacing);
    ctx.fillStyle = taxi_color;

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
    } else if (aircraft.state.type === 'taxiing' && isSelected && mod()) {
      let text = shortTaxiingState(aircraft.state.value.state);
      let textWidth = ctx.measureText(text).width + 10;
      ctx.fillStyle = colors.text_background;
      ctx.fillRect(
        pos[0] + spacing * 0.5,
        pos[1] - spacing - fontSize() * 0.5 + fontSize(),
        textWidth,
        fontSize()
      );
      ctx.fillStyle = taxi_color;
      ctx.fillText(text, pos[0] + spacing, pos[1] - spacing + fontSize());
    }

    // // Draw speed
    // ctx.fillText(
    //   Math.round(aircraft.speed).toString(),
    //   pos[0] + spacing,
    //   pos[1] - spacing + fontSize() * (drawAlt ? 2 : 1)
    // );
  }

  function drawFlightPlanWaypoints(ctx: Ctx, aircraft: Aircraft) {
    resetTransform(ctx);
    let pos = scalePoint(aircraft.pos, radar().scale, radar().shiftPoint);

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

  function drawTower(ctx: Ctx, world: World, aircrafts: Array<Aircraft>) {
    for (let waypoint of world.waypoints) {
      drawWaypoint(ctx, waypoint.name, waypoint.data, colors.special.waypoint);
    }

    for (let airport of world.airports) {
      drawAirspace(ctx, airport);

      for (let runway of airport.runways) {
        drawRunway(ctx, runway);
      }
    }

    for (let aircraft of aircrafts.filter(
      (a) => a.altitude >= 1000 && a.id !== selectedAircraft()
    )) {
      drawBlip(ctx, aircraft);
    }

    for (let aircraft of aircrafts.filter(
      (a) => a.altitude >= 1000 && a.id === selectedAircraft()
    )) {
      drawBlip(ctx, aircraft);
    }

    for (let aircraft of aircrafts.filter((a) => a.id == selectedAircraft())) {
      drawFlightPlanWaypoints(ctx, aircraft);
    }
  }

  function drawGround(ctx: Ctx, world: World, aircrafts: Array<Aircraft>) {
    for (let airport of world.airports) {
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

    for (let aircraft of aircrafts.filter(
      (a) => a.altitude < 1000 && a.id !== selectedAircraft()
    )) {
      drawBlipGround(ctx, aircraft);
    }

    for (let aircraft of aircrafts.filter(
      (a) => a.altitude < 1000 && a.id === selectedAircraft()
    )) {
      drawBlipGround(ctx, aircraft);
    }
  }

  return <canvas id="canvas" ref={canvas}></canvas>;
}
