use anyhow::Result;
//use std::time::Duration;
use chrono::{Duration, Utc, Timelike, Datelike};
use tokio_cron_scheduler::{Job, JobScheduler, JobToRun};
use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;
use std::sync::Arc;

#[derive(Debug, Clone)]
struct JobData{
    notification_string: String,
    schedule_time_string: String,
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");
    let sched = JobScheduler::new().await;
    let sched = sched.unwrap();
    run_example(sched).await;
}

async fn run_example(mut sched: JobScheduler) -> Result<()> {
    #[cfg(feature = "signal")]
    sched.shutdown_on_ctrl_c();

    sched.set_shutdown_handler(Box::new(|| {
        Box::pin(async move {
            info!("Shut down done");
        })
    }));

    /// test data for start job
    let now = Utc::now();
    let schedule_time = get_cron_schedule_string_by_datetime(now.checked_add_signed(Duration::seconds(10)).unwrap());
    // let start = now.checked_add_signed(Duration::seconds(10));
    // let schedule_time = match start {
    //     Some(x) => {
    //         get_cron_schedule_string_by_datetime(x)
    //     },
    //     None => {
    //         println!("----Something wong!");
    //         get_cron_schedule_string_by_datetime(now)
    //     },
    // };

    let notification_string:String = String::from("TEXT");

    let job_data = JobData{
        notification_string: String::from(notification_string),
        schedule_time_string: schedule_time,
    };

    /// crate new async job
    let mut async_one_shot_job = Job::new_async(&*job_data.schedule_time_string, |uuid, mut l| {
        Box::pin(async move {
            info!(">>>>>>>> RUN RUN RUN >>>>>>>>> I run async id {:#?}", uuid);
            info!("!(>>>>>>>> = {:#?}", job_data.notification_string);
            // let next_tick = l.next_tick_for_job(uuid).await;
            // match next_tick {
            //     Ok(Some(ts)) => info!("Next time for 4s is {:?}", ts),
            //     _ => warn!("Could not get next tick for 4s job"),
            // }
        })
    })
    .unwrap();
    
    /// configure on_start_notification
    let async_one_shot_job_clone = async_one_shot_job.clone();
    let js = sched.clone();
    info!("4s job id {:?}", async_one_shot_job.guid());
    async_one_shot_job.on_start_notification_add(&sched, Box::new(move |job_id, notification_id, type_of_notification| {
        let async_one_shot_job_clone = async_one_shot_job_clone.clone();
        let js = js.clone();
        Box::pin(async move {
            info!(" ->ON START>- Job {:?} ran on start notification {:?} ({:?})", job_id, notification_id, type_of_notification);
            // info!(" ->ON START>- This should only run once since we're going to remove this notification immediately.");
            // info!(" ->ON START>- Removed? {:?}", async_one_shot_job_clone.on_start_notification_remove(&js, &notification_id).await);
        })
    })).await?;

    /// configure on_stop_notification_add
    async_one_shot_job
        .on_stop_notification_add(
            &sched,
            Box::new(|job_id, notification_id, type_of_notification| {
                Box::pin(async move {
                    info!(
                        " -<ON STOP<- | Job {:?} was completed, notification {:?} ran ({:?})",
                        job_id, notification_id, type_of_notification
                    );
                })
            }),
        )
        .await?;

    /// configure on_done_notification_add
    async_one_shot_job
        .on_done_notification_add(
            &sched,
            Box::new(|job_id, notification_id, type_of_notification| {
                Box::pin(async move {
                    info!(
                        " -<ON DONE>- | Job {:?} completed and ran notification {:?} ({:?})",
                        job_id, notification_id, type_of_notification
                    );
                })
            }),
        )
        .await?;

    /// configure on_removed_notification_add
    async_one_shot_job
        .on_removed_notification_add(
            &sched,
            Box::new(|job_id, notification_id, type_of_notification| {
                Box::pin(async move {
                    info!(
                        " -> ON REMOVED<- | Job {:?} was removed, notification {:?} ran ({:?})",
                        job_id, notification_id, type_of_notification
                    );
                })
            }),
        )
        .await?;

    let async_one_shot_job_guid = async_one_shot_job.guid();
    sched.add(async_one_shot_job).await?;

    let start = sched.start().await;
    if start.is_err() {
        error!("Error starting scheduler");
        return Ok(());
    }
    tokio::time::sleep(std::time::Duration::from_secs(40)).await;

    info!("Remove jobs");
    sched.remove(&async_one_shot_job_guid).await?;
    tokio::time::sleep(std::time::Duration::from_secs(40)).await;

    info!("Goodbye.");
    sched.shutdown().await?;
    Ok(())
}

fn get_cron_schedule_string_by_datetime(x: chrono::DateTime<Utc>) -> String {
    [x.second().to_string(), 
    x.minute().to_string(), 
    x.hour().to_string(),
    x.day().to_string(),
    x.month().to_string(),
    x.weekday().to_string(),
    x.year().to_string()].join(" ")
}
