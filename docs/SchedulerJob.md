��ʱ����
======


```rust
/// ÿ1���ӷ���һ�� Hello
#[scheduler_job(cron = "0 0/1 * * * ?")]
async fn handle_scheduler(c:Arc<Client>) {
    let chain = MessageChain::default()
        .append(Text::new("Hello".to_owned()));
    c.rq_client.send_friend_message(123123,chain).await.expect("sent message failed");
}

/// ÿ3���� ��ȡһ������״̬
#[scheduler_job(cron = "0 0/3 * * * ?")]
async fn handle_scheduler02(c:Arc<Client>) {
    println!("{}",c.rq_client.get_status());
}

/// scheduler
pub fn scheduler() -> SchedulerJob {
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