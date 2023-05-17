use anyhow::Context;
use proc_qq::re_exports::ricq::Client;
use proc_qq::re_exports::ricq_core::msg::elem::Text;
use proc_qq::re_exports::ricq_core::msg::MessageChain;
use proc_qq::{scheduler, scheduler_job, MessageChainAppendTrait, Scheduler};
use std::sync::Arc;

/// 每1分钟发送一次 Hello std::time::Duration  秒
#[scheduler_job(repeat = 60)]
async fn handle_scheduler(c: Arc<Client>) -> anyhow::Result<()> {
    let chain = MessageChain::default().append(Text::new("Hello".to_owned()));
    c.send_friend_message(123123, chain)
        .await
        .with_context(|| "sent message failed")?;
    Ok(())
}

/// 每3分钟 获取一次网络状态
#[scheduler_job(cron = "0 */3 * * * ?")]
async fn handle_scheduler02(c: Arc<Client>) -> anyhow::Result<()> {
    println!("{}", c.get_status());
    Ok(())
}

/// scheduler
pub fn scheduler() -> Scheduler {
    scheduler!("hello_jobs", handle_scheduler, handle_scheduler02,)
}
