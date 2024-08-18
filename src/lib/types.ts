export type TaxiPoint =
  | {
      type: 'taxiway';
      value: Taxiway;
    }
  | {
      type: 'runway';
      value: Runway;
    }
  | {
      type: 'gate';
      value: [Terminal, Gate];
    };

export type TaxiWaypoint = {
  pos: Vec2;
  wp: TaxiPoint;
  hold: boolean;
};

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
    | { type: 'flying' }
    | { type: 'landing'; value: Runway }
    | {
        type: 'taxiing';
        value: {
          current: TaxiWaypoint;
          waypoints: Array<TaxiWaypoint>;
        };
      };
  intention:
    | { type: 'land' }
    | { type: 'flyover' }
    | { type: 'depart'; value: number };

  created: number;
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
  kind: { type: 'normal' | 'holdshort' | 'apron'; value: string };
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

  gates: Array<Gate>;
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
  | { type: 'runways'; value: Runway[] }
  | { type: 'taxiways'; value: Taxiway[] }
  | { type: 'terminals'; value: Terminal[] }
  | {
      type: 'atcreply';
      value: { id: string; frequency: number; reply: string };
    }
  | { type: 'reply'; value: RadioMessage }
  | { type: 'size'; value: number };
