use engine::engine::{Aircraft, Engine, Event};

fn main() {
  let mut engine = Engine::default();

  let mut aircraft = Aircraft::default();
  aircraft.speed = 200.0;
  // aircraft.target.speed = 250.0;

  aircraft.altitude = 2000.0;
  // aircraft.target.altitude = 4000.0;

  aircraft.heading = 0.0;
  // aircraft.target.heading = 45.0;

  engine.aircraft.push(aircraft);
  engine.events.push(Event::TargetSpeed(250.0));
  engine.events.push(Event::TargetAltitude(4000.0));
  engine.events.push(Event::TargetHeading(45.0));

  println!("{engine:?}");

  engine.tick();
  println!("{engine:?}");

  engine.tick();
  println!("{engine:?}");
}
