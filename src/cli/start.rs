use std::thread;

use anyhow::Error;
use clap_derive::Args;

use crate::core_loop::CoreLoop;
// use crate::database::Database;
use crate::server::Server;

const MQTT_CLIENT_NAME: &str = "HL-3-RZ-POWER-01";
const MQTT_BROKER_ADDRESS: &str = "10.15.40.33";
const MQTT_BROKER_PORT: u16 = 1883;

#[derive(Clone, Args)]
pub struct StartCommand {
    #[arg(long)]
    port: String,

    #[arg(long, default_value = "false")]
    verbose: bool,
}

impl StartCommand {
    pub fn run(self) -> Result<(), Error> {
        // let database = Database::load()?;
        let mut mqttoptions =
            rumqttc::MqttOptions::new(MQTT_CLIENT_NAME, MQTT_BROKER_ADDRESS, MQTT_BROKER_PORT);
        mqttoptions.set_keep_alive(std::time::Duration::from_secs(10));
        let (mqtt_client, mut _connection) = rumqttc::Client::new(mqttoptions, 10);

        let core_loop = CoreLoop::new(self.port, self.verbose);
        let latest_reading_cell = core_loop.get_latest_reading_cell();

        let server_thread = thread::spawn(|| Server::create(3000, latest_reading_cell).enter());

        core_loop.enter(mqtt_client)?;

        server_thread.join().unwrap()?;
        Ok(())
    }
}
