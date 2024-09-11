use engine::entities::airport::Airport;

pub mod new_v_pattern;
pub mod parallel;

pub type AirportSetupFn = fn(airport: &mut Airport);
pub mod tutorial;
