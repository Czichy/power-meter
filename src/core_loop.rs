use std::{io::Read, sync::Arc, time::Duration};

use anyhow::{Context, Error};
use crossbeam_utils::atomic::AtomicCell;
use serialport::{Parity, StopBits};

use crate::meter_reading::MeterReading;

const MQTT_TOPIC_PREFIX: &str = "power-meter/1-HLY03-0207-2343";

#[derive(Clone)]
pub struct CoreLoop {
    port:           String,
    // database:       &'a Database,
    latest_reading: Arc<AtomicCell<Option<MeterReading>>>,
    verbose:        bool,
}

impl CoreLoop {
    pub fn new(port: String, verbose: bool) -> Self {
        Self {
            port,
            // database,
            latest_reading: Arc::new(AtomicCell::new(None)),
            verbose,
        }
    }

    pub async fn get_data_and_publish(
        &self,
        mqtt_client: &rumqttc::AsyncClient,
    ) -> Result<(), Error> {
        let port = serialport::new(&self.port, 9_600)
            .stop_bits(StopBits::One)
            .parity(Parity::None)
            .timeout(Duration::from_millis(5000))
            .open()
            .expect("Failed to open port");

        // let mut current_ball_position = 1;
        let mut decoder = sml_rs::transport::Decoder::<Vec<u8>>::new();

        log::info!("Now listening for SML messages on {}...", self.port);

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
                    let _ = publish_data(&reading, mqtt_client).await;

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
}

pub async fn publish_data(
    reading: &MeterReading,
    mqtt_client: &rumqttc::AsyncClient,
) -> Result<(), Error> {
    let _ = mqtt_client
        .publish(
            "power-meter",
            rumqttc::QoS::AtLeastOnce,
            false,
            "1-HLY03-0207-2343 alive",
        )
        .await;
    if let Some(meter_time) = reading.meter_time {
        if let (Some(total_energy_inbound), Some(total_energy_inbound_unit)) = (
            &reading.total_energy_inbound,
            &reading.total_energy_inbound_unit,
        ) {
            mqtt_client
                .publish(
                    MQTT_TOPIC_PREFIX.to_string(),
                    rumqttc::QoS::AtLeastOnce,
                    false,
                    format!(
                        "{{ \"timestamp\": {meter_time}, \"total_inbound\": \
                         {total_energy_inbound}, \"unit\" : \"{total_energy_inbound_unit}\" }}",
                    ),
                )
                .await
                .context("Failed to publish current consumption message")?;
        }

        if let (Some(total_energy_outbound), Some(total_energy_outbound_unit)) = (
            &reading.total_energy_outbound,
            &reading.total_energy_outbound_unit,
        ) {
            mqtt_client
                .publish(
                    MQTT_TOPIC_PREFIX.to_string(),
                    rumqttc::QoS::AtLeastOnce,
                    false,
                    format!(
                        "{{ \"timestamp\": {meter_time},\"total_outbound\": \
                         {total_energy_outbound}, \"unit\" : \"{total_energy_outbound_unit}\"
         }}",
                    ),
                )
                .await
                .context("Failed to publish current consumption message")?;
        }

        if let (Some(current_net_power), Some(current_net_power_unit)) =
            (&reading.current_net_power, &reading.current_net_power_unit)
        {
            mqtt_client
                .publish(
                    MQTT_TOPIC_PREFIX.to_string(),
                    rumqttc::QoS::AtMostOnce,
                    false,
                    format!(
                        "{{ \"timestamp\": {meter_time}, \"current_net_power\":  \
                         {current_net_power}, \"unit\" : \"{current_net_power_unit}\" }}",
                    ),
                )
                .await
                .context("Failed to publish current consumption message")?;
        }

        if let (Some(line_one), Some(line_one_unit)) = (&reading.line_one, &reading.line_one_unit) {
            mqtt_client
                .publish(
                    MQTT_TOPIC_PREFIX.to_string(),
                    rumqttc::QoS::AtMostOnce,
                    false,
                    format!(
                        "{{ \"timestamp\": {meter_time}, \"line_one\": {line_one}, \"unit\" : \
                         \"{line_one_unit}\" }}",
                    ),
                )
                .await
                .context("Failed to publish current consumption message")?;
        }
        if let (Some(line_two), Some(line_two_unit)) = (&reading.line_two, &reading.line_two_unit) {
            mqtt_client
                .publish(
                    MQTT_TOPIC_PREFIX.to_string(),
                    rumqttc::QoS::AtMostOnce,
                    false,
                    format!(
                        "{{ \"timestamp\": {meter_time}, \"line_two\": {line_two}, \"unit\" : \
                         \"{line_two_unit}\" }}",
                    ),
                )
                .await
                .context("Failed to publish current consumption message")?;
        }
        if let (Some(line_three), Some(line_three_unit)) =
            (&reading.line_three, &reading.line_three_unit)
        {
            mqtt_client
                .publish(
                    MQTT_TOPIC_PREFIX.to_string(),
                    rumqttc::QoS::AtMostOnce,
                    false,
                    format!(
                        "{{ \"timestamp\": {meter_time}, \"line_three\": {line_three}, \"unit\" : \
                         \"{line_three_unit}\"
         }}",
                    ),
                )
                .await
                .context("Failed to publish current consumption message")?;
        }
    }

    Ok(())
}
