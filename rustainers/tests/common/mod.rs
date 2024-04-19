use rstest::fixture;
use tracing::{debug, Level};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::time;

use rustainers::runner::Runner;

pub mod images;

pub fn init_test_tracing(level: Level) {
    tracing_subscriber::fmt()
        .pretty()
        .with_line_number(true)
        .with_thread_names(true)
        .with_span_events(FmtSpan::NONE)
        .with_timer(time::uptime())
        .with_max_level(level)
        .with_test_writer()
        .init();
}

#[fixture]
#[once]
pub fn runner() -> Runner {
    init_test_tracing(Level::INFO);

    #[allow(clippy::expect_used)]
    let runner = Runner::auto().expect("Should find a valid runner");
    debug!("Using runner {runner:?}");
    runner
}
