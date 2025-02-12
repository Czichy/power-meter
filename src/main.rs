use clap::Parser;

// use crate::cli::root_command::RootCommand;

mod cli;
mod meter_reading;
mod obis_code;
mod server;
mod unit;

// fn main() -> Result<(), Error> { RootCommand::parse().run() }

#[tokio::main()]
async fn main() -> Result<(), anyhow::Error> {
    let formatter = syslog::Formatter3164 {
        facility: syslog::Facility::LOG_DAEMON,
        hostname: None,
        process:  "power-meter".into(),
        pid:      0,
    };

    env_logger::init();
    syslog::unix(formatter).expect("Failed to initialize syslog");

    println!(
        "Starting Power-Meter (power-meter) v{}",
        env!("CARGO_PKG_VERSION")
    );

    cli::root_command::RootCommand::parse().run().await
}
