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
use std::borrow::Cow;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::io::ErrorKind;
use std::path::Path;

#[derive(Debug)]
pub struct SomeDbError;

impl Display for SomeDbError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "Some error to be clearly identified at some point!")
  }
}

impl std::error::Error for SomeDbError {}

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

  pub(crate) fn raw(key: String) -> Self {
    ProjectKey { key }
  }

  pub(crate) fn as_bytes(&self) -> &[u8] {
    self.key.as_bytes()
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

  pub fn add_project(&mut self, name: String) -> Result<Cow<Project>, SomeDbError> {
    let entry = self.projects.entry(ProjectKey::new(&name));
    match entry {
      Entry::Vacant(_) => Self::apply_action(&mut self.storage, entry, Action::ProjectAdd { name }),
      Entry::Occupied(_) => Err(SomeDbError),
    }
  }

  pub fn remove_project(&mut self, name: String) -> Result<Cow<Project>, SomeDbError> {
    let key = ProjectKey::new(&name);
    let entry = self.projects.entry(key.clone());
    match entry {
      Entry::Occupied(_) => {
        Self::apply_action(&mut self.storage, entry, Action::ProjectDel { key })
      }
      Entry::Vacant(_) => Err(SomeDbError),
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

  pub fn start_on(&mut self, name: String) -> Result<Cow<Project>, SomeDbError> {
    match self.silent_stop() {
      Ok(_) => {
        let key = ProjectKey::new(&name);
        let entry = self.projects.entry(key.clone());
        let now = Local::now();
        match entry {
          Entry::Occupied(_) => Self::apply_action(
            &mut self.storage,
            entry,
            Action::RecordStart {
              key,
              ts: now.timestamp(),
              tz: now.offset().utc_minus_local(),
            },
          ),
          Entry::Vacant(_) => Err(SomeDbError),
        }
      }
      Err(_) => Err(SomeDbError),
    }
  }

  pub fn stop(&mut self) -> Result<Cow<Project>, SomeDbError> {
    if self.current_project().is_none() {
      return Err(SomeDbError);
    }
    self.silent_stop().map(|o| o.unwrap())
  }

  fn silent_stop(&mut self) -> Result<Option<Cow<Project>>, SomeDbError> {
    if self.last_project.is_none() {
      return Ok(None);
    }

    let key = self.last_project.take();
    let entry = self.projects.entry(key.unwrap());
    let now = Local::now();
    match entry {
      Entry::Occupied(e) => {
        if e.get().in_flight() {
          Self::apply_action(
            &mut self.storage,
            Entry::Occupied(e),
            Action::RecordStop {
              ts: now.timestamp(),
              tz: now.offset().utc_minus_local(),
            },
          )
        } else {
          Ok(Cow::Borrowed(e.into_mut()))
        }
      }
      Entry::Vacant(_) => Err(SomeDbError),
    }
    .map(Some)
  }

  fn apply_action<'a>(
    storage: &'a mut FsStorage,
    entry: Entry<'a, ProjectKey, Project>,
    action: Action,
  ) -> Result<Cow<'a, Project>, SomeDbError> {
    match storage.record_action(action) {
      Ok(action) => action.apply(entry),
      Err(_) => Err(SomeDbError),
    }
  }
}

fn load_all(mut database: Database) -> Result<Database, ()> {
  for (key, action) in database.storage.replay_actions() {
    let key = key.unwrap_or_else(|| database.last_project.take().expect("We need a key here!"));
    let project = action
      .apply(database.projects.entry(key))
      .expect("Something is off with our WAL!");
    if project.in_flight() {
      database.last_project = Some(ProjectKey::new(project.name()));
    }
  }
  Ok(database)
}
