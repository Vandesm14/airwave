use std::{borrow::Borrow, collections::HashMap, ops::Deref};

use glam::Vec2;
use shared::{
  inverse_degrees, move_point,
  structs::{Runway, Taxiway, TaxiwayKind},
};

use crate::{Action, Entity, EntityData, RefOrValue, RefType};

pub type EntityMap = HashMap<String, EntityData>;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EntityConstructor {
  pub entities: EntityMap,

  pub runways: Vec<Runway>,
  pub taxiways: Vec<Taxiway>,
}

impl<T> RefOrValue<T>
where
  T: Clone,
{
  pub fn only_value(&self) -> Option<T> {
    if let RefOrValue::Value(v) = self {
      Some(v.clone())
    } else {
      None
    }
  }
}

impl RefOrValue<f32> {
  pub fn value(&self, map: &EntityMap, traceback_id: &str) -> Option<f32> {
    match self {
      RefOrValue::Action(action) => match action.deref() {
        Action::AddDegrees(a, b) => {
          todo!()
        }

        // Move and AddVec2 are only for Vec2 values.
        Action::Move(_, _, _) => todo!(),
        Action::AddVec2(_, _) => todo!(),
      },
      RefOrValue::Value(v) => Some(*v),
      RefOrValue::Ref(r) => match r {
        RefType::R(_) => todo!(),

        // A and B (Vec2) aren't f32 values.
        RefType::A(_) => panic!("Cannot get an f32 value from a Ref type of A (Vec2). Entity: {traceback_id}"),
        RefType::B(_) => panic!("Cannot get an f32 value from a Ref type of B (Vec2). Entity: {traceback_id}"),
      },
    }
  }
}

impl RefOrValue<Vec2> {
  pub fn value(&self, map: &EntityMap, traceback_id: &str) -> Option<Vec2> {
    match self {
      RefOrValue::Action(action) => match action.deref() {
        Action::Move(_, _, _) => todo!(),
        Action::AddVec2(_, _) => todo!(),

        // AddDegrees is only for angles (f32).
        Action::AddDegrees(_, _) => {
          panic!("Cannot AddDegrees to a Vec2 value. Entity: {traceback_id}")
        }
      },
      RefOrValue::Value(v) => Some(*v),
      RefOrValue::Ref(r) => match r {
        RefType::A(a) => map.get(a).and_then(|entity_data| match entity_data {
          EntityData::Taxiway { a, .. } => a.only_value(),
          EntityData::Runway {
            pos,
            heading,
            length,
          } => pos
            .only_value()
            .zip(heading.only_value())
            .zip(length.only_value())
            .map(|((pos, heading), length)| {
              move_point(pos, inverse_degrees(heading), length * 0.5)
            }),
        }),
        RefType::B(b) => map.get(b).and_then(|entity_data| match entity_data {
          EntityData::Taxiway { b, .. } => b.only_value(),
          EntityData::Runway {
            pos,
            heading,
            length,
          } => pos
            .only_value()
            .zip(heading.only_value())
            .zip(length.only_value())
            .map(|((pos, heading), length)| {
              move_point(pos, inverse_degrees(heading), length * 0.5)
            }),
        }),

        // R (f32) isn't a Vec2 value.
        RefType::R(_) => panic!("Cannot get a Vec2 value from a Ref type of R (rotation). Entity: {traceback_id}"),
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
    }
  }

  pub fn add_entity(&mut self, entity: Entity) {
    let data: EntityData = match entity.data {
      EntityData::Taxiway { a, b } => {
        let a = a.value(&self.entities, &entity.id).unwrap();
        let b = b.value(&self.entities, &entity.id).unwrap();

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
      EntityData::Runway {
        pos,
        heading,
        length,
      } => {
        todo!()
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
