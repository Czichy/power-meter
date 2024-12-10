use anyhow::Error;
use clap::Parser;

use crate::cli::root_command::RootCommand;

mod cli;
mod meter_reading;
mod obis_code;
mod unit;
// mod database;
mod core_loop;
mod server;

fn main() -> Result<(), Error> { RootCommand::parse().run() }
