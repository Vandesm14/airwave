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

  let runway_27r = Runway {
    id: "27R".into(),
    pos: airport.center + Vec2::Y * RUNWAY_SPACING / 2.0,
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
    pos: airport.center + Vec2::Y * -(RUNWAY_SPACING / 2.0),
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

  // MARK: Right Arrival Waypoints.

  let waypoint_tack = Node {
    name: "TACK".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: move_point(
      runway_27r.start(),
      inverse_degrees(runway_27r.heading),
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

  // MARK: Left Arrival Waypoints.

  let waypoint_lord = Node {
    name: "LORD".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: move_point(
      runway_27l.start(),
      inverse_degrees(runway_27l.heading),
      NAUTICALMILES_TO_FEET * 14.0,
    ),
  };

  let waypoint_jest = Node {
    name: "JEST".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: move_point(waypoint_lord.value, RIGHT, NAUTICALMILES_TO_FEET * 4.0),
  };

  let waypoint_ball = Node {
    name: "BALL".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: move_point(
      waypoint_jest.value,
      RIGHT + 45.0,
      NAUTICALMILES_TO_FEET * 8.0,
    ),
  };

  waypoint_sets.approach.insert(
    "BALL".to_owned(),
    vec![
      waypoint_ball.name.clone(),
      waypoint_jest.name.clone(),
      waypoint_lord.name.clone(),
    ],
  );

  waypoints.push(waypoint_lord);
  waypoints.push(waypoint_jest);
  waypoints.push(waypoint_ball);

  // MARK: Right Departure Waypoints.

  let waypoint_note = Node {
    name: "NOTE".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: move_point(
      runway_27r.end(),
      runway_27r.heading,
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

  // MARK: Left Departure Waypoints.

  let waypoint_king = Node {
    name: "KING".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: move_point(
      runway_27l.end(),
      runway_27l.heading,
      NAUTICALMILES_TO_FEET * 6.0,
    ),
  };

  let waypoint_town = Node {
    name: "TOWN".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: move_point(waypoint_king.value, LEFT, NAUTICALMILES_TO_FEET * 8.0),
  };

  let waypoint_gold = Node {
    name: "GOLD".to_owned(),
    kind: NodeKind::Runway,
    behavior: NodeBehavior::GoTo,
    value: move_point(
      waypoint_king.value,
      LEFT - 45.0,
      NAUTICALMILES_TO_FEET * 8.0,
    ),
  };

  waypoint_sets.departure.insert(
    "TOWN".to_owned(),
    vec![waypoint_king.name.clone(), waypoint_town.name.clone()],
  );

  waypoint_sets.departure.insert(
    "GOLD".to_owned(),
    vec![waypoint_king.name.clone(), waypoint_gold.name.clone()],
  );

  waypoints.push(waypoint_king);
  waypoints.push(waypoint_town);
  waypoints.push(waypoint_gold);

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
