use tracing::Level;
use tracing_subscriber::fmt::format;
#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt().event_format(format().compact()).with_max_level(Level::INFO).init();
    tokio::join!(server::start(),);
}
