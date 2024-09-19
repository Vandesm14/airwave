use engine::{
  engine::Engine,
  entities::{
    aircraft::{Aircraft, Event},
    airspace::Airspace,
    world::World,
  },
  NAUTICALMILES_TO_FEET,
};

const MANUAL_TOWER_AIRSPACE_RADIUS: f32 = NAUTICALMILES_TO_FEET * 30.0;

fn main() {
  let mut engine = Engine::default();
  let mut world = World::default();
  let mut aircrafts: Vec<Aircraft> = Vec::new();

  // Create a controlled KSFO airspace
  let airspace_ksfo = Airspace {
    id: "KSFO".into(),
    size: MANUAL_TOWER_AIRSPACE_RADIUS,
    ..Default::default()
  };

  world.airspaces.push(airspace_ksfo);

  let aircraft = Aircraft {
    speed: 200.0,
    altitude: 2000.0,
    heading: 0.0,
    ..Default::default()
  };

  aircrafts.push(aircraft);

  engine.events.push(Event::TargetSpeed(250.0));
  engine.events.push(Event::TargetAltitude(4000.0));
  engine.events.push(Event::TargetHeading(45.0));

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
