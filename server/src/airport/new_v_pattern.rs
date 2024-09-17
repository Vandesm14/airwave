use glam::Vec2;

use engine::{
  add_degrees, find_projected_intersection, inverse_degrees, move_point,
  objects::{
    airport::{Airport, Gate, Runway, Taxiway, Terminal},
    command::Task,
    world::WaypointSet,
  },
  pathfinder::{Node, NodeBehavior, NodeKind, WaypointNodeData},
  Line, CLOCKWISE, COUNTERCLOCKWISE, NAUTICALMILES_TO_FEET,
};

// TODO: Add tasks to the correct waypoints to clear landings, et cetera.

pub fn setup(
  airport: &mut Airport,
  waypoints: &mut Vec<Node<WaypointNodeData>>,
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
    id: 'A',
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
      id: format!("A{}", i),
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
      id: format!("A{}", i + total_gates),
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
    id: 'B',
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
      id: format!("B{}", i),
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
      id: format!("B{}", i + total_gates),
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

  let waypoint_vista = Node {
    name: "VISTA".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: WaypointNodeData {
      to: move_point(
        runway_13.start(),
        inverse_degrees(runway_13.heading),
        NAUTICALMILES_TO_FEET * 12.0,
      ),
      then: vec![Task::Land(runway_13.id.clone())],
    },
  };

  let waypoint_orbit = Node {
    name: "ORBIT".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: WaypointNodeData {
      to: move_point(
        waypoint_vista.value.to,
        inverse_degrees(runway_13.heading),
        NAUTICALMILES_TO_FEET * 4.0,
      ),
      then: vec![Task::Altitude(4000.0), Task::Speed(250.0)],
    },
  };

  let waypoint_crest = Node {
    name: "CREST".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: WaypointNodeData {
      to: move_point(
        waypoint_orbit.value.to,
        inverse_degrees(runway_13.heading),
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![Task::Altitude(4000.0), Task::Speed(250.0)],
    },
  };

  let waypoint_blaze = Node {
    name: "BLAZE".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: WaypointNodeData {
      to: move_point(
        waypoint_orbit.value.to,
        add_degrees(inverse_degrees(runway_13.heading), -45.0),
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![Task::Altitude(4000.0), Task::Speed(250.0)],
    },
  };

  let waypoint_swift = Node {
    name: "SWIFT".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: WaypointNodeData {
      to: move_point(
        waypoint_orbit.value.to,
        add_degrees(inverse_degrees(runway_13.heading), 45.0),
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![Task::Altitude(4000.0), Task::Speed(250.0)],
    },
  };

  //

  let waypoint_sonic = Node {
    name: "SONIC".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: WaypointNodeData {
      to: move_point(
        runway_22.start(),
        inverse_degrees(runway_22.heading),
        NAUTICALMILES_TO_FEET * 12.0,
      ),
      then: vec![Task::Land(runway_22.id.clone())],
    },
  };

  let waypoint_ready = Node {
    name: "READY".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: WaypointNodeData {
      to: move_point(
        waypoint_sonic.value.to,
        inverse_degrees(runway_22.heading),
        NAUTICALMILES_TO_FEET * 4.0,
      ),
      then: vec![Task::Altitude(4000.0), Task::Speed(250.0)],
    },
  };

  let waypoint_short = Node {
    name: "SHORT".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: WaypointNodeData {
      to: move_point(
        waypoint_ready.value.to,
        inverse_degrees(runway_22.heading),
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![Task::Altitude(4000.0), Task::Speed(250.0)],
    },
  };

  let waypoint_quick = Node {
    name: "QUICK".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: WaypointNodeData {
      to: move_point(
        waypoint_ready.value.to,
        add_degrees(inverse_degrees(runway_22.heading), -45.0),
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![Task::Altitude(4000.0), Task::Speed(250.0)],
    },
  };

  let waypoint_arrow = Node {
    name: "ARROW".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: WaypointNodeData {
      to: move_point(
        waypoint_ready.value.to,
        add_degrees(inverse_degrees(runway_22.heading), 45.0),
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![Task::Altitude(4000.0), Task::Speed(250.0)],
    },
  };

  //

  let waypoint_paper = Node {
    name: "PAPER".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: WaypointNodeData {
      to: move_point(
        runway_13.end(),
        runway_13.heading,
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![],
    },
  };

  let waypoint_ghost = Node {
    name: "GHOST".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: WaypointNodeData {
      to: move_point(
        waypoint_paper.value.to,
        runway_13.heading,
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![],
    },
  };

  let waypoint_ocean = Node {
    name: "OCEAN".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: WaypointNodeData {
      to: move_point(
        waypoint_ghost.value.to,
        add_degrees(runway_13.heading, -45.0),
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![],
    },
  };

  let waypoint_goose = Node {
    name: "GOOSE".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: WaypointNodeData {
      to: move_point(
        waypoint_ghost.value.to,
        add_degrees(runway_13.heading, 45.0),
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![],
    },
  };

  //

  let waypoint_quack = Node {
    name: "QUACK".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: WaypointNodeData {
      to: move_point(
        runway_22.end(),
        runway_22.heading,
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![],
    },
  };

  let waypoint_state = Node {
    name: "STATE".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: WaypointNodeData {
      to: move_point(
        waypoint_quack.value.to,
        runway_22.heading,
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![],
    },
  };

  let waypoint_unite = Node {
    name: "UNITE".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: WaypointNodeData {
      to: move_point(
        waypoint_state.value.to,
        add_degrees(runway_22.heading, -45.0),
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![],
    },
  };

  let waypoint_royal = Node {
    name: "ROYAL".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: WaypointNodeData {
      to: move_point(
        waypoint_state.value.to,
        add_degrees(runway_22.heading, 45.0),
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![],
    },
  };

  //

  waypoint_sets.arrival.insert(
    "A6".into(),
    vec![
      "A6".into(),
      waypoint_blaze.name.clone(),
      waypoint_orbit.name.clone(),
      waypoint_vista.name.clone(),
    ],
  );

  waypoint_sets.arrival.insert(
    "E4".into(),
    vec![
      "E4".into(),
      waypoint_crest.name.clone(),
      waypoint_orbit.name.clone(),
      waypoint_vista.name.clone(),
    ],
  );

  waypoint_sets.arrival.insert(
    "A5".into(),
    vec![
      "A5".into(),
      waypoint_swift.name.clone(),
      waypoint_orbit.name.clone(),
      waypoint_vista.name.clone(),
    ],
  );

  waypoint_sets.arrival.insert(
    "A9".into(),
    vec![
      "A9".into(),
      waypoint_short.name.clone(),
      waypoint_ready.name.clone(),
      waypoint_sonic.name.clone(),
    ],
  );

  waypoint_sets.arrival.insert(
    "B1".into(),
    vec![
      "B1".into(),
      waypoint_arrow.name.clone(),
      waypoint_ready.name.clone(),
      waypoint_sonic.name.clone(),
    ],
  );

  //

  waypoint_sets.approach.insert(
    waypoint_blaze.name.clone(),
    vec![
      waypoint_blaze.name.clone(),
      waypoint_orbit.name.clone(),
      waypoint_vista.name.clone(),
    ],
  );

  waypoint_sets.approach.insert(
    waypoint_crest.name.clone(),
    vec![
      waypoint_crest.name.clone(),
      waypoint_orbit.name.clone(),
      waypoint_vista.name.clone(),
    ],
  );

  waypoint_sets.approach.insert(
    waypoint_swift.name.clone(),
    vec![
      waypoint_swift.name.clone(),
      waypoint_orbit.name.clone(),
      waypoint_vista.name.clone(),
    ],
  );

  waypoint_sets.approach.insert(
    waypoint_royal.name.clone(),
    vec![
      waypoint_royal.name.clone(),
      waypoint_blaze.name.clone(),
      waypoint_orbit.name.clone(),
      waypoint_vista.name.clone(),
    ],
  );

  //

  waypoint_sets.approach.insert(
    waypoint_quick.name.clone(),
    vec![
      waypoint_quick.name.clone(),
      waypoint_ready.name.clone(),
      waypoint_sonic.name.clone(),
    ],
  );

  waypoint_sets.approach.insert(
    waypoint_short.name.clone(),
    vec![
      waypoint_short.name.clone(),
      waypoint_ready.name.clone(),
      waypoint_sonic.name.clone(),
    ],
  );

  waypoint_sets.approach.insert(
    waypoint_arrow.name.clone(),
    vec![
      waypoint_arrow.name.clone(),
      waypoint_ready.name.clone(),
      waypoint_sonic.name.clone(),
    ],
  );

  waypoint_sets.approach.insert(
    waypoint_ocean.name.clone(),
    vec![
      waypoint_ocean.name.clone(),
      waypoint_arrow.name.clone(),
      waypoint_ready.name.clone(),
      waypoint_sonic.name.clone(),
    ],
  );

  //

  waypoint_sets.departure.insert(
    waypoint_royal.name.clone(),
    vec![
      waypoint_quack.name.clone(),
      waypoint_state.name.clone(),
      waypoint_royal.name.clone(),
    ],
  );

  waypoint_sets.departure.insert(
    waypoint_state.name.clone(),
    vec![waypoint_quack.name.clone(), waypoint_state.name.clone()],
  );

  waypoint_sets.departure.insert(
    waypoint_unite.name.clone(),
    vec![
      waypoint_quack.name.clone(),
      waypoint_state.name.clone(),
      waypoint_unite.name.clone(),
    ],
  );

  waypoint_sets.departure.insert(
    waypoint_blaze.name.clone(),
    vec![
      waypoint_quack.name.clone(),
      waypoint_state.name.clone(),
      waypoint_royal.name.clone(),
      waypoint_blaze.name.clone(),
    ],
  );

  //

  waypoint_sets.departure.insert(
    waypoint_goose.name.clone(),
    vec![
      waypoint_paper.name.clone(),
      waypoint_ghost.name.clone(),
      waypoint_goose.name.clone(),
    ],
  );

  waypoint_sets.departure.insert(
    waypoint_ghost.name.clone(),
    vec![waypoint_paper.name.clone(), waypoint_ghost.name.clone()],
  );

  waypoint_sets.departure.insert(
    waypoint_ocean.name.clone(),
    vec![
      waypoint_paper.name.clone(),
      waypoint_ghost.name.clone(),
      waypoint_ocean.name.clone(),
    ],
  );

  waypoint_sets.departure.insert(
    waypoint_arrow.name.clone(),
    vec![
      waypoint_paper.name.clone(),
      waypoint_ghost.name.clone(),
      waypoint_ocean.name.clone(),
      waypoint_arrow.name.clone(),
    ],
  );

  waypoints.push(waypoint_vista);
  waypoints.push(waypoint_orbit);
  waypoints.push(waypoint_crest);
  waypoints.push(waypoint_blaze);
  waypoints.push(waypoint_swift);

  waypoints.push(waypoint_sonic);
  waypoints.push(waypoint_ready);
  waypoints.push(waypoint_short);
  waypoints.push(waypoint_quick);
  waypoints.push(waypoint_arrow);

  waypoints.push(waypoint_paper);
  waypoints.push(waypoint_ghost);
  waypoints.push(waypoint_ocean);
  waypoints.push(waypoint_goose);

  waypoints.push(waypoint_quack);
  waypoints.push(waypoint_state);
  waypoints.push(waypoint_unite);
  waypoints.push(waypoint_royal);

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
