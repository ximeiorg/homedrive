#[tokio::main]
async fn main() {
    tokio::join!(server::start(),);
}
