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
use std::collections::BTreeMap;

pub enum Action<'a> {
  ProjectAdd { name: &'a str },
  // ProjectDel { name: &'a str },
}

impl<'a> Action<'a> {
  pub fn apply(&self, data: &mut BTreeMap<String, Project>) -> Result<(), ()> {
    match *self {
      Action::ProjectAdd { name } => {
        match data.insert(name.to_lowercase(), Project::new(name.to_string())) {
          None => Ok(()),
          Some(_) => Err(()),
        }
      }
    }
  }
}
