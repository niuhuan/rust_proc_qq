use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub mod hello_module;
pub mod result_handlers;
pub mod scheduler_handlers;

pub fn init_tracing_subscriber() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .without_time(),
        )
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("ricq", Level::DEBUG)
                .with_target("proc_qq", Level::DEBUG)
                // 如果需搬运, 这里换成自己的项目名
                .with_target("proc_qq_examples", Level::DEBUG),
        )
        .init();
}
