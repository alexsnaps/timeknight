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
use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::Path;

pub struct Database {
  storage: FsStorage,
  projects: BTreeMap<String, Project>,
}

impl Database {
  pub fn open(location: &Path) -> Result<Self, ErrorKind> {
    match FsStorage::new(location) {
      Ok(storage) => {
        let database = Database {
          storage,
          projects: BTreeMap::new(),
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
    let key = name.to_lowercase();
    if !self.projects.contains_key(&key) {
      let action = Action::ProjectAdd { name };
      return match self.storage.record_action(&action) {
        Ok(_) => action.apply(&mut self.projects),
        Err(_) => Err(()),
      };
    }
    Err(())
  }

  pub fn clear(&mut self) {
    self.projects.clear();
  }
}

fn load_all(mut database: Database) -> Result<Database, ()> {
  database.clear();
  Ok(database)
}
