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

pub mod core;
pub mod db;

use db::Database;
use std::fs;

use crate::core::Project;
use chrono::{DateTime, Datelike, Local};
use clap::{arg, ArgMatches, Command};
use console::{style, Term};
use itertools::Itertools;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::Duration;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_DIRECTORY: &str = ".timeknight";

fn main() {
  let matches = Command::new("timeknight")
    .about("Traces where all that time goes...")
    .version(VERSION)
    .subcommand_required(true)
    .subcommand(
      Command::new("project")
        .about("Project management")
        .subcommand_required(true)
        .subcommand(Command::new("add").arg(arg!(<NAME> "The project name to create")))
        .subcommand(Command::new("del").arg(arg!(<NAME> "The project name to delete")))
        .subcommand(Command::new("list")),
    )
    .subcommand(
      Command::new("start")
        .about("Starts tracking time for a project")
        .arg(arg!(<NAME> "the project's name to start tracking time for")),
    )
    .subcommand(Command::new("stop").about("Stops tracking time"))
    .subcommand(Command::new("status").about("Displays current status"))
    .subcommand(
      Command::new("report")
        .about("Reports")
        .arg(
          arg!(<PERIOD> "Period to produce the report for")
            .required(false)
            .value_parser(clap::builder::PossibleValuesParser::new([
              "ever",
              "today",
              "yesterday",
              "week",
              "lastweek",
              "month",
              "lastmonth",
            ]))
            .default_value("ever"),
        )
        .arg(
          arg!(--"by" <GROUPING>)
            .value_parser(clap::builder::PossibleValuesParser::new(["day"]))
            .required(false),
        ),
    )
    .get_matches();

  let location = db_location();
  init_if_needed(&location);

  match Database::open(location.as_path()) {
    Ok(mut database) => handle_command(matches, &mut database),
    Err(err) => match err {
      ErrorKind::InvalidInput => {
        eprintln!(
          "{} Location {} doesn't appear to be a directory!",
          style("FAIL").red().bold(),
          location.display(),
        )
      }
      _ => {
        eprintln!(
          "{} Couldn't access storage: {}",
          style("FAIL").red().bold(),
          location.display(),
        )
      }
    },
  }
}

fn handle_command(matches: ArgMatches, database: &mut Database) {
  match matches.subcommand() {
    Some(("project", sub_matches)) => match sub_matches.subcommand() {
      Some(("add", sub_matches)) => {
        let project = sub_matches
          .get_one::<String>("NAME")
          .expect("required")
          .clone();
        match database.add_project(project.clone()) {
          Ok(project) => {
            println!(
              "{} project '{}'",
              style("Created").green().bold(),
              project.name(),
            );
          }
          Err(_) => {
            println!(
              "{} to create project '{}'",
              style("Failed").red().bold(),
              project,
            );
          }
        }
      }
      Some(("del", sub_matches)) => {
        let project = sub_matches
          .get_one::<String>("NAME")
          .expect("required")
          .clone();
        match database.remove_project(project.clone()) {
          Ok(project) => {
            println!(
              "{} project '{}'",
              style("Deleted").green().bold(),
              project.name(),
            );
          }
          Err(_) => {
            println!(
              "{} to delete project '{}'",
              style("Failed").red().bold(),
              project,
            );
          }
        }
      }
      Some(("list", _)) => {
        let projects = database.list_projects();
        if projects.is_empty() {
          println!(
            "{} use 'add' to create one",
            style("No projects").yellow().bold(),
          );
        }
        projects.iter().for_each(|p| println!("{}", p.name()));
      }
      _ => unreachable!("clap should ensure we don't get here"),
    },
    Some(("start", sub_matches)) => {
      let name = sub_matches.get_one::<String>("NAME").expect("required");
      if database.start_on(name.to_string()).is_ok() {
        println!(
          "{} tracking time on '{}'",
          style("Started").green().bold(),
          name,
        );
      }
    }
    Some(("stop", _sub_matches)) => match database.stop() {
      Ok(project) => {
        println!(
          "{} tracking on {} - {} recorded",
          style("Stopped").green().bold(),
          style(project.name()).green().bold(),
          style(display_duration(
            project.records().last().unwrap().duration()
          ))
          .green(),
        );
      }
      Err(_) => {
        println!(
          "{} to be stopped",
          style("No tracked project").yellow().bold(),
        );
      }
    },
    Some(("status", _sub_matches)) => match database.current_project() {
      None => println!("Nothing going on!"),
      Some(project) => {
        let r = project.records().last().unwrap();
        if r.is_on_going() {
          println!(
            "Working on {} for {}",
            style(project.name()).green().bold(),
            style(display_duration(r.duration())).green(),
          );
        }
      }
    },
    Some(("report", sub_matches)) => {
      let mut projects = database.list_projects();
      projects.sort_by_key(|p| p.name().to_lowercase());
      let now = Local::now();
      let period = sub_matches.get_one::<String>("PERIOD").unwrap();
      let lines = build_report(
        &projects,
        now,
        period,
        sub_matches.get_one::<String>("by").is_some(),
      );
      print_report(lines);
    }
    _ => unreachable!("clap should ensure we don't get here"),
  }
}

fn build_report(
  projects: &[&Project],
  now: DateTime<Local>,
  period: &str,
  by_day: bool,
) -> Vec<(String, String, String)> {
  let tz = now.offset();
  let (start, end) = match period {
    "ever" => {
      let min = chrono::MIN_DATE;
      let max = chrono::MAX_DATE;
      (min.with_timezone(tz), max.with_timezone(tz))
    }
    "today" => {
      let today = now.with_timezone(now.offset());
      (today.date(), today.date())
    }
    "yesterday" => {
      let yesterday = now.with_timezone(now.offset()) - chrono::Duration::days(1);
      (yesterday.date(), yesterday.date())
    }
    "week" => {
      let off = now.weekday().num_days_from_monday();
      let today = now.with_timezone(now.offset());
      let start = today - chrono::Duration::days(off as i64);
      (start.date(), today.date())
    }
    "lastweek" => {
      let off = now.weekday().num_days_from_monday();
      let start = now - chrono::Duration::days(off as i64 + 7);
      let end = now - chrono::Duration::days(off as i64 + 1);
      (
        start.with_timezone(start.offset()).date(),
        end.with_timezone(end.offset()).date(),
      )
    }

    "month" => {
      let start = now.date().with_day(1).unwrap();
      let today = now.with_timezone(now.offset());
      (start.with_timezone(start.offset()), today.date())
    }
    "lastmonth" => {
      let start = now
        .date()
        .with_day(1)
        .unwrap()
        .with_month(now.month() - 1)
        .unwrap();
      let end = start.with_month(now.month()).unwrap() - chrono::Duration::days(1);
      (
        start.with_timezone(start.offset()),
        end.with_timezone(start.offset()),
      )
    }
    _ => unreachable!("clap should ensure we don't get here"),
  };
  let lines: Vec<(String, String, String)> = if by_day {
    projects
      .iter()
      .flat_map(|p| {
        p.records()
          .group_by(|r| r.start().date())
          .into_iter()
          .filter(|(day, _)| day >= &start && day <= &end)
          .map(|(day, records)| {
            (
              p.name().to_string(),
              format!("{}", day.naive_local()),
              display_duration(
                records
                  .into_iter()
                  .filter(|r| r.start().date() >= start && r.start().date() <= end)
                  .map(|r| r.duration())
                  .sum(),
              ),
            )
          })
          .collect::<Vec<(String, String, String)>>()
      })
      .collect()
  } else {
    projects
      .iter()
      .map(|p| {
        (
          p.name().to_string(),
          period.to_string(),
          display_duration(
            p.records()
              .filter(|r| {
                r.start().date().naive_local() >= start.naive_local()
                  && r.start().date().naive_local() <= end.naive_local()
              })
              .map(|r| r.duration())
              .sum(),
          ),
        )
      })
      .collect()
  };
  lines
}

fn print_report(lines: Vec<(String, String, String)>) {
  let h1 = "Project";
  let h2 = "Period";
  let h3 = "Duration";

  let (n_width, p_width, d_width) = lines
    .iter()
    .map(|(n, p, d)| (n.len(), p.len(), d.len()))
    .fold((h1.len(), h2.len(), h3.len()), |(m1, m2, m3), (n, p, d)| {
      (m1.max(n), m2.max(p), m3.max(d))
    });

  println!(
    "┏━{0:━>w1$}━┯━{0:━>w2$}━┯━{0:━^w3$}━┓",
    "━",
    w1 = n_width,
    w2 = p_width,
    w3 = d_width
  );
  println!(
    "┃ {0: ^w1$} │ {1: ^w2$} │ {2: ^w3$} ┃",
    h1,
    h2,
    h3,
    w1 = n_width,
    w2 = p_width,
    w3 = d_width
  );
  println!(
    "┠─{0:─>w1$}─┼─{0:─>w2$}─┼─{0:─^w3$}─┨",
    "─",
    w1 = n_width,
    w2 = p_width,
    w3 = d_width
  );
  lines.iter().for_each(|(project, period, duration)| {
    println!(
      "┃ {0: >w1$} │ {1: ^w2$} │ {2: <w3$} ┃",
      project,
      period,
      duration,
      w1 = n_width,
      w2 = p_width,
      w3 = d_width,
    );
  });
  println!(
    "┗━{0:━>w1$}━┷━{0:━>w2$}━┷━{0:━^w3$}━┛",
    "━",
    w1 = n_width,
    w2 = p_width,
    w3 = d_width
  );
}

fn display_duration(duration: Duration) -> String {
  match (
    duration.as_secs() % 60,
    (duration.as_secs() / 60) % 60,
    (duration.as_secs() / 60) / 60,
  ) {
    (0, 0, 0) => "None".to_string(),
    (1, 0, 0) => "one second".to_string(),
    (s, 0, 0) => format!("{s} seconds"),
    (1, 1, 0) => "one minute one second".to_string(),
    (s, 1, 0) => format!("one minute {s} second"),
    (1, m, 0) => format!("{m} minutes one second"),
    (s, m, 0) => format!("{m} minutes {s} seconds"),
    (_, 0, 1) => "an hour".to_string(),
    (_, 1, 1) => "an hour one minute".to_string(),
    (_, m, 1) => format!("one hour {m} minute"),
    (_, m, h) => format!("{h} hours {m} minutes"),
  }
}

fn db_location() -> PathBuf {
  dirs::home_dir()
    .get_or_insert_with(|| {
      eprintln!(
        "{} Could not find a home directory, falling back to current directory",
        style("Ugh!").cyan(),
      );
      match std::env::current_dir() {
        Ok(location) => location,
        Err(err) => {
          eprintln!("{}: {}", style("Can't access current directory").red(), err);
          std::process::exit(1);
        }
      }
    })
    .join(DEFAULT_DIRECTORY)
}

fn init_if_needed(location: &Path) {
  if !location.exists() {
    println!(
      "{} Looks like the environment wasn't ever set up...",
      style("Welcome!").cyan(),
    );
    println!("Should we initialize it in {} ?", location.display());
    match Term::stdout().read_char() {
      Ok('y') | Ok('Y') => match fs::create_dir(location) {
        Ok(_) => {
          println!(
            "{} db... {}",
            style("Initializing").green().bold(),
            style("Done!").green(),
          );
        }
        Err(err) => {
          eprintln!(
            "{} initializing db: {}",
            style("Error").red().bold(),
            style(format!("{}", err)).red().bold(),
          );
          std::process::exit(1);
        }
      },
      _ => {
        eprintln!("{} bye!", style("Aborting...").yellow());
        std::process::exit(1);
      }
    };
  }
}
