use glam::Vec2;

use engine::{
  add_degrees,
  entities::{
    aircraft::events::EventKind,
    airport::{Airport, Gate, Runway, Taxiway, Terminal},
    world::WaypointSet,
  },
  find_projected_intersection, inverse_degrees, move_point,
  pathfinder::{Node, NodeBehavior, NodeKind, NodeVORData},
  Line, CLOCKWISE, COUNTERCLOCKWISE, NAUTICALMILES_TO_FEET,
};
use internment::Intern;

pub fn setup(
  airport: &mut Airport,
  waypoints: &mut Vec<Node<NodeVORData>>,
  waypoint_sets: &mut WaypointSet,
) {
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

  let waypoint_vista = Node {
    name: Intern::from_ref("VISTA"),
    kind: NodeKind::VOR,
    behavior: NodeBehavior::GoTo,
    value: NodeVORData {
      to: move_point(
        runway_13.start(),
        inverse_degrees(runway_13.heading),
        NAUTICALMILES_TO_FEET * 12.0,
      ),
      then: vec![EventKind::Land(runway_13.id)],
    },
  };

  let waypoint_orbit = Node {
    name: Intern::from_ref("ORBIT"),
    kind: NodeKind::VOR,
    behavior: NodeBehavior::GoTo,
    value: NodeVORData {
      to: move_point(
        waypoint_vista.value.to,
        inverse_degrees(runway_13.heading),
        NAUTICALMILES_TO_FEET * 4.0,
      ),
      then: vec![EventKind::Altitude(4000.0), EventKind::Speed(250.0)],
    },
  };

  let waypoint_crest = Node {
    name: Intern::from_ref("CREST"),
    kind: NodeKind::VOR,
    behavior: NodeBehavior::GoTo,
    value: NodeVORData {
      to: move_point(
        waypoint_orbit.value.to,
        inverse_degrees(runway_13.heading),
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![EventKind::Altitude(4000.0), EventKind::Speed(250.0)],
    },
  };

  let waypoint_blaze = Node {
    name: Intern::from_ref("BLAZE"),
    kind: NodeKind::VOR,
    behavior: NodeBehavior::GoTo,
    value: NodeVORData {
      to: move_point(
        waypoint_orbit.value.to,
        add_degrees(inverse_degrees(runway_13.heading), -45.0),
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![EventKind::Altitude(4000.0), EventKind::Speed(250.0)],
    },
  };

  let waypoint_swift = Node {
    name: Intern::from_ref("SWIFT"),
    kind: NodeKind::VOR,
    behavior: NodeBehavior::GoTo,
    value: NodeVORData {
      to: move_point(
        waypoint_orbit.value.to,
        add_degrees(inverse_degrees(runway_13.heading), 45.0),
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![EventKind::Altitude(4000.0), EventKind::Speed(250.0)],
    },
  };

  //

  let waypoint_sonic = Node {
    name: Intern::from_ref("SONIC"),
    kind: NodeKind::VOR,
    behavior: NodeBehavior::GoTo,
    value: NodeVORData {
      to: move_point(
        runway_22.start(),
        inverse_degrees(runway_22.heading),
        NAUTICALMILES_TO_FEET * 12.0,
      ),
      then: vec![EventKind::Land(runway_22.id)],
    },
  };

  let waypoint_ready = Node {
    name: Intern::from_ref("READY"),
    kind: NodeKind::VOR,
    behavior: NodeBehavior::GoTo,
    value: NodeVORData {
      to: move_point(
        waypoint_sonic.value.to,
        inverse_degrees(runway_22.heading),
        NAUTICALMILES_TO_FEET * 4.0,
      ),
      then: vec![EventKind::Altitude(4000.0), EventKind::Speed(250.0)],
    },
  };

  let waypoint_short = Node {
    name: Intern::from_ref("SHORT"),
    kind: NodeKind::VOR,
    behavior: NodeBehavior::GoTo,
    value: NodeVORData {
      to: move_point(
        waypoint_ready.value.to,
        inverse_degrees(runway_22.heading),
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![EventKind::Altitude(4000.0), EventKind::Speed(250.0)],
    },
  };

  let waypoint_quick = Node {
    name: Intern::from_ref("QUICK"),
    kind: NodeKind::VOR,
    behavior: NodeBehavior::GoTo,
    value: NodeVORData {
      to: move_point(
        waypoint_ready.value.to,
        add_degrees(inverse_degrees(runway_22.heading), -45.0),
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![EventKind::Altitude(4000.0), EventKind::Speed(250.0)],
    },
  };

  let waypoint_arrow = Node {
    name: Intern::from_ref("ARROW"),
    kind: NodeKind::VOR,
    behavior: NodeBehavior::GoTo,
    value: NodeVORData {
      to: move_point(
        waypoint_ready.value.to,
        add_degrees(inverse_degrees(runway_22.heading), 45.0),
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![EventKind::Altitude(4000.0), EventKind::Speed(250.0)],
    },
  };

  //

  let waypoint_paper = Node {
    name: Intern::from_ref("PAPER"),
    kind: NodeKind::VOR,
    behavior: NodeBehavior::GoTo,
    value: NodeVORData {
      to: move_point(
        runway_13.end(),
        runway_13.heading,
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![],
    },
  };

  let waypoint_ghost = Node {
    name: Intern::from_ref("GHOST"),
    kind: NodeKind::VOR,
    behavior: NodeBehavior::GoTo,
    value: NodeVORData {
      to: move_point(
        waypoint_paper.value.to,
        runway_13.heading,
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![],
    },
  };

  let waypoint_ocean = Node {
    name: Intern::from_ref("OCEAN"),
    kind: NodeKind::VOR,
    behavior: NodeBehavior::GoTo,
    value: NodeVORData {
      to: move_point(
        waypoint_ghost.value.to,
        add_degrees(runway_13.heading, -45.0),
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![],
    },
  };

  let waypoint_goose = Node {
    name: Intern::from_ref("GOOSE"),
    kind: NodeKind::VOR,
    behavior: NodeBehavior::GoTo,
    value: NodeVORData {
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
    name: Intern::from_ref("QUACK"),
    kind: NodeKind::VOR,
    behavior: NodeBehavior::GoTo,
    value: NodeVORData {
      to: move_point(
        runway_22.end(),
        runway_22.heading,
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![],
    },
  };

  let waypoint_state = Node {
    name: Intern::from_ref("STATE"),
    kind: NodeKind::VOR,
    behavior: NodeBehavior::GoTo,
    value: NodeVORData {
      to: move_point(
        waypoint_quack.value.to,
        runway_22.heading,
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![],
    },
  };

  let waypoint_unite = Node {
    name: Intern::from_ref("UNITE"),
    kind: NodeKind::VOR,
    behavior: NodeBehavior::GoTo,
    value: NodeVORData {
      to: move_point(
        waypoint_state.value.to,
        add_degrees(runway_22.heading, -45.0),
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![],
    },
  };

  let waypoint_royal = Node {
    name: Intern::from_ref("ROYAL"),
    kind: NodeKind::VOR,
    behavior: NodeBehavior::GoTo,
    value: NodeVORData {
      to: move_point(
        waypoint_state.value.to,
        add_degrees(runway_22.heading, 45.0),
        NAUTICALMILES_TO_FEET * 6.0,
      ),
      then: vec![],
    },
  };

  // Runway 13 Approaches

  waypoint_sets.approach.insert(
    waypoint_blaze.name,
    vec![
      waypoint_blaze.clone().with_behavior(vec![
        EventKind::AltitudeAtOrBelow(7000.0),
        EventKind::SpeedAtOrBelow(300.0),
      ]),
      waypoint_orbit.clone().with_behavior(vec![
        EventKind::AltitudeAtOrBelow(4000.0),
        EventKind::SpeedAtOrBelow(230.0),
      ]),
      waypoint_vista
        .clone()
        .with_behavior(vec![EventKind::Land(Intern::from_ref("13"))]),
    ],
  );

  waypoint_sets.approach.insert(
    waypoint_crest.name,
    vec![
      waypoint_crest.clone().with_behavior(vec![
        EventKind::AltitudeAtOrBelow(7000.0),
        EventKind::SpeedAtOrBelow(300.0),
      ]),
      waypoint_orbit.clone().with_behavior(vec![
        EventKind::AltitudeAtOrBelow(4000.0),
        EventKind::SpeedAtOrBelow(230.0),
      ]),
      waypoint_vista
        .clone()
        .with_behavior(vec![EventKind::Land(Intern::from_ref("13"))]),
    ],
  );

  waypoint_sets.approach.insert(
    waypoint_swift.name,
    vec![
      waypoint_swift.clone().with_behavior(vec![
        EventKind::AltitudeAtOrBelow(7000.0),
        EventKind::SpeedAtOrBelow(300.0),
      ]),
      waypoint_orbit.clone().with_behavior(vec![
        EventKind::AltitudeAtOrBelow(4000.0),
        EventKind::SpeedAtOrBelow(230.0),
      ]),
      waypoint_vista
        .clone()
        .with_behavior(vec![EventKind::Land(Intern::from_ref("13"))]),
    ],
  );

  waypoint_sets.approach.insert(
    waypoint_royal.name,
    vec![
      waypoint_royal.clone().with_behavior(vec![
        EventKind::AltitudeAtOrBelow(7000.0),
        EventKind::SpeedAtOrBelow(300.0),
      ]),
      waypoint_blaze.clone(),
      waypoint_orbit.clone().with_behavior(vec![
        EventKind::AltitudeAtOrBelow(4000.0),
        EventKind::SpeedAtOrBelow(230.0),
      ]),
      waypoint_vista
        .clone()
        .with_behavior(vec![EventKind::Land(Intern::from_ref("13"))]),
    ],
  );

  // Runway 22 Approaches

  waypoint_sets.approach.insert(
    waypoint_quick.name,
    vec![
      waypoint_quick.clone().with_behavior(vec![
        EventKind::AltitudeAtOrBelow(7000.0),
        EventKind::SpeedAtOrBelow(300.0),
      ]),
      waypoint_ready.clone().with_behavior(vec![
        EventKind::AltitudeAtOrBelow(4000.0),
        EventKind::SpeedAtOrBelow(230.0),
      ]),
      waypoint_sonic
        .clone()
        .with_behavior(vec![EventKind::Land(Intern::from_ref("22"))]),
    ],
  );

  waypoint_sets.approach.insert(
    waypoint_short.name,
    vec![
      waypoint_short.clone().with_behavior(vec![
        EventKind::AltitudeAtOrBelow(7000.0),
        EventKind::SpeedAtOrBelow(300.0),
      ]),
      waypoint_ready.clone().with_behavior(vec![
        EventKind::AltitudeAtOrBelow(4000.0),
        EventKind::SpeedAtOrBelow(230.0),
      ]),
      waypoint_sonic
        .clone()
        .with_behavior(vec![EventKind::Land(Intern::from_ref("22"))]),
    ],
  );

  waypoint_sets.approach.insert(
    waypoint_arrow.name,
    vec![
      waypoint_arrow.clone().with_behavior(vec![
        EventKind::AltitudeAtOrBelow(7000.0),
        EventKind::SpeedAtOrBelow(300.0),
      ]),
      waypoint_ready.clone().with_behavior(vec![
        EventKind::AltitudeAtOrBelow(4000.0),
        EventKind::SpeedAtOrBelow(230.0),
      ]),
      waypoint_sonic
        .clone()
        .with_behavior(vec![EventKind::Land(Intern::from_ref("22"))]),
    ],
  );

  waypoint_sets.approach.insert(
    waypoint_ocean.name,
    vec![
      waypoint_ocean.clone().with_behavior(vec![
        EventKind::AltitudeAtOrBelow(7000.0),
        EventKind::SpeedAtOrBelow(300.0),
      ]),
      waypoint_arrow.clone(),
      waypoint_ready.clone().with_behavior(vec![
        EventKind::AltitudeAtOrBelow(4000.0),
        EventKind::SpeedAtOrBelow(230.0),
      ]),
      waypoint_sonic
        .clone()
        .with_behavior(vec![EventKind::Land(Intern::from_ref("22"))]),
    ],
  );

  //

  waypoint_sets.departure.insert(
    waypoint_royal.name,
    vec![
      waypoint_quack.clone().with_behavior(vec![
        EventKind::AltitudeAtOrAbove(7000.0),
        EventKind::SpeedAtOrAbove(250.0),
      ]),
      waypoint_state.clone(),
      waypoint_royal.clone().with_behavior(vec![
        EventKind::AltitudeAtOrAbove(13000.0),
        EventKind::SpeedAtOrAbove(350.0),
      ]),
    ],
  );

  waypoint_sets.departure.insert(
    waypoint_state.name,
    vec![
      waypoint_quack.clone().with_behavior(vec![
        EventKind::AltitudeAtOrAbove(7000.0),
        EventKind::SpeedAtOrAbove(250.0),
      ]),
      waypoint_state.clone().with_behavior(vec![
        EventKind::AltitudeAtOrAbove(13000.0),
        EventKind::SpeedAtOrAbove(350.0),
      ]),
    ],
  );

  waypoint_sets.departure.insert(
    waypoint_unite.name,
    vec![
      waypoint_quack.clone().with_behavior(vec![
        EventKind::AltitudeAtOrAbove(7000.0),
        EventKind::SpeedAtOrAbove(250.0),
      ]),
      waypoint_state.clone(),
      waypoint_unite.clone().with_behavior(vec![
        EventKind::AltitudeAtOrAbove(13000.0),
        EventKind::SpeedAtOrAbove(350.0),
      ]),
    ],
  );

  waypoint_sets.departure.insert(
    waypoint_blaze.name,
    vec![
      waypoint_quack.clone().with_behavior(vec![
        EventKind::AltitudeAtOrAbove(7000.0),
        EventKind::SpeedAtOrAbove(250.0),
      ]),
      waypoint_state.clone(),
      waypoint_royal.clone().with_behavior(vec![
        EventKind::AltitudeAtOrAbove(13000.0),
        EventKind::SpeedAtOrAbove(350.0),
      ]),
      waypoint_blaze.clone(),
    ],
  );

  //

  waypoint_sets.departure.insert(
    waypoint_goose.name,
    vec![
      waypoint_paper.clone().with_behavior(vec![
        EventKind::AltitudeAtOrAbove(7000.0),
        EventKind::SpeedAtOrAbove(250.0),
      ]),
      waypoint_ghost.clone(),
      waypoint_goose.clone().with_behavior(vec![
        EventKind::AltitudeAtOrAbove(13000.0),
        EventKind::SpeedAtOrAbove(350.0),
      ]),
    ],
  );

  waypoint_sets.departure.insert(
    waypoint_ghost.name,
    vec![
      waypoint_paper.clone().with_behavior(vec![
        EventKind::AltitudeAtOrAbove(7000.0),
        EventKind::SpeedAtOrAbove(250.0),
      ]),
      waypoint_ghost.clone().with_behavior(vec![
        EventKind::AltitudeAtOrAbove(13000.0),
        EventKind::SpeedAtOrAbove(350.0),
      ]),
    ],
  );

  waypoint_sets.departure.insert(
    waypoint_ocean.name,
    vec![
      waypoint_paper.clone().with_behavior(vec![
        EventKind::AltitudeAtOrAbove(7000.0),
        EventKind::SpeedAtOrAbove(250.0),
      ]),
      waypoint_ghost.clone(),
      waypoint_ocean.clone().with_behavior(vec![
        EventKind::AltitudeAtOrAbove(13000.0),
        EventKind::SpeedAtOrAbove(350.0),
      ]),
    ],
  );

  waypoint_sets.departure.insert(
    waypoint_arrow.name,
    vec![
      waypoint_paper.clone().with_behavior(vec![
        EventKind::AltitudeAtOrAbove(7000.0),
        EventKind::SpeedAtOrAbove(250.0),
      ]),
      waypoint_ghost.clone(),
      waypoint_ocean.clone().with_behavior(vec![
        EventKind::AltitudeAtOrAbove(13000.0),
        EventKind::SpeedAtOrAbove(350.0),
      ]),
      waypoint_arrow.clone(),
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
