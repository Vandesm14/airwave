use engine::{
  move_point,
  structs::{Airport, Gate, Runway, Taxiway, Terminal},
  Line, DOWN, UP,
};
use glam::Vec2;

pub fn setup(airport: &mut Airport) {
  const ENTRYWAY_TAXIWAY_DISTANCE: f32 = 500.0;

  // MARK: Right.

  let runway_27r = Runway {
    id: "27R".into(),
    pos: airport.center + Vec2::Y * 1500.0,
    heading: 270.0,
    length: 7000.0,
  };

  let taxiway_b = Taxiway {
    id: "B".into(),
    a: move_point(runway_27r.start(), DOWN, ENTRYWAY_TAXIWAY_DISTANCE),
    b: move_point(runway_27r.end(), DOWN, ENTRYWAY_TAXIWAY_DISTANCE),
  };

  let taxiway_a1 = Taxiway {
    id: "A1".into(),
    a: runway_27r.end(),
    b: move_point(runway_27r.end(), DOWN, ENTRYWAY_TAXIWAY_DISTANCE),
  };

  let taxiway_a2 = Taxiway {
    id: "A2".into(),
    a: runway_27r.start().lerp(runway_27r.end(), 0.75),
    b: move_point(
      runway_27r.start().lerp(runway_27r.end(), 0.75),
      DOWN,
      ENTRYWAY_TAXIWAY_DISTANCE,
    ),
  };

  let taxiway_a3 = Taxiway {
    id: "A3".into(),
    a: runway_27r.start().lerp(runway_27r.end(), 0.5),
    b: move_point(
      runway_27r.start().lerp(runway_27r.end(), 0.5),
      DOWN,
      ENTRYWAY_TAXIWAY_DISTANCE,
    ),
  };

  let taxiway_a4 = Taxiway {
    id: "A4".into(),
    a: runway_27r.start().lerp(runway_27r.end(), 0.25),
    b: move_point(
      runway_27r.start().lerp(runway_27r.end(), 0.25),
      DOWN,
      ENTRYWAY_TAXIWAY_DISTANCE,
    ),
  };

  let taxiway_a5 = Taxiway {
    id: "A5".into(),
    a: runway_27r.start().lerp(runway_27r.end(), 0.0),
    b: move_point(
      runway_27r.start().lerp(runway_27r.end(), 0.0),
      DOWN,
      ENTRYWAY_TAXIWAY_DISTANCE,
    ),
  };

  // MARK: Left.

  let runway_27l = Runway {
    id: "27L".into(),
    pos: airport.center + Vec2::Y * -1500.0,
    heading: 270.0,
    length: 7000.0,
  };

  let taxiway_c = Taxiway {
    id: "C".into(),
    a: move_point(runway_27l.start(), UP, ENTRYWAY_TAXIWAY_DISTANCE),
    b: move_point(runway_27l.end(), UP, ENTRYWAY_TAXIWAY_DISTANCE),
  };

  let taxiway_d1 = Taxiway {
    id: "D1".into(),
    a: runway_27l.end(),
    b: move_point(runway_27l.end(), UP, ENTRYWAY_TAXIWAY_DISTANCE),
  };

  let taxiway_d2 = Taxiway {
    id: "D2".into(),
    a: runway_27l.start().lerp(runway_27l.end(), 0.75),
    b: move_point(
      runway_27l.start().lerp(runway_27l.end(), 0.75),
      UP,
      ENTRYWAY_TAXIWAY_DISTANCE,
    ),
  };

  let taxiway_d3 = Taxiway {
    id: "D3".into(),
    a: runway_27l.start().lerp(runway_27l.end(), 0.5),
    b: move_point(
      runway_27l.start().lerp(runway_27l.end(), 0.5),
      UP,
      ENTRYWAY_TAXIWAY_DISTANCE,
    ),
  };

  let taxiway_d4 = Taxiway {
    id: "D4".into(),
    a: runway_27l.start().lerp(runway_27l.end(), 0.25),
    b: move_point(
      runway_27l.start().lerp(runway_27l.end(), 0.25),
      UP,
      ENTRYWAY_TAXIWAY_DISTANCE,
    ),
  };

  let taxiway_d5 = Taxiway {
    id: "D5".into(),
    a: runway_27l.start().lerp(runway_27l.end(), 0.0),
    b: move_point(
      runway_27l.start().lerp(runway_27l.end(), 0.0),
      UP,
      ENTRYWAY_TAXIWAY_DISTANCE,
    ),
  };

  // MARK: Terminals.

  let mut terminal_a = Terminal {
    id: 'A',
    a: taxiway_a2.b,
    b: taxiway_a3.b,
    c: move_point(taxiway_a3.b, DOWN, 750.0),
    d: move_point(taxiway_a2.b, DOWN, 750.0),
    apron: Line::new(taxiway_a2.b, taxiway_a3.b),
    gates: Vec::new(),
  };

  let mut terminal_b = Terminal {
    id: 'B',
    a: taxiway_d2.b,
    b: taxiway_d3.b,
    c: move_point(taxiway_d3.b, UP, 750.0),
    d: move_point(taxiway_d2.b, UP, 750.0),
    apron: Line::new(taxiway_d2.b, taxiway_d3.b),
    gates: Vec::new(),
  };

  const GATES_PER_TERMINAL: usize = 4;

  // TODO: Shift the gates back over to where they're supposed to be
  for i in 1..=GATES_PER_TERMINAL {
    terminal_a.gates.push(Gate {
      id: format!("{}{i}", terminal_a.id),
      heading: DOWN,
      pos: move_point(
        terminal_a
          .c
          .lerp(terminal_a.d, (1.0 / GATES_PER_TERMINAL as f32) * i as f32),
        UP,
        150.0,
      ),
    });

    terminal_b.gates.push(Gate {
      id: format!("{}{i}", terminal_b.id),
      heading: UP,
      pos: move_point(
        terminal_b
          .c
          .lerp(terminal_b.d, (1.0 / GATES_PER_TERMINAL as f32) * i as f32),
        DOWN,
        150.0,
      ),
    });
  }

  // MARK: Right.

  airport.add_runway(runway_27r);

  airport.add_taxiway(taxiway_b);
  airport.add_taxiway(taxiway_a1);
  airport.add_taxiway(taxiway_a2);
  airport.add_taxiway(taxiway_a3);
  airport.add_taxiway(taxiway_a4);
  airport.add_taxiway(taxiway_a5);

  // MARK: Left.

  airport.add_runway(runway_27l);

  airport.add_taxiway(taxiway_c);
  airport.add_taxiway(taxiway_d1);
  airport.add_taxiway(taxiway_d2);
  airport.add_taxiway(taxiway_d3);
  airport.add_taxiway(taxiway_d4);
  airport.add_taxiway(taxiway_d5);

  // MARK: Terminals.

  airport.terminals.push(terminal_a);
  airport.terminals.push(terminal_b);
}
