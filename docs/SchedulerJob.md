定时任务
======

需要设置 `features = [ "scheduler" ]`

```rust
/// 每1分钟发送一次 Hello
#[scheduler_job(cron = "0 0/1 * * * ?")]
async fn handle_scheduler(c:Arc<Client>) -> anyhow::Result<()> {
    let chain = MessageChain::default()
        .append(Text::new("Hello".to_owned()));
    c.send_friend_message(123123,chain).await.expect("sent message failed");
    Ok(())
}

/// 每3分钟 获取一次网络状态
#[scheduler_job(repeat = 180)]
async fn handle_scheduler02(c:Arc<Client>) -> anyhow::Result<()> {
    println!("{}",c.get_status());
    Ok(())
}

/// scheduler
pub fn scheduler() -> Scheduler {
    scheduler!(
        "hello_jobs",
        handle_scheduler,
        handle_scheduler02,
    )
}
```

```rust
ClientBuilder::new()
.modules(vec![hello_module::module()])
.schedulers(vec![scheduler_handlers::scheduler()])
.build()
```

## 自己实现Scheduler

可以实现有状态的SchedulerJob

```rust

#[allow(non_camel_case_types)]
pub struct handle_scheduler02 ;

#[async_trait]
impl::proc_qq::SchedulerJobHandler for handle_scheduler02 {
    async fn call(&self, c: Arc<Client>) -> ::anyhow::Result<()> {
        println! ("{}", c.get_status()) ; Ok(()) 
    }
}

fn test(){
    ::proc_qq::Scheduler {
        id : "hello_jobs".to_owned(),
        jobs : vec![
            :: proc_qq :: SchedulerJob {
                id: "handle_scheduler02".to_owned(),
                job: Box::new(handle_scheduler02{}),
            }
        ],
    }
}
```