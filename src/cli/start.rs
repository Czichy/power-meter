use anyhow::{Context, Error};
use chrono::Utc;
use clap_derive::Args;
use tokio::io::AsyncRead;
use tokio_serial::SerialStream;
use tokio_stream::StreamExt;

use crate::meter_reading::MeterReading;

const MQTT_CLIENT_NAME: &str = "HL-3-RZ-POWER-01";
const MQTT_BROKER_ADDRESS: &str = "10.15.40.33";
const MQTT_BROKER_PORT: u16 = 1883;
const MQTT_TOPIC_PREFIX: &str = "power-meter/1-HLY03-0207-2343";

#[derive(Clone, Args)]
pub struct StartCommand {
    #[arg(long)]
    port: String,

    #[arg(long, default_value = "false")]
    verbose: bool,
}

impl StartCommand {
    pub async fn run(self) -> Result<(), Error> {
        let uart = uart_ir_sensor_data_stream(self.port);
        let mut stream = crate::meter_reading::sml_message_stream(uart);

        let mut mqttoptions =
            rumqttc::MqttOptions::new(MQTT_CLIENT_NAME, MQTT_BROKER_ADDRESS, MQTT_BROKER_PORT);
        mqttoptions.set_keep_alive(std::time::Duration::from_secs(10));

        let (client, mut eventloop) = rumqttc::AsyncClient::new(mqttoptions, 5);

        tokio::spawn(async move {
            loop {
                let _ = eventloop.poll().await;
            }
        });
        while let Some(event) = stream.next().await {
            let _ = publish_data(&event, &client).await;
        }

        Ok(())
    }
}

pub(crate) fn uart_ir_sensor_data_stream(port: String) -> impl AsyncRead {
    let serial = tokio_serial::new(port, 9600);
    SerialStream::open(&serial).unwrap()
}
pub async fn publish_data(
    reading: &MeterReading,
    mqtt_client: &rumqttc::AsyncClient,
) -> Result<(), Error> {
    let meter_install_date: chrono::DateTime<Utc> =
        chrono::DateTime::from_timestamp(1728985109, 0).expect("invalid timestamp");

    let _ = mqtt_client
        .publish(
            "power-meter",
            rumqttc::QoS::AtLeastOnce,
            false,
            "1-HLY03-0207-2343 alive",
        )
        .await;
    if let Some(meter_time) = reading.meter_time {
        let meter_time =
            (meter_install_date + chrono::Duration::seconds(meter_time as i64)).timestamp();
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
