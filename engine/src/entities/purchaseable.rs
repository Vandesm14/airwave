use serde::{Deserialize, Serialize};

use super::aircraft::AircraftKind;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PurchasableAircraft {
  pub id: usize,
  pub cost: usize,
  pub kind: AircraftKind,
}
