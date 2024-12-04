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
  value: { waypoints: Array<NodeVOR>; enroute: boolean };
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
  frequencies: Frequencies;
};

export function DefaultAirspace(): Airspace {
  return {
    id: 'KSFO',
    pos: [0, 0],
    radius: 500,
    airports: [],
    frequencies: {
      approach: 118.5,
      departure: 118.5,
      tower: 118.5,
      ground: 118.5,
      center: 118.5,
    },
  };
}

export type Connection = {
  id: string;
  state: 'inactive' | 'active';
  pos: Vec2;
  transition: Vec2;
};

export type World = {
  airspace: Airspace;
  connections: Array<Connection>;
};

export function DefaultWorld(): World {
  return {
    airspace: DefaultAirspace(),
    connections: [],
  };
}

export type RadioMessage = {
  id: string;
  frequency: number;
  reply: string;
  created: Duration;
};

export type Game = {
  points: Points;
};

export type Points = {
  landings: number;
  landing_rate: {
    rate: Duration;
    marks: Duration[];
  };
  takeoffs: number;
  takeoff_rate: {
    rate: Duration;
    marks: Duration[];
  };
};

export type Flight = {
  id: number;
  kind: 'inbound' | 'outbound';
  spawn_at: Duration;
  status:
    | { type: 'scheduled' }
    | { type: 'ongoing'; value: string }
    | { type: 'completed'; value: string };
};
