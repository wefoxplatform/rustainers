use rstest::fixture;
use tracing::{debug, Level};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::time;

use rustainers::images::Redis;
use rustainers::runner::Runner;

pub fn init_tracing(level: Level) {
    tracing_subscriber::fmt()
        .pretty()
        .with_line_number(true)
        .with_thread_names(true)
        .with_span_events(FmtSpan::NONE)
        .with_timer(time::uptime())
        .with_max_level(level)
        .init();
}

#[fixture]
#[once]
pub fn runner() -> Runner {
    init_tracing(Level::INFO);

    let runner = Runner::auto().expect("Should find a valid runner");
    debug!("Using runner {runner:?}");
    runner
}

#[fixture]
#[once]
pub fn redis() -> Redis {
    Redis::default()
}
