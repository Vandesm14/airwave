use std::collections::HashMap;

use glam::Vec2;
use shared::{inverse_degrees, move_point};

use crate::{Entity, EntityData, RefOrValue, RefType};

pub type EntityMap = HashMap<String, EntityData>;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EntityConstructor {
  entities: EntityMap,
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
  pub fn value(&self, map: &EntityMap) -> Option<f32> {
    match self {
      RefOrValue::Action(a) => todo!(),
      RefOrValue::Value(v) => Some(*v),
      RefOrValue::Ref(r) => match r {
        RefType::A(_) => None,
        RefType::B(_) => None,
        RefType::R(_) => todo!(),
      },
    }
  }
}

impl RefOrValue<Vec2> {
  pub fn value(&self, map: &EntityMap) -> Option<Vec2> {
    match self {
      RefOrValue::Action(a) => todo!(),
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
        RefType::R(_) => None,
      },
    }
  }
}

impl EntityConstructor {
  pub fn new() -> Self {
    Self {
      entities: HashMap::new(),
    }
  }

  pub fn add_entity(&mut self, entity: Entity) {
    let data: EntityData = match entity.data {
      EntityData::Taxiway { a, b } => {
        let a = a.value(&self.entities).unwrap();
        let b = b.value(&self.entities).unwrap();

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
