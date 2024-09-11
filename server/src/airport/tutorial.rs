use engine::{
  inverse_degrees, move_point,
  objects::{
    airport::{Airport, Gate, Runway, Taxiway, Terminal},
    world::WaypointSet,
  },
  pathfinder::{Node, NodeBehavior, NodeKind},
  Line, DOWN, LEFT, NAUTICALMILES_TO_FEET, RIGHT, UP,
};
use glam::Vec2;

pub fn setup(
  airport: &mut Airport,
  waypoints: &mut Vec<Node<Vec2>>,
  waypoint_sets: &mut WaypointSet,
) {
  /// In feet (ft).
  ///
  /// This is the minimum runway spacing approved by the ICAO and FAA.
  // TODO: Is the above information definitely correct?
  const RUNWAY_SPACING: f32 = 3400.0;
  /// In feet (ft).
  ///
  /// This is the minimum separation distance between the centre lines of two
  /// taxiways approved by the ICAO.
  // TODO: Is the above information definitely correct?
  const ENTRYWAY_TAXIWAY_DISTANCE: f32 = 300.0;

  // MARK: Right.

  let runway_27 = Runway {
    id: "27".into(),
    pos: airport.center + Vec2::Y * RUNWAY_SPACING / 2.0,
    heading: 270.0,
    length: 7000.0,
  };

  let taxiway_b = Taxiway {
    id: "B".into(),
    a: move_point(runway_27.start(), DOWN, ENTRYWAY_TAXIWAY_DISTANCE),
    b: move_point(runway_27.end(), DOWN, ENTRYWAY_TAXIWAY_DISTANCE),
  };

  let taxiway_a1 = Taxiway {
    id: "A1".into(),
    a: runway_27.end(),
    b: move_point(runway_27.end(), DOWN, ENTRYWAY_TAXIWAY_DISTANCE),
  };

  let taxiway_a2 = Taxiway {
    id: "A2".into(),
    a: runway_27.start().lerp(runway_27.end(), 0.75),
    b: move_point(
      runway_27.start().lerp(runway_27.end(), 0.75),
      DOWN,
      ENTRYWAY_TAXIWAY_DISTANCE,
    ),
  };

  let taxiway_a3 = Taxiway {
    id: "A3".into(),
    a: runway_27.start().lerp(runway_27.end(), 0.5),
    b: move_point(
      runway_27.start().lerp(runway_27.end(), 0.5),
      DOWN,
      ENTRYWAY_TAXIWAY_DISTANCE,
    ),
  };

  let taxiway_a4 = Taxiway {
    id: "A4".into(),
    a: runway_27.start().lerp(runway_27.end(), 0.25),
    b: move_point(
      runway_27.start().lerp(runway_27.end(), 0.25),
      DOWN,
      ENTRYWAY_TAXIWAY_DISTANCE,
    ),
  };

  let taxiway_a5 = Taxiway {
    id: "A5".into(),
    a: runway_27.start().lerp(runway_27.end(), 0.0),
    b: move_point(
      runway_27.start().lerp(runway_27.end(), 0.0),
      DOWN,
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
  }

  // MARK: Right Arrival Waypoints.

  let waypoint_tack = Node {
    name: "TACK".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: move_point(
      runway_27.start(),
      inverse_degrees(runway_27.heading),
      NAUTICALMILES_TO_FEET * 12.0,
    ),
  };

  let waypoint_cork = Node {
    name: "CORK".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: move_point(waypoint_tack.value, RIGHT, NAUTICALMILES_TO_FEET * 4.0),
  };

  let waypoint_foam = Node {
    name: "FOAM".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: move_point(
      waypoint_cork.value,
      RIGHT - 45.0,
      NAUTICALMILES_TO_FEET * 8.0,
    ),
  };

  waypoint_sets.approach.insert(
    "FOAM".to_owned(),
    vec![
      waypoint_foam.name.clone(),
      waypoint_cork.name.clone(),
      waypoint_tack.name.clone(),
    ],
  );

  waypoints.push(waypoint_tack);
  waypoints.push(waypoint_cork);
  waypoints.push(waypoint_foam);

  // MARK: Right Departure Waypoints.

  let waypoint_note = Node {
    name: "NOTE".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: move_point(
      runway_27.end(),
      runway_27.heading,
      NAUTICALMILES_TO_FEET * 8.0,
    ),
  };

  let waypoint_idea = Node {
    name: "IDEA".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: move_point(waypoint_note.value, LEFT, NAUTICALMILES_TO_FEET * 8.0),
  };

  let waypoint_bulb = Node {
    name: "BULB".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: move_point(
      waypoint_note.value,
      LEFT + 45.0,
      NAUTICALMILES_TO_FEET * 8.0,
    ),
  };

  waypoint_sets.departure.insert(
    "IDEA".to_owned(),
    vec![waypoint_note.name.clone(), waypoint_idea.name.clone()],
  );

  waypoint_sets.departure.insert(
    "BULB".to_owned(),
    vec![waypoint_note.name.clone(), waypoint_bulb.name.clone()],
  );

  waypoints.push(waypoint_note);
  waypoints.push(waypoint_idea);
  waypoints.push(waypoint_bulb);

  // MARK: Right.

  airport.add_runway(runway_27);

  airport.add_taxiway(taxiway_b);
  airport.add_taxiway(taxiway_a1);
  airport.add_taxiway(taxiway_a2);
  airport.add_taxiway(taxiway_a3);
  airport.add_taxiway(taxiway_a4);
  airport.add_taxiway(taxiway_a5);

  // MARK: Terminals.

  airport.terminals.push(terminal_a);
}
