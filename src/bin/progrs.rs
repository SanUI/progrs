use std::io;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
  progrs::main().await
}
