use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio_cron_scheduler::Job;
use crate::{Client, SchedulerJob};


pub struct SchedulerHandler {
    pub(crate) client: Arc<Client>,
    pub(crate) scheduler_job: Vec<SchedulerJob>,
}

impl SchedulerHandler {

    /// 启动定时任务执行器
    pub async fn start(&self)  -> anyhow::Result<()>{
        let scheduler = tokio_cron_scheduler::JobScheduler::new().await?;
        let client = self.client.clone();
        let scheduler_job = self.scheduler_job.clone();
        for job in scheduler_job {
            tracing::info!("Add {} Job",job.name);
            for job in job.handles {
                let _teak = Arc::clone(&job);
                let client = Arc::clone(&client);
                let job = Job::new_async(_teak.cron().as_str(), move |_, _| _teak.call(client.clone()))?;
                scheduler.add(job).await?;
            }
        }
        scheduler.start().await?;
        Ok(())
    }
    
}

pub trait ScheduledJobHandler: Sync + Send {
    fn cron(&self) -> String;
    fn call(&self, bot: Arc<Client>) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>;
}

