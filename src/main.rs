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

use ansi_term::Colour;
use clap::{arg, App, AppSettings, ArgMatches};
use console::Term;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::Duration;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_DIRECTORY: &str = ".timeknight";

fn main() {
  let matches = App::new("timeknight")
    .about("Traces where all that time goes...")
    .setting(AppSettings::SubcommandRequiredElseHelp)
    .version(VERSION)
    .subcommand(
      App::new("project")
        .about("Project management")
        .subcommand(
          App::new("add")
            .arg(arg!(<NAME> "The project name to create"))
            .setting(AppSettings::ArgRequiredElseHelp),
        )
        .subcommand(
          App::new("del")
            .arg(arg!(<NAME> "The project name to delete"))
            .setting(AppSettings::ArgRequiredElseHelp),
        )
        .subcommand(App::new("list"))
        .setting(AppSettings::ArgRequiredElseHelp),
    )
    .subcommand(
      App::new("start")
        .about("Starts tracking time for a project")
        .arg(arg!(<NAME> "the project's name to start tracking time for"))
        .setting(AppSettings::ArgRequiredElseHelp),
    )
    .subcommand(App::new("stop").about("Stops tracking time"))
    .subcommand(App::new("status").about("Displays current status"))
    .get_matches();

  let location = db_location();
  init_if_needed(&location);

  match Database::open(location.as_path()) {
    Ok(mut database) => handle_command(matches, &mut database),
    Err(err) => match err {
      ErrorKind::InvalidInput => {
        eprintln!(
          "{} Location {} doesn't appear to be a directory!",
          Colour::Red.bold().paint("FAIL"),
          location.display(),
        )
      }
      _ => {
        eprintln!(
          "{} Couldn't access storage: {}",
          Colour::Red.bold().paint("FAIL"),
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
        let project = sub_matches.value_of("NAME").expect("required");
        match database.add_project(project.to_string()) {
          Ok(project) => {
            println!(
              "{} project '{}'",
              Colour::Green.bold().paint("Created"),
              project.name(),
            );
          }
          Err(_) => {
            println!(
              "{} to create project '{}'",
              Colour::Red.bold().paint("Failed"),
              project,
            );
          }
        }
      }
      Some(("del", sub_matches)) => {
        let project = sub_matches.value_of("NAME").expect("required");
        match database.remove_project(project.to_string()) {
          Ok(project) => {
            println!(
              "{} project '{}'",
              Colour::Green.bold().paint("Deleted"),
              project.name(),
            );
          }
          Err(_) => {
            println!(
              "{} to delete project '{}'",
              Colour::Red.bold().paint("Failed"),
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
            Colour::Yellow.bold().paint("No projects"),
          );
        }
        projects.iter().for_each(|p| println!("{}", p.name()));
      }
      _ => unreachable!("clap should ensure we don't get here"),
    },
    Some(("start", sub_matches)) => {
      let name = sub_matches.value_of("NAME").expect("required");
      if database.start_on(name.to_string()).is_ok() {
        println!(
          "{} tracking time on '{}'",
          Colour::Green.bold().paint("Started"),
          name,
        );
      }
    }
    Some(("stop", _sub_matches)) => match database.stop() {
      Ok(project) => {
        println!(
          "{} tracking on {} - {} recorded",
          Colour::Green.bold().paint("Stopped"),
          Colour::Green.bold().paint(project.name()),
          Colour::Green.paint(ddisplay(project.records().last().unwrap().duration())),
        );
      }
      Err(_) => {
        println!(
          "{} to be stopped",
          Colour::Yellow.bold().paint("No tracked project"),
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
            Colour::Green.bold().paint(project.name()),
            Colour::Green.paint(ddisplay(r.duration())),
          );
        }
      }
    },
    _ => unreachable!("clap should ensure we don't get here"),
  }
}

fn ddisplay(duration: Duration) -> String {
  match (
    duration.as_secs() % 60,
    (duration.as_secs() / 60) % 60,
    (duration.as_secs() / 60) / 60,
  ) {
    (1, 0, 0) => "one second".to_string(),
    (s, 0, 0) => format!("{s} seconds"),
    (1, 1, 0) => "one minute one second".to_string(),
    (s, 1, 0) => format!("one minute {s} second"),
    (1, m, 0) => format!("{m} minutes one second"),
    (s, m, 0) => format!("{m} minutes {s} seconds"),
    (_, 0, 1) => "an hour".to_string(),
    (_, 1, 1) => "an hour one minute".to_string(),
    (_, m, 1) => format!("one hour {m} minute"),
    (_, m, h) => format!("{h} hours {m} minute"),
  }
}

fn db_location() -> PathBuf {
  dirs::home_dir()
    .get_or_insert_with(|| {
      eprintln!(
        "{} Could not find a home directory, falling back to current directory",
        Colour::Purple.paint("Ugh!")
      );
      match std::env::current_dir() {
        Ok(location) => location,
        Err(err) => {
          eprintln!(
            "{}: {}",
            Colour::Red.paint("Can't access current directory"),
            err
          );
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
      Colour::Purple.paint("Welcome!")
    );
    println!("Should we initialize it in {} ?", location.display());
    match Term::stdout().read_char() {
      Ok('y') | Ok('Y') => match fs::create_dir(location) {
        Ok(_) => {
          println!(
            "{} db... {}",
            Colour::Green.bold().paint("Initializing"),
            Colour::Green.paint("Done!"),
          );
        }
        Err(err) => {
          eprintln!(
            "{} initializing db: {}",
            Colour::Red.bold().paint("Error"),
            Colour::Red.paint(format!("{}", err)),
          );
          std::process::exit(1);
        }
      },
      _ => {
        eprintln!("{} bye!", Colour::Purple.paint("Aborting..."));
        std::process::exit(1);
      }
    };
  }
}
