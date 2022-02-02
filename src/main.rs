pub mod core;
pub mod storage;

use ansi_term::Colour;
use clap::{arg, App, AppSettings};
use std::io::ErrorKind;
use storage::FsStorage;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_DIRECTORY: &str = ".tracetime";

fn main() {
  let matches = App::new("tracetime")
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

  let location = dirs::home_dir()
    .expect("Couldn't find home directory")
    .join(DEFAULT_DIRECTORY);

  match FsStorage::new(location.as_path()) {
    Ok(storage) => match matches.subcommand() {
      Some(("project", sub_matches)) => {
        let project = sub_matches.value_of("NAME").expect("required");
        storage.create_project(project);
        println!(
          "{} project '{}'",
          Colour::Green.bold().paint("Created"),
          project,
        );
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
          "{} tracking time on '{}'",
          Colour::Green.bold().paint("Stopped"),
          "unknown",
        );
      }
      _ => unreachable!("clap should ensure we don't get here"),
    },
    Err(err) => match err {
      ErrorKind::InvalidInput => {
        eprintln!(
          "{} Location {:?} doesn't appear to be a directory!",
          Colour::Red.bold().paint("Failure"),
          location,
        )
      }
      _ => {
        eprintln!(
          "{} Couldn't access storage: {:?}",
          Colour::Red.bold().paint("Failure"),
          location,
        )
      }
    },
  }
}
