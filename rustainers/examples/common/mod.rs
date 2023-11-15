use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::time;

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
