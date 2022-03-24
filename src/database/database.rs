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

use crate::database::storage::Action;
use crate::database::storage::FsStorage;
use std::io::ErrorKind;
use std::path::Path;

pub struct Database {
  storage: FsStorage,
}

impl Database {
  pub fn open(location: &Path) -> Result<Self, ErrorKind> {
    match FsStorage::new(location) {
      Ok(storage) => Ok(Database { storage }),
      Err(e) => Err(e),
    }
  }

  pub fn create_project(&mut self, name: &str) {
    self.storage.add_action(Action::ProjectAdd { name })
  }
}
