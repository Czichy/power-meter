use std::fmt::Display;

use anyhow::{anyhow, bail, Error};
use serde::Serialize;
use sml_rs::parser::{common::{Time, Value},
                     complete::{File, MessageBody}};
use tokio::{io::{AsyncRead, AsyncReadExt},
            sync::mpsc::{self, Sender}};
use tokio_stream::{wrappers::ReceiverStream, Stream};

use crate::{obis_code::ObisCode, unit::Unit};

#[derive(Serialize)]
pub struct MeterReading {
    pub meter_time: Option<u32>,

    pub total_energy_inbound:      Option<f64>,
    pub total_energy_inbound_unit: Option<Unit>,

    pub total_energy_outbound:      Option<f64>,
    pub total_energy_outbound_unit: Option<Unit>,

    pub current_net_power:      Option<f64>,
    pub current_net_power_unit: Option<Unit>,

    pub line_one:      Option<f64>, // watts
    pub line_one_unit: Option<Unit>,

    pub line_two:      Option<f64>, // watts
    pub line_two_unit: Option<Unit>,

    pub line_three:      Option<f64>, // watts
    pub line_three_unit: Option<Unit>,
}

const OBIS_TOTAL_INBOUND_COUNT: ObisCode = ObisCode::from_octet_str(&[1, 0, 1, 8, 0, 255]);
const OBIS_TOTAL_OUTBOUND_COUNT: ObisCode = ObisCode::from_octet_str(&[1, 0, 2, 8, 0, 255]);
const OBIS_CURRENT_NET_POWER: ObisCode = ObisCode::from_octet_str(&[1, 0, 16, 7, 0, 255]);
const OBIS_LINE_ONE: ObisCode = ObisCode::from_octet_str(&[1, 0, 36, 7, 0, 255]);
const OBIS_LINE_TWO: ObisCode = ObisCode::from_octet_str(&[1, 0, 56, 7, 0, 255]);
const OBIS_LINE_THREE: ObisCode = ObisCode::from_octet_str(&[1, 0, 76, 7, 0, 255]);

impl MeterReading {
    pub fn parse(sml_file: File) -> Result<Self, Error> {
        println!("SML file \"{:#?}\"", sml_file);
        // The payload must contain 3 messages. An open response, a get list response
        // and a close response.
        if sml_file.messages.len() != 3 {
            bail!("Invalid number of messages: {}", sml_file.messages.len());
        }

        let list_response = &sml_file.messages[1];

        let MessageBody::GetListResponse(get_list_response) = &list_response.message_body else {
            bail!("Unexpected message type: {:?}", list_response.message_body);
        };

        let mut meter_values = MeterReading {
            meter_time:                 None,
            total_energy_inbound:       None,
            total_energy_inbound_unit:  None,
            total_energy_outbound:      None,
            total_energy_outbound_unit: None,
            current_net_power:          None,
            current_net_power_unit:     None,
            line_one:                   None,
            line_one_unit:              None,
            line_two:                   None,
            line_two_unit:              None,
            line_three:                 None,
            line_three_unit:            None,
        };

        for entry in &get_list_response.val_list {
            let obis_code =
                ObisCode::try_from_octet_str(entry.obj_name).map_err(|e| anyhow!("{e:?}"));
            let obis_code = match obis_code {
                Ok(obis_code) => obis_code,
                Err(e) => {
                    println!("Invalid obis code \"{:?}\": {:?}", entry.obj_name, e);
                    continue;
                },
            };
            let unit = entry.unit.and_then(Unit::from_u8);

            match obis_code {
                OBIS_TOTAL_INBOUND_COUNT => {
                    let value = match entry.value {
                        Value::I8(value) => value as f64,
                        Value::I16(value) => value as f64,
                        Value::I32(value) => value as f64,
                        Value::I64(value) => value as f64,
                        Value::U8(value) => value as f64,
                        Value::U16(value) => value as f64,
                        Value::U32(value) => value as f64,
                        Value::U64(value) => value as f64,
                        _ => {
                            println!("Non 32bit integer: {:?}", entry.value);
                            continue;
                        },
                    };

                    let value = if let Some(scaler) = entry.scaler {
                        value as f64 / 10f64.powi(-scaler as i32)
                    } else {
                        value as f64
                    };

                    meter_values.total_energy_inbound = Some(value);
                    meter_values.total_energy_inbound_unit = unit;

                    if let Some(Time::SecIndex(secs)) = entry.val_time {
                        meter_values.meter_time = Some(secs);
                    } else {
                        meter_values.meter_time = None;
                    }
                },
                OBIS_TOTAL_OUTBOUND_COUNT => {
                    let value = match entry.value {
                        Value::I8(value) => value as f64,
                        Value::I16(value) => value as f64,
                        Value::I32(value) => value as f64,
                        Value::I64(value) => value as f64,
                        Value::U8(value) => value as f64,
                        Value::U16(value) => value as f64,
                        Value::U32(value) => value as f64,
                        Value::U64(value) => value as f64,
                        _ => {
                            println!("Non 32bit integer: {:?}", entry.value);
                            continue;
                        },
                    };

                    let value = if let Some(scaler) = entry.scaler {
                        value as f64 / 10f64.powi(-scaler as i32)
                    } else {
                        value as f64
                    };

                    meter_values.total_energy_outbound = Some(value);
                    meter_values.total_energy_outbound_unit = unit;

                    if let Some(Time::SecIndex(secs)) = entry.val_time {
                        meter_values.meter_time = Some(secs);
                    } else {
                        meter_values.meter_time = None;
                    }
                },
                OBIS_CURRENT_NET_POWER => {
                    let value = match entry.value {
                        Value::I8(value) => value as f64,
                        Value::I16(value) => value as f64,
                        Value::I32(value) => value as f64,
                        Value::I64(value) => value as f64,
                        Value::U8(value) => value as f64,
                        Value::U16(value) => value as f64,
                        Value::U32(value) => value as f64,
                        Value::U64(value) => value as f64,
                        _ => {
                            // discard non-64bit integer values
                            println!("Non 16bit integer: {:?}", entry.value);
                            continue;
                        },
                    };

                    let value = if let Some(scaler) = entry.scaler {
                        value as f64 / 10f64.powi(-scaler as i32)
                    } else {
                        value as f64
                    };

                    meter_values.current_net_power = Some(value);
                    meter_values.current_net_power_unit = unit;
                },
                OBIS_LINE_ONE => {
                    let value = match entry.value {
                        Value::I8(value) => value as f64,
                        Value::I16(value) => value as f64,
                        Value::I32(value) => value as f64,
                        Value::I64(value) => value as f64,
                        Value::U8(value) => value as f64,
                        Value::U16(value) => value as f64,
                        Value::U32(value) => value as f64,
                        Value::U64(value) => value as f64,
                        _ => {
                            println!("Non 32bit integer: {:?}", entry.value);
                            continue;
                        },
                    };

                    meter_values.line_one = Some(value);
                    meter_values.line_one_unit = unit;
                },
                OBIS_LINE_TWO => {
                    let value = match entry.value {
                        Value::I8(value) => value as f64,
                        Value::I16(value) => value as f64,
                        Value::I32(value) => value as f64,
                        Value::I64(value) => value as f64,
                        Value::U8(value) => value as f64,
                        Value::U16(value) => value as f64,
                        Value::U32(value) => value as f64,
                        Value::U64(value) => value as f64,
                        _ => {
                            println!("Non 32bit integer: {:?}", entry.value);
                            continue;
                        },
                    };

                    meter_values.line_two = Some(value);
                    meter_values.line_two_unit = unit;
                },
                OBIS_LINE_THREE => {
                    let value = match entry.value {
                        Value::I8(value) => value as f64,
                        Value::I16(value) => value as f64,
                        Value::I32(value) => value as f64,
                        Value::I64(value) => value as f64,
                        Value::U8(value) => value as f64,
                        Value::U16(value) => value as f64,
                        Value::U32(value) => value as f64,
                        Value::U64(value) => value as f64,
                        _ => {
                            println!("Non 32bit integer: {:?}", entry.value);
                            continue;
                        },
                    };

                    meter_values.line_three = Some(value);
                    meter_values.line_three_unit = unit;
                },
                _ => {
                    // discard unknown obis codes
                },
            }
        }

        Ok(meter_values)
    }

    pub fn display_compact(&self) -> String {
        format!(
            "{}s, {} {}, {} {}, {} {}, {} {}, {} {}, {} {}",
            map_unknown(&self.meter_time),
            map_unknown(&self.total_energy_inbound),
            map_unknown(&self.total_energy_inbound_unit),
            map_unknown(&self.total_energy_outbound),
            map_unknown(&self.total_energy_outbound_unit),
            map_unknown(&self.current_net_power),
            map_unknown(&self.current_net_power_unit),
            map_unknown(&self.line_one),
            map_unknown(&self.line_one_unit),
            map_unknown(&self.line_two),
            map_unknown(&self.line_two_unit),
            map_unknown(&self.line_three),
            map_unknown(&self.line_three_unit)
        )
    }
}

fn map_unknown(option: &Option<impl Display>) -> String {
    match option {
        Some(value) => format!("{}", value),
        None => "Unknown".to_string(),
    }
}

impl Display for MeterReading {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Meter Time: {}", map_unknown(&self.meter_time))?;
        writeln!(
            f,
            "Total Energy Inbound: {} {}",
            map_unknown(&self.total_energy_inbound),
            map_unknown(&self.total_energy_inbound_unit)
        )?;
        writeln!(
            f,
            "Total Energy Outbound: {} {}",
            map_unknown(&self.total_energy_outbound),
            map_unknown(&self.total_energy_outbound_unit)
        )?;
        writeln!(
            f,
            "Current Power: {} {}",
            map_unknown(&self.current_net_power),
            map_unknown(&self.current_net_power_unit)
        )?;
        writeln!(
            f,
            "Line One: {} {}",
            map_unknown(&self.line_one),
            map_unknown(&self.line_one_unit)
        )?;
        writeln!(
            f,
            "Line Two: {} {}",
            map_unknown(&self.line_two),
            map_unknown(&self.line_two_unit)
        )?;
        writeln!(
            f,
            "Line Three: {} {}",
            map_unknown(&self.line_three),
            map_unknown(&self.line_three_unit)
        )
    }
}

/// Read SML message stream from a reader
///
/// ```
/// use std::io::Cursor;
/// use hackdose_sml_parser::message_stream::sml_message_stream;
/// use tokio_stream::StreamExt;
///
/// #[tokio::main]
/// async fn main() {
///     let cursor = Cursor::new(vec![0x01, 0x02, 0x03]);
///     let mut message_stream = sml_message_stream(cursor);
///     while let Some(message) = message_stream.next().await {
///         println!("Message: {:?}", message);
///     }
/// }
/// ```
pub fn sml_message_stream(
    mut stream: impl AsyncRead + Unpin + Send + 'static,
) -> impl Stream<Item = MeterReading> {
    let (tx, rx) = mpsc::channel::<MeterReading>(256);

    let mut buf = [0; 512];
    let mut decoder = sml_rs::transport::Decoder::<Vec<u8>>::new();

    tokio::spawn(async move {
        while let Ok(n) = stream.read(&mut buf).await {
            if n == 0 {
                break;
            }
            emit_message(&mut decoder, &buf[..n], tx.clone()).await;
        }
    });

    ReceiverStream::new(rx)
}

async fn emit_message<'a>(
    decoder: &'a mut sml_rs::transport::Decoder<Vec<u8>>,
    buf: &'a [u8],
    tx: Sender<MeterReading>,
) -> Result<(), Error> {
    let mut to_process = buf.to_vec();
    for byte in to_process {
        match decoder.push_byte(byte) {
            Ok(None) => {},
            Ok(Some(decoded_bytes)) => {
                let result = sml_rs::parser::complete::parse(decoded_bytes);
                let Ok(sml_file) = result else {
                    // if self.verbose {
                    println!("Err({:?})", result);
                    // }
                    continue;
                };

                let reading = MeterReading::parse(sml_file);
                let Ok(reading) = reading else {
                    continue;
                };
                // if self.verbose {
                println!("{}", reading.display_compact());
                // }
                let _ = tx.send(reading).await;

                // let _ = publish_data(&reading, mqtt_client).await;

                // self.latest_reading.store(Some(reading));
            },
            Err(e) => {
                // if self.verbose {
                println!("Err({:?})", e);
                // }
            },
        }
    }
    Ok(())
}
