pub mod core;
pub mod storage;

use ansi_term::Colour;
use clap::{arg, App, AppSettings, ArgMatches};
use std::io::ErrorKind;
use std::path::PathBuf;
use storage::FsStorage;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_DIRECTORY: &str = ".timeknight";

fn main() {
  let matches = App::new("timeknight")
    .about("Traces where all that time goes...")
    .setting(AppSettings::SubcommandRequiredElseHelp)
    .version(VERSION)
    .subcommand(
      App::new("project")
        .about("Creates a project")
        .arg(arg!(<NAME> "The project name to create"))
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

  match FsStorage::new(location.as_path()) {
    Ok(mut storage) => handle_command(matches, &mut storage),
    Err(err) => match err {
      ErrorKind::InvalidInput => {
        eprintln!(
          "{} Location {:?} doesn't appear to be a directory!",
          Colour::Red.bold().paint("FAIL"),
          location,
        )
      }
      _ => {
        eprintln!(
          "{} Couldn't access storage: {:?}",
          Colour::Red.bold().paint("FAIL"),
          location,
        )
      }
    },
  }
}

fn handle_command(matches: ArgMatches, storage: &mut FsStorage) {
  match matches.subcommand() {
    Some(("project", sub_matches)) => {
      let project = sub_matches.value_of("NAME").expect("required");
      match create_project(storage, project) {
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

fn create_project(storage: &mut FsStorage, project: &str) -> Result<(), ()> {
  storage.create_project(project);
  Ok(())
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
