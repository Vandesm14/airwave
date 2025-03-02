export type TaxiWaypointBehavior = {
  type: 'goto' | 'holdshort' | 'takeoff' | 'park';
};

export type NodeVec2 = {
  name: string;
  kind: 'taxiway' | 'gate' | 'apron' | 'runway';
  behavior: 'goto' | 'holdshort' | 'park';
  value: Vec2;
};

export type NodeVOR = {
  name: string;
  kind: 'vor';
  behavior: 'goto' | 'holdshort';
  value: { to: Vec2; then: Array<unknown> };
};

export type AircraftStateFlying = {
  type: 'flying';
  value: { waypoints: Array<NodeVOR> };
};

export type LandingState =
  | 'before-turn'
  | 'turning'
  | 'correcting'
  | 'localizer'
  | 'glideslope'
  | 'touchdown'
  | 'go-around';

export type TaxiingState = 'armed' | 'stopped' | 'override' | 'holding';

export type AircraftStateLanding = {
  type: 'landing';
  value: {
    runway: Runway;
    state: LandingState;
  };
};

export type AircraftStateTaxiing = {
  type: 'taxiing';
  value: {
    current: NodeVec2;
    waypoints: Array<NodeVec2>;
    state: TaxiingState;
  };
};

export type AircraftStateParked = {
  type: 'parked';
  value: {
    at: NodeVec2;
    active: boolean;
  };
};

export type AircraftState =
  | AircraftStateFlying
  | AircraftStateLanding
  | AircraftStateTaxiing
  | AircraftStateParked;

type Duration = {
  secs: number;
  nanos: number;
};

export function newDuration(secs: number, nanos: number): Duration {
  return { secs, nanos };
}

export type Aircraft = {
  id: string;
  is_colliding: boolean;

  pos: Vec2;
  /** In Knots */
  speed: number;
  /** In Degrees (0 is north; up) */
  heading: number;
  /** In Feet */
  altitude: number;

  state: AircraftState;
  target: {
    /** In Knots */
    speed: number;
    /** In Degrees (0 is north; up) */
    heading: number;
    /** In Feet */
    altitude: number;
  };
  flight_plan: {
    departing: string;
    arriving: string;

    speed: number;
    altitude: number;
  };

  frequency: number;
};

export function isAircraftFlying(
  state: AircraftState
): state is AircraftStateFlying {
  return state.type === 'flying';
}

export function isAircraftLanding(
  state: AircraftState
): state is AircraftStateLanding {
  return state.type === 'landing';
}

export function isAircraftTaxiing(
  state: AircraftState
): state is AircraftStateTaxiing {
  return state.type === 'taxiing';
}

export function isAircraftParked(
  state: AircraftState
): state is AircraftStateParked {
  return state.type === 'parked';
}

export type Vec2 = [number, number];

export type Runway = {
  id: string;
  pos: Vec2;
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

  apron: [Vec2, Vec2];

  gates: Array<Gate>;
};

export type Airport = {
  id: string;
  center: Vec2;
  runways: Array<Runway>;
  taxiways: Array<Taxiway>;
  terminals: Array<Terminal>;
  frequencies: Frequencies;
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
  radius: number;
  airports: Array<Airport>;
};

export function DefaultAirspace(): Airspace {
  return {
    id: 'KSFO',
    pos: [0, 0],
    radius: 500,
    airports: [],
  };
}

export type World = {
  airspaces: Array<Airspace>;
};

export function DefaultWorld(): World {
  return {
    airspaces: [DefaultAirspace()],
  };
}

export type RadioMessage = {
  id: string;
  frequency: number;
  reply: string;
  created: Duration;
};

export type Game = {};
