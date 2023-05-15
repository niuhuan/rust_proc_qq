use anyhow::anyhow;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::SchedulerJob;

pub struct SchedulerHandler {
    pub(crate) jobs_scheduler: JobScheduler,
    pub(crate) scheduler_job: Vec<SchedulerJob>,
    pub(crate) client: Arc<ricq::Client>,
}

impl SchedulerHandler {
    pub async fn new(client: Arc<ricq::Client>) -> anyhow::Result<Self> {
        let jobs_scheduler = JobScheduler::new()
            .await
            .map_err(|_| anyhow!("JobScheduler 初始化失败"))?;
        Ok(Self {
            jobs_scheduler,
            scheduler_job: vec![],
            client,
        })
    }

    /// 初始化
    pub async fn init(&mut self) -> anyhow::Result<JoinHandle<JobScheduler>> {
        if !self.scheduler_job.is_empty() {
            for j in &self.scheduler_job {
                for h in &j.handles {
                    let job = Arc::clone(h);
                    let c = Arc::clone(&self.client);
                    let job_locked = match job.time_type() {
                        TimeType::Cron(cron) => {
                            Job::new_async(cron.as_str(), move |_, _| job.call(c.clone()))
                        }
                        TimeType::Duration(time) => Job::new_repeated_async(
                            std::time::Duration::from_secs(time),
                            move |_, _| job.call(c.clone()),
                        ),
                    }
                    .map_err(|_| anyhow!("Failed to create the {} task", j.name))?;
                    self.jobs_scheduler.add(job_locked).await?;
                }
                tracing::debug!("Add {} Job", j.name);
            }
        }
        let jobs_scheduler_locked = self.jobs_scheduler.clone();
        let handle = tokio::spawn(async move {
            jobs_scheduler_locked
                .start()
                .await
                .expect("定时任务启动失败!");
            jobs_scheduler_locked
        });
        Ok(handle)
    }
}
pub enum TimeType {
    Cron(String),
    Duration(u64),
}
pub trait ScheduledJobHandler: Sync + Send {
    fn time_type(&self) -> TimeType;
    fn call(&self, c: Arc<ricq::Client>) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>;
}
