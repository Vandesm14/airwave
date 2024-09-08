use engine::{
  add_degrees, inverse_degrees, move_point,
  structs::{Airport, Gate, Runway, Taxiway, Terminal},
  Line, DOWN, LEFT, RIGHT, UP,
};
use glam::Vec2;

pub fn setup(airport: &mut Airport) {
  let runway_20 = Runway {
    id: "20".into(),
    pos: Vec2::new(0.0, 0.0),
    heading: 200.0,
    length: 7000.0,
  };

  let runway_27: Runway = Runway {
    id: "27".into(),
    pos: Vec2::new(-1000.0, 2400.0),
    heading: 270.0,
    length: 7000.0,
  };

  let taxiway_b = Taxiway {
    id: "B".into(),
    a: move_point(
      runway_27.start(),
      add_degrees(runway_27.heading, 90.0),
      500.0,
    ),
    b: move_point(runway_27.end(), add_degrees(runway_27.heading, 90.0), 500.0),
  };

  let taxiway_c = Taxiway {
    id: "C".into(),
    a: move_point(
      runway_20.start(),
      add_degrees(runway_20.heading, 90.0),
      500.0,
    ),
    b: move_point(runway_20.end(), add_degrees(runway_20.heading, 90.0), 500.0),
  };

  let taxiway_hs_20 = Taxiway {
    id: "D4".into(),
    a: runway_20.start(),
    b: taxiway_c.a,
  };

  let taxiway_hs_27 = Taxiway {
    id: "A4".into(),
    a: runway_27.start(),
    b: move_point(
      runway_27.start(),
      add_degrees(runway_27.heading, 90.0),
      500.0,
    ),
  };

  let a = move_point(taxiway_b.b, UP, 500.0);
  let b = move_point(a, RIGHT, 4000.0);
  let c = move_point(b, UP, 1500.0);
  let d = move_point(c, LEFT, 4000.0);
  let mut terminal_a = Terminal {
    id: 'A',
    a,
    b,
    c,
    d,
    gates: Vec::new(),
    apron: Line::default(),
  };
  terminal_a.apron = Line::new(terminal_a.a, terminal_a.b);

  let gates_line_start = move_point(terminal_a.a, UP, 1200.0);
  let gates = 5;
  let padding = 400.0;
  let spacing = 4000.0 / gates as f32;
  for i in 0..gates {
    let gate = Gate {
      id: format!("A{}", i + 1),
      pos: move_point(gates_line_start, RIGHT, spacing * i as f32 + padding),
      heading: 0.0,
    };
    terminal_a.gates.push(gate);
  }

  let tw_a = move_point(a, RIGHT, 200.0);
  let taxiway_a1 = Taxiway {
    id: "A1".into(),
    a: tw_a,
    b: move_point(tw_a, DOWN, 1000.0),
  };

  let tw_a = move_point(a, RIGHT, 2000.0);
  let taxiway_a2 = Taxiway {
    id: "A2".into(),
    a: tw_a,
    b: move_point(tw_a, DOWN, 1000.0),
  };

  let tw_a = move_point(a, RIGHT, 3800.0);
  let taxiway_a3 = Taxiway {
    id: "A3".into(),
    a: tw_a,
    b: move_point(tw_a, DOWN, 1000.0),
  };

  let taxiway_d1 = Taxiway {
    id: "D1".into(),
    a: taxiway_c.b,
    b: runway_20.end(),
  };

  let taxiway_d2 = Taxiway {
    id: "D2".into(),
    a: move_point(taxiway_c.b, inverse_degrees(runway_20.heading), 1000.0),
    b: move_point(runway_20.end(), inverse_degrees(runway_20.heading), 1000.0),
  };

  let taxiway_d3 = Taxiway {
    id: "D3".into(),
    a: move_point(taxiway_c.b, inverse_degrees(runway_20.heading), 2500.0),
    b: move_point(runway_20.end(), inverse_degrees(runway_20.heading), 2500.0),
  };

  airport.add_runway(runway_20);
  airport.add_runway(runway_27);

  airport.add_taxiway(taxiway_a1);
  airport.add_taxiway(taxiway_a2);
  airport.add_taxiway(taxiway_a3);
  airport.add_taxiway(taxiway_b);
  airport.add_taxiway(taxiway_c);
  airport.add_taxiway(taxiway_d1);
  airport.add_taxiway(taxiway_d2);
  airport.add_taxiway(taxiway_d3);
  airport.add_taxiway(taxiway_hs_20);
  airport.add_taxiway(taxiway_hs_27);

  airport.terminals.push(terminal_a);
}
