use proc_qq::re_exports::ricq_core::msg::elem::Text;
use proc_qq::re_exports::ricq_core::msg::MessageChain;
use proc_qq::{scheduler, scheduler_job, Client, MessageChainAppendTrait, SchedulerJob};
use std::sync::Arc;

/// 每1分钟发送一次 Hello
#[scheduler_job(cron = "0 0/1 * * * ?")]
async fn handle_scheduler(c: Arc<Client>) {
    let chain = MessageChain::default().append(Text::new("Hello".to_owned()));
    c.rq_client
        .send_friend_message(2939599299, chain)
        .await
        .expect("sent message failed");
}

/// 每3分钟 获取一次网络状态
#[scheduler_job(cron = "0 0/3 * * * ?")]
async fn handle_scheduler02(c: Arc<Client>) {
    println!("{}", c.rq_client.get_status());
}

/// scheduler
pub fn scheduler() -> SchedulerJob {
    scheduler!("hello_jobs", handle_scheduler, handle_scheduler02,)
}
