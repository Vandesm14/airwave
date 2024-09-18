use engine::{
  engine::Engine,
  entities::aircraft::{Aircraft, Event},
};

fn main() {
  let mut engine = Engine::default();

  let aircraft = Aircraft {
    speed: 200.0,
    altitude: 2000.0,
    heading: 0.0,
    ..Default::default()
  };

  engine.aircraft.push(aircraft);
  engine.events.push(Event::TargetSpeed(250.0));
  engine.events.push(Event::TargetAltitude(4000.0));
  engine.events.push(Event::TargetHeading(45.0));

  println!("{engine:?}");
  engine.tick();
  println!("{engine:?}");
  engine.tick();
  println!("{engine:?}");
  engine.tick();
  println!("{engine:?}");
  engine.tick();
  println!("{engine:?}");
  engine.tick();
  println!("{engine:?}");
  engine.tick();
  println!("{engine:?}");
  engine.tick();
  println!("{engine:?}");
  engine.tick();
  println!("{engine:?}");
  engine.tick();
  println!("{engine:?}");
  engine.tick();
  println!("{engine:?}");
}
