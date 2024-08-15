export type Aircraft = {
  pos: [number, number];
  x: number;
  y: number;

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

  state: { type: string; value: Runway };
};

export type Runway = {
  id: string;
  pos: [number, number];
  x: number;
  y: number;
  /** In Degrees (0 is north; up) */
  heading: number;
  /** In Feet */
  length: number;
};

export type RadioMessage = { id: string; reply: string };

export type ServerEvent =
  | {
      type: 'aircraft';
      value: Aircraft[];
    }
  | { type: 'runways'; value: Runway[] }
  | { type: 'atcreply'; value: string }
  | { type: 'reply'; value: RadioMessage }
  | { type: 'size'; value: number };
