use anyhow::Result;
use chrono::Duration as cDuration;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::time::{self, Duration, Instant};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

/// A structure that encapsulates the data to be notified.
#[derive(Debug)]
pub struct NotificationData {
    /// The date to which you need to send a notification
    notification_date: DateTime<Utc>,
    /// The text of the message that will be sent to the recipient during the notification
    notification_string: Arc<String>,
}

impl NotificationData {
    pub fn get_notification_date(&self) -> DateTime<Utc> {
        self.notification_date
    }

    pub fn get_notification_string(&self) -> Arc<String> {
        Arc::clone(&self.notification_string)
    }
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");
    let sched = JobScheduler::new().await;
    let sched = sched.unwrap();

    // test data for start job
    let now = Utc::now();
    let end_time_1 = now.checked_add_signed(cDuration::seconds(80)).unwrap();
    let notification_data_1 = NotificationData {
        notification_date: end_time_1,
        notification_string: Arc::from(String::from("TEXT_1")),
    };

    let end_time_2 = now.checked_add_signed(cDuration::seconds(20)).unwrap();
    let notification_data_2 = NotificationData {
        notification_date: end_time_2,
        notification_string: Arc::from(String::from("TEXT_2")),
    };
    let notification_datas: Vec<NotificationData> = vec![notification_data_1, notification_data_2];

    // run notification
    run_notification(sched, notification_datas).await;

    // run periodicity job to fetch data from google sheet
}

async fn run_notification(
    mut sched: JobScheduler,
    notification_datas: Vec<NotificationData>,
) -> Result<()> {
    #[cfg(feature = "signal")]
    sched.shutdown_on_ctrl_c();

    sched.set_shutdown_handler(Box::new(|| {
        Box::pin(async move {
            info!("Shut down done");
        })
    }));

    for notification_data in notification_datas.iter() {
        // Creating a new job and adding this job to the scheduler
        info!(
            "create job with notification_data = {:#?}",
            notification_data
        );
        notification_job(notification_data, &sched).await?;
    }

    repeated_job(&sched).await?;

    let start = sched.start().await;
    if start.is_err() {
        error!("Error starting scheduler");
        return Ok(());
    }
    tokio::time::sleep(Duration::from_secs(140)).await;

    info!("Goodbye.");
    sched.shutdown().await?;
    Ok(())
}

/// Creating a new job and adding this job to the scheduler
async fn notification_job(
    notification_data: &NotificationData,
    sched: &JobScheduler,
) -> Result<(), anyhow::Error> {
    let duration_between_now_and_notification_date = notification_data
        .get_notification_date()
        .signed_duration_since(Utc::now())
        .to_std()
        .unwrap();
    let notification_instant = Instant::now()
        .checked_add(duration_between_now_and_notification_date)
        .unwrap();
    let notification_string: Arc<String> = Arc::clone(&notification_data.get_notification_string());
    let async_one_shot_job =
        Job::new_one_shot_at_instant_async(notification_instant, move |uuid, mut l| {
            let notification_string = Arc::clone(&notification_string);
            Box::pin(async move {
                do_notify(notification_string, uuid).await;
            })
        })
        .unwrap();
    sched.add(async_one_shot_job).await?;
    Ok(())
}

/// Creating a new repeated job periodicity run for fetch data from source and adding this job to the scheduler
async fn repeated_job(sched: &JobScheduler) -> Result<(), anyhow::Error> {
    let duration = cDuration::seconds(5).to_std().unwrap();
    let async_repeated_job = Job::new_repeated_async(duration, move |uuid, mut l| {
        Box::pin(async move {
            do_repeat().await;
        })
    })
    .unwrap();
    sched.add(async_repeated_job).await?;
    Ok(())
}

async fn do_notify(notification_string: Arc<String>, uuid: uuid::Uuid) {
    let now = chrono::Utc::now();
    info!("!>>>>>>>> time start = {:?}", now);
    //tokio::time::sleep(Duration::from_secs(40)).await;
    info!("!>>>>>>>> notification_string = {:#?}", notification_string);
    info!("!>>>>>>>> RUN RUN RUN >>>>>>>>> I run async id {:#?}", uuid);
}

async fn do_repeat() {
    info!("!>>>>>>>> Fetch data from source periodically <<<<<<<<<!");
    info!("!>>>>>>>> time start = {:?} <<<<<<<<<!", chrono::Utc::now());
    tokio::time::sleep(Duration::from_secs(40)).await;
    info!("!>>>>>>>> time end = {:?} <<<<<<<<<!", chrono::Utc::now());
}
