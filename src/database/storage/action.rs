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

use crate::core::{Project, Record};
use crate::database::database::ProjectKey;
use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use std::collections::btree_map::Entry;

pub enum Action {
  ProjectAdd { name: String },
  ProjectDel { name: String },
  RecordStart { name: String, ts: i64, tz: i32 },
  RecordStop { ts: i64, tz: i32 },
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
      Action::RecordStart { name: _, ts, tz } => match entry {
        Entry::Occupied(mut e) => {
          let utc = Utc.timestamp_millis(*ts);
          let offset = FixedOffset::from_offset(&FixedOffset::west(*tz));
          let start: DateTime<FixedOffset> = utc.with_timezone(&offset);
          e.get_mut()
            .add_record(Record::started_on(start))
            .expect("Replay start failed");
          Ok(())
        }
        Entry::Vacant(_) => Err(()),
      },
      Action::RecordStop { ts, tz } => match entry {
        Entry::Occupied(mut e) => {
          let utc = Utc.timestamp_millis(*ts);
          let offset = FixedOffset::from_offset(&FixedOffset::west(*tz));
          let end: DateTime<FixedOffset> = utc.with_timezone(&offset);
          e.get_mut().end_at(end).expect("Replay end failed");
          Ok(())
        }
        Entry::Vacant(_) => Err(()),
      },
    }
  }

  pub fn from_bytes(data: &[u8]) -> Result<(Option<ProjectKey>, Action), ()> {
    match data[0] {
      127 => {
        let name = String::from_utf8_lossy(&data[1..]).to_string();
        Ok((Some(ProjectKey::new(&name)), Action::ProjectAdd { name }))
      }
      126 => {
        let name = String::from_utf8_lossy(&data[1..]).to_string();
        Ok((Some(ProjectKey::new(&name)), Action::ProjectDel { name }))
      }
      125 => {
        let name = String::from_utf8_lossy(&data[13..]).to_string();
        let ts = i64::from_le_bytes(data[1..9].try_into().expect("Wrong math!"));
        let tz = i32::from_le_bytes(data[9..13].try_into().expect("Wrong math!"));
        Ok((
          Some(ProjectKey::new(&name)),
          Action::RecordStart { name, ts, tz },
        ))
      }
      124 => {
        let ts = i64::from_le_bytes(data[1..9].try_into().expect("Wrong math!"));
        let tz = i32::from_le_bytes(data[9..13].try_into().expect("Wrong math!"));
        Ok((None, Action::RecordStop { ts, tz }))
      }
      _ => Err(()),
    }
  }
}

impl Into<Vec<u8>> for &Action {
  fn into(self) -> Vec<u8> {
    match self {
      Action::ProjectAdd { name } => {
        let raw = name.as_bytes();
        let mut buffer = Vec::with_capacity(raw.len() + 2);
        buffer.push(127);
        buffer.extend_from_slice(raw);
        buffer.push(b'\n');
        buffer
      }
      Action::ProjectDel { name } => {
        let raw = name.as_bytes();
        let mut buffer = Vec::with_capacity(raw.len() + 2);
        buffer.push(126);
        buffer.extend_from_slice(raw);
        buffer.push(b'\n');
        buffer
      }
      Action::RecordStart { name, ts, tz } => {
        let raw = name.as_bytes();
        let mut buffer = Vec::with_capacity(raw.len() + 14);
        buffer.push(125);
        buffer.extend_from_slice(&ts.to_le_bytes());
        buffer.extend_from_slice(&tz.to_le_bytes());
        buffer.extend_from_slice(raw);
        buffer.push(b'\n');
        buffer
      }
      Action::RecordStop { ts, tz } => {
        let mut buffer = Vec::with_capacity(14);
        buffer.push(124);
        buffer.extend_from_slice(&ts.to_le_bytes());
        buffer.extend_from_slice(&tz.to_le_bytes());
        buffer.push(b'\n');
        buffer
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::database::database::ProjectKey;
  use crate::database::storage::Action;

  #[test]
  fn record_start_serializes_alright() {
    let record_start = Action::RecordStart {
      name: "ourName".to_string(),
      ts: 321456789,
      tz: -123456,
    };
    let buffer: Vec<u8> = (&record_start).into();
    assert_eq!(21, buffer.len());
    assert_eq!(21, buffer.capacity());
    assert_eq!(
      [125, 149, 10, 41, 19, 0, 0, 0, 0, 192, 29, 254, 255, 111, 117, 114, 78, 97, 109, 101, 10],
      buffer.as_slice()
    );
    let (key, action) = Action::from_bytes(&buffer[..buffer.len() - 1]).unwrap();
    assert_eq!(key, Some(ProjectKey::new("OURNAME")));
    match action {
      Action::RecordStart { name, ts, tz } => {
        assert_eq!(name, "ourName");
        assert_eq!(ts, 321456789);
        assert_eq!(tz, -123456);
      }
      _ => assert!(false),
    }
  }
}
