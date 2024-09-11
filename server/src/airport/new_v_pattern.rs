use glam::Vec2;

use engine::{
  add_degrees, find_line_intersection, find_projected_intersection,
  inverse_degrees, move_point,
  objects::{
    airport::{Airport, Gate, Runway, Taxiway, Terminal},
    world::WaypointSet,
  },
  pathfinder::{Node, NodeBehavior, NodeKind},
  Line, CLOCKWISE, COUNTERCLOCKWISE,
};

pub fn setup(
  airport: &mut Airport,
  waypoints: &mut Vec<Node<Vec2>>,
  waypoint_sets: &mut WaypointSet,
) {
  const TAXIWAY_DISTANCE: f32 = 400.0;

  let runway_13 = Runway {
    id: "13".into(),
    pos: airport.center + Vec2::new(1000.0, 0.0),
    heading: 135.0,
    length: 7000.0,
  };

  let runway_22 = Runway {
    id: "22".into(),
    pos: airport.center + Vec2::new(-1000.0, 0.0),
    heading: 225.0,
    length: 7000.0,
  };

  let taxiway_a = Taxiway {
    id: "A".into(),
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
    id: "B".into(),
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
    id: "C".into(),
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
    id: "D".into(),
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
    id: "E1".into(),
    a: taxiway_a.b.lerp(taxiway_a.a, 1.0),
    b: taxiway_b.b.lerp(taxiway_b.a, 1.0),
  };

  let taxiway_e2 = Taxiway {
    id: "E2".into(),
    a: taxiway_a.b.lerp(taxiway_a.a, 0.5),
    b: taxiway_b.b.lerp(taxiway_b.a, 0.5),
  };

  let taxiway_e3 = Taxiway {
    id: "E3".into(),
    a: taxiway_a.b.lerp(taxiway_a.a, 0.25),
    b: taxiway_b.b.lerp(taxiway_b.a, 0.25),
  };

  let taxiway_e4 = Taxiway {
    id: "E4".into(),
    a: taxiway_a.b.lerp(taxiway_a.a, 0.0),
    b: taxiway_b.b.lerp(taxiway_b.a, 0.0),
  };

  let taxiway_f1 = Taxiway {
    id: "F1".into(),
    a: taxiway_c.b.lerp(taxiway_c.a, 1.0),
    b: taxiway_d.b.lerp(taxiway_d.a, 1.0),
  };

  let taxiway_f2 = Taxiway {
    id: "F2".into(),
    a: taxiway_c.b.lerp(taxiway_c.a, 0.5),
    b: taxiway_d.b.lerp(taxiway_d.a, 0.5),
  };

  let taxiway_f3 = Taxiway {
    id: "F3".into(),
    a: taxiway_c.b.lerp(taxiway_c.a, 0.25),
    b: taxiway_d.b.lerp(taxiway_d.a, 0.25),
  };

  let taxiway_f4 = Taxiway {
    id: "F4".into(),
    a: taxiway_c.b.lerp(taxiway_c.a, 0.0),
    b: taxiway_d.b.lerp(taxiway_d.a, 0.0),
  };

  let terminal_a_a = find_projected_intersection(
    taxiway_e2.clone().into(),
    taxiway_f2.clone().into(),
  )
  .unwrap();
  let terminal_a_b = move_point(
    terminal_a_a,
    runway_13.heading,
    taxiway_f2.a.distance(taxiway_f4.a),
  );
  let terminal_a_c = find_projected_intersection(
    taxiway_e4.clone().into(),
    taxiway_f4.clone().into(),
  )
  .unwrap();
  let terminal_a_d = move_point(
    terminal_a_c,
    inverse_degrees(runway_13.heading),
    taxiway_e2.b.distance(taxiway_e4.b),
  );

  let terminal_a = Terminal {
    id: 'A',
    a: terminal_a_a,
    b: terminal_a_b,
    c: terminal_a_c,
    d: terminal_a_d,
    gates: Vec::new(),
    apron: Line::default(),
  };

  let taxiway_g1 = Taxiway {
    id: "G1".into(),
    a: terminal_a_d.lerp(terminal_a_a, 0.1),
    b: taxiway_e4.b.lerp(taxiway_e2.b, 0.1),
  };

  let taxiway_g2 = Taxiway {
    id: "G2".into(),
    a: terminal_a_d.lerp(terminal_a_a, 0.5),
    b: taxiway_e4.b.lerp(taxiway_e2.b, 0.5),
  };

  let taxiway_g3 = Taxiway {
    id: "G3".into(),
    a: terminal_a_d.lerp(terminal_a_a, 0.9),
    b: taxiway_e4.b.lerp(taxiway_e2.b, 0.9),
  };

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

  airport.add_taxiway(taxiway_g1);
  airport.add_taxiway(taxiway_g2);
  airport.add_taxiway(taxiway_g3);

  airport.terminals.push(terminal_a);
}
