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
use crate::db::storage::Action;
use crate::db::storage::FsStorage;
use chrono::Local;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::Path;

pub struct Database {
  storage: FsStorage,
  projects: BTreeMap<ProjectKey, Project>,
  last_project: Option<ProjectKey>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
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

  pub fn list_projects(&self) -> Vec<&Project> {
    let mut projects = self.projects.values().collect::<Vec<&Project>>();
    projects.sort_by_key(|a| a.name().to_lowercase());
    projects
  }

  pub fn current_project(&self) -> Option<&Project> {
    match &self.last_project {
      Some(key) => self.projects.get(key),
      None => None,
    }
  }

  pub fn start_on(&mut self, name: &str) -> Result<(), ()> {
    match self.silent_stop() {
      Ok(_) => {
        let entry = self.projects.entry(ProjectKey::new(name));
        let now = Local::now();
        match entry {
          Entry::Occupied(_) => Self::apply_action(
            &mut self.storage,
            entry,
            Action::RecordStart {
              name: name.to_string(),
              ts: now.timestamp(),
              tz: now.offset().utc_minus_local(),
            },
          ),
          Entry::Vacant(_) => Err(()),
        }
      }
      Err(_) => Err(()),
    }
  }

  pub fn stop(&mut self) -> Result<(), ()> {
    if self.current_project().is_none() {
      return Err(());
    }
    self.silent_stop()
  }

  fn silent_stop(&mut self) -> Result<(), ()> {
    let key = self.last_project.as_ref().unwrap();
    let entry = self.projects.entry(key.clone());
    let now = Local::now();
    match entry {
      Entry::Occupied(_) => Self::apply_action(
        &mut self.storage,
        entry,
        Action::RecordStop {
          ts: now.timestamp(),
          tz: now.offset().utc_minus_local(),
        },
      ),
      Entry::Vacant(_) => Err(()),
    }
  }

  fn apply_action(
    storage: &mut FsStorage,
    entry: Entry<ProjectKey, Project>,
    action: Action,
  ) -> Result<(), ()> {
    match storage.record_action(action) {
      Ok(action) => action.apply(entry),
      Err(_) => Err(()),
    }
  }
}

fn load_all(mut database: Database) -> Result<Database, ()> {
  for (key, action) in database.storage.replay_actions() {
    let key = match key {
      None => database.last_project.expect("We need a key here!"),
      Some(project) => project,
    };
    action
      .apply(database.projects.entry(key.clone()))
      .expect("Something is off with our WAL!");
    database.last_project = Some(key);
  }
  Ok(database)
}
