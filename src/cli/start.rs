use anyhow::{Context, Error};
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
        // Last Will: broker marks us offline if the connection drops, so evcc
        // sees a stale meter instead of a silently frozen last value.
        mqttoptions.set_last_will(rumqttc::LastWill::new(
            format!("{MQTT_TOPIC_PREFIX}/status"),
            "offline",
            rumqttc::QoS::AtLeastOnce,
            true,
        ));

        let (client, mut eventloop) = rumqttc::AsyncClient::new(mqttoptions, 10);

        tokio::spawn(async move {
            loop {
                let _ = eventloop.poll().await;
            }
        });

        // Announce availability (retained) once connected.
        let _ = client
            .publish(
                format!("{MQTT_TOPIC_PREFIX}/status"),
                rumqttc::QoS::AtLeastOnce,
                true,
                "online",
            )
            .await;

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

/// Publish every reading as **one raw numeric value per subtopic**, retained.
///
/// This is the layout evcc's `mqtt` plugin consumes directly (one topic = one
/// value, no JSON/jq), e.g. the grid meter reads `<prefix>/power`:
///   - `<prefix>/power`         momentary net power in W (+ import / − export, OBIS 16.7.0)
///   - `<prefix>/energy_import` total drawn from grid in Wh (OBIS 1.8.0)
///   - `<prefix>/energy_export` total fed into grid in Wh   (OBIS 2.8.0)
///   - `<prefix>/l1` `/l2` `/l3` per-phase power in W
///
/// Retained so a reconnecting subscriber (evcc, Grafana) gets the last value
/// immediately instead of waiting for the next SML telegram.
pub async fn publish_data(
    reading: &MeterReading,
    mqtt_client: &rumqttc::AsyncClient,
) -> Result<(), Error> {
    async fn publish_field(
        client: &rumqttc::AsyncClient,
        topic: String,
        value: f64,
    ) -> Result<(), Error> {
        client
            .publish(topic, rumqttc::QoS::AtLeastOnce, true, format!("{value}"))
            .await
            .context("Failed to publish meter value")?;
        Ok(())
    }

    if let Some(value) = reading.current_net_power {
        publish_field(mqtt_client, format!("{MQTT_TOPIC_PREFIX}/power"), value).await?;
    }
    if let Some(value) = reading.total_energy_inbound {
        publish_field(mqtt_client, format!("{MQTT_TOPIC_PREFIX}/energy_import"), value).await?;
    }
    if let Some(value) = reading.total_energy_outbound {
        publish_field(mqtt_client, format!("{MQTT_TOPIC_PREFIX}/energy_export"), value).await?;
    }
    if let Some(value) = reading.line_one {
        publish_field(mqtt_client, format!("{MQTT_TOPIC_PREFIX}/l1"), value).await?;
    }
    if let Some(value) = reading.line_two {
        publish_field(mqtt_client, format!("{MQTT_TOPIC_PREFIX}/l2"), value).await?;
    }
    if let Some(value) = reading.line_three {
        publish_field(mqtt_client, format!("{MQTT_TOPIC_PREFIX}/l3"), value).await?;
    }

    Ok(())
}
