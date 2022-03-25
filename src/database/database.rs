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
use crate::database::storage::Action;
use crate::database::storage::FsStorage;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::Path;

pub struct Database {
  storage: FsStorage,
  projects: BTreeMap<ProjectKey, Project>,
  last_project: Option<ProjectKey>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct ProjectKey {
  key: String,
}

impl ProjectKey {
  pub(crate) fn new(key: &str) -> Self {
    ProjectKey {
      key: key.to_lowercase(),
    }
  }
}

impl Database {
  pub fn open(location: &Path) -> Result<Self, ErrorKind> {
    match FsStorage::new(location) {
      Ok(storage) => {
        let database = Database {
          storage,
          projects: BTreeMap::new(),
          last_project: None,
        };
        match load_all(database) {
          Ok(database) => Ok(database),
          Err(_) => Err(ErrorKind::InvalidData),
        }
      }
      Err(e) => Err(e),
    }
  }

  pub fn add_project(&mut self, name: &str) -> Result<(), ()> {
    let entry = self.projects.entry(ProjectKey::new(name));
    match entry {
      Entry::Vacant(_) => Self::apply_action(
        &mut self.storage,
        entry,
        Action::ProjectAdd {
          name: name.to_string(),
        },
      ),
      Entry::Occupied(_) => Err(()),
    }
  }

  pub fn remove_project(&mut self, name: &str) -> Result<(), ()> {
    let entry = self.projects.entry(ProjectKey::new(name));
    match entry {
      Entry::Occupied(_) => Self::apply_action(
        &mut self.storage,
        entry,
        Action::ProjectDel {
          name: name.to_string(),
        },
      ),
      Entry::Vacant(_) => Err(()),
    }
  }

  pub fn clear(&mut self) {
    self.projects.clear();
  }

  fn apply_action(
    storage: &mut FsStorage,
    entry: Entry<ProjectKey, Project>,
    action: Action,
  ) -> Result<(), ()> {
    return match storage.record_action(action) {
      Ok(action) => action.apply(entry),
      Err(_) => Err(()),
    };
  }
}

fn load_all(mut database: Database) -> Result<Database, ()> {
  for (key, action) in database.storage.replay_actions() {
    action
      .apply(database.projects.entry(key.clone()))
      .expect("Something is off with our WAL!");
    database.last_project = Some(key);
  }
  Ok(database)
}
