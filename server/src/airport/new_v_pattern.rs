use glam::Vec2;

use engine::{
  add_degrees,
  entities::airport::{Airport, Gate, Runway, Taxiway, Terminal},
  find_projected_intersection, inverse_degrees, move_point,
  Line, CLOCKWISE, COUNTERCLOCKWISE,
};
use internment::Intern;

pub fn setup(airport: &mut Airport) {
  const TAXIWAY_DISTANCE: f32 = 400.0;

  let runway_13 = Runway {
    id: Intern::from_ref("13"),
    pos: airport.center + Vec2::new(1000.0, 0.0),
    heading: 135.0,
    length: 7000.0,
  };

  let runway_22 = Runway {
    id: Intern::from_ref("22"),
    pos: airport.center + Vec2::new(-1000.0, 0.0),
    heading: 225.0,
    length: 7000.0,
  };

  let taxiway_a = Taxiway {
    id: Intern::from_ref("A"),
    a: move_point(
      runway_22.start(),
      add_degrees(runway_22.heading, CLOCKWISE),
      TAXIWAY_DISTANCE,
    ),
    b: move_point(
      runway_22.end(),
      add_degrees(runway_22.heading, CLOCKWISE),
      TAXIWAY_DISTANCE,
    ),
  };

  let taxiway_b = Taxiway {
    id: Intern::from_ref("B"),
    a: move_point(
      runway_22.start(),
      add_degrees(runway_22.heading, COUNTERCLOCKWISE),
      TAXIWAY_DISTANCE,
    ),
    b: move_point(
      runway_22.end(),
      add_degrees(runway_22.heading, COUNTERCLOCKWISE),
      TAXIWAY_DISTANCE,
    ),
  };

  let taxiway_c = Taxiway {
    id: Intern::from_ref("C"),
    a: move_point(
      runway_13.start(),
      add_degrees(runway_13.heading, CLOCKWISE),
      TAXIWAY_DISTANCE,
    ),
    b: move_point(
      runway_13.end(),
      add_degrees(runway_13.heading, CLOCKWISE),
      TAXIWAY_DISTANCE,
    ),
  };

  let taxiway_d = Taxiway {
    id: Intern::from_ref("D"),
    a: move_point(
      runway_13.start(),
      add_degrees(runway_13.heading, COUNTERCLOCKWISE),
      TAXIWAY_DISTANCE,
    ),
    b: move_point(
      runway_13.end(),
      add_degrees(runway_13.heading, COUNTERCLOCKWISE),
      TAXIWAY_DISTANCE,
    ),
  };

  let taxiway_e1 = Taxiway {
    id: Intern::from_ref("E1"),
    a: taxiway_a.b.lerp(taxiway_a.a, 1.0),
    b: taxiway_b.b.lerp(taxiway_b.a, 1.0),
  };

  let taxiway_e2 = Taxiway {
    id: Intern::from_ref("E2"),
    a: taxiway_a.b.lerp(taxiway_a.a, 0.5),
    b: taxiway_b.b.lerp(taxiway_b.a, 0.5),
  };

  let taxiway_e3 = Taxiway {
    id: Intern::from_ref("E3"),
    a: taxiway_a.b.lerp(taxiway_a.a, 0.25),
    b: taxiway_b.b.lerp(taxiway_b.a, 0.25),
  };

  let taxiway_e4 = Taxiway {
    id: Intern::from_ref("E4"),
    a: taxiway_a.b.lerp(taxiway_a.a, 0.0),
    b: taxiway_b.b.lerp(taxiway_b.a, 0.0),
  };

  let taxiway_f1 = Taxiway {
    id: Intern::from_ref("F1"),
    a: taxiway_c.b.lerp(taxiway_c.a, 1.0),
    b: taxiway_d.b.lerp(taxiway_d.a, 1.0),
  };

  let taxiway_f2 = Taxiway {
    id: Intern::from_ref("F2"),
    a: taxiway_c.b.lerp(taxiway_c.a, 0.5),
    b: taxiway_d.b.lerp(taxiway_d.a, 0.5),
  };

  let taxiway_f3 = Taxiway {
    id: Intern::from_ref("F3"),
    a: taxiway_c.b.lerp(taxiway_c.a, 0.25),
    b: taxiway_d.b.lerp(taxiway_d.a, 0.25),
  };

  let taxiway_f4 = Taxiway {
    id: Intern::from_ref("F4"),
    a: taxiway_c.b.lerp(taxiway_c.a, 0.0),
    b: taxiway_d.b.lerp(taxiway_d.a, 0.0),
  };

  let terminal_a_a = taxiway_e4.b;
  let terminal_a_b = taxiway_e3.b;
  let terminal_a_c = find_projected_intersection(
    taxiway_f3.clone().into(),
    taxiway_e3.clone().into(),
  )
  .unwrap();
  let terminal_a_d = find_projected_intersection(
    taxiway_f3.clone().into(),
    taxiway_e4.clone().into(),
  )
  .unwrap();

  let mut terminal_a = Terminal {
    id: Intern::from_ref("A"),
    a: terminal_a_a,
    b: terminal_a_b,
    c: terminal_a_c,
    d: terminal_a_d,
    gates: Vec::new(),
    apron: Line::new(
      terminal_a_a.lerp(terminal_a_b, 0.5),
      terminal_a_c.lerp(terminal_a_d, 0.5),
    )
    .extend(10.0),
  };

  let total_gates = 6;
  for i in 1..=total_gates {
    let gate = Gate {
      id: Intern::from(format!("A{}", i)),
      pos: move_point(
        terminal_a
          .apron
          .0
          .lerp(terminal_a.apron.1, i as f32 / (total_gates + 1) as f32),
        inverse_degrees(runway_22.heading),
        terminal_a.a.distance(terminal_a.b) * 0.35,
      ),
      heading: inverse_degrees(runway_22.heading),
    };
    terminal_a.gates.push(gate);
  }
  for i in 1..=total_gates {
    let gate = Gate {
      id: Intern::from(format!("A{}", i + total_gates)),
      pos: move_point(
        terminal_a
          .apron
          .0
          .lerp(terminal_a.apron.1, i as f32 / (total_gates + 1) as f32),
        runway_22.heading,
        terminal_a.a.distance(terminal_a.b) * 0.35,
      ),
      heading: runway_22.heading,
    };
    terminal_a.gates.push(gate);
  }

  let terminal_b_a = taxiway_f3.a;
  let terminal_b_b = taxiway_f4.a;
  let terminal_b_c = find_projected_intersection(
    taxiway_f4.clone().into(),
    taxiway_e3.clone().into(),
  )
  .unwrap();
  let terminal_b_d = find_projected_intersection(
    taxiway_f3.clone().into(),
    taxiway_e3.clone().into(),
  )
  .unwrap();

  let mut terminal_b = Terminal {
    id: Intern::from_ref("B"),
    a: terminal_b_a,
    b: terminal_b_b,
    c: terminal_b_c,
    d: terminal_b_d,
    gates: Vec::new(),
    apron: Line::new(
      terminal_b_a.lerp(terminal_b_b, 0.5),
      terminal_b_c.lerp(terminal_b_d, 0.5),
    )
    .extend(10.0),
  };

  let total_gates = 6;
  for i in 1..=total_gates {
    let gate = Gate {
      id: Intern::from(format!("B{}", i)),
      pos: move_point(
        terminal_b
          .apron
          .0
          .lerp(terminal_b.apron.1, i as f32 / (total_gates + 1) as f32),
        inverse_degrees(runway_13.heading),
        terminal_b.a.distance(terminal_b.b) * 0.35,
      ),
      heading: inverse_degrees(runway_13.heading),
    };
    terminal_b.gates.push(gate);
  }
  for i in 1..=total_gates {
    let gate = Gate {
      id: Intern::from(format!("B{}", i + total_gates)),
      pos: move_point(
        terminal_b
          .apron
          .0
          .lerp(terminal_b.apron.1, i as f32 / (total_gates + 1) as f32),
        runway_13.heading,
        terminal_b.a.distance(terminal_b.b) * 0.35,
      ),
      heading: runway_13.heading,
    };
    terminal_b.gates.push(gate);
  }

  airport.add_runway(runway_13);
  airport.add_runway(runway_22);

  airport.add_taxiway(taxiway_a);
  airport.add_taxiway(taxiway_b);
  airport.add_taxiway(taxiway_c);
  airport.add_taxiway(taxiway_d);

  airport.add_taxiway(taxiway_e1);
  airport.add_taxiway(taxiway_e2);
  airport.add_taxiway(taxiway_e3);
  airport.add_taxiway(taxiway_e4);

  airport.add_taxiway(taxiway_f1);
  airport.add_taxiway(taxiway_f2);
  airport.add_taxiway(taxiway_f3);
  airport.add_taxiway(taxiway_f4);

  airport.terminals.push(terminal_a);
  airport.terminals.push(terminal_b);
}
