use engine::engine::{Aircraft, Engine};

fn main() {
  let mut engine = Engine::default();

  let mut aircraft = Aircraft::default();
  aircraft.speed = 200.0;
  aircraft.target.speed = 200.0;

  aircraft.altitude = 2000.0;
  aircraft.target.altitude = 4000.0;

  engine.aircraft.push(aircraft);
}
