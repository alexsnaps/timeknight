pub mod core;

use chrono::offset::{Local, TimeZone};
use crate::core::{Project, Record};

fn main() {
  // let time = chrono::Local::now();
  // println!("{:?}", time);
  let start = Local.ymd(2021, 03, 14).and_hms(0, 0, 0).naive_utc();
  let end = Local.ymd(2021, 03, 14).and_hms(4, 0, 0).naive_utc();
  // let start = DateTime::parse_from_rfc3339("2021-03-14T00:00:00-05:00").unwrap();
  // let end = DateTime::parse_from_rfc3339("2021-03-14T04:00:00-04:00").unwrap();

  // FixedOffset::from_utc_date();

  println!("{:?}", start);
  println!("{:?}", end);
  let diff = end.signed_duration_since(start);
  println!("{}", diff.num_hours());

  let mut project = Project::new("foo".to_owned());
  let started = project.start();
  println!("Project {} got started: {:?}", project.name(), started);
  println!("with {} record", project.records().count());
}
