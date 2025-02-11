use anyhow::Error;
use clap_derive::Args;
use log::{error, info};
use tokio_cron_scheduler::Job;

use crate::core_loop::CoreLoop;

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
    pub async fn run(self) -> Result<(), Error> {
        let core_loop = CoreLoop::new(self.port, self.verbose);
        let sched = tokio_cron_scheduler::JobScheduler::new().await?;
        let mut handles = Vec::new();

        let mut mqttoptions =
            rumqttc::MqttOptions::new(MQTT_CLIENT_NAME, MQTT_BROKER_ADDRESS, MQTT_BROKER_PORT);
        mqttoptions.set_keep_alive(std::time::Duration::from_secs(10));

        let (client, mut eventloop) = rumqttc::AsyncClient::new(mqttoptions, 5);

        // This job runs every 10 seconds and retrieves the current power consumption
        // from the power metet
        // let mut sml_job = Job::new_async("1/10 * * * * *", move |_, _| {
        //     let client = client.clone();
        //     let core = core_loop.clone();
        //     Box::pin(async move {
        //         if let Err(e) = core.get_data_and_publish(&client).await {
        //             error!("Failed SML API job: {:?}", e);
        //         }
        //     })
        // })?;

        // sml_job
        //     .on_stop_notification_add(
        //         &sched,
        //         Box::new(|job_id, notification_id, type_of_notification| {
        //             Box::pin(async move {
        //                 info!(
        //                     "Job {:?} was completed, notification {:?} ran ({:?})",
        //                     job_id, notification_id, type_of_notification
        //                 );
        //             })
        //         }),
        //     )
        //     .await?;

        // sched.add(sml_job).await?;
        // sched.start().await?;
        handles.push(tokio::task::spawn(async move {
            loop {
                println!("core loop start");
                let client = client.clone();
                let core = core_loop.clone();
                if let Err(e) = core.get_data_and_publish(&client).await {
                    error!("Failed SML API job: {:?}", e);
                }
                println!("core loop end");
            }
        }));

        handles.push(tokio::task::spawn(async move {
            loop {
                println!("event loop start");
                if let Err(e) = eventloop.poll().await {
                    // In case of an error stop event loop and terminate task
                    // this will result in aborting the program
                    error!("Error MQTT Event loop returned: {:?}", e);
                    break;
                }
                println!("event loop end");
            }
        }));

        // In case any of the tasks panic abort the program
        for handle in handles {
            if let Err(e) = handle.await {
                error!("Task panicked: {:?}", e);
                std::process::exit(1);
            }
        }

        Ok(())
    }
}
