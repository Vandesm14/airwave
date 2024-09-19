use engine::{
  engine::Engine,
  entities::{
    aircraft::{
      events::{Event, EventKind},
      Aircraft,
    },
    airport::{Airport, Runway},
    airspace::Airspace,
    world::World,
  },
  NAUTICALMILES_TO_FEET,
};
use internment::Intern;

const MANUAL_TOWER_AIRSPACE_RADIUS: f32 = NAUTICALMILES_TO_FEET * 30.0;

fn main() {
  let mut engine = Engine::default();
  let mut world = World::default();
  let mut aircrafts: Vec<Aircraft> = Vec::new();

  // Create a controlled KSFO airspace
  let mut airspace_ksfo = Airspace {
    id: Intern::from_ref("KSFO"),
    size: MANUAL_TOWER_AIRSPACE_RADIUS,
    ..Default::default()
  };

  let mut airport_ksfo = Airport {
    id: Intern::from_ref("KSFO"),
    ..Default::default()
  };

  airport_ksfo.add_runway(Runway {
    id: Intern::from_ref("27"),
    heading: 270.0,
    length: 7000.0,
    ..Default::default()
  });

  airspace_ksfo.airports.push(airport_ksfo);
  world.airspaces.push(airspace_ksfo);

  let aircraft = Aircraft {
    speed: 200.0,
    altitude: 2000.0,
    heading: 0.0,
    ..Default::default()
  };
  let aircraft_id = aircraft.id;

  aircrafts.push(aircraft);

  engine
    .events
    .push(Event::new(aircraft_id, EventKind::TargetSpeed(250.0)));
  engine
    .events
    .push(Event::new(aircraft_id, EventKind::TargetAltitude(4000.0)));
  engine
    .events
    .push(Event::new(aircraft_id, EventKind::TargetHeading(45.0)));

  println!("{aircrafts:?}");
  engine.tick(&world, &mut aircrafts);
  println!("{aircrafts:?}");
  engine.tick(&world, &mut aircrafts);
  println!("{aircrafts:?}");
  engine.tick(&world, &mut aircrafts);
  println!("{aircrafts:?}");
  engine.tick(&world, &mut aircrafts);
  println!("{aircrafts:?}");
  engine.tick(&world, &mut aircrafts);
  println!("{aircrafts:?}");
  engine.tick(&world, &mut aircrafts);
  println!("{aircrafts:?}");
  engine.tick(&world, &mut aircrafts);
  println!("{aircrafts:?}");
  engine.tick(&world, &mut aircrafts);
  println!("{aircrafts:?}");
  engine.tick(&world, &mut aircrafts);
  println!("{aircrafts:?}");
  engine.tick(&world, &mut aircrafts);
  println!("{aircrafts:?}");
}
