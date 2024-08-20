use std::collections::HashMap;

use crate::{Entity, EntityData, RefOrValue, RefType};

pub type EntityMap = HashMap<String, EntityData>;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EntityConstructor {
  entities: EntityMap,
}

impl<T> RefOrValue<T> {
  pub fn value(&self, map: &EntityMap) -> Option<&T> {
    match self {
      RefOrValue::Action(a) => todo!(),
      RefOrValue::Value(v) => Some(v),
      RefOrValue::Ref(r) => match r {
        RefType::A(a) => {
          map.get(a);
          todo!()
        }
        RefType::B(_) => todo!(),
        RefType::R(_) => todo!(),
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
          a: RefOrValue::Value(*a),
          b: RefOrValue::Value(*b),
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
