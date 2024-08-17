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
    | { type: 'approach' }
    | { type: 'landing'; value: Runway }
    | { type: 'willdepart'; value: { runway: Runway; heading: number } }
    | { type: 'departing'; value: number };
  created: number;
};

type Vec2 = { x: number; y: number };

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
  type: 'normal' | 'holdshort' | 'apron';
  value: {
    id: string;
    a: Vec2;
    b: Vec2;
  };
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
  | {
      type: 'atcreply';
      value: { id: string; frequency: number; reply: string };
    }
  | { type: 'reply'; value: RadioMessage }
  | { type: 'size'; value: number };
