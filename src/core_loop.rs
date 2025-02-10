use std::{io::Read, sync::Arc, time::Duration};

use anyhow::Error;
use crossbeam_utils::atomic::AtomicCell;
use serialport::{Parity, StopBits};

use crate::meter_reading::MeterReading;

const MQTT_CLIENT_NAME: &str = "HL-3-RZ-POWER-01";
const MQTT_BROKER_ADDRESS: &str = "10.15.40.33";
const MQTT_BROKER_PORT: u16 = 1883;
const MQTT_TOPIC_PREFIX: &str = "power-meter/1-HLY03-0207-2343/";

pub struct CoreLoop {
    port:           String,
    // database:       &'a Database,
    latest_reading: Arc<AtomicCell<Option<MeterReading>>>,
    mqtt_client:    rumqttc::Client,
    verbose:        bool,
}

impl CoreLoop {
    pub fn new(port: String, verbose: bool) -> Self {
        // Set MQTT connection options
        let mut mqttoptions =
            rumqttc::MqttOptions::new(MQTT_CLIENT_NAME, MQTT_BROKER_ADDRESS, MQTT_BROKER_PORT);
        mqttoptions.set_keep_alive(Duration::from_secs(10));
        let (mqtt_client, mut _connection) = rumqttc::Client::new(mqttoptions, 10);
        Self {
            port,
            // database,
            latest_reading: Arc::new(AtomicCell::new(None)),
            mqtt_client,
            verbose,
        }
    }

    pub fn enter(&self) -> Result<(), Error> {
        let port = serialport::new(&self.port, 9_600)
            .stop_bits(StopBits::One)
            .parity(Parity::None)
            .timeout(Duration::from_millis(5000))
            .open()
            .expect("Failed to open port");

        // let mut current_ball_position = 1;
        let mut decoder = sml_rs::transport::Decoder::<Vec<u8>>::new();

        println!("Now listening for SML messages on {}...", self.port);

        for res in port.bytes() {
            let byte = res?;

            match decoder.push_byte(byte) {
                Ok(None) => {},
                Ok(Some(decoded_bytes)) => {
                    let result = sml_rs::parser::complete::parse(decoded_bytes);
                    let Ok(sml_file) = result else {
                        if self.verbose {
                            println!("Err({:?})", result);
                        }
                        continue;
                    };

                    let reading = MeterReading::parse(sml_file);
                    let Ok(reading) = reading else {
                        continue;
                    };
                    if self.verbose {
                        println!("{}", reading.display_compact());
                    }

                    // self.database.insert_reading(&reading)?;
                    if let Some(meter_time) = reading.meter_time {
                        if let (Some(total_energy_inbound), Some(total_energy_inbound_unit)) = (
                            &reading.total_energy_inbound,
                            &reading.total_energy_inbound_unit,
                        ) {
                            let Ok(sent) = self.mqtt_client.publish(
                                format!("{MQTT_TOPIC_PREFIX}/meter_time"),
                                rumqttc::QoS::AtLeastOnce,
                                false,
                                format!(
                                    "{{ \"timestamp\": {meter_time}, \"value\": \
                                     {total_energy_inbound}, \"unit\" : \
                                     \"{total_energy_inbound_unit}\" }}",
                                ),
                            ) else {
                                println!("Cannot send MQTT message!");
                                // continue;
                            };
                        }

                        if let (Some(total_energy_outbound), Some(total_energy_outbound_unit)) = (
                            &reading.total_energy_outbound,
                            &reading.total_energy_outbound_unit,
                        ) {
                            match self.mqtt_client.publish(
                                format!("{MQTT_TOPIC_PREFIX}/meter_time"),
                                rumqttc::QoS::AtLeastOnce,
                                false,
                                format!(
                                    "{{ \"timestamp\": {meter_time}, \"value\": \
                                     {total_energy_outbound}, \"unit\" : \
                                     \"{total_energy_outbound_unit}\" }}",
                                ),
                            ) {
                                Ok(_) => {},
                                Err(err) => {
                                    println!("Cannot send to MQTT {err}");
                                    // continue;
                                },
                            };
                        }

                        if let (Some(current_net_power), Some(current_net_power_unit)) =
                            (&reading.current_net_power, &reading.current_net_power_unit)
                        {
                            let Ok(sent) = self.mqtt_client.publish(
                                format!("{MQTT_TOPIC_PREFIX}/meter_time"),
                                rumqttc::QoS::AtLeastOnce,
                                false,
                                format!(
                                    "{{ \"timestamp\": {meter_time}, \"value\": \
                                     {current_net_power}, \"unit\" : \"{current_net_power_unit}\" \
                                     }}",
                                ),
                            ) else {
                                // continue;
                            };
                        }

                        if let (Some(line_one), Some(line_one_unit)) =
                            (&reading.line_one, &reading.line_one_unit)
                        {
                            let Ok(sent) = self.mqtt_client.publish(
                                format!("{MQTT_TOPIC_PREFIX}/meter_time"),
                                rumqttc::QoS::AtLeastOnce,
                                false,
                                format!(
                                    "{{ \"timestamp\": {meter_time}, \"value\": {line_one}, \
                                     \"unit\" : \"{line_one_unit}\" }}",
                                ),
                            ) else {
                                // continue;
                            };
                        }
                        if let (Some(line_two), Some(line_two_unit)) =
                            (&reading.line_two, &reading.line_two_unit)
                        {
                            let Ok(sent) = self.mqtt_client.publish(
                                format!("{MQTT_TOPIC_PREFIX}/meter_time"),
                                rumqttc::QoS::AtLeastOnce,
                                false,
                                format!(
                                    "{{ \"timestamp\": {meter_time}, \"value\": {line_two}, \
                                     \"unit\" : \"{line_two_unit}\" }}",
                                ),
                            ) else {
                                // continue;
                            };
                        }
                        if let (Some(line_three), Some(line_three_unit)) =
                            (&reading.line_three, &reading.line_three_unit)
                        {
                            let Ok(sent) = self.mqtt_client.publish(
                                format!("{MQTT_TOPIC_PREFIX}/meter_time"),
                                rumqttc::QoS::AtLeastOnce,
                                false,
                                format!(
                                    "{{ \"timestamp\": {meter_time}, \"value\": {line_three}, \
                                     \"unit\" : \"{line_three_unit}\" }}",
                                ),
                            ) else {
                                // continue;
                            };
                        }
                    }
                    self.latest_reading.store(Some(reading));
                },
                Err(e) => {
                    if self.verbose {
                        println!("Err({:?})", e);
                    }
                },
            }
        }

        Ok(())
    }

    pub fn get_latest_reading_cell(&self) -> Arc<AtomicCell<Option<MeterReading>>> {
        self.latest_reading.clone()
    }
}
