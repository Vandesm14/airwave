use serde::{Deserialize, Serialize};

use super::aircraft::AircraftKind;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PurchasableAircraft {
  id: usize,
  cost: usize,
  kind: AircraftKind,
}
