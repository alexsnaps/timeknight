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

use chrono::{DateTime, FixedOffset};
use std::cmp::Ordering;
use std::slice::Iter;

use crate::core::record::{IllegalStateError, RResult, RecordEnded};
use crate::core::Record;

type AdditionResult = Result<RecordAdded, IllegalStateError>;

#[derive(Debug)]
pub enum RecordAdded {
  Started,
  Switched,
  Cropped,
}

pub struct Project {
  name: String,
  records: Vec<Record>,
}

impl Project {
  pub fn new(name: String) -> Self {
    Project {
      name,
      records: Vec::new(),
    }
  }

  pub fn name(&self) -> &str {
    self.name.as_str()
  }

  pub fn records(&self) -> Iter<'_, Record> {
    self.records.iter()
  }

  pub fn start(&mut self) -> AdditionResult {
    self.add_record(Record::new())
  }

  pub fn add_record(&mut self, record: Record) -> AdditionResult {
    // End last record if in flight still or crop it
    match self
      .records
      .iter_mut()
      .last()
      .map(|r| r.crop(record.start()))
    {
      None => {
        self.records.push(record);
        Ok(RecordAdded::Started)
      }
      Some(r) => match r {
        Ok(ok) => {
          self.records.push(record);
          match ok {
            RecordEnded::Noop => Ok(RecordAdded::Started),
            RecordEnded::Ended => Ok(RecordAdded::Switched),
            RecordEnded::Cropped => Ok(RecordAdded::Cropped),
          }
        }
        Err(err) => Err(err),
      },
    }
  }

  pub fn end_at(&mut self, end: DateTime<FixedOffset>) -> RResult {
    self
      .records
      .last_mut()
      .expect("No record present!")
      .crop(end)
  }

  pub fn in_flight(&self) -> bool {
    match self.records.last() {
      None => false,
      Some(record) => record.is_on_going(),
    }
  }
}

impl Eq for Record {}

impl PartialEq<Self> for Record {
  fn eq(&self, other: &Self) -> bool {
    self.start().eq(&other.start())
      && self.duration().eq(&other.duration())
      && self.is_billable().eq(&other.is_billable())
      && self.is_on_going().eq(&other.is_on_going())
  }
}

impl PartialOrd<Self> for Record {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for Record {
  fn cmp(&self, other: &Self) -> Ordering {
    self.start().cmp(&other.start())
  }
}
