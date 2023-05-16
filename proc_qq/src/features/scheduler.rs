use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio_cron_scheduler::Job;

pub struct Scheduler {
    pub id: String,
    pub jobs: Vec<SchedulerJob>,
}

pub struct SchedulerJob {
    pub id: String,
    pub period: SchedulerJobPeriod,
    pub handler: Arc<Box<dyn SchedulerJobHandler>>,
}

pub enum SchedulerJobPeriod {
    Cron(String),
    Repeat(u64),
}

#[async_trait::async_trait]
pub trait SchedulerJobHandler: Sync + Send {
    async fn call(&self, c: Arc<ricq::Client>) -> anyhow::Result<()>;
}

#[allow(dead_code)]
struct SchedulerJobProcess {
    scheduler_id: String,
    job_id: String,
    client: Arc<ricq::Client>,
    handler: Arc<Box<dyn SchedulerJobHandler>>,
}

impl SchedulerJobProcess {
    pub fn do_process(&self) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        let handler = Arc::clone(&self.handler);
        let client = Arc::clone(&self.client);
        let scheduler_id = self.scheduler_id.clone();
        let job_id = self.job_id.clone();
        Box::pin(async move {
            match handler.call(client).await {
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!("定时任务执行失败 : {scheduler_id} : {job_id} : {e}");
                }
            };
        })
    }
}

pub(crate) async fn put_scheduler(
    js: &mut tokio_cron_scheduler::JobScheduler,
    scs: Arc<Vec<Scheduler>>,
    client: Arc<ricq::Client>,
) -> anyhow::Result<()> {
    for sc in scs.clone().iter() {
        for job in &sc.jobs {
            let process = SchedulerJobProcess {
                scheduler_id: sc.id.clone(),
                job_id: job.id.clone(),
                client: Arc::clone(&client),
                handler: job.handler.clone(),
            };
            let lock = match &job.period {
                SchedulerJobPeriod::Cron(cron) => {
                    Job::new_async(cron.as_str(), move |_, _| process.do_process())?
                }
                SchedulerJobPeriod::Repeat(time) => Job::new_repeated_async(
                    std::time::Duration::from_secs(time.clone()),
                    move |_, _| process.do_process(),
                )?,
            };
            js.add(lock).await?;
        }
    }
    Ok(())
}
