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

use chrono::{DateTime, FixedOffset, Local};
use std::cmp::Ordering;
use std::time::Duration;

#[derive(Debug)]
pub enum RecordEnded {
  Noop,
  Cropped,
  Ended,
}

#[derive(Debug)]
pub enum IllegalStateError {
  NegativeDuration,
  NoDuration,
}

pub type RResult = Result<RecordEnded, IllegalStateError>;

pub struct Record {
  start: chrono::DateTime<FixedOffset>,
  end: Option<chrono::DateTime<FixedOffset>>,
  billable: bool,
}

impl Record {
  pub fn new() -> Self {
    Record::started_on(Record::now())
  }

  pub fn started_on(start: DateTime<FixedOffset>) -> Self {
    Record {
      start,
      end: None,
      billable: true,
    }
  }

  pub fn start(&self) -> DateTime<FixedOffset> {
    self.start
  }

  pub fn is_on_going(&self) -> bool {
    self.end.is_none()
  }

  pub fn duration(&self) -> Duration {
    let end = self.end.or_else(|| Some(Record::now())).unwrap();
    let duration = end.signed_duration_since(self.start);
    if duration < chrono::Duration::zero() {
      return Duration::ZERO;
    }
    Duration::from_secs(duration.num_seconds() as u64)
  }

  pub fn crop(&mut self, new_end: DateTime<FixedOffset>) -> RResult {
    match self.start.cmp(&new_end) {
      Ordering::Greater => Err(IllegalStateError::NegativeDuration),
      Ordering::Less => Err(IllegalStateError::NoDuration),
      Ordering::Equal => match self.end {
        None => {
          self.end = Some(new_end);
          Ok(RecordEnded::Ended)
        }
        Some(end) => {
          if new_end < end {
            self.end = Some(new_end);
            Ok(RecordEnded::Cropped)
          } else {
            Ok(RecordEnded::Noop)
          }
        }
      },
    }
  }

  pub fn is_billable(&self) -> bool {
    self.billable
  }

  fn now() -> DateTime<FixedOffset> {
    let now = Local::now();
    now.with_timezone(now.offset())
  }
}

impl Default for Record {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use crate::core::record::Record;

  use chrono::{DateTime, FixedOffset, TimeZone, Utc};
  use std::ops::Sub;
  use std::time::Duration;

  #[test]
  fn new_record_has_proper_defaults() {
    let record = Record::new();
    assert_eq!(
      record
        .start()
        .signed_duration_since(Record::now())
        .num_seconds(),
      0
    );
    assert_eq!(record.duration(), Duration::ZERO);
    assert!(record.is_billable());
    assert!(record.is_on_going());
  }

  #[test]
  fn duration_math_works() {
    let two_seconds = chrono::Duration::seconds(2);
    let start = Record::now().sub(two_seconds);
    let record = Record::started_on(start);
    assert_eq!(record.start(), start);
    assert_eq!(record.duration(), Duration::from_secs(2));
    assert!(record.is_billable());
    assert!(record.is_on_going());
  }

  #[test]
  fn negative_duration_is_zero() {
    let two_seconds = chrono::Duration::seconds(2);
    let start = Record::now() + two_seconds;
    let record = Record::started_on(start);
    assert_eq!(record.start(), start);
    assert_eq!(record.duration(), Duration::ZERO);
    assert!(record.is_billable());
    assert!(record.is_on_going());
  }

  #[test]
  fn deconstruct() {
    let now = Record::now();
    let ts = now.timestamp_millis();
    let tz = now.offset().utc_minus_local();
    let utc = Utc.timestamp_millis(ts);
    let offset = FixedOffset::from_offset(&FixedOffset::west(tz));
    let other: DateTime<FixedOffset> = utc.with_timezone(&offset);
    println!("{} == {}", now.to_rfc3339(), other.to_rfc3339());
    assert_eq!(now.timezone(), other.timezone());
    assert_eq!(ts, other.timestamp_millis());
  }
}
