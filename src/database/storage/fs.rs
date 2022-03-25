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

use crate::database::database::ProjectKey;
use crate::database::storage::Action;
use std::fs::{remove_file, File, OpenOptions};
use std::io;
use std::io::{BufRead, ErrorKind, Write};
use std::path::{Path, PathBuf};

pub struct FsStorage {
  location: PathBuf,
  wal: File,
}

const LOCK_FILE: &str = ".lock";

const WAL: &'static str = "entries.wal";

impl FsStorage {
  pub fn new(location: &Path) -> Result<Self, ErrorKind> {
    if !location.is_dir() {
      return Err(ErrorKind::InvalidInput);
    }

    let lock_location = Self::lock_file(location);

    match OpenOptions::new()
      .write(true)
      .create_new(true)
      .open(lock_location)
    {
      Ok(_) => match OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(location.join(WAL))
      {
        Ok(wal) => Ok(FsStorage {
          location: location.to_path_buf(),
          wal,
        }),
        Err(err) => Err(err.kind()),
      },
      Err(err) => Err(err.kind()),
    }
  }

  pub fn record_action<'a>(&mut self, action: Action) -> Result<Action, ()> {
    let mut buffer = vec![];
    buffer.push(action.id());
    let data = action.data();
    buffer.extend_from_slice(data);
    buffer.push(b'\n');
    match self.wal.write(&buffer) {
      Ok(_) => Ok(action),
      Err(_) => Err(()),
    }
  }

  pub fn replay_actions(&self) -> impl Iterator<Item = (ProjectKey, Action)> + '_ {
    io::BufReader::new(File::open(self.location.join(WAL)).unwrap())
      .lines()
      .map(|line| {
        let data = line.unwrap().into_bytes();
        Action::from_bytes(&data).unwrap()
      })
      .into_iter()
  }

  fn lock_file(location: &Path) -> PathBuf {
    location.join(LOCK_FILE)
  }

  fn close(&mut self) -> Result<(), io::Error> {
    remove_file(Self::lock_file(self.location.as_path()))
  }
}

impl Drop for FsStorage {
  fn drop(&mut self) {
    if self.close().is_err() {
      eprintln!(
        "Failed to remove lock file: {:?}!",
        Self::lock_file(self.location.as_path())
      )
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::database::storage::fs::FsStorage;
  use std::env;
  use std::fs::{create_dir, remove_dir};
  use std::io::ErrorKind;
  use std::io::ErrorKind::InvalidInput;
  use std::path::Path;

  #[test]
  fn test_create_errs_on_not_a_valid_dir() {
    assert_eq!(
      FsStorage::new(Path::new("/noWayThisExitsPleaseTellMeNo")).err(),
      Some(InvalidInput)
    );
  }

  #[test]
  fn test_succeeds_on_proper_dir() {
    let location = env::temp_dir().join("timeknightTest_succeeds_on_proper_dir");
    create_dir(location.as_path()).expect("failed to create temp directory");
    {
      let _working_storage = FsStorage::new(location.as_path()).expect("Failed creating Storage");
      assert_eq!(
        FsStorage::new(location.as_path()).err(),
        Some(ErrorKind::AlreadyExists)
      );
    }
    remove_dir(location.as_path()).expect("couldn't cleanup our test directory!")
  }
}
