use core::panic;
use std::{collections::HashMap, ops::Deref};

use glam::Vec2;
use serde::Serialize;
use shared::{
  angle_between_points, degrees_to_heading, move_point,
  structs::{Runway, Taxiway, TaxiwayKind, Terminal},
};
use thiserror::Error;

use crate::{
  Action, Degrees, Entity, EntityData, Feet, RefOrValue, RefType, Var,
};

pub type EntityMap = HashMap<String, EntityData>;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EntityConstructor {
  pub entities: EntityMap,

  pub runways: Vec<Runway>,
  pub taxiways: Vec<Taxiway>,
  pub terminals: Vec<Terminal>,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum ValueError<T, R>
where
  T: Clone + Serialize,
  R: Clone + Serialize,
{
  #[error("Cannot call action: {0:?} for value type: {1:?}")]
  InvalidActionForProperty(Action<T>, RefOrValue<R>),
  #[error("Invalid ref call on entity: {0:?} for value type: {1:?}")]
  InvalidRefForEntity(RefType<T>, RefOrValue<R>),
}

impl<T> RefOrValue<T>
where
  T: Clone + Serialize,
{
  pub fn only_value(&self) -> Option<T> {
    if let RefOrValue::Value(v) = self {
      Some(v.clone())
    } else {
      None
    }
  }
}

impl RefOrValue<Feet> {
  pub fn value(&self, map: &EntityMap) -> Option<Feet> {
    match self {
      RefOrValue::Action(action) => match action.deref() {
        Action::Move(_, _, _) => panic!(
          "{}",
          ValueError::InvalidActionForProperty(*action.clone(), self.clone())
        ),
        Action::Add(a, b) => {
          let a = a.value(map).unwrap();
          let b = b.value(map).unwrap();

          Some(Feet(a.0 + b.0))
        }
      },
      RefOrValue::Value(v) => Some(*v),
      RefOrValue::Ref(r) => match r {
        RefType::A(v) => map.get(v).and_then(|entity_data| match entity_data {
          EntityData::Taxiway { .. } => panic!(
            "{}",
            ValueError::InvalidRefForEntity(r.clone(), self.clone())
          ),
          EntityData::Runway { .. } => panic!(
            "{}",
            ValueError::InvalidRefForEntity(r.clone(), self.clone())
          ),
          EntityData::Var(var) => match var {
            Var::Position(_) => panic!(
              "{}",
              ValueError::InvalidRefForEntity(r.clone(), self.clone())
            ),
            Var::Degrees(_) => panic!(
              "{}",
              ValueError::InvalidRefForEntity(r.clone(), self.clone())
            ),
            Var::Feet(v) => v.only_value(),
          },
        }),

        // Invalid RefType for a Feet value.
        RefType::R(_) => panic!(
          "{}",
          ValueError::InvalidRefForEntity(r.clone(), self.clone())
        ),
        RefType::B(_) => panic!(
          "{}",
          ValueError::InvalidRefForEntity(r.clone(), self.clone())
        ),
      },
    }
  }
}

impl RefOrValue<Degrees> {
  pub fn value(&self, map: &EntityMap) -> Option<Degrees> {
    match self {
      RefOrValue::Action(action) => match action.deref() {
        Action::Add(a, b) => {
          let a = a.value(map).unwrap();
          let b = b.value(map).unwrap();

          Some(Degrees(a.0 + b.0))
        }
        Action::Move(_, _, _) => panic!(
          "{}",
          ValueError::InvalidActionForProperty(*action.clone(), self.clone())
        ),
      },
      RefOrValue::Value(v) => Some(*v),
      RefOrValue::Ref(r) => match r {
        RefType::R(_) => todo!("get angle of runway or taxiway"),

        // Invalid RefType for a Degrees value.
        RefType::A(v) => map.get(v).and_then(|entity_data| match entity_data {
          EntityData::Taxiway { .. } => panic!(
            "{}",
            ValueError::InvalidRefForEntity(r.clone(), self.clone())
          ),
          EntityData::Runway { .. } => panic!(
            "{}",
            ValueError::InvalidRefForEntity(r.clone(), self.clone())
          ),
          EntityData::Var(var) => match var {
            Var::Position(_) => panic!(
              "{}",
              ValueError::InvalidRefForEntity(r.clone(), self.clone())
            ),
            Var::Degrees(v) => v.only_value(),
            Var::Feet(_) => panic!(
              "{}",
              ValueError::InvalidRefForEntity(r.clone(), self.clone())
            ),
          },
        }),
        RefType::B(_) => panic!(
          "{}",
          ValueError::InvalidRefForEntity(r.clone(), self.clone())
        ),
      },
    }
  }
}

impl RefOrValue<Vec2> {
  pub fn value(&self, map: &EntityMap) -> Option<Vec2> {
    match self {
      RefOrValue::Action(action) => match action.deref() {
        Action::Move(pos, heading, length) => {
          let pos = pos.value(map)?;
          let heading = heading.value(map)?;
          let length = length.value(map)?;

          Some(move_point(pos, heading.0, length.0))
        }
        Action::Add(a, b) => {
          let a = a.value(map).unwrap();
          let b = b.value(map).unwrap();

          Some(a + b)
        }
      },
      RefOrValue::Value(v) => Some(*v),
      RefOrValue::Ref(r) => match r {
        RefType::A(a) => map.get(a).and_then(|entity_data| match entity_data {
          EntityData::Taxiway { a, .. } => a.only_value(),
          EntityData::Runway { a, .. } => a.only_value(),
          EntityData::Var(var) => match var {
            Var::Position(v) => v.only_value(),
            Var::Degrees(_) => panic!(
              "{}",
              ValueError::InvalidRefForEntity(r.clone(), self.clone())
            ),
            Var::Feet(_) => panic!(
              "{}",
              ValueError::InvalidRefForEntity(r.clone(), self.clone())
            ),
          },
        }),
        RefType::B(b) => map.get(b).and_then(|entity_data| match entity_data {
          EntityData::Taxiway { b, .. } => b.only_value(),
          EntityData::Runway { b, .. } => b.only_value(),
          EntityData::Var(_) => panic!(
            "{}",
            ValueError::InvalidRefForEntity(r.clone(), self.clone())
          ),
        }),

        // Invalid RefType for a Vec2 value.
        RefType::R(_) => panic!(
          "{}",
          ValueError::InvalidRefForEntity(r.clone(), self.clone())
        ),
      },
    }
  }
}

impl EntityConstructor {
  pub fn new() -> Self {
    Self {
      entities: HashMap::new(),

      runways: Vec::new(),
      taxiways: Vec::new(),
      terminals: Vec::new(),
    }
  }

  pub fn add_entity(&mut self, entity: Entity) {
    let data: EntityData = match entity.data {
      // Airport Objects
      EntityData::Taxiway { a, b } => {
        let a = a.value(&self.entities).unwrap();
        let b = b.value(&self.entities).unwrap();

        self.taxiways.push(Taxiway {
          id: entity.id.clone(),
          a,
          b,
          kind: TaxiwayKind::Normal,
        });

        EntityData::Taxiway {
          a: RefOrValue::Value(a),
          b: RefOrValue::Value(b),
        }
      }
      EntityData::Runway { a, b } => {
        let a = a.value(&self.entities).unwrap();
        let b = b.value(&self.entities).unwrap();

        let pos = a.midpoint(b);
        let heading = degrees_to_heading(angle_between_points(a, b));
        let length = a.distance(b);

        self.runways.push(Runway {
          id: entity.id.clone(),
          pos,
          heading,
          length,
        });

        EntityData::Runway {
          a: RefOrValue::Value(a),
          b: RefOrValue::Value(b),
        }
      }

      // Variables
      EntityData::Var(Var::Degrees(degrees)) => {
        let degrees = degrees.value(&self.entities).unwrap();

        EntityData::Var(Var::Degrees(RefOrValue::Value(degrees)))
      }
      EntityData::Var(Var::Feet(feet)) => {
        let feet = feet.value(&self.entities).unwrap();

        EntityData::Var(Var::Feet(RefOrValue::Value(feet)))
      }
      EntityData::Var(Var::Position(position)) => {
        let position = position.value(&self.entities).unwrap();

        EntityData::Var(Var::Position(RefOrValue::Value(position)))
      }
    };

    self.entities.insert(entity.id.clone(), data);
  }

  pub fn entities(&self) -> &EntityMap {
    &self.entities
  }

  pub fn entities_mut(&mut self) -> &mut EntityMap {
    &mut self.entities
  }
}
