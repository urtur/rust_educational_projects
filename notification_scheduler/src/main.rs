use anyhow::Result;
use std::time::{Duration, self};
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");
    let sched = JobScheduler::new().await;
    let sched = sched.unwrap();
    /// test data for start job
    let (diff, notification_string) = {
        let start_time = time::Instant::now();
        let end_time = start_time
            .checked_add(Duration::from_secs(40))
            .unwrap();
        let diff = end_time.elapsed();
        let notification_string = Arc::new(String::from("TEXT"));
        (diff, notification_string)
    };
    run_atomic_notification(sched, diff, notification_string).await;
}

async fn run_atomic_notification(mut sched: JobScheduler, diff: Duration, notification_string: Arc<String>) -> Result<()> {
    #[cfg(feature = "signal")]
    sched.shutdown_on_ctrl_c();

    sched.set_shutdown_handler(Box::new(|| {
        Box::pin(async move {
            info!("Shut down done");
        })
    }));

    /// crate new async job
    //let mut async_one_shot_job = Job::new_async(schedule_time.as_str(), move |uuid, mut l|
    let async_one_shot_job = Job::new_one_shot_async(diff, move |uuid, mut l| {
        let notification_string = Arc::clone(&notification_string);
        Box::pin(async move {
            //let notification_string = Arc::clone(&notification_string);
            info!(
                "!(>>>>>>>> notification_string = {:#?}",
                notification_string
            );
            info!(">>>>>>>> RUN RUN RUN >>>>>>>>> I run async id {:#?}", uuid);
        })
    })
    .unwrap();

    let async_one_shot_job_guid = async_one_shot_job.guid();
    sched.add(async_one_shot_job).await?;

    let start = sched.start().await;
    if start.is_err() {
        error!("Error starting scheduler");
        return Ok(());
    }
    tokio::time::sleep(Duration::from_secs(40)).await;

    info!("Remove jobs");
    sched.remove(&async_one_shot_job_guid).await?;
    tokio::time::sleep(Duration::from_secs(40)).await;

    info!("Goodbye.");
    sched.shutdown().await?;
    Ok(())
}
