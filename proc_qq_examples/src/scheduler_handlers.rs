use std::sync::Arc;
use proc_qq::{Client, ClientTrait, scheduler, scheduler_job, SchedulerJob};

/// 每1分钟获取一次 bot uin
#[scheduler_job(cron = "0 0/1 * * * ?")]
async fn handle_scheduler(c:Arc<Client>) {
    let bot_uin = c.bot_uin().await;
    println!("{}", bot_uin);
}

/// scheduler
pub fn scheduler() -> SchedulerJob {
    scheduler!(
        "hello_jobs",
        handle_scheduler
    )
}