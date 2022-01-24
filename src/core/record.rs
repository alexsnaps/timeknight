use chrono::{DateTime, FixedOffset, Local};
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
    let end = self.end.or(Some(Record::now())).unwrap();
    let duration = end.signed_duration_since(self.start);
    if duration < chrono::Duration::zero() {
      return Duration::ZERO;
    }
    Duration::from_secs(duration.num_seconds() as u64)
  }

  pub fn crop(&mut self, new_end: DateTime<FixedOffset>) -> RResult {
    if self.start > new_end {
      return Err(IllegalStateError::NegativeDuration);
    } else if self.start == new_end {
      return Err(IllegalStateError::NoDuration);
    }

    match self.end {
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

#[cfg(test)]
mod tests {
  use crate::core::record::Record;

  use std::ops::Sub;
  use std::time::Duration;

  #[test]
  fn new_record_has_proper_defaults() {
    let record = Record::new();
    assert_eq!(record.start().signed_duration_since(Record::now()).num_seconds(), 0);
    assert_eq!(record.duration(), Duration::ZERO);
    assert_eq!(record.is_billable(), true);
    assert_eq!(record.is_on_going(), true);
  }

  #[test]
  fn duration_math_works() {
    let two_seconds = chrono::Duration::seconds(2);
    let start = Record::now().sub(two_seconds);
    let record = Record::started_on(start);
    assert_eq!(record.start(), start);
    assert_eq!(record.duration(), Duration::from_secs(2));
    assert_eq!(record.is_billable(), true);
    assert_eq!(record.is_on_going(), true);
  }


  #[test]
  fn negative_duration_is_zero() {
    let two_seconds = chrono::Duration::seconds(2);
    let start = Record::now() + two_seconds;
    let record = Record::started_on(start);
    assert_eq!(record.start(), start);
    assert_eq!(record.duration(), Duration::ZERO);
    assert_eq!(record.is_billable(), true);
    assert_eq!(record.is_on_going(), true);
  }
}
