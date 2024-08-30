import { useAtom } from 'solid-jotai';
import { radarAtom, renderAtom, worldAtom } from './lib/atoms';
import {
  Aircraft,
  Airspace,
  Runway,
  Taxiway,
  Terminal,
  Vec2,
  World,
} from './lib/types';
import { Accessor, createEffect, createMemo, onMount } from 'solid-js';

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
  let groundScale = createMemo(() => (radar().mode === 'ground' ? 10 : 1));
  let fontSize = createMemo(() => 16 * (radar().scale * 200));

  function scaleFeet(num: number): number {
    const FEET_TO_PIXELS = 0.0025;
    return num * FEET_TO_PIXELS * radar().scale * groundScale();
  }

  function scalePoint(vec2: Vec2): Vec2 {
    let x = scaleFeet(vec2.x) * radar().scale;
    let y = scaleFeet(vec2.y) * radar().scale;

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
          let maxScale = 2.0;
          let minScale = 0.1;

          radar.scale += e.deltaY * -0.0006;
          radar.scale = Math.max(Math.min(radar.scale, maxScale), minScale);

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
    // ctx.translate(radar().scale * -0.5, radar().scale * -0.5);

    // Set 0.0 to be the center of the canvas
    ctx.translate(canvas.width * 0.5, canvas.height * 0.5);
  }

  function loopDraw(canvas: HTMLCanvasElement, dts: number) {
    const width = canvas.width;
    const height = canvas.height;

    let ctx = canvas.getContext('2d');
    if (ctx) {
      ctx.font = `900 ${fontSize()}px monospace`;
      ctx.fillStyle = 'black';

      ctx.resetTransform();
      ctx.fillRect(0, 0, width, height);
      resetTransform(ctx);
      drawCompass(ctx);

      if (radar().mode === 'tower') {
        drawTower(ctx, world(), aircrafts());
      } else if (radar().mode === 'ground') {
        drawGround(ctx, world(), aircrafts());
      }
    }
  }

  function drawCompass(ctx: Ctx) {
    // let half_size = airspaceSize() * 0.5 * radar().scale;
    // let airspace_radius = half_size - 50;
    // ctx.fillStyle = '#888';
    // ctx.textAlign = 'center';
    // ctx.textBaseline = 'middle';
    // let padding = radar().scale * 5000;
    // for (let i = 0; i < 36; i++) {
    //   let text = degreesToHeading(i * 10)
    //     .toString()
    //     .padStart(3, '0');
    //   if (text === '000') {
    //     text = '360';
    //   }
    //   ctx.fillText(
    //     text,
    //     Math.cos(toRadians(i * 10)) * (airspace_radius + padding) + half_size,
    //     Math.sin(toRadians(i * 10)) * (airspace_radius + padding) + half_size
    //   );
    // }
  }

  function drawAirspace(ctx: Ctx, airspace: Airspace) {
    let pos = scalePoint(airspace.pos);

    ctx.strokeStyle = 'white';
    ctx.fillStyle = 'white';
    ctx.beginPath();
    ctx.arc(pos.x, pos.y, scaleFeet(airspace.size), 0, Math.PI * 2);
    ctx.stroke();
  }
  function drawRunway(ctx: Ctx, runway: Runway) {}
  function drawBlip(ctx: Ctx, aircraft: Aircraft) {}

  function drawTerminal(ctx: Ctx, terminal: Terminal) {}
  function drawTaxiway(ctx: Ctx, taxiway: Taxiway) {}
  function drawTaxiwayLabel(ctx: Ctx, taxiway: Taxiway) {}
  function drawRunwayGround(ctx: Ctx, runway: Runway) {}
  function drawBlipGround(ctx: Ctx, aircraft: Aircraft) {}

  function drawTower(ctx: Ctx, world: World, aircrafts: Array<Aircraft>) {
    for (let airspace of world.airspaces) {
      drawAirspace(ctx, airspace);
    }

    for (let airport of world.airports) {
      for (let runway of airport.runways) {
        drawRunway(ctx, runway);
      }
    }

    for (let aircraft of aircrafts) {
      drawBlip(ctx, aircraft);
    }
  }

  function drawGround(ctx: Ctx, world: World, aircrafts: Array<Aircraft>) {
    for (let airport of world.airports) {
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

    for (let aircraft of aircrafts) {
      drawBlipGround(ctx, aircraft);
    }
  }

  return <canvas id="canvas" ref={canvas}></canvas>;
}
