pub mod core;

use ansi_term::Colour;
use clap::{arg, App, AppSettings};

const VERSION: &str = env!("CARGO_PKG_VERSION");

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

  match matches.subcommand() {
    Some(("project", sub_matches)) => {
      println!(
        "{} project '{}'",
        Colour::Green.bold().paint("Created"),
        sub_matches.value_of("NAME").expect("required"),
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
  }
}
