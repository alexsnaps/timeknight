/*
 * Copyright 2022 Alex Snaps
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use crate::core::Project;
use crate::database::database::ProjectKey;
use std::collections::btree_map::Entry;

pub enum Action {
  ProjectAdd { name: String },
  ProjectDel { name: String },
}

impl Action {
  pub fn apply(&self, entry: Entry<ProjectKey, Project>) -> Result<(), ()> {
    match self {
      Action::ProjectAdd { name } => match entry {
        Entry::Vacant(e) => {
          e.insert(Project::new(name.to_string()));
          Ok(())
        }
        Entry::Occupied(_) => Err(()),
      },
      Action::ProjectDel { name: _ } => match entry {
        Entry::Occupied(e) => {
          e.remove();
          Ok(())
        }
        Entry::Vacant(_) => Err(()),
      },
    }
  }

  pub fn id(&self) -> u8 {
    match self {
      Action::ProjectAdd { .. } => 127 as u8,
      Action::ProjectDel { .. } => 126 as u8,
    }
  }

  pub fn data(&self) -> &[u8] {
    match self {
      Action::ProjectAdd { name } => name.as_bytes(),
      Action::ProjectDel { name } => name.as_bytes(),
    }
  }

  pub fn from_bytes(data: &[u8]) -> Result<(ProjectKey, Action), ()> {
    match data[0] {
      127 => {
        let name = String::from_utf8_lossy(&data[1..]).to_string();
        Ok((ProjectKey::new(&name), Action::ProjectAdd { name }))
      }
      126 => {
        let name = String::from_utf8_lossy(&data[1..]).to_string();
        Ok((ProjectKey::new(&name), Action::ProjectDel { name }))
      }
      _ => Err(()),
    }
  }
}
