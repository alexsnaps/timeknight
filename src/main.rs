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
pub mod database;

use database::Database;
use std::fs;

use ansi_term::Colour;
use clap::{arg, App, AppSettings, ArgMatches};
use console::Term;
use std::io::ErrorKind;
use std::path::PathBuf;

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
        match database.add_project(project) {
          Ok(()) => {
            println!(
              "{} project '{}'",
              Colour::Green.bold().paint("Created"),
              project,
            );
          }
          Err(()) => {
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
        match database.remove_project(project) {
          Ok(()) => {
            println!(
              "{} project '{}'",
              Colour::Green.bold().paint("Deleted"),
              project,
            );
          }
          Err(()) => {
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
      println!(
        "{} tracking time on '{}'",
        Colour::Green.bold().paint("Started"),
        sub_matches.value_of("NAME").expect("required"),
      );
    }
    Some(("stop", _sub_matches)) => {
      println!(
        "{} to be stopped",
        Colour::Yellow.bold().paint("No tracked project"),
      );
    }
    _ => unreachable!("clap should ensure we don't get here"),
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

fn init_if_needed(location: &PathBuf) {
  if !location.exists() {
    println!(
      "{} Looks like the environment wasn't ever set up...",
      Colour::Purple.paint("Welcome!")
    );
    println!(
      "Should we initialize it in {} ?",
      location.to_str().unwrap()
    );
    match Term::stdout().read_char() {
      Ok('y') | Ok('Y') => match fs::create_dir(location.as_path()) {
        Ok(_) => {
          println!(
            "{} database... {}",
            Colour::Green.bold().paint("Initializing"),
            Colour::Green.paint("Done!"),
          );
        }
        Err(err) => {
          eprintln!(
            "{} initializing database: {}",
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
