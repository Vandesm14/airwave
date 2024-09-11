export type TaxiWaypointBehavior = {
  type: 'goto' | 'holdshort' | 'takeoff' | 'park';
};

export type NodeVec2 = {
  name: string;
  kind: 'taxiway' | 'runway' | 'gate' | 'apron';
  behavior: 'goto' | 'holdshort';
  value: [number, number];
};

export function arrToVec2(arr: [number, number]): Vec2 {
  return { x: arr[0], y: arr[1] };
}

export type Aircraft = {
  x: number;
  y: number;

  frequency: number;

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

  state:
    | {
        type: 'flying';
        value: { waypoints: Array<NodeVec2> };
      }
    | { type: 'landing'; value: Runway }
    | {
        type: 'taxiing';
        value: {
          current: NodeVec2;
          waypoints: Array<NodeVec2>;
        };
      };
  flight_plan: [string, string];

  created: number;

  airspace: string | null;
};

export type Vec2 = { x: number; y: number };

export type Runway = {
  id: string;
  x: number;
  y: number;
  /** In Degrees (0 is north; up) */
  heading: number;
  /** In Feet */
  length: number;
};

export type Taxiway = {
  id: string;
  a: Vec2;
  b: Vec2;
};

export type Gate = {
  id: string;
  pos: Vec2;
  heading: number;
};

export type Terminal = {
  id: string;
  a: Vec2;
  b: Vec2;
  c: Vec2;
  d: Vec2;

  apron: [[number, number], [number, number]];

  gates: Array<Gate>;
};

export type Airport = {
  id: string;
  center: Vec2;
  runways: Array<Runway>;
  taxiways: Array<Taxiway>;
  terminals: Array<Terminal>;
  altitudeRange: [number, number];
};

export type Frequencies = {
  approach: number;
  departure: number;
  tower: number;
  ground: number;
  center: number;
};

export type Airspace = {
  id: string;
  pos: Vec2;
  size: number;
  airports: Array<Airport>;
  auto: boolean;
  frequencies: Frequencies;
};

export type World = {
  airspaces: Array<Airspace>;
  waypoints: Array<NodeVec2>;
};

export type RadioMessage = {
  id: string;
  frequency: number;
  reply: string;
};

export type ServerEvent =
  | {
      type: 'aircraft';
      value: Aircraft[];
    }
  | { type: 'world'; value: World }
  | {
      type: 'atcreply';
      value: { id: string; frequency: number; reply: string };
    }
  | { type: 'reply'; value: RadioMessage };
